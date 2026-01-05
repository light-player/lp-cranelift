//! Filesystem abstraction trait

use crate::error::Error;

/// Platform-agnostic filesystem trait
pub trait Filesystem {
    /// Read a file from the filesystem
    ///
    /// Returns the file contents as a byte vector, or an error if the file doesn't exist
    /// or cannot be read.
    fn read_file(&self, path: &str) -> Result<alloc::vec::Vec<u8>, Error>;

    /// Write data to a file in the filesystem
    ///
    /// Creates the file if it doesn't exist, overwrites if it does.
    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), Error>;

    /// Check if a file exists in the filesystem
    fn file_exists(&self, path: &str) -> Result<bool, Error>;
}

