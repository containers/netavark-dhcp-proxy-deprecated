pub mod cache;
pub mod g_rpc {
    include!("../proto-build/netavark_proxy.rs");
    use mozim::DhcpV4Lease as MozimV4Lease;
    impl From<MozimV4Lease> for Lease {
        fn from(l: MozimV4Lease) -> Lease {
            let domain_name = match l.domain_name{
                None => String::from(""),
                Some(l) => l
            };
            Lease {
                ip_response: Some(IpResponse {
                    t1: l.t1,
                    t2: l.t2,
                    lease_time: l.lease_time,
                    mtu: 0,
                    domain_name,
                    mac_addr: None,
                }),
                v4: Some(DhcpV4Lease {
                    siaddr: Some(Ipv4Addr::from(l.siaddr)),
                    yiaddr: Some(Ipv4Addr::from(l.yiaddr)),
                    srv_id: Some(Ipv4Addr::from(l.srv_id)),
                    subnet_mask: Some(Ipv4Addr::from(l.subnet_mask)),
                    broadcast_addr: Some(Ipv4Addr::from(l.broadcast_addr)),
                    dns_servers: handle_ip_vectors(l.dns_srvs),
                    gateways: handle_ip_vectors(l.gateways),
                    ntp_servers: handle_ip_vectors(l.ntp_srvs),
                    host_name: l.host_name.unwrap_or(String::from("")),
                }),
                v6: None,
            }
        }
    }

    fn handle_ip_vectors(ip: Option<Vec<std::net::Ipv4Addr>>) -> Vec<Ipv4Addr> {
        let mut ips: Vec<Ipv4Addr> = Vec::new();
        if let Some(j) = ip {
            for ip in j {
                ips.push(Ipv4Addr::from(ip));
            }
        }
        ips
    }

    impl From<std::net::Ipv4Addr> for Ipv4Addr {
        fn from(ip: std::net::Ipv4Addr) -> Ipv4Addr {
            return Ipv4Addr {
                octets: Vec::from(ip.octets())
            };
        }
    }

    impl From<Option<std::net::Ipv4Addr>> for Ipv4Addr {
        fn from(ip: Option<std::net::Ipv4Addr>) -> Self {
            if let Some(addr) = ip {
                return Ipv4Addr {
                    octets: Vec::from(addr.octets())
                };
            }
            return Ipv4Addr {
                octets: Vec::from([0, 0, 0, 0])
            };
        }
    }

    impl MacAddress {
        /// Create a new instance of a mac address
        pub fn new(addr: String) -> Self {
            MacAddress { addr }
        }
        /// Validate the mac address by decoding it then encoding and checking its the same as the
        /// original
        pub fn validate(&self) -> bool {
            let bytes = match self.decode_address_from_hex() {
                Ok(bytes) => bytes,
                Err(e) => {
                    log::debug!("{}", e.to_string());
                    return false;
                }
            };
            if MacAddress::encode_address_to_hex(bytes) == self.addr {
                return true;
            }
            false
        }

        fn decode_address_from_hex(&self) -> Result<Vec<u8>, std::io::Error> {
            let bytes: Result<Vec<u8>, _> = self
                .addr
                .split(|c| c == ':' || c == '-')
                .into_iter()
                .map(|b| u8::from_str_radix(b, 16))
                .collect();

            let result = match bytes {
                Ok(bytes) => {
                    if bytes.len() != 6 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("invalid mac length for address: {}", self.addr),
                        ));
                    }
                    bytes
                }
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("unable to parse mac address {}: {}", self.addr, e),
                    ));
                }
            };

            Ok(result)
        }

        fn encode_address_to_hex(bytes: Vec<u8>) -> String {
            let address: String = bytes
                .iter()
                .map(|x| format!("{:02x}", x))
                .collect::<Vec<String>>()
                .join(":");

            address
        }
    }
}
