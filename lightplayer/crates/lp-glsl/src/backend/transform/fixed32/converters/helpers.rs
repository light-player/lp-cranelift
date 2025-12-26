//! Common helper functions for instruction conversion.
//!
//! This module provides shared utilities used across different converter modules.
//! Most utilities have been moved to `crate::util` for reuse across the codebase.

// This module is kept for backwards compatibility and re-exports.
// All functionality has been moved to crate::util modules.

pub use crate::backend::ir_utils::fixed_point::{
    create_max_fixed_const, create_min_fixed_const, create_zero_const, max_fixed_value,
    min_fixed_value,
};
pub use crate::backend::ir_utils::instruction::{
    extract_binary_operands, extract_unary_operand, get_first_result,
    unexpected_format_error, verify_converted_type,
};
pub use crate::backend::ir_utils::value_map::{map_operand, map_operands, map_value};
