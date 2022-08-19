use tokio;
use tonic::{Response, Status, transport::Server, Code::Internal, Request};
use mozim::{DhcpError, DhcpV4Client, DhcpV4Config, DhcpV4Lease as MozimV4Lease};

pub mod g_rpc {
    include!("grpc/netavark_proxy.rs");
}

use g_rpc::{
    Lease as NetavarkLease, DhcpV4Lease as NetavarkV4Lease, NetworkConfig, IpResponse,
    Ipv4Addr as NetavarkIpv4Addr, MacAddress, OperationResponse
};
use g_rpc::netavark_proxy_server::{NetavarkProxy, NetavarkProxyServer};

use netavark_proxy::cache::{LeaseCache};

const POLL_WAIT_TIME: isize = 5;

#[derive(Debug)]
struct NetavarkProxyService(LeaseCache);

// gRPC request and response methods
#[tonic::async_trait]
impl NetavarkProxy for NetavarkProxyService {
    async fn get_lease(
        &self,
        request: Request<NetworkConfig>,
    ) -> Result<Response<NetavarkLease>, Status> {
        log::debug!("Request from client: {:?}", request.remote_addr());

        //Spawn a new thread to avoid tokio runtime issues
        std::thread::spawn(move || {
            let network_config: NetworkConfig = request.into_inner();
            let mut client = match get_client(network_config) {
                Ok(c) => c,
                Err(e) => return Err(Status::new(Internal, e.to_string()))
            };

            // assume that there is no lease and no error finding one
            let mut lease: Result<Option<NetavarkLease>, DhcpError> = Ok(None);

            while let Ok(None) = lease {
                let events = client.poll(POLL_WAIT_TIME).unwrap();
                for event in events {
                    match client.process(event) {
                        //No DhcpError and a lease is successfully found.
                        Ok(Some(l)) => {
                            lease = Ok(Some(<NetavarkLease as From<MozimV4Lease>>::from(l)));
                        }
                        //No DhcpError but no lease found
                        Ok(None) => {
                            lease = Ok(None);
                        }
                        Err(err) => {
                            lease = Err(err);
                        }
                    };
                }
            }
            return match lease {
                Ok(Some(l)) => Ok(Response::new(l)),
                Ok(None) => Ok(Response::new(<NetavarkLease as From<MozimV4Lease>>::from(MozimV4Lease::default()))),
                Err(err) => Err(Status::new(Internal, err.to_string()))
            };
        }).join().expect("Error joining thread")
    }
    async fn tear_down(
        &self,
        request: Request<NetworkConfig>,
    ) -> Result<Response<OperationResponse>, Status> {
        log::debug!("Request from client: {:?}", request.remote_addr());
        if let Err(e) = self.0.teardown() {
            log::info!("Error tearing down: {}", e);
            return Ok(Response::new(OperationResponse { success: false }));
        }
        Ok(Response::new(OperationResponse {
            success: true
        }))
    }
}

fn get_client(network_config: NetworkConfig) -> Result<DhcpV4Client, DhcpError> {
    let iface: String = network_config.iface;
    match network_config.version {
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
        _ => {
            return Err(DhcpError::new(ErrorKind::InvalidArgument, String::from("Must select a valid IP protocol 0=v4, 1=v6")));
        }
    }
}


#[tokio::main]
#[allow(unused)]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse().unwrap();
    let cache = match LeaseCache::new() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("IO error with the cache fs");
            return Ok(());
        }
    };
    let netavark_proxy_service = NetavarkProxyService(LeaseCache::new().unwrap());
    Server::builder()
        .add_service(NetavarkProxyServer::new(netavark_proxy_service))
        .serve(addr)
        .await?;
    Ok(())
}