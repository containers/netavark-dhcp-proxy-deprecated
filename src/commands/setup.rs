use crate::g_rpc::netavark_proxy_client::NetavarkProxyClient;
use crate::g_rpc::{Lease, NetworkConfig};
use clap::Parser;
use log::debug;
use std::num::ParseIntError;
use std::str::FromStr;
use tonic::transport::Channel;
use tonic::{Request, Status};

#[derive(Parser, Debug)]
pub struct Setup {
    /// Network namespace path
    #[clap(forbid_empty_values = true, required = true)]
    config: NetworkConfig,
}

//  maybe move this to types.rs
impl FromStr for NetworkConfig {
    type Err = ParseIntError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        // s is actually a string so if we intend to generate
        // a `NetworkConfig` object from `s` parse `s` and populate
        // ifcace, mac_addr, domain_name, host_name and version
        // instead of default empty values
        Ok(NetworkConfig {
            iface: "".to_string(),
            mac_addr: None,
            domain_name: "".to_string(),
            host_name: "".to_string(),
            version: 0,
        })
    }
}

impl Setup {
    pub fn new(config: NetworkConfig) -> Self {
        Self { config }
    }

    pub async fn exec(&self, mut conn: NetavarkProxyClient<Channel>) -> Result<Lease, Status> {
        println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        debug!("{:?}", "Setting up...");
        println!(
            "--> {:#?}",
            serde_json::to_string_pretty(&self.config.clone())
        );
        let response = conn.setup(Request::new(self.config.clone())).await?;
        Ok(response.into_inner())
    }
}
