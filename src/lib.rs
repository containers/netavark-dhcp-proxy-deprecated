use crate::g_rpc::{Lease, NetworkConfig};
use std::error::Error;

pub mod cache;
pub mod dhcp_service;
pub mod ip;
pub mod types;

use crate::g_rpc::netavark_proxy_client::NetavarkProxyClient;
use http::Uri;
use log::debug;
use std::fs::File;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Status};
use tower::service_fn;

// TODO these constant destinations are not final.
// Default UDS path for gRPC to communicate on.
pub const DEFAULT_UDS_PATH: &str = "/run/podman/nv-proxy.sock";
// Default configuration directory.
pub const DEFAULT_CONFIG_DIR: &str = "";
// Default Network configuration path
pub const DEFAULT_NETWORK_CONFIG: &str = "/dev/stdin";
// Default epoll wait time before dhcp socket times out
pub const DEFAULT_TIMEOUT: isize = 8;
#[allow(clippy::unwrap_used)]
pub mod g_rpc {
    include!("../proto-build/netavark_proxy.rs");
    use mozim::DhcpV4Lease as MozimV4Lease;

    impl Lease {
        /// Add mac address to a lease
        pub fn add_mac_address(&mut self, mac_addr: &String) {
            self.mac_address = mac_addr.to_string()
        }
        /// Update the domain name of the lease
        pub fn add_domain_name(&mut self, domain_name: &String) {
            self.domain_name = domain_name.to_string();
        }
    }

    impl From<MozimV4Lease> for Lease {
        fn from(l: MozimV4Lease) -> Lease {
            // Since these fields are optional as per mozim. Match them first and then set them
            let domain_name = match l.domain_name {
                None => String::from(""),
                Some(l) => l,
            };
            let mtu = l.mtu.unwrap_or(0) as u32;

            Lease {
                t1: l.t1,
                t2: l.t2,
                lease_time: l.lease_time,
                mtu,
                domain_name,
                mac_address: "".to_string(),
                siaddr: l.siaddr.to_string(),
                yiaddr: l.yiaddr.to_string(),
                srv_id: l.srv_id.to_string(),
                subnet_mask: l.subnet_mask.to_string(),
                // TODO something is jacked with8 broadcast, moving on
                broadcast_addr: "".to_string(),
                dns_servers: handle_ip_vectors(l.dns_srvs),
                gateways: handle_ip_vectors(l.gateways),
                ntp_servers: handle_ip_vectors(l.ntp_srvs),
                host_name: l.host_name.unwrap_or_else(|| String::from("")),
                is_v6: false,
            }
        }
    }

    fn handle_ip_vectors(ip: Option<Vec<std::net::Ipv4Addr>>) -> Vec<String> {
        let mut ips: Vec<String> = Vec::new();
        if let Some(j) = ip {
            for ip in j {
                ips.push(ip.to_string());
            }
        }
        ips
    }

    impl From<std::net::Ipv4Addr> for Ipv4Addr {
        fn from(ip: std::net::Ipv4Addr) -> Ipv4Addr {
            Ipv4Addr {
                octets: Vec::from(ip.octets()),
            }
        }
    }

    impl From<Option<std::net::Ipv4Addr>> for Ipv4Addr {
        fn from(ip: Option<std::net::Ipv4Addr>) -> Self {
            if let Some(addr) = ip {
                return Ipv4Addr {
                    octets: Vec::from(addr.octets()),
                };
            }
            Ipv4Addr {
                octets: Vec::from([0, 0, 0, 0]),
            }
        }
    }

    #[test]
    fn test_handle_gw() {
        use std::str::FromStr;
        let mut ips: Vec<std::net::Ipv4Addr> = Vec::new();
        for i in 0..5 {
            let ip = format!("10.1.{}.1", i);
            let ipv4 = std::net::Ipv4Addr::from_str(&ip).expect("failed hard");
            ips.push(ipv4);
        }
        let response = handle_ip_vectors(Some(ips));
        // Len of response should be same as ips
        assert_eq!(response.len(), 5);
        assert_eq!(response[0].to_string(), "10.1.0.1");
    }
}

// A collection of functions for client side connections to the proxy server
impl NetworkConfig {
    pub fn load(path: &str) -> Result<NetworkConfig, Box<dyn Error>> {
        let file = std::io::BufReader::new(File::open(path)?);
        Ok(serde_json::from_reader(file)?)
    }

    /// get_client is an internal function to obtain the uds endpoint
    ///
    /// # Arguments
    ///
    /// * `p`: path to uds
    ///
    /// returns: Result<NetavarkProxyClient<Channel>, Status>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    async fn get_client(p: String) -> Result<NetavarkProxyClient<Channel>, Status> {
        // We do not know why the uds connections need to be done like this.  The
        // maintainer suggested it is part of the their API.
        let endpoint = Endpoint::try_from("http://[::1]:10000")
            .map_err(|e| Status::internal(e.to_string()))?;

        let channel = endpoint
            .connect_with_connector(service_fn(move |_: Uri| {
                let pp = p.clone();
                debug!("using uds path: {}", pp);
                UnixStream::connect(pp)
            }))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(NetavarkProxyClient::new(channel))
    }

    /// get_lease is a wrapper function for obtaining a lease
    /// over grpc from the nvproxy-server
    ///
    /// # Arguments
    ///
    /// * `p`: path to uds
    ///
    /// returns: Result<Lease, Status>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub async fn get_lease(self, p: &str) -> Result<Lease, Status> {
        let mut client = NetworkConfig::get_client(p.to_string()).await?;
        let lease = match client.setup(Request::new(self)).await {
            Ok(l) => l.into_inner(),
            Err(s) => return Err(s),
        };
        Ok(lease)
    }

    /// drop_lease is a wrapper function to release the current
    /// DHCP lease via the nvproxy
    ///
    ///
    /// # Arguments
    ///
    /// * `p`:  path to udsz
    ///
    /// returns: Result<Lease, Status>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub async fn drop_lease(self, p: &str) -> Result<Lease, Status> {
        let mut client = NetworkConfig::get_client(p.to_string()).await?;
        client
            .teardown(Request::new(self))
            .await
            .map(|l| l.into_inner())
    }
}
