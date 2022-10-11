use clap::Parser;
use log::debug;
use netavark_proxy::g_rpc::netavark_proxy_client::NetavarkProxyClient;
use netavark_proxy::g_rpc::{Lease, NetworkConfig};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

#[derive(Parser, Debug)]
pub struct Setup {
    /// Network namespace path
    #[clap(forbid_empty_values = false, required = false)]
    config: NetworkConfig,
}

impl Setup {
    pub fn new(config: NetworkConfig) -> Self {
        Self { config }
    }

    pub async fn exec(
        &self,
        mut conn: NetavarkProxyClient<Channel>,
    ) -> Result<Response<Lease>, Status> {
        debug!("{:?}", "Setting up...");
        debug!(
            "input: {:#?}",
            serde_json::to_string_pretty(&self.config.clone())
        );
        conn.setup(Request::new(self.config.clone())).await
    }
}
