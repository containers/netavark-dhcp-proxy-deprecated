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
        //V6
        1 => {
            unimplemented!();
        }
        _ => {
            return Err(DhcpError::new(mozim::ErrorKind::InvalidArgument, String::from("Must select a valid IP protocol 0=v4, 1=v6")))
        }
    }
}
impl NetavarkLease {
    pub fn add_mac_addr(self, mac_addr: MacAddress) {
        if let Some(mut ip_response) = self.ip_response {
            ip_response.mac_addr = Some(mac_addr);
        }
    }
}

#[tokio::main]
#[allow(unused)]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse().unwrap();
    let netavark_proxy_service = NetavarkProxyService(LeaseCache::new().unwrap());
    Server::builder()
        .add_service(NetavarkProxyServer::new(netavark_proxy_service))
        .serve(addr)
        .await?;
    Ok(())
}

impl From<MozimV4Lease> for NetavarkLease {
    fn from(l: MozimV4Lease) -> NetavarkLease {
        NetavarkLease {
            ip_response: Some(IpResponse {
                t1: l.t1,
                t2: l.t2,
                lease_time: l.lease_time,
                mtu: 0,
                domain_name: l.domain_name,
                mac_addr: None
            }),
            v4: Some(NetavarkV4Lease {
                siaddr: Some(NetavarkIpv4Addr::from(l.siaddr)),
                yiaddr: Some(NetavarkIpv4Addr::from(l.yiaddr)),
                srv_id: Some(NetavarkIpv4Addr::from(l.srv_id)),
                subnet_mask: Some(NetavarkIpv4Addr::from(l.subnet_mask)),
                broadcast_addr: Some(NetavarkIpv4Addr::from(l.broadcast_addr)),
                dns_servers: handle_ip_vectors(l.dns_srvs),
                gateways: handle_ip_vectors(l.gateways),
                ntp_servers: handle_ip_vectors(l.ntp_srvs),
                host_name: l.host_name.unwrap_or(String::from("")),
            }),
            v6: None,

        }
    }
}

fn handle_ip_vectors(ip: Option<Vec<std::net::Ipv4Addr>>) -> Vec<NetavarkIpv4Addr> {
    let mut ips: Vec<g_rpc::Ipv4Addr> = Vec::new();
    if let Some(j) = ip {
        for ip in j {
            ips.push(g_rpc::Ipv4Addr::from(ip));
        }
    }
    ips
}

impl From<std::net::Ipv4Addr> for NetavarkIpv4Addr {
    fn from(ip: std::net::Ipv4Addr) -> NetavarkIpv4Addr {
        return NetavarkIpv4Addr {
            octets: Vec::from(ip.octets())
        };
    }
}

impl From<Option<std::net::Ipv4Addr>> for NetavarkIpv4Addr {
    fn from(ip: Option<std::net::Ipv4Addr>) -> Self {
        if let Some(addr) = ip {
            return NetavarkIpv4Addr {
                octets: Vec::from(addr.octets())
            };
        }
        return NetavarkIpv4Addr {
            octets: Vec::from([0, 0, 0, 0])
        };
    }
}
