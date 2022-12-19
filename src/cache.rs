use crate::g_rpc::{Lease as NetavarkLease, Lease};
use log::debug;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;

/// LeaseCache holds the locked memory map of the mac address to a vector of leases - for multi homing
/// It also stores a locked path buffer to change the FS cache
#[derive(Debug)]
pub struct LeaseCache {
    mem: HashMap<String, Vec<NetavarkLease>>,
    path: PathBuf,
}

impl LeaseCache {
    /// Initiate the memory and file system cache. Will create and open the specified directory for
    /// the cache and create an empty memory map
    ///
    /// # Arguments
    /// * `dir`: Optional directory string that allows user to define their own cache directory.
    /// Otherwise the default directory is
    ///
    /// returns: Result<LeaseCache, Error>
    ///
    /// On success a new lease cache instance will be returned. On failure an IO error will
    /// be returned.
    /// This likely means it could not find the file directory
    pub fn new(file_path: PathBuf) -> Result<LeaseCache, io::Error> {
        debug!(
            "lease cache file: {:?}",
            file_path.to_str().unwrap_or_default()
        );

        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&file_path)?;

        Ok(LeaseCache {
            mem: HashMap::new(),
            path: file_path,
        })
    }

    /// Add a new lease to a memory and file system cache
    ///
    /// # Arguments
    ///
    /// * `mac_addr`: Mac address of the container
    /// * `lease`: New lease that should be saved in the cache
    ///
    /// returns: Result<(), Error>
    ///
    /// On success this the method will return Ok. On a failure it will return an IO error due to
    /// not being able to write or read the file system cache
    pub fn add_lease(&mut self, mac_addr: &str, lease: &NetavarkLease) -> Result<(), io::Error> {
        debug!("add lease: {:?}", mac_addr);
        let cache = &mut self.mem;
        // write to the memory cache
        // HashMap::insert uses a owned reference and key, must clone the referenced mac address and lease
        cache.insert(mac_addr.to_string(), vec![lease.clone()]);
        // write updated memory cache to the file system
        self.save_memory_to_fs()
    }

    /// When lease information changes significantly, update the cache to reflect the changes
    ///
    /// # Arguments
    ///
    /// * `mac_addr`: Mac address of the container
    /// * `lease`: Newest lease information
    ///
    /// returns: Result<(), Error>
    ///
    /// On success returns Ok. On failure returns an io error, likely means that the it could not
    /// find the file
    pub fn update_lease(&mut self, mac_addr: &str, lease: NetavarkLease) -> Result<(), io::Error> {
        let cache = &mut self.mem;
        // write to the memory cache
        cache.insert(mac_addr.to_string(), vec![lease]);
        // write updated memory cache to the file system
        self.save_memory_to_fs()
    }

    /// When a singular container is taken down. Remove that lease from the cache memory and fs
    ///
    /// # Arguments
    ///
    /// * `mac_addr`: Mac address of the container
    pub fn remove_lease(&mut self, mac_addr: &str) -> Result<Lease, io::Error> {
        debug!("remove lease: {:?}", mac_addr);
        let mem = &mut self.mem;
        // the remove function uses a reference key, so we borrow and dereference the MadAddress
        // if !mem.contains_key(mac_addr) {
        // let lease = match self.mem.get()
        let lease = match mem.get(mac_addr) {
            None => Lease {
                t1: 0,
                t2: 0,
                lease_time: 0,
                mtu: 0,
                domain_name: "".to_string(),
                mac_address: "".to_string(),
                is_v6: false,
                siaddr: "".to_string(),
                yiaddr: "".to_string(),
                srv_id: "".to_string(),
                subnet_mask: "".to_string(),
                broadcast_addr: "".to_string(),
                dns_servers: vec![],
                gateways: vec![],
                ntp_servers: vec![],
                host_name: "".to_string(),
            },
            Some(l) => l[0].clone(),
        };
        mem.remove(mac_addr);
        // write updated memory cache to the file system
        match self.save_memory_to_fs() {
            Ok(_) => Ok(lease),
            Err(e) => Err(e),
        }
    }

    /// Clean up the memory and file system on tear down of the proxy server
    pub fn teardown(&mut self) -> Result<(), io::Error> {
        self.mem.clear();
        return match OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    }

    /// Save the memory contents to the file system. This will remove the contents in the file,
    /// then write the memory map to the file. This method will be called any the lease memory cache
    /// changes (new lease, remove lease, update lease)
    fn save_memory_to_fs(&self) -> io::Result<()> {
        let path = &self.path;
        let mem = &self.mem;
        // Write and truncate options set to true to clear the file
        let file = OpenOptions::new().write(true).truncate(true).open(path)?;
        serde_json::to_writer(&file, &mem)?;
        file.sync_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod cache_tests {
    #[test]
    fn new() {
        // 1. Clean the directory to the lease

        // 2. Create a new cache instance
        // 3. Check that the file to the cache exists
        // 4. Clean the directory to the lease
    }

    #[test]
    fn update() {
        // 1. Clean the directory to the lease
        // 2. Create a new cache instance
        // 3. Check that the file to the cache exists
        // 4. Change the value of that lease and call the update method
        // 5. Check that the old lease doesnt exist and the new lease is up to date
        // 6. Clean the directory to the lease
    }

    #[test]
    fn remove() {
        // 1. Clean the directory to the lease
        // 2. Create a new cache instance
        // 3. Check that the file to the cache exists
        // 4. Add a lease entry to the cache
        // 5. Check that both leases exist in the cache
        // 6. Remove the lease
        // 7. Check to make sure the lease is gone
        // 8. Clean the directory to the lease
    }

    #[test]
    fn teardown() {
        // 1. Clean the directory to the lease
        // 2. Create a new cache instance
        // 3. Check that the file to the cache exists
        // 4. Add a lease entry to the cache
        // 5. tear down the cache
        // 7. Check to make sure the no leases remain and the fs cache is empty
    }
}
