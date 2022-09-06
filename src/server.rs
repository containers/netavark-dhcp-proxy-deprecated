#![cfg_attr(not(unix), allow(unused_imports))]
use mozim::{DhcpError, DhcpV4Client, DhcpV4Config, DhcpV4Lease as MozimV4Lease, ErrorKind};
use netavark_proxy::cache::LeaseCache;
use netavark_proxy::g_rpc::netavark_proxy_server::{NetavarkProxy, NetavarkProxyServer};
use netavark_proxy::g_rpc::{Empty, Lease as NetavarkLease, NetworkConfig, OperationResponse};
use netavark_proxy::{DEFAULT_CONFIG_DIR, DEFAULT_UDS_PATH};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;

use clap::Parser;
use log::{debug, warn};
use tonic::{transport::Server, Code, Code::Internal, Request, Response, Status};

const POLL_WAIT_TIME: isize = 5;

#[derive(Debug)]
/// This is the tonic netavark proxy service that is required to impl the Netavark Proxy trait which
/// includes the gRPC methods defined in proto/proxy.proto. We can store a atomically referenced counted
/// mutex cache in the structure tuple.
///
/// The cache needs to be **safely mutable across multiple threads**. We need to share the lease cache
/// across multiple threads for 2 reasons
/// 1. Each tonic request is spawned in its own new thread.
/// 2. A new thread must be spawned in any request that uses mozim, such as get_lease. This is because
///    tonic creates its own runtime for each request and mozim trys to make its own runtime inside of
///    a runtime.
///
struct NetavarkProxyService(Arc<Mutex<LeaseCache>>);

// gRPC request and response methods
#[tonic::async_trait]
impl NetavarkProxy for NetavarkProxyService {
    /// A new lease will be sent back to netavark on request. get_lease() is responsible for also
    /// updating the cache based on the dhcp event that happens.
    async fn setup(
        &self,
        request: Request<NetworkConfig>,
    ) -> Result<Response<NetavarkLease>, Status> {
        debug!("Request from client {:?}", request.remote_addr());
        //Spawn a new thread to avoid tokio runtime issues
        let cache = self.0.clone();
        std::thread::spawn(move || {
            // Set up some common values
            let network_config: NetworkConfig = request.into_inner();
            println!("{:#?}", serde_json::to_string_pretty(&network_config));
            // Make sure a mac address was supplied in the NetworkConfig and validate the addr if it exists
            let mac_addr = network_config.mac_addr;

            // DHCP client will be in charge of making the DORA requests to the DHCP server
            let mut client = match get_client(&network_config.iface, &network_config.version) {
                Ok(c) => c,
                Err(e) => return Err(Status::new(Internal, e.to_string())),
            };
            // Attempt to process for a lease 31 times. Sleep for a second every 8 attempts
            for i in 1..31 {
                let events = client.poll(POLL_WAIT_TIME).unwrap();
                for event in events {
                    match client.process(event) {
                        // Lease successfully found
                        Ok(Some(new_lease)) => {
                            // the lease must be mutable in order to add the mac address and domain name
                            let mut netavark_lease =
                                <NetavarkLease as From<MozimV4Lease>>::from(new_lease);
                            netavark_lease.add_mac_address(&mac_addr);
                            netavark_lease.add_domain_name(network_config.domain_name);
                            if let Err(e) =
                                cache.lock().unwrap().add_lease(&mac_addr, &netavark_lease)
                            {
                                return Err(Status::new(
                                    Internal,
                                    format!("Error caching the lease: {}", e),
                                ));
                            }
                            return Ok(Response::new(netavark_lease));
                        }
                        Err(err) => {
                            return Err(Status::new(Internal, err.to_string()));
                        }
                        Ok(None) => {}
                    };
                }
                // Sleep for one second every 8 attempts
                if i % 8 == 0 {
                    log::debug!("No DHCP response. Sleeping 1s");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
            Err(Status::new(
                Code::Unavailable,
                "Could not find a lease. Likely could not find a dhcp server",
            ))
        })
        .join()
        .expect("Error joining thread")
    }
    /// When a container is shut down this method should be called. It will clear the lease information
    /// from the caching system.
    async fn teardown(
        &self,
        request: Request<NetworkConfig>,
    ) -> Result<Response<NetavarkLease>, Status> {
        let nc = request.into_inner();
        let empty_lease = NetavarkLease {
            t1: 0,
            t2: 0,
            lease_time: 0,
            mtu: 0,
            domain_name: "".to_string(),
            mac_address: nc.mac_addr.clone(),
            is_v6: false,
            v4: None,
            v6: None,
        };

        self.0.clone().lock().unwrap().remove_lease(&nc.mac_addr)?;
        Ok(Response::new(empty_lease))
    }

    /// On teardown of the proxy the cache will be cleared gracefully.
    async fn clean(&self, request: Request<Empty>) -> Result<Response<OperationResponse>, Status> {
        log::debug!("Request from client: {:?}", request.remote_addr());
        self.0.clone().lock().unwrap().teardown()?;
        Ok(Response::new(OperationResponse { success: true }))
    }
}

/// Create a DHCP client using mozim. This method takes a interface name and version and generates
/// a DHCP client that can be processed for DHCP events
///
/// # Arguments
///
/// * `iface`: network interface name
/// * `version`: Version - can be Ipv4 or Ipv6
///
/// returns: Result<DhcpV4Client, DhcpError>
///
/// On success a DHCP client of the version type will be returned. On failure a DHCP error will be
/// returned
fn get_client(iface: &str, version: &i32) -> Result<DhcpV4Client, DhcpError> {
    match version {
        //V4
        0 => {
            let config = DhcpV4Config::new(iface)?;
            let client = DhcpV4Client::init(config, None)?;
            Ok(client)
        }
        //V6 TODO implement DHCPv6
        1 => {
            unimplemented!();
        }
        // No valid version found in the network configuration sent by the client
        _ => Err(DhcpError::new(
            ErrorKind::InvalidArgument,
            String::from("Must select a valid IP protocol 0=v4, 1=v6"),
        )),
    }
}
#[derive(Parser, Debug)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// location to store backup files
    #[clap(short, long)]
    dir: Option<String>,
    /// alternative uds location
    #[clap(short, long)]
    uds: Option<String>,
}

#[tokio::main]
#[allow(unused)]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().format_timestamp(None).init();
    let opts = Opts::parse();

    // where we store the cache file
    let conf_dir = opts.dir.unwrap_or(DEFAULT_CONFIG_DIR.to_string());
    // location of the grpc port
    let uds_path = opts.uds.unwrap_or(DEFAULT_UDS_PATH.to_string());
    // Match because parent reruns an option
    match Path::new(&uds_path).parent() {
        None => {
            log::error!("Could not find uds path");
            return Ok(());
        }
        Some(f) => tokio::fs::create_dir_all(f).await?,
    }
    // Listen on UDS path
    let uds = UnixListener::bind(&uds_path)?;
    let uds_stream = UnixListenerStream::new(uds);

    let cache = match LeaseCache::new(None) {
        Ok(c) => Arc::new(Mutex::new(c)),
        Err(e) => {
            log::error!("Could not setup the cache: {}", e.to_string());
            return Ok(());
        }
    };
    let netavark_proxy_service = NetavarkProxyService(cache);
    Server::builder()
        .add_service(NetavarkProxyServer::new(netavark_proxy_service))
        .serve_with_incoming(uds_stream)
        .await?;

    //Clean up UDS on exit
    match fs::remove_file(uds_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            warn!("Could not remove the file: {}", e);
            Ok(())
        }
    }
}
