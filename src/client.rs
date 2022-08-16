//    ** This client represents the netavark binary which will establish a connection **
use tonic::{Request};
pub mod g_rpc {
    tonic::include_proto!("netavark_proxy");
}

use g_rpc::netavark_proxy_client::NetavarkProxyClient;
use g_rpc::{NetworkConfig};

#[tokio::main]
#[allow(unused)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = NetavarkProxyClient::connect("http://[::1]:10000").await?;
    let response = client.get_lease(
        Request::new(NetworkConfig {
            iface: String::from("wlp5s0"),
            lease: None,
            version: 1
        }
    ))
        .await?;
    println!("Response {:#?}", response);
    Ok(())
}
