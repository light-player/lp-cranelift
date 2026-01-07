//! Host OutputProvider implementation

use lp_core::error::Error;
use lp_core::nodes::output::config::OutputNode;
use lp_core::traits::{LedOutput, OutputProvider};

use crate::led_output::HostLedOutput;

/// Host implementation of OutputProvider
///
/// Creates HostLedOutput instances for visualization in egui.
pub struct HostOutputProvider;

impl HostOutputProvider {
    /// Create a new HostOutputProvider
    pub fn new() -> Self {
        Self
    }
}

impl Default for HostOutputProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputProvider for HostOutputProvider {
    fn create_output(&self, config: &OutputNode) -> Result<std::boxed::Box<dyn LedOutput>, Error> {
        match config {
            OutputNode::GpioStrip { chip, count, .. } => {
                // Derive bytes_per_pixel from chip type
                // For now, assume ws2812 = 3 bytes (RGB)
                let bytes_per_pixel = match chip.as_str() {
                    "ws2812" | "ws2812b" => 3,
                    _ => {
                        return Err(Error::Validation(format!(
                            "Unknown chip type: {}",
                            chip
                        )));
                    }
                };

                let pixel_count = *count as usize;
                let output = HostLedOutput::new(pixel_count, bytes_per_pixel);

                Ok(std::boxed::Box::new(output))
            }
        }
    }
}

