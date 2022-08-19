use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;
use crate::g_rpc::{Lease as NetavarkLease, MacAddress};

/// Path to the lease json cache
pub const XDGRUNTIME: &str = "/run/user/1000/nv-leases";

/// Let the lease cache store multiple Leases for multi-homing in the future
#[derive(Debug)]
pub struct LeaseCache(Mutex<HashMap<MacAddress, Vec<NetavarkLease>>>);

impl LeaseCache {
    /// Create a new Lease Cache instance
    pub fn new() -> Result<LeaseCache, io::Error> {
        let path = Path::new(XDGRUNTIME);
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)?;
        Ok(LeaseCache(Mutex::new(HashMap::<MacAddress, Vec<NetavarkLease>>::new())))
    }

    pub fn add_lease(&self, mac_addr: &MacAddress, lease: &NetavarkLease) -> Result<(), io::Error> {
        let path = Path::new(XDGRUNTIME);
        let mut cache = self.0.lock().unwrap();
        // write to the memory cache
        cache.insert(mac_addr.clone(), vec![lease.clone()]);
        // write to fs cache
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(&file);
        let mut contents: Vec<NetavarkLease> = serde_json::from_reader(reader)?;
        contents.push(lease.clone());
        serde_json::to_writer_pretty(&file, &contents)?;
        Ok(())
    }

    /// TODO - when the information on a lease has changed rewrite the new lease to the mac address
    /// on the cache
    ///
    /// # Arguments
    ///
    /// * `mac_addr`: Mac address of the container
    /// * `lease`: lease to add to the cache
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

    /// TODO - When a container goes down remove the lease information from cache
    /// on the cache
    ///
    /// # Arguments
    ///
    /// * `mac_addr`: Mac address of the container
    pub fn remove_lease(&self, mac_addr: MacAddress) -> Result<(), io::Error> {
        let mut cache = self.0.lock().unwrap();
        cache.remove(&mac_addr);
        Ok(())
    }
    /// On tear down of the proxy remove all values from the memory cache and truncate the file cache
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



