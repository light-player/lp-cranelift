//! Shared utilities for function transformation
//!
//! This module provides common utilities for copying CLIF function components
//! between functions. These utilities are used by all transforms.

pub mod blocks;
pub mod instruction_copy;
pub mod stack_slots;
pub mod value_aliases;

// Re-export all utilities for easy importing
pub use blocks::{
    create_blocks, ensure_block_params, map_entry_block_params,
};
pub use instruction_copy::copy_instruction;
pub use stack_slots::copy_stack_slots;
pub use value_aliases::copy_value_aliases;

