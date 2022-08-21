use std::fs::OpenOptions;
use std::io::{Error, Read};
//    ** This client represents the netavark binary which will establish a connection **
use netavark_proxy::g_rpc::netavark_proxy_client::NetavarkProxyClient;
use netavark_proxy::g_rpc::{MacAddress, NetworkConfig, Teardown};
use tonic::Request;
pub const XDGRUNTIME: &str = "/run/user/1000/nv-leases";
#[tokio::main]
#[allow(unused)]
// This client assumes you use the default lease directory
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = NetavarkProxyClient::connect("http://[::1]:10000").await?;
    print_default_lease_file();
    let mac_addr = Some(MacAddress::new("00:00:5e:00:53:af".to_string()));
    let mac_addr1 = Some(MacAddress::new("10:a3:5e:1d:53:af".to_string()));

    println!("Grabbing a lease");
    let lease = client
        .get_lease(Request::new(NetworkConfig {
            iface: String::from("wlp5s0"),
            mac_addr,
            domain_name: "Jack Baude".to_string(),
            host_name: "Jacks Machine".to_string(),
            version: 0,
        }))
        .await?;
    println!("Response {:?}", lease.into_inner());
    print_default_lease_file();

    println!("Grabbing a second ease");
    let lease_1 = client
        .get_lease(Request::new(NetworkConfig {
            iface: String::from("wlp5s0"),
            mac_addr: mac_addr1,
            domain_name: "Jack Baude 2".to_string(),
            host_name: "Jacks 2nd Machine".to_string(),
            version: 0,
        }))
        .await?;
    println!("Response {:?}", lease_1.into_inner());
    print_default_lease_file();

    println!("Removing the second lease");
    let remove = client
        .remove_lease(Request::new(MacAddress {
            addr: "10:a3:5e:1d:53:af".to_string(),
        }))
        .await?;
    println!("Remove lease_1: {:?}", remove.into_inner());
    print_default_lease_file();

    println!("Tearing down");
    let tear_down = client.tear_down(Teardown {}).await?;
    println!("teardown: {:?}", tear_down.into_inner());
    print_default_lease_file();

    Ok(())
}

fn print_default_lease_file() -> Result<(), Error> {
    let mut default_lease_file = match OpenOptions::new().read(true).open(XDGRUNTIME) {
        Ok(leases) => leases,
        Err(e) => {
            log::warn!("Could not find the lease file");
            return Err(e);
        }
    };
    let mut contents = String::new();
    default_lease_file.read_to_string(&mut contents)?;
    println!("Lease file cache: {}\n", contents);
    Ok(())
}
