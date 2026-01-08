//! Error types for lp-core

use alloc::{string::String, vec::Vec};
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
            Error::Node(msg) => {
                // For node errors, preserve multi-line formatting
                // If the message contains newlines, format it nicely with indentation
                if msg.contains('\n') {
                    // Split message into lines and indent continuation lines
                    let lines: Vec<&str> = msg.lines().collect();
                    if lines.is_empty() {
                        write!(f, "Node error: {}", msg)
                    } else {
                        write!(f, "Node error: {}", lines[0])?;
                        for line in lines.iter().skip(1) {
                            write!(f, "\n         {}", line)?;
                        }
                        Ok(())
                    }
                } else {
                    write!(f, "Node error: {}", msg)
                }
            }
        }
    }
}
