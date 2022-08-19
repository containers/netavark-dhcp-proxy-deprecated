use mozim::{DhcpError, DhcpV4Client, DhcpV4Config, DhcpV4Lease as MozimV4Lease, ErrorKind};
use netavark_proxy::cache::LeaseCache;
use netavark_proxy::g_rpc::netavark_proxy_server::{NetavarkProxy, NetavarkProxyServer};
use netavark_proxy::g_rpc::{Lease as NetavarkLease, NetworkConfig, OperationResponse};
use tokio;
use tonic::Code::InvalidArgument;
use tonic::{transport::Server, Code::Internal, Request, Response, Status};

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
            // Set up some common values
            let network_config: NetworkConfig = request.into_inner();
            let _mac_addr = match &network_config.mac_addr {
                None => return Err(Status::new(InvalidArgument, "No mac address supplied")),
                Some(m) => m,
            };

            // DHCP client will be in charge of making the DORA requests to the DHCP server
            let mut client = match get_client(&network_config) {
                Ok(c) => c,
                Err(e) => return Err(Status::new(Internal, e.to_string())),
            };

            // Begin processing the DHCP events to grab a lease
            let mut lease: Result<Option<NetavarkLease>, DhcpError> = Ok(None);
            // While a lease has not been found start processing the DHCP events
            while let Ok(None) = lease {
                let events = client.poll(POLL_WAIT_TIME).unwrap();
                for event in events {
                    match client.process(event) {
                        // Lease successfully found
                        Ok(Some(new_lease)) => {
                            // TODO call the cache add lease method on the cache.
                            lease =
                                Ok(Some(<NetavarkLease as From<MozimV4Lease>>::from(new_lease)));
                        }
                        Err(err) => {
                            lease = Err(err);
                        }
                        Ok(None) => {}
                    };
                }
            }
            return match lease {
                Ok(Some(l)) => Ok(Response::new(l)),
                Err(err) => Err(Status::new(Internal, err.to_string())),
                _ => Err(Status::new(Internal, "No lease was found")),
            };
        })
        .join()
        .expect("Error joining thread")
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
        Ok(Response::new(OperationResponse { success: true }))
    }
}

// Get a DHCP client based on the ip version
fn get_client(network_config: &NetworkConfig) -> Result<DhcpV4Client, DhcpError> {
    let iface: &String = &network_config.iface;
    // Proto enumerations define a const int to each type (e.g. V4 = 0, V6 = 1).
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
        Ok(c) => c,
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
