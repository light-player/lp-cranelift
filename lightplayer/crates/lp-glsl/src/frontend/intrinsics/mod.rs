//! Intrinsic math function implementations.
//!
//! This module provides GLSL-based implementations of math functions
//! that are compiled and inserted into the module on demand.

mod compiler;
mod dependency;
pub mod loader;

pub use compiler::compile_intrinsic_functions;
pub use dependency::{DependencyGraph, build_dependency_graph, compute_transitive_closure};
pub use loader::{IntrinsicCache, get_or_create_intrinsic};
