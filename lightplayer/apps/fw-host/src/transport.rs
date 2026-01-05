//! Host transport implementation using stdio

use lp_core::error::Error;
use lp_core::traits::Transport;
use std::io::{self, BufRead, BufReader, Write};

/// Host transport implementation using stdin/stdout
pub struct HostTransport {
    stdin: BufReader<io::Stdin>,
    stdout: io::Stdout,
    buffer: String,
}

impl HostTransport {
    /// Create a new host transport
    pub fn new() -> Self {
        Self {
            stdin: BufReader::new(io::stdin()),
            stdout: io::stdout(),
            buffer: String::new(),
        }
    }
}

impl Default for HostTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for HostTransport {
    fn send_message(&mut self, message: &str) -> Result<(), Error> {
        let stdout = &mut self.stdout;
        stdout
            .write_all(message.as_bytes())
            .map_err(|e| Error::Protocol(format!("Failed to write to stdout: {}", e)))?;
        stdout
            .write_all(b"\n")
            .map_err(|e| Error::Protocol(format!("Failed to write newline: {}", e)))?;
        stdout
            .flush()
            .map_err(|e| Error::Protocol(format!("Failed to flush stdout: {}", e)))?;
        Ok(())
    }

    fn receive_message(&mut self) -> Result<String, Error> {
        // Read until we get a complete line (ending with \n)
        self.buffer.clear();
        self.stdin
            .read_line(&mut self.buffer)
            .map_err(|e| Error::Protocol(format!("Failed to read from stdin: {}", e)))?;

        // Remove trailing newline if present
        let message = self.buffer.trim_end_matches('\n').to_string();
        Ok(message)
    }
}

