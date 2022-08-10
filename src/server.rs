use tonic::{transport::Server, Response};
use tonic::{Code};
use tokio;
use mozim::{DhcpV4Client, DhcpV4Config};
use nispor::{
    AddressFamily, IfaceConf, IfaceState, IpAddrConf, IpConf, NetConf,
    NetState, RouteConf, RouteProtocol,
};

use netavark_proxy::{DhcpReply, NetworkConfig};
use netavark_proxy::netavark_proxy_server::{NetavarkProxy, NetavarkProxyServer};

// See target/debug/build/netavark_proxy-xxxxxxxx/out/netavark_proxy.rs
pub mod netavark_proxy {
    tonic::include_proto!("netavark_proxy");
}

#[derive(Debug, Default)]
struct NetavarkProxyService;

#[tonic::async_trait]
impl NetavarkProxy for NetavarkProxyService {
    async fn get_lease(
        &self,
        request: tonic::Request<NetworkConfig>,
    ) -> Result<tonic::Response<DhcpReply>, tonic::Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let mut iface: String = request.into_inner().iface;
        iface = iface.as_str().parse().unwrap();

        let client: Result<DhcpV4Client, tonic::Status> = std::thread::spawn(move || {
            purge_dhcp_ip_route(&iface);
            let config = match DhcpV4Config::new(&iface) {
                Ok(c) => c,
                Err(err) => return Err(
                    tonic::Status::new(
                        Code::Internal,
                        err.to_string(),
                    )
                )
            };
            // client could be instantiated with a known lease (re-lease) instead of None
            let client: DhcpV4Client = match DhcpV4Client::init(config, None) {
                Ok(client) => client,
                Err(err) =>
                    return Err(tonic::Status::new(
                        Code::Internal,
                        err.to_string(),
                    ))
            };
            println!("{:?}", client);
            Ok(client)
        }).join().expect("Thread Panicked");

        match client {
            Ok(client) => println!("{:?}", client),
            Err(e) => return Err(e)
        };
        let reply = netavark_proxy::DhcpReply {
            ip: String::from("home")
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
#[allow(unused)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse().unwrap();
    let netavark_proxy_service = NetavarkProxyService::default();
    Server::builder()
        .add_service(NetavarkProxyServer::new(netavark_proxy_service))
        .serve(addr)
        .await?;
    Ok(())
}

fn new_net_conf_with_ip_conf(iface_name: &str, ip_conf: IpConf) -> NetConf {
    let mut iface_conf = IfaceConf::default();
    iface_conf.name = iface_name.to_string();
    iface_conf.state = IfaceState::Up;
    iface_conf.ipv4 = Some(ip_conf);
    let mut net_conf = NetConf::default();
    net_conf.ifaces = Some(vec![iface_conf]);
    net_conf
}

// Remove all dynamic IP and dhcp routes of specified interface
fn purge_dhcp_ip_route(iface_name: &str) {
    let state = NetState::retrieve().unwrap();
    if let Some(ip_info) =
    state.ifaces.get(iface_name).and_then(|i| i.ipv4.as_ref())
    {
        let mut addrs_to_remove = Vec::new();
        for addr in ip_info
            .addresses
            .as_slice()
            .iter()
            .filter(|a| a.valid_lft != "forever")
        {
            let mut addr_conf = IpAddrConf::default();
            addr_conf.remove = true;
            addr_conf.address = addr.address.clone();
            addr_conf.prefix_len = addr.prefix_len;
            addrs_to_remove.push(addr_conf);
        }
        if !addrs_to_remove.is_empty() {
            let mut ip_conf = IpConf::default();
            ip_conf.addresses = addrs_to_remove;
            new_net_conf_with_ip_conf(iface_name, ip_conf)
                .apply()
                .expect("Likely not ran with root");

        }
    }
    let mut routes_to_remove = Vec::new();
    for rt in state.routes.as_slice().iter().filter(|rt| {
        rt.oif.as_deref() == Some(iface_name)
            && rt.protocol == RouteProtocol::Dhcp
            && rt.address_family == AddressFamily::IPv4
    }) {
        routes_to_remove.push(gen_rt_conf(
            true,
            rt.dst.as_deref().unwrap_or("0.0.0.0/0"),
            iface_name,
            rt.via
                .as_deref()
                .unwrap_or_else(|| rt.gateway.as_deref().unwrap_or("0.0.0.0")),
            None,
        ));
    }
    let mut net_conf = NetConf::default();
    net_conf.routes = Some(routes_to_remove);
    net_conf.apply().expect("Likely not ran with root");
}

fn gen_rt_conf(
    remove: bool,
    dst: &str,
    oif: &str,
    via: &str,
    metric: Option<u32>,
) -> RouteConf {
    let mut rt = RouteConf::default();
    rt.remove = remove;
    rt.dst = dst.to_string();
    rt.oif = Some(oif.to_string());
    rt.via = Some(via.to_string());
    rt.table = Some(254);
    rt.metric = metric;
    rt.protocol = Some(RouteProtocol::Dhcp);
    rt
}
