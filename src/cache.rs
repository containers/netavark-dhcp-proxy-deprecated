use crate::g_rpc::{Lease as NetavarkLease, MacAddress};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Path to the lease json cache
pub const XDGRUNTIME: &str = "/run/user/1000/nv-leases";

/// LeaseCache holds the locked memory map of the mac address to a vector of leases - for multi homing
/// It also stores a locked path buffer to change the FS cache
#[derive(Debug)]
pub struct LeaseCache {
    mem: Mutex<HashMap<MacAddress, Vec<NetavarkLease>>>,
    path: Mutex<PathBuf>,
}

impl LeaseCache {
    /// Initiate the memory and file system cache. Will create and open the specified directory for
    /// the cache and create an empty memory map
    ///
    /// # Arguments
    /// * `dir`: Optional directory string that allows user to define their own cache directory.
    /// Otherwise the default directory is `/run/user/1000/nv-leases`
    ///
    /// returns: Result<LeaseCache, Error>
    ///
    /// On success a new lease cache instance will be returned. On failure a io error will be returned.
    /// This likely means it could not find the file directory
    pub fn new(dir: Option<String>) -> Result<LeaseCache, io::Error> {
        let path = dir.as_deref().unwrap_or(XDGRUNTIME);
        OpenOptions::new().write(true).create(true).open(path)?;
        Ok(LeaseCache {
            mem: Mutex::new(HashMap::new()),
            path: Mutex::new(PathBuf::from(path)),
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
    /// On success this the method will return Ok(). On a failure it will return an IO error due to
    /// not being able to write or read the file system cache
    pub fn add_lease(&self, mac_addr: &MacAddress, lease: &NetavarkLease) -> Result<(), io::Error> {
        let mut cache = self.mem.lock().unwrap();
        // write to the memory cache
        cache.insert(mac_addr.clone(), vec![lease.clone()]);
        // write updated memory cache to the file system
        self.save_memory_to_fs()
    }

    /// When a mac address lease information changes significantly, update the cache to reflect the
    /// changes
    ///
    /// # Arguments
    ///
    /// * `mac_addr`: Mac address of the container
    /// * `lease`: Newest lease information
    ///
    /// returns: Result<(), Error>
    ///
    /// On success returns Ok(). On failure returns an io error, likely means that the it could not
    /// find the file
    pub fn update_lease(
        &self,
        mac_addr: MacAddress,
        lease: NetavarkLease,
    ) -> Result<(), io::Error> {
        let mut cache = self.mem.lock().unwrap();
        // write to the memory cache
        cache.insert(mac_addr, vec![lease.clone()]);
        // write updated memory cache to the file system
        self.save_memory_to_fs()
    }

    /// When a singular container is taken down. Remove that lease from the cache memory and fs
    ///
    /// # Arguments
    ///
    /// * `mac_addr`: Mac address of the container
    pub fn remove_lease(&self, mac_addr: MacAddress) -> Result<(), io::Error> {
        let mut cache = self.mem.lock().unwrap();
        cache.remove(&mac_addr);
        // write updated memory cache to the file system
        self.save_memory_to_fs()
    }

    /// Clean up the memory and file system on tear down of the proxy server
    pub fn teardown(&self) -> Result<(), io::Error> {
        self.mem.lock().unwrap().clear();
        let path = Path::new(XDGRUNTIME);
        return match OpenOptions::new().truncate(true).open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    }

    /// Save the memory contents to the file system. This will remove the contents in the file,
    /// then write the memory map to the file. This method will be called any the lease memory cache
    /// changes (new lease, remove lease, update lease)
    fn save_memory_to_fs(&self) -> std::io::Result<()> {
        let path_binding = self.path.lock().unwrap();
        let path = path_binding.deref();
        let mem_binding = self.mem.lock().unwrap();
        let mem = mem_binding.deref();
        // Write and truncate options set to true to clear the file
        let file = OpenOptions::new().write(true).truncate(true).open(path)?;
        serde_json::to_writer_pretty(&file, &mem)?;
        Ok(())
    }
}

impl Default for LeaseCache {
    fn default() -> Self {
        LeaseCache {
            mem: Mutex::new(HashMap::<MacAddress, Vec<NetavarkLease>>::new()),
            path: Mutex::from(PathBuf::from(XDGRUNTIME)),
        }
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
