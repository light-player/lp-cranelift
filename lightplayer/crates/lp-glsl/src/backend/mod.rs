//! Backend2: New architecture for direct module building
//!
//! This module provides a cleaner architecture that builds functions directly
//! in the final Module without a linking step.

pub mod codegen;
pub mod module;
pub mod target;
pub mod transform;
