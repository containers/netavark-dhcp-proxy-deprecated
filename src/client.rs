//    ** This client represents the netavark binary which will establish a connection **
use netavark_proxy::g_rpc::netavark_proxy_client::NetavarkProxyClient;
use netavark_proxy::g_rpc::{MacAddress, NetworkConfig};
use tonic::Request;
#[tokio::main]
#[allow(unused)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = NetavarkProxyClient::connect("http://[::1]:10000").await?;
    let response = client
        .get_lease(Request::new(NetworkConfig {
            iface: String::from("wlp5s0"),
            mac_addr: Some(MacAddress::new("00:00:5e:00:53:af".to_string())),
            version: 0,
        }))
        .await?;
    // This complies fine but inspection thinks `request.into_inner()` does not implement debug
    println!("Response {:#?}", response.into_inner());
    Ok(())
}
