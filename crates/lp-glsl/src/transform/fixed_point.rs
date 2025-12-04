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

/// Convert F32const to iconst with fixed-point value
fn convert_f32const(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
) -> Result<(), TransformError> {
    use cranelift_codegen::ir::InstructionData;
    
    // Get the float constant value
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::UnaryIeee32 { opcode: _, imm } = inst_data {
        let f32_value = f32::from_bits(imm.bits());
        let _result = func.dfg.first_result(inst);
        
        // Convert to fixed-point
        let fixed_value = match format {
            FixedPointFormat::Fixed16x16 => float_to_fixed16x16(f32_value) as i64,
            FixedPointFormat::Fixed32x32 => float_to_fixed32x32(f32_value),
        };
        
        // Replace with iconst
        let target_type = format.cranelift_type();
        func.dfg.replace(inst).iconst(target_type, fixed_value);
    }
    
    Ok(())
}

/// Convert Fadd to Iadd (fixed-point addition is direct integer addition)
fn convert_fadd(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
) -> Result<(), TransformError> {
    use cranelift_codegen::ir::InstructionData;
    
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = args[0];
        let arg2 = args[1];
        
        // Replace Fadd with Iadd
        func.dfg.replace(inst).iadd(arg1, arg2);
        
        // Update result type
        let result = func.dfg.first_result(inst);
        func.dfg.change_to_alias(result, result);
    }
    
    Ok(())
}

/// Convert Fsub to Isub (fixed-point subtraction is direct integer subtraction)
fn convert_fsub(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
) -> Result<(), TransformError> {
    use cranelift_codegen::ir::InstructionData;
    
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = args[0];
        let arg2 = args[1];
        
        // Replace Fsub with Isub
        func.dfg.replace(inst).isub(arg1, arg2);
        
        // Update result type
        let result = func.dfg.first_result(inst);
        func.dfg.change_to_alias(result, result);
    }
    
    Ok(())
}

/// Convert Fmul to fixed-point multiplication sequence
/// For fixed-point multiply: result = (a * b) >> shift_amount
fn convert_fmul(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
) -> Result<(), TransformError> {
    use cranelift_codegen::ir::InstructionData;
    use cranelift_codegen::cursor::{Cursor, FuncCursor};
    
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = args[0];
        let arg2 = args[1];
        let result = func.dfg.first_result(inst);
        let shift_amount = format.shift_amount();
        let _target_type = format.cranelift_type();
        
        // Create a cursor positioned at this instruction
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        
        match format {
            FixedPointFormat::Fixed16x16 => {
                // For 16.16: result = (a * b) >> 16
                // We can use a simpler approach: 
                // hi = (a * b) >> 32 (using smulhi for signed multiplication high)
                // lo = (a * b) & 0xFFFFFFFF (using regular mul)
                // result = (hi << 16) | (lo >> 16)
                
                // Actually, simpler: Just do 64-bit math
                // Extend to 64-bit, multiply, shift, truncate
                let a_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, arg1);
                let b_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, arg2);
                let mul_64 = cursor.ins().imul(a_ext, b_ext);
                let shift_const_64 = cursor.ins().iconst(cranelift_codegen::ir::types::I64, shift_amount);
                let shifted = cursor.ins().sshr(mul_64, shift_const_64);
                let result_32 = cursor.ins().ireduce(cranelift_codegen::ir::types::I32, shifted);
                
                // Replace original instruction's result with our final value
                cursor.func.dfg.change_to_alias(result, result_32);
                
                // Remove the original instruction
                cursor.goto_inst(inst);
                cursor.remove_inst();
            }
            FixedPointFormat::Fixed32x32 => {
                // For 32.32: result = (a * b) >> 32
                // Use i128 arithmetic
                let a_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I128, arg1);
                let b_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I128, arg2);
                let mul_128 = cursor.ins().imul(a_ext, b_ext);
                let shift_const_128 = cursor.ins().iconst(cranelift_codegen::ir::types::I64, shift_amount);
                let shifted = cursor.ins().sshr(mul_128, shift_const_128);
                let result_64 = cursor.ins().ireduce(cranelift_codegen::ir::types::I64, shifted);
                
                // Replace original instruction's result
                cursor.func.dfg.change_to_alias(result, result_64);
                
                // Remove the original instruction
                cursor.goto_inst(inst);
                cursor.remove_inst();
            }
        }
    }
    
    Ok(())
}

