use crate::g_rpc::netavark_proxy_client::NetavarkProxyClient;
use crate::g_rpc::{Lease, NetworkConfig};
use clap::Parser;
use log::debug;
use tonic::transport::Channel;
use tonic::{Request, Status};

#[derive(Parser, Debug)]
pub struct Teardown {
    /// Network namespace path
    #[clap(forbid_empty_values = true, required = true)]
    config: NetworkConfig,
}

impl Teardown {
    pub fn new(config: NetworkConfig) -> Self {
        Self { config }
    }

    pub async fn exec(&self, mut conn: NetavarkProxyClient<Channel>) -> Result<Lease, Status> {
        debug!("Entering teardown");
        let response = conn.setup(Request::new(self.config.clone())).await?;
        Ok(response.into_inner())
    }
}
