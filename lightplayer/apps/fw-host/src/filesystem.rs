//! Host filesystem implementation using std::fs

use lp_core::error::Error;
use lp_core::traits::Filesystem;
use std::fs;
use std::path::PathBuf;

/// Host filesystem implementation using std::fs
pub struct HostFilesystem {
    base_path: PathBuf,
}

impl HostFilesystem {
    /// Create a new host filesystem with the given base path
    pub fn new(base_path: PathBuf) -> Self {
        // Ensure the base directory exists
        if let Err(e) = fs::create_dir_all(&base_path) {
            eprintln!("Warning: Failed to create base directory {:?}: {}", base_path, e);
        }
        Self { base_path }
    }

    /// Get the full path for a file
    fn get_path(&self, path: &str) -> PathBuf {
        self.base_path.join(path)
    }
}

impl Filesystem for HostFilesystem {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, Error> {
        let full_path = self.get_path(path);
        fs::read(&full_path).map_err(|e| {
            Error::Filesystem(format!("Failed to read file {:?}: {}", full_path, e))
        })
    }

    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), Error> {
        let full_path = self.get_path(path);
        // Create parent directory if it doesn't exist
        if let Some(parent) = full_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(Error::Filesystem(format!(
                    "Failed to create directory {:?}: {}",
                    parent, e
                )));
            }
        }
        fs::write(&full_path, data).map_err(|e| {
            Error::Filesystem(format!("Failed to write file {:?}: {}", full_path, e))
        })
    }

    fn file_exists(&self, path: &str) -> Result<bool, Error> {
        let full_path = self.get_path(path);
        Ok(full_path.exists())
    }

    fn list_dir(&self, _path: &str) -> Result<Vec<String>, Error> {
        // TODO: Implement in Phase 10
        todo!("list_dir() will be implemented in Phase 10")
    }
}

