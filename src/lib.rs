use crate::g_rpc::NetworkConfig;
use std::error::Error;

pub mod cache;
pub mod commands;
pub mod dhcp_service;

use std::fs::File;
// TODO these constant destinations are not final.
// Default UDS path for gRPC to communicate on.
pub const DEFAULT_UDS_PATH: &str = "/run/nv-dhcp/nv-dhcp-uds.sock";
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

    impl DhcpV4Lease {
        /// update the host name. This is only applicable to dhcpv4 leases
        pub fn add_host_name(&mut self, host_name: String) {
            self.host_name = host_name;
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
                v4: Some(DhcpV4Lease {
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
                }),
                v6: None,
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
}

impl NetworkConfig {
    pub fn load(path: &str) -> Result<NetworkConfig, Box<dyn Error>> {
        let file = std::io::BufReader::new(File::open(path)?);
        Ok(serde_json::from_reader(file)?)
    }
}
