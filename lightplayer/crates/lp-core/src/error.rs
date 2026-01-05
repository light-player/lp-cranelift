//! Error types for lp-core

use alloc::string::String;
use core::fmt;

/// Error type for lp-core operations
#[derive(Debug, Clone)]
pub enum Error {
    /// Serialization/deserialization error
    Serialization(String),
    /// Filesystem error
    Filesystem(String),
    /// Protocol error
    Protocol(String),
    /// Project validation error
    Validation(String),
    /// Node-specific error
    Node(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Filesystem(msg) => write!(f, "Filesystem error: {}", msg),
            Error::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            Error::Validation(msg) => write!(f, "Validation error: {}", msg),
            Error::Node(msg) => write!(f, "Node error: {}", msg),
        }
    }
}
