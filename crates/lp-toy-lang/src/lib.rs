//! Toy language compiler using Cranelift JIT.
//!
//! This crate provides a simple toy language that compiles to native machine code
//! using Cranelift's JIT compiler. It supports both `std` and `no_std` environments.

#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod executor;
pub mod frontend;
pub mod jit;

pub use executor::{execute_function, execute_function_0args, execute_function_1arg, execute_function_2args};
pub use jit::JIT;
