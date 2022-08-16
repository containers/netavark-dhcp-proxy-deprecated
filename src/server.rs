use std::io::Read;
// This is a binary file for running the netavark DHCP proxy server.
use std::net::Ipv4Addr;
use tokio;
use tonic::{Response, Status, transport::Server, Code::Internal, Request};
use mozim::{DhcpError, DhcpV4Client, DhcpV4Config, DhcpV4Lease as MozimLease};
use serde::ser::{Serialize, Serializer, SerializeStruct};

// See target/debug/build/netavark_proxy-xxxxxxxx/out/netavark_proxy.rs
pub mod g_rpc {
    tonic::include_proto!("netavark_proxy");
}

use g_rpc::{DhcpV4Lease as NetavarkLease, NetworkConfig, Ipv4Addr as NetavarkIpv4Addr, OperationResponse, MacAddress as NetavarkMacAddress};
use g_rpc::netavark_proxy_server::{NetavarkProxy, NetavarkProxyServer};

use netavark_proxy::cache::{LeaseCache, MacAddress};

const POLL_WAIT_TIME: isize = 5;

#[derive(Debug)]
struct NetavarkProxyService(LeaseCache);

#[tonic::async_trait]
impl NetavarkProxy for NetavarkProxyService {
    async fn get_lease(
        &self,
        request: Request<NetworkConfig>,
    ) -> Result<Response<NetavarkLease>, Status> {
        log::debug!("Request from client: {:?}", request.remote_addr());
        //Spawn a new thread to avoid tokio runtime issues
        std::thread::spawn(move || {
            let iface: String = request.into_inner().iface;
            let config = match DhcpV4Config::new(&iface) {
                Ok(config) => config,
                Err(err) => return Err(Status::new(Internal, err.to_string()))
            };
            let mut client = match DhcpV4Client::init(config, None) {
                Ok(client) => client,
                Err(err) => return Err(Status::new(Internal, err.to_string()))
            };
            // assume that there is no lease and no error finding one
            let mut lease: Result<Option<NetavarkLease>, DhcpError> = Ok(None);
            // TODO check if this will timeout so not to hang
            while let Ok(None) = lease {
                let events = client.poll(POLL_WAIT_TIME).unwrap();
                for event in events {
                    match client.process(event) {
                        Ok(Some(l)) => {
                            lease = Ok(Some(<NetavarkLease as From<MozimLease>>::from(l)));
                        }
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
                Ok(None) => Ok(Response::new(<NetavarkLease as From<MozimLease>>::from(MozimLease::default()))),
                Err(err) => Err(Status::new(Internal, err.to_string()))
            };
        }).join().expect("Error joining thread")
    }
    async fn tear_down(
        &self,
        request: Request<NetworkConfig>,
    ) -> Result<Response<OperationResponse>, Status> {
        log::debug!("Request from client: {:?}", request.remote_addr());
        Ok(Response::new(OperationResponse {
            success: true
        }))
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
            mac_addr: Some(NetavarkMacAddress::from(MacAddress::default())),
            version: 1
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
impl From<MacAddress> for NetavarkMacAddress {
    fn from(m: MacAddress) -> Self {
        NetavarkMacAddress {
            bytes: m.bytes.to_vec()
        }
    }
}

