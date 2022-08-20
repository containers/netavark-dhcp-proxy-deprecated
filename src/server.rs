use std::sync::{Arc, Mutex};
use mozim::{DhcpError, DhcpV4Client, DhcpV4Config, DhcpV4Lease as MozimV4Lease, ErrorKind};
use netavark_proxy::cache::LeaseCache;
use netavark_proxy::g_rpc::netavark_proxy_server::{NetavarkProxy, NetavarkProxyServer};
use netavark_proxy::g_rpc::{Lease as NetavarkLease, NetworkConfig, OperationResponse, MacAddress};
use tokio;
use tonic::Code::InvalidArgument;
use tonic::{transport::Server, Code::Internal, Request, Response, Status, Code};

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
    async fn get_lease(
        &self,
        request: Request<NetworkConfig>,
    ) -> Result<Response<NetavarkLease>, Status> {
        println!("Request from client: {:?}", request.remote_addr());
        //Spawn a new thread to avoid tokio runtime issues
        let cache = self.0.clone();
        std::thread::spawn(move || {
            // Set up some common values
            let network_config: NetworkConfig = request.into_inner();
            // Make sure a mac address was supplied in the NetworkConfig and validate the addr if it exists
            let mac_addr = match network_config.mac_addr {
                Some(addr) => {
                    if !addr.validate() {
                        return Err(Status::new(InvalidArgument, "Invalid Mac address"));
                    }
                    addr
                }
                None => {
                    return Err(Status::new(InvalidArgument, "No mac address supplied"));
                }
            };

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
                            let mut netavark_lease = <NetavarkLease as From<MozimV4Lease>>::from(new_lease);
                            netavark_lease.add_mac_address(&mac_addr);
                            netavark_lease.add_domain_name(network_config.domain_name);
                            if let Err(e) = cache.lock().unwrap().add_lease(&mac_addr, &netavark_lease) {
                                return Err(Status::new(Internal, format!("Error caching the lease: {}", e.to_string())));
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
            return Err(Status::new(Code::Unavailable, "Could not find a lease. Likely could not find a dhcp server"));
        })
            .join()
            .expect("Error joining thread")
    }
    /// When a container is shut down this method should be called. It will clear the lease information
    /// from the caching system.
    async fn remove_lease(
        &self,
        request: Request<MacAddress>,
    ) -> Result<Response<OperationResponse>, Status> {
        self.0.clone().lock().unwrap().remove_lease(request.into_inner())?;
        Ok(Response::new(OperationResponse { success: true }))
    }

    /// On teardown of the proxy the cache will be cleared gracefully.
    async fn tear_down(
        &self,
        request: Request<NetworkConfig>,
    ) -> Result<Response<OperationResponse>, Status> {
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
fn get_client(iface: &String, version: &i32) -> Result<DhcpV4Client, DhcpError> {
    match version {
        //V4
        0 => {
            let config = DhcpV4Config::new(&iface)?;
            let client = DhcpV4Client::init(config, None)?;
            return Ok(client);
        }
        //V6 TODO implement DHCPv6
        1 => {
            unimplemented!();
        }
        // No valid version found in the network configuration sent by the client
        _ => {
            return Err(DhcpError::new(
                ErrorKind::InvalidArgument,
                String::from("Must select a valid IP protocol 0=v4, 1=v6"),
            ));
        }
    }
}

#[tokio::main]
#[allow(unused)]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse().unwrap();
    let cache = match LeaseCache::new(None) {
        Ok(c) => Arc::new(Mutex::new(c)),
        Err(e) => {
            log::warn!("IO error with the cache fs");
            return Ok(());
        }
    };
    let netavark_proxy_service = NetavarkProxyService(cache);
    Server::builder()
        .add_service(NetavarkProxyServer::new(netavark_proxy_service))
        .serve(addr)
        .await?;
    Ok(())
}
