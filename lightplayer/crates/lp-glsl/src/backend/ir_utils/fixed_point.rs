//! Fixed-point format utilities.

use crate::backend::transform::FixedPointFormat;
use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;

/// Get the maximum representable value for a fixed-point format.
pub fn max_fixed_value(format: FixedPointFormat) -> i32 {
    match format {
        FixedPointFormat::Fixed16x16 => 0x7FFF0000i32, // 32767.0 in 16.16 format
        FixedPointFormat::Fixed32x32 => i32::MAX,      // Not fully implemented
    }
}

/// Get the minimum representable value for a fixed-point format.
pub fn min_fixed_value(format: FixedPointFormat) -> i32 {
    match format {
        FixedPointFormat::Fixed16x16 => 0x80000000u32 as i32, // -32768.0 in 16.16 format (i32::MIN)
        FixedPointFormat::Fixed32x32 => i32::MIN,             // Not fully implemented
    }
}

/// Create a constant for the maximum fixed-point value.
pub fn create_max_fixed_const(builder: &mut FunctionBuilder, format: FixedPointFormat) -> Value {
    let target_type = format.cranelift_type();
    let max_val = max_fixed_value(format);
    builder.ins().iconst(target_type, max_val as i64)
}

/// Create a constant for the minimum fixed-point value.
pub fn create_min_fixed_const(builder: &mut FunctionBuilder, format: FixedPointFormat) -> Value {
    let target_type = format.cranelift_type();
    let min_val = min_fixed_value(format);
    builder.ins().iconst(target_type, min_val as i64)
}

/// Create a zero constant for the target fixed-point type.
pub fn create_zero_const(builder: &mut FunctionBuilder, format: FixedPointFormat) -> Value {
    let target_type = format.cranelift_type();
    builder.ins().iconst(target_type, 0)
}
