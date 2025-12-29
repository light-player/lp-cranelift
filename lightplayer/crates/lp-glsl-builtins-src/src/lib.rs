#![no_std]

extern crate alloc;

mod builtins;
mod test_util;

// Re-export run_runtests macro for easy usage (will be added when macro is implemented)
pub use test_util::run_runtests::run_runtests;
