//! Output provider trait for creating LED output handles

use crate::error::Error;
use crate::nodes::output::config::OutputNode;
use crate::traits::LedOutput;

/// Trait for creating LED output handles from configuration
///
/// Implemented by firmware-specific code (ESP32, host, etc.)
/// to set up hardware based on output configuration.
pub trait OutputProvider {
    /// Create and configure an LED output handle from configuration
    ///
    /// For `GpioStrip`: configures GPIO pin from `config.gpio_pin`,
    /// sets up chip driver (ws2812, etc.), and returns a handle.
    ///
    /// Returns an error if the output cannot be created.
    fn create_output(&self, config: &OutputNode)
    -> Result<alloc::boxed::Box<dyn LedOutput>, Error>;
}
