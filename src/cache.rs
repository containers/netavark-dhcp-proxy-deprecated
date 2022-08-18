use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;

pub mod g_rpc {
    include!("grpc/netavark_proxy.rs");
}

use g_rpc::{
            MacAddress, Lease as NetavarkLease};

const XDGRUNTIME: &str = "/run/user/UID/nv-dhcp";


// Let the lease cache store multiple Leases for multi-homing in the future
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
    pub fn add_lease(&self, mac_addr: MacAddress, lease: NetavarkLease) -> Result<(), io::Error> {
        let path = Path::new(XDGRUNTIME);
        let mut cache = self.0.lock().unwrap();
        // write to the memory cache
        cache.insert(mac_addr, vec![lease.clone()]);
        // write to fs cache
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(&file);
        let mut contents: Vec<NetavarkLease> = serde_json::from_reader(reader)?;
        contents.push(lease.clone());
        serde_json::to_writer_pretty(&file, &contents)?;
        Ok(())
    }

    pub fn update_lease(&self, mac_addr: MacAddress, lease: NetavarkLease) -> Result<(), io::Error> {
        let path = Path::new(XDGRUNTIME);
        let mut cache = self.0.lock().unwrap();
        // write to the memory cache
        cache.insert(mac_addr, vec![lease.clone()]);
        // write to fs cache
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(&file);
        let mut contents: Vec<NetavarkLease> = serde_json::from_reader(reader)?;
        contents.push(lease.clone());
        serde_json::to_writer_pretty(&file, &contents)?;
        Ok(())
    }
    pub fn remove_lease(&self, mac_addr: MacAddress) -> Result<(), io::Error> {
        let mut cache = self.0.lock().unwrap();
        cache.remove(&mac_addr);
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), io::Error> {
        self.0.lock().unwrap().clear();
        let path = Path::new(XDGRUNTIME);
        return match OpenOptions::new().truncate(true).open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        };
    }

}

impl Default for LeaseCache {
    fn default() -> Self { LeaseCache(Mutex::new(HashMap::<MacAddress, Vec<NetavarkLease>>::new())) }
}



