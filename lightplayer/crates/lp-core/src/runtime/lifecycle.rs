//! Node lifecycle trait

use crate::error::Error;
use crate::runtime::contexts::InitContext;

/// Trait for node lifecycle management
pub trait NodeLifecycle {
    /// Configuration type for this node
    type Config;
    /// Render context type for this node
    type RenderContext;

    /// Initialize the node from configuration
    ///
    /// Validates dependencies, allocates resources, compiles shaders, etc.
    fn init(&mut self, config: &Self::Config, ctx: &InitContext) -> Result<(), Error>;

    /// Update the node state
    ///
    /// Called each frame with a render context providing access to other nodes.
    fn update(&mut self, ctx: &Self::RenderContext) -> Result<(), Error>;

    /// Cleanup resources
    ///
    /// Called when unloading the project.
    fn destroy(&mut self) -> Result<(), Error>;
}
