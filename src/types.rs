use std::num::ParseIntError;
use std::str::FromStr;

use crate::NetworkConfig;
impl FromStr for NetworkConfig {
    type Err = ParseIntError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        // s is actually a string so if we intend to generate
        // a `NetworkConfig` object from `s` parse `s` and populate
        // ifcace, mac_addr, domain_name, host_name and version
        // instead of default empty values
        Ok(NetworkConfig {
            iface: "".to_string(),
            mac_addr: "".to_string(),
            domain_name: "".to_string(),
            host_name: "".to_string(),
            version: 0,
        })
    }
}
