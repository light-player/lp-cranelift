//! Toy language compiler using automatic SSA construction.
//!
//! This crate ports the cranelift-jit-demo to use our LPIR and SSABuilder API.
//! It serves as validation that the new SSABuilder API works correctly.

pub mod executor;
pub mod frontend;
pub mod translator;

pub use executor::execute_function;
pub use translator::Translator;
