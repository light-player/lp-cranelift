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
pub mod math;
pub mod memory;

pub(crate) use helpers::*;
