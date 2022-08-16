use std::collections::HashMap;
use std::fs::{OpenOptions};
use std::io;
use std::path::Path;
use std::sync::Mutex;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

pub mod g_rpc {
    tonic::include_proto!("netavark_proxy");
}

use g_rpc::{DhcpV4Lease as NetavarkLease, Ipv4Addr as NetavarkIpv4Addr, MacAddress as NetavarkMacAddress};

const XDGRUNTIME: &str = "/run/user/UID/nv-dhcp";

pub enum LeaseCacheEvent {
    NewLease,
    UpdateLease,
    RemoveLease,
    TearDown,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct MacAddress {
    pub bytes: [u8; 6],
}
impl Default for MacAddress {
    fn default() -> Self {
        MacAddress {
            bytes: [0,0,0,0,0,0]
        }
    }
}
// Let the lease cache store multiple Leases for multi-homing in the future
// TODO - should this hold a pointer to the lease file?
#[derive(Debug)]
pub struct LeaseCache(Mutex<HashMap<MacAddress, Vec<NetavarkLease>>>);

impl LeaseCache {
    /// Create a new Lease Cache
    pub fn new() -> Result<LeaseCache, io::Error> {
        let path = Path::new(XDGRUNTIME);
        OpenOptions::new()
            .create(true)
            .open(path)?;
        Ok(LeaseCache(Mutex::new(HashMap::<MacAddress, Vec<NetavarkLease>>::new())))
    }

    pub fn on_event(&self, event: LeaseCacheEvent, mac_addr: MacAddress, lease: NetavarkLease) -> Result<(), std::io::Error> {
        let path = Path::new(XDGRUNTIME);
        let mut cache = self.0.lock().unwrap();
        match event {
            LeaseCacheEvent::NewLease => {
                Ok(())
            }

            LeaseCacheEvent::UpdateLease => {
                Ok(())
            }
            LeaseCacheEvent::RemoveLease => {
                Ok(())
            }
            LeaseCacheEvent::TearDown => {
                self.0.lock().unwrap().clear();
                let path = Path::new(XDGRUNTIME);
                return match OpenOptions::new().truncate(true).open(path) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e)
                };
            }
        }
    }


    fn update_file(&self) -> () {
        let path = Path::new(XDGRUNTIME);
        OpenOptions::new().append(true).open(&path).unwrap();
    }
}

impl Default for LeaseCache {
    fn default() -> Self { LeaseCache(Mutex::new(HashMap::<MacAddress, Vec<NetavarkLease>>::new())) }
}
impl Serialize for NetavarkIpv4Addr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("IpvAddr", 1)?;
        state.serialize_field("octets", &self.v4)?;
        state.end()
    }
}
impl Serialize for NetavarkMacAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("NetavarkMacAddress", 1)?;
        state.serialize_field("bytes", &self.bytes)?;
        state.end()
    }
}
impl Serialize for NetavarkLease {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("NetavarkLease", 16)?;
        state.serialize_field("siaddr", &self.siaddr)?;
        state.serialize_field("yiaddr", &self.siaddr)?;
        state.serialize_field("t1", &self.t1)?;
        state.serialize_field("t2", &self.t2)?;
        state.serialize_field("lease_time", &self.lease_time)?;
        state.serialize_field("srv_id", &self.srv_id)?;
        state.serialize_field("subnet_mask", &self.subnet_mask)?;
        state.serialize_field("broadcast_addr", &self.broadcast_addr)?;
        state.serialize_field("dns_servers", &self.dns_servers)?;
        state.serialize_field("gateways", &self.gateways)?;
        state.serialize_field("ntp_servers", &self.ntp_servers)?;
        state.serialize_field("mtu" ,&self.mtu)?;
        state.serialize_field("host_name" ,&self.host_name)?;
        state.serialize_field("domain_name" ,&self.domain_name)?;
        state.serialize_field("mac_address", &self.mac_addr)?;
        state.serialize_field("version", &self.version)?;
        state.end()
    }
}
