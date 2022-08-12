use std::net::Ipv4Addr;
use tokio;
use tonic::{Response, Status, transport::Server, Code::Internal};
use mozim::{DhcpError, DhcpV4Client, DhcpV4Config, DhcpV4Lease as MozimLease};
// See target/debug/build/netavark_proxy-xxxxxxxx/out/netavark_proxy.rs
pub mod g_rpc {
    tonic::include_proto!("netavark_proxy");
}
use g_rpc::{DhcpV4Lease as NetavarkLease, NetworkConfig, Ipv4Addr as NetavarkIpv4Addr};
use g_rpc::netavark_proxy_server::{NetavarkProxy, NetavarkProxyServer};

pub mod dhcp_utils;
use dhcp_utils::{purge_dhcp_ip_route, apply_dhcp_ip_route};

const POLL_WAIT_TIME: isize = 5;

#[derive(Debug, Default)]
struct NetavarkProxyService;

#[tonic::async_trait]
impl NetavarkProxy for NetavarkProxyService {
    async fn get_lease(
        &self,
        request: tonic::Request<NetworkConfig>,
    ) -> Result<tonic::Response<NetavarkLease>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());
        //Spawn a new thread to avoid tokio runtime issues
        std::thread::spawn(move || {
            let iface: String = request.into_inner().iface;
            purge_dhcp_ip_route(&iface);
            let config = match DhcpV4Config::new(&iface) {
                Ok(config) => config,
                Err(err) => return Err(tonic::Status::new(Internal, err.to_string()))
            };
            let mut client = match DhcpV4Client::init(config, None) {
                Ok(client) => client,
                Err(err) => return Err(tonic::Status::new(Internal, err.to_string()))
            };

            let mut lease: Result<Option<NetavarkLease>, DhcpError> = Ok(None);
            while let Ok(None) = lease {
                let events = client.poll(POLL_WAIT_TIME).unwrap();
                for event in events {
                    match client.process(event) {
                        Ok(Some(l)) => {
                            apply_dhcp_ip_route(&iface, &l);
                            lease = Ok(Some(<NetavarkLease as From<MozimLease>>::from(l)));
                            break;
                        },
                        Ok(None) => {
                            lease = Ok(None);
                        },
                        Err(err) => {
                            purge_dhcp_ip_route(&iface);
                            lease = Err(err);
                        }
                    };
                }
            }
            return match lease {
                Ok(Some(l)) => Ok(Response::new(l)),
                Ok(None) => Ok(Response::new(<NetavarkLease as From<MozimLease>>::from(MozimLease::default()))),
                Err(err) => Err(Status::new(Internal, err.to_string()))
            }
        }).join().expect("Error joining thread")
    }
}

#[tokio::main]
#[allow(unused)]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse().unwrap();
    let netavark_proxy_service = NetavarkProxyService::default();
    Server::builder()
        .add_service(NetavarkProxyServer::new(netavark_proxy_service))
        .serve(addr)
        .await?;
    Ok(())
}



impl From<MozimLease> for NetavarkLease {
    fn from(l: MozimLease) -> NetavarkLease {
        NetavarkLease {
            siaddr: Some(NetavarkIpv4Addr::from(l.siaddr)),
            yiaddr: Some(NetavarkIpv4Addr::from(l.yiaddr)),
            t1: l.t1,
            t2: l.t2,
            lease_time: l.lease_time,
            srv_id: Some(NetavarkIpv4Addr::from(l.srv_id)),
            subnet_mask: Some(NetavarkIpv4Addr::from(l.subnet_mask)),
            broadcast_addr: Some(NetavarkIpv4Addr::from(l.broadcast_addr)),
            dns_servers: handle_ip_vectors(l.dns_srvs),
            gateways: handle_ip_vectors(l.gateways),
            ntp_servers: handle_ip_vectors(l.ntp_srvs),
            mtu: None,
            host_name: l.host_name,
            domain_name: l.domain_name,
        }
    }
}


fn handle_ip_vectors(ip: Option<Vec<Ipv4Addr>>) -> Vec<NetavarkIpv4Addr> {
    let mut ips: Vec<g_rpc::Ipv4Addr> = Vec::new();
    if let Some(j) = ip {
        for ip in j {
            ips.push(g_rpc::Ipv4Addr::from(ip));
        }
    }
    ips
}

impl From<Ipv4Addr> for NetavarkIpv4Addr {
    fn from(ip: Ipv4Addr) -> NetavarkIpv4Addr {
        return g_rpc::Ipv4Addr {
            v4: Vec::from(ip.octets())
        };
    }
}

impl From<Option<Ipv4Addr>> for NetavarkIpv4Addr {
    fn from(ip: Option<Ipv4Addr>) -> Self {
        if let Some(addr) = ip {
            return NetavarkIpv4Addr {
                v4: Vec::from(addr.octets())
            };
        }
        return NetavarkIpv4Addr {
            v4: Vec::from([0, 0, 0, 0])
        };
    }
}