/// Convert Fdiv to fixed-point division sequence
/// For fixed-point divide: result = (a << shift_amount) / b
fn convert_fdiv(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
) -> Result<(), TransformError> {
    use cranelift_codegen::ir::InstructionData;
    use cranelift_codegen::cursor::{Cursor, FuncCursor};
    
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = args[0];
        let arg2 = args[1];
        let result = func.dfg.first_result(inst);
        let shift_amount = format.shift_amount();
        
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        
        match format {
            FixedPointFormat::Fixed16x16 => {
                // For 16.16: result = (a << 16) / b
                // Extend to 64-bit to avoid overflow
                let a_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, arg1);
                let shift_const = cursor.ins().iconst(cranelift_codegen::ir::types::I64, shift_amount);
                let a_shifted = cursor.ins().ishl(a_ext, shift_const);
                let b_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, arg2);
                let div_result = cursor.ins().sdiv(a_shifted, b_ext);
                let result_32 = cursor.ins().ireduce(cranelift_codegen::ir::types::I32, div_result);
                
                cursor.func.dfg.change_to_alias(result, result_32);
                cursor.goto_inst(inst);
                cursor.remove_inst();
            }
            FixedPointFormat::Fixed32x32 => {
                // For 32.32: result = (a << 32) / b
                // Extend to 128-bit
                let a_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I128, arg1);
                let shift_const = cursor.ins().iconst(cranelift_codegen::ir::types::I64, shift_amount);
                let a_shifted = cursor.ins().ishl(a_ext, shift_const);
                let b_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I128, arg2);
                let div_result = cursor.ins().sdiv(a_shifted, b_ext);
                let result_64 = cursor.ins().ireduce(cranelift_codegen::ir::types::I64, div_result);
                
                cursor.func.dfg.change_to_alias(result, result_64);
                cursor.goto_inst(inst);
                cursor.remove_inst();
            }
        }
    }
    
    Ok(())
}

/// Convert Fcmp to Icmp with appropriate condition code
fn convert_fcmp(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
) -> Result<(), TransformError> {
    use cranelift_codegen::ir::InstructionData;
    
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::FloatCompare { opcode: _, cond, args } = inst_data {
        let arg1 = args[0];
        let arg2 = args[1];
        let cond = *cond;
        
        // Convert FloatCC to IntCC
        let int_cond = match cond {
            FloatCC::Equal => IntCC::Equal,
            FloatCC::NotEqual => IntCC::NotEqual,
            FloatCC::LessThan => IntCC::SignedLessThan,
            FloatCC::LessThanOrEqual => IntCC::SignedLessThanOrEqual,
            FloatCC::GreaterThan => IntCC::SignedGreaterThan,
            FloatCC::GreaterThanOrEqual => IntCC::SignedGreaterThanOrEqual,
            // For unordered/ordered: no NaN in fixed-point
            FloatCC::Ordered => IntCC::Equal, // Always true, use a == a
            FloatCC::Unordered => IntCC::NotEqual, // Always false, use a != a
            FloatCC::OrderedNotEqual => IntCC::NotEqual,
            FloatCC::UnorderedOrEqual => IntCC::Equal,
            FloatCC::UnorderedOrLessThan => IntCC::SignedLessThan,
            FloatCC::UnorderedOrLessThanOrEqual => IntCC::SignedLessThanOrEqual,
            FloatCC::UnorderedOrGreaterThan => IntCC::SignedGreaterThan,
            FloatCC::UnorderedOrGreaterThanOrEqual => IntCC::SignedGreaterThanOrEqual,
        };
        
        // Replace Fcmp with Icmp
        func.dfg.replace(inst).icmp(int_cond, arg1, arg2);
    }
    
    Ok(())
}

/// Convert Load with F32 type to Load with I32/I64 type
fn convert_load(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
) -> Result<(), TransformError> {
    use cranelift_codegen::ir::InstructionData;
    
    let inst_data = &func.dfg.insts[inst];
    
    // Check if this is a load of F32 type
    let result = func.dfg.first_result(inst);
    if func.dfg.value_type(result) != cranelift_codegen::ir::types::F32 {
        return Ok(()); // Not an F32 load, skip
    }
    
    if let InstructionData::Load { opcode: _, flags, offset, arg } = inst_data {
        let addr = *arg;
        let flags = *flags;
        let offset = *offset;
        let target_type = format.cranelift_type();
        
        // Replace with load of target type
        func.dfg.replace(inst).load(target_type, flags, addr, offset);
    }
    
    Ok(())
}

/// Convert Store with F32 type to Store with I32/I64 type
fn convert_store(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
) -> Result<(), TransformError> {
    use cranelift_codegen::ir::InstructionData;
    
    let inst_data = &func.dfg.insts[inst];
    
    if let InstructionData::Store { opcode: _, flags, offset, args } = inst_data {
        let addr = args[0];
        let value = args[1];
        
        // Check if we're storing an F32 value
        if func.dfg.value_type(value) != cranelift_codegen::ir::types::F32 {
            return Ok(()); // Not an F32 store, skip
        }
        
        let flags = *flags;
        let offset = *offset;
        
        // Replace with store of target type
        func.dfg.replace(inst).store(flags, value, addr, offset);
    }
    
    Ok(())
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
    
    // Collect all values that need updating
    let mut values_to_update = Vec::new();
    
    // Check all instruction results
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
    
    // Update types by replacing each value with a new one of the target type
    for old_value in values_to_update {
        let new_value = func.dfg.replace_result(old_value, target_type);
        // Use the new value wherever the old one was used
        func.dfg.change_to_alias(old_value, new_value);
    }
}

