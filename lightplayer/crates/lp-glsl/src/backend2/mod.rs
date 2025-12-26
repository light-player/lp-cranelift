//! Backend2: New architecture for direct module building
//!
//! This module provides a cleaner architecture that builds functions directly
//! in the final Module without a linking step.

pub mod target;
pub mod module;
pub mod codegen;
