//! Fixed32 transform for backend2
//!
//! This module adapts the existing fixed32 transform to work with the new
//! backend2 architecture using the Transform trait.

mod transform;

pub use transform::Fixed32Transform;
// Re-export FixedPointFormat - use the public re-export from backend::transform::fixed32
pub use crate::backend::transform::fixed32::FixedPointFormat;

