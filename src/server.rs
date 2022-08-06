use std::net::SocketAddr;
#[allow(unused_imports)]
use tonic::{transport::Server, Request, Response, Status};
use tonic::{include_proto, IntoRequest};
use tokio;
use mozim;
use mozim::DhcpV4Client;

// See target/debug/build/netavark_proxy-xxxxxxxx/out/netavark_proxy.rs
pub mod netavark_proxy {
    tonic::include_proto!("netavark_proxy");
}
use netavark_proxy::{DhcpReply, NetworkConfig};
use netavark_proxy::netavark_proxy_server::{NetavarkProxy, NetavarkProxyServer};
#[derive(Debug, Default)]
struct NetavarkProxyService;

#[tonic::async_trait]
impl NetavarkProxy for NetavarkProxyService {
    async fn get_lease(
        &self,
        request: tonic::Request<NetworkConfig>
    ) -> Result<tonic::Response<DhcpReply>, tonic::Status> {
        println!("Got a request from {:?} from ", request.remote_addr());
        print!("{:?}", request.message().iface);
        let reply = netavark_proxy::DhcpReply {
            ip: String::from("home")
        };
        Ok(Response::new(reply))
    }

}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse().unwrap();
    let netavark_proxy_service = NetavarkProxyService::default();
    Server::builder()
        .add_service(NetavarkProxyServer::new(netavark_proxy_service))
        .serve(addr)
        .await?;
    Ok(())

}
