#![cfg_attr(not(unix), allow(unused_imports))]
use netavark_proxy::cache::LeaseCache;
use netavark_proxy::g_rpc::netavark_proxy_server::{NetavarkProxy, NetavarkProxyServer};
use netavark_proxy::g_rpc::{Empty, Lease as NetavarkLease, NetworkConfig, OperationResponse};
use netavark_proxy::{DEFAULT_CONFIG_DIR, DEFAULT_TIMEOUT, DEFAULT_UDS_PATH};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;

use clap::Parser;
use log::{debug, warn};
use netavark_proxy::dhcp_service::DhcpService;
use tonic::{transport::Server, Code::Internal, Request, Response, Status};

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
struct NetavarkProxyService(Arc<Mutex<LeaseCache>>, isize);

// gRPC request and response methods
#[tonic::async_trait]
impl NetavarkProxy for NetavarkProxyService {
    /// gRPC connection to get a lease
    async fn setup(
        &self,
        request: Request<NetworkConfig>,
    ) -> Result<Response<NetavarkLease>, Status> {
        debug!("Request from client {:?}", request.remote_addr());

        let cache = self.0.clone();
        let timeout = self.1.clone();
        //Spawn a new thread to avoid tokio runtime issues
        std::thread::spawn(move || {
            // Set up some common values
            let network_config = request.into_inner();
            let mac_addr = network_config.mac_addr.clone();
            // create a dhcp service to get a lease.
            let lease = DhcpService::new(network_config, timeout)?.get_lease()?;
            // Try and add the lease information to the cache
            if let Err(e) = cache
                .lock()
                .expect("Could not unlock cache. A thread was poisoned")
                .add_lease(&mac_addr, &lease)
            {
                return Err(Status::new(
                    Internal,
                    format!("Error caching the lease: {}", e),
                ));
            }
            Ok(Response::new(lease))
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

        self.0
            .clone()
            .lock()
            .expect("Could not unlock cache. A thread was poisoned")
            .remove_lease(&nc.mac_addr)?;
        Ok(Response::new(empty_lease))
    }

    /// On teardown of the proxy the cache will be cleared gracefully.
    async fn clean(&self, request: Request<Empty>) -> Result<Response<OperationResponse>, Status> {
        log::debug!("Request from client: {:?}", request.remote_addr());
        self.0
            .clone()
            .lock()
            .expect("Could not unlock cache. A thread was poisoned")
            .teardown()?;
        Ok(Response::new(OperationResponse { success: true }))
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
    /// optional time in seconds to time out after looking for a lease
    #[clap(short, long)]
    timeout: Option<isize>,
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
    // timeout time if no leases are found
    let timeout = opts.timeout.unwrap_or(DEFAULT_TIMEOUT);

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
    // let dhcp_service = DhcpService::new()
    let netavark_proxy_service = NetavarkProxyService(cache, timeout);
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
