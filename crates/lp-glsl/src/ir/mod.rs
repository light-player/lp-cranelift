//! Intermediate representation for GLSL compilation.
//!
//! This module contains types and utilities for representing GLSL shaders
//! in Cranelift IR form before linking and execution.

mod clif_module;

pub use clif_module::{ClifModule, ClifModuleBuilder};
