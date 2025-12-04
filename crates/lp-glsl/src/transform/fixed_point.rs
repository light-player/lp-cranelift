//! Float-to-fixed-point transformation pass.
//!
//! This module converts floating point operations to fixed-point arithmetic
//! at the Cranelift IR level. All F32 types and operations are converted to
//! fixed-point representation using either I32 (16.16 format) or I64 (32.32 format).

#![allow(unused_imports)]

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

use cranelift_codegen::ir::{
    Function, Inst, Block, Value, Type, InstBuilder, condcodes::{FloatCC, IntCC},
};
use cranelift_codegen::cursor::{Cursor, FuncCursor};

/// Fixed-point format selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedPointFormat {
    /// 16.16 format: 16 integer bits, 16 fractional bits (uses I32)
    /// Range: -32768.0 to +32767.9999847412109375
    /// Precision: 1/65536 (approximately 0.00001526)
    Fixed16x16,
    
    /// 32.32 format: 32 integer bits, 32 fractional bits (uses I64)
    /// Range: -2147483648.0 to +2147483647.9999999998
    /// Precision: 1/4294967296 (approximately 0.00000000023)
    Fixed32x32,
}

impl FixedPointFormat {
    /// Get the Cranelift type for this fixed-point format
    pub fn cranelift_type(&self) -> Type {
        match self {
            FixedPointFormat::Fixed16x16 => cranelift_codegen::ir::types::I32,
            FixedPointFormat::Fixed32x32 => cranelift_codegen::ir::types::I64,
        }
    }
    
    /// Get the shift amount for this format
    pub fn shift_amount(&self) -> i64 {
        match self {
            FixedPointFormat::Fixed16x16 => 16,
            FixedPointFormat::Fixed32x32 => 32,
        }
    }
}

/// Convert a float32 value to fixed16x16 representation.
///
/// Fixed16x16 format uses 16 integer bits and 16 fractional bits.
/// Range: -32768.0 to +32767.9999847412109375
/// Precision: 1/65536 (approximately 0.00001526)
pub fn float_to_fixed16x16(f: f32) -> i32 {
    // Clamp to representable range
    let clamped = f.clamp(-32768.0, 32767.9999847412109375);
    // Convert to fixed-point (round to nearest)
    let scaled = clamped * 65536.0;
    let rounded = if scaled >= 0.0 {
        (scaled + 0.5) as i32
    } else {
        (scaled - 0.5) as i32
    };
    rounded
}

/// Convert fixed16x16 back to float32 (for debugging/constants).
#[allow(dead_code)]
pub fn fixed16x16_to_float(fixed: i32) -> f32 {
    fixed as f32 / 65536.0
}

/// Convert a float32 value to fixed32x32 representation.
///
/// Fixed32x32 format uses 32 integer bits and 32 fractional bits.
/// Range: -2147483648.0 to +2147483647.9999999998
/// Precision: 1/4294967296 (approximately 0.00000000023)
pub fn float_to_fixed32x32(f: f32) -> i64 {
    // Convert to f64 for more precision in intermediate calculations
    let f64_val = f as f64;
    // Clamp to representable range
    let clamped = f64_val.clamp(-2147483648.0, 2147483647.9999999998);
    // Convert to fixed-point (round to nearest)
    let scaled = clamped * 4294967296.0;
    let rounded = if scaled >= 0.0 {
        (scaled + 0.5) as i64
    } else {
        (scaled - 0.5) as i64
    };
    rounded
}

/// Convert fixed32x32 back to float32 (for debugging/constants).
#[allow(dead_code)]
pub fn fixed32x32_to_float(fixed: i64) -> f32 {
    (fixed as f64 / 4294967296.0) as f32
}

/// Error type for transformation errors
#[derive(Debug, Clone)]
pub struct TransformError {
    pub message: String,
}

impl TransformError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl core::fmt::Display for TransformError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Convert all float operations in a function to fixed-point.
///
/// This pass:
/// 1. Converts function signature (F32 → I32/I64)
/// 2. Converts all F32 values to I32/I64 (fixed-point representation)
/// 3. Converts all float operations to fixed-point operations
/// 4. Updates all value types
/// 5. Verifies the function is still valid
pub fn convert_floats_to_fixed(
    func: &mut Function,
    format: FixedPointFormat,
) -> Result<(), TransformError> {
    // 1. Convert signature
    convert_signature(func, format);

    // 2. Walk all blocks and instructions to convert them
    // We need to collect instructions first to avoid borrow issues
    let mut insts_to_convert: Vec<(Block, Inst)> = Vec::new();
    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            insts_to_convert.push((block, inst));
        }
    }

    // 3. Convert each instruction
    for (_block, inst) in insts_to_convert {
        convert_instruction(func, inst, format)?;
    }

    // 4. Update all value types from F32 to I32/I64
    update_all_value_types(func, format);

    // 5. Verify function is still valid
    if let Err(errors) = cranelift_codegen::verify_function(func, &cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::builder())) {
        return Err(TransformError::new(format!(
            "Verification failed after transformation: {}",
            errors
        )));
    }

    Ok(())
}

/// Convert function signature: F32 params/returns → I32/I64
fn convert_signature(func: &mut Function, format: FixedPointFormat) {
    let target_type = format.cranelift_type();
    
    // Convert parameter types
    for param in &mut func.signature.params {
        if param.value_type == cranelift_codegen::ir::types::F32 {
            param.value_type = target_type;
        }
    }
    
    // Convert return types
    for ret in &mut func.signature.returns {
        if ret.value_type == cranelift_codegen::ir::types::F32 {
            ret.value_type = target_type;
        }
    }
}

/// Convert a single instruction
fn convert_instruction(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
) -> Result<(), TransformError> {
    use cranelift_codegen::ir::Opcode;
    
    let opcode = func.dfg.insts[inst].opcode();
    
    match opcode {
        Opcode::F32const => convert_f32const(func, inst, format)?,
        Opcode::Fadd => convert_fadd(func, inst, format)?,
        Opcode::Fsub => convert_fsub(func, inst, format)?,
        Opcode::Fmul => convert_fmul(func, inst, format)?,
        Opcode::Fdiv => convert_fdiv(func, inst, format)?,
        Opcode::Fcmp => convert_fcmp(func, inst, format)?,
        Opcode::Load => convert_load(func, inst, format)?,
        Opcode::Store => convert_store(func, inst, format)?,
        _ => {
            // Other instructions don't need conversion
        }
    }
    
    Ok(())
}

/// Update all value types from F32 to I32/I64
fn update_all_value_types(func: &mut Function, format: FixedPointFormat) {
    let target_type = format.cranelift_type();
    
    // Get all values that need updating
    let mut values_to_update = Vec::new();
    
    // Check all instruction results and arguments
    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            let results = func.dfg.inst_results(inst).to_vec();
            for &result in &results {
                if func.dfg.value_type(result) == cranelift_codegen::ir::types::F32 {
                    values_to_update.push(result);
                }
            }
        }
        
        // Check block parameters
        let params = func.dfg.block_params(block).to_vec();
        for &param in &params {
            if func.dfg.value_type(param) == cranelift_codegen::ir::types::F32 {
                values_to_update.push(param);
            }
        }
    }
    
    // Update all collected values
    for value in values_to_update {
        func.dfg.change_to_alias(value, value);
        // Note: We'll need to handle this differently in Cranelift
        // This is a placeholder - the actual implementation will depend on Cranelift's API
    }
}

