//! Instruction converters for fixed-point transformation.
//!
//! This module contains converters that transform F32 instructions into
//! fixed-point equivalents using the builder-based rewrite approach.

pub mod arithmetic;
pub mod calls;
pub mod comparison;
pub mod constants;
pub mod control;
pub mod conversions;
pub mod helpers;
pub mod instruction_copy;
pub mod math;
pub mod memory;

// Re-export instruction_copy for use in link.rs
pub use instruction_copy::{copy_instruction_as_is, copy_instruction_as_is_with_stack_slot_map};

pub(crate) use helpers::*;
