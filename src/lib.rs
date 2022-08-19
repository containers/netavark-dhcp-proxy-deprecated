pub mod cache;
pub mod g_rpc {
    include!("grpc/netavark_proxy.rs");
    use mozim::DhcpV4Lease as MozimV4Lease;
    impl From<MozimV4Lease> for Lease {
        fn from(l: MozimV4Lease) -> Lease {
            Lease {
                ip_response: Some(IpResponse {
                    t1: l.t1,
                    t2: l.t2,
                    lease_time: l.lease_time,
                    mtu: 0,
                    domain_name: l.domain_name,
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
        pub fn new(bytes: Vec<u8>) -> Self {
            MacAddress {
                bytes
            }
        }
    }
}