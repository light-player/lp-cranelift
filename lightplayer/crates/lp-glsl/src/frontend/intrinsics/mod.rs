//! Intrinsic math function implementations.
//!
//! This module provides GLSL-based implementations of math functions
//! that are compiled and inserted into the module on demand.

mod compiler;
pub mod loader;

pub use compiler::compile_intrinsic_functions;
pub use loader::{IntrinsicCache, get_or_create_intrinsic};
