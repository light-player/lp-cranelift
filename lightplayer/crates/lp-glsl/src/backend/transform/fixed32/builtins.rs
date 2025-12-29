//! Builtin function generators for fixed32 transform.
//!
//! This module generates CLIF for builtin functions that are too complex to inline,
//! such as sqrt using reciprocal multiplication.

use crate::backend::module::gl_module::GlModule;
use crate::backend::transform::fixed32::instructions::Fixed32Builtin;
use crate::backend::transform::fixed32::types::FixedPointFormat;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{AbiParam, Function, InstBuilder, Signature, condcodes::IntCC, types};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage, Module};

/// Generate and add a builtin function to the module.
///
/// Returns the FuncId of the added function.
pub fn generate_and_add_builtin<M: Module>(
    builtin: Fixed32Builtin,
    module: &mut GlModule<M>,
    format: FixedPointFormat,
) -> Result<FuncId, GlslError> {
    match builtin {
        Fixed32Builtin::SqrtRecip => generate_sqrt_recip_builtin(module, format),
    }
}

/// Generate the __fixed32_sqrt_recip builtin function.
///
/// Signature: (i32) -> i32
/// Implements square root using Newton-Raphson with reciprocal multiplication.
fn generate_sqrt_recip_builtin<M: Module>(
    module: &mut GlModule<M>,
    format: FixedPointFormat,
) -> Result<FuncId, GlslError> {
    const FUNC_NAME: &str = "__fixed32_sqrt_recip";
    const SHIFT: i64 = 16;

    // Create signature: (i32) -> i32
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::I32));
    sig.returns.push(AbiParam::new(types::I32));

    // Declare function in module
    let func_id = module
        .declare_function(FUNC_NAME, Linkage::Local, sig.clone())
        .map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                alloc::format!("Failed to declare builtin function '{}': {}", FUNC_NAME, e),
            )
        })?;

    // Create function builder context
    let mut ctx = module.module_internal().make_context();
    ctx.func.signature = sig.clone();
    ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32());

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    // Create entry block
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    // Get parameter
    let x_fixed = builder.block_params(entry_block)[0];

    // Handle edge cases: if x_fixed <= 0, return 0
    let zero = builder.ins().iconst(types::I32, 0);
    let is_zero = builder.ins().icmp(IntCC::Equal, x_fixed, zero);
    let is_negative = builder.ins().icmp(IntCC::SignedLessThan, x_fixed, zero);
    let is_invalid = builder.ins().bor(is_zero, is_negative);

    // Convert to i64 and scale up: x_scaled = x_fixed << 16
    let x_fixed_i64 = builder.ins().sextend(types::I64, x_fixed);
    let shift_16_const = builder.ins().iconst(types::I64, SHIFT);
    let x_scaled = builder.ins().ishl(x_fixed_i64, shift_16_const);

    // Create constants once (outside loop)
    let zero_i64 = builder.ins().iconst(types::I64, 0);
    let one_i64 = builder.ins().iconst(types::I64, 1);
    let one_i32 = builder.ins().iconst(types::I32, 1);
    let zero_i32 = builder.ins().iconst(types::I32, 0);
    let two_i64 = builder.ins().iconst(types::I64, 2);
    let shift_1_const = builder.ins().iconst(types::I64, 1);
    let shift_const = builder.ins().iconst(types::I64, SHIFT);
    let recip_base_const = builder.ins().iconst(types::I32, 0x8000_0000i64);

    // Initial guess: x_scaled >> 9
    let shift_9_const = builder.ins().iconst(types::I64, 9);
    let guess = builder.ins().sshr(x_scaled, shift_9_const);
    let guess = builder.ins().smax(guess, one_i64);

    // Newton-Raphson iterations: 6 iterations
    let mut current_guess = guess;
    
    for _ in 0..6 {
        // Compute x_scaled / current_guess using reciprocal multiplication
        // 1. Truncate guess to i32 for reciprocal calculation
        let guess_i32 = builder.ins().ireduce(types::I32, current_guess);
        
        // Compute absolute value of guess_i32
        let guess_is_neg_i32 = builder.ins().icmp(IntCC::SignedLessThan, guess_i32, zero_i32);
        let guess_negated_i32 = builder.ins().ineg(guess_i32);
        let guess_abs = builder.ins().select(guess_is_neg_i32, guess_negated_i32, guess_i32);
        
        // Ensure guess is not zero
        let guess_is_zero = builder.ins().icmp(IntCC::Equal, guess_abs, zero_i32);
        let guess_safe = builder.ins().select(guess_is_zero, one_i32, guess_abs);

        // 2. Compute reciprocal: 0x8000_0000 / guess_safe (i32 division)
        let recip = builder.ins().udiv(recip_base_const, guess_safe);

        // 3. Multiply x_scaled by reciprocal: (x_scaled * recip * 2) >> 16
        // Compute absolute value of x_scaled (i64)
        let x_scaled_is_neg = builder.ins().icmp(IntCC::SignedLessThan, x_scaled, zero_i64);
        let x_scaled_negated = builder.ins().ineg(x_scaled);
        let x_scaled_abs = builder.ins().select(x_scaled_is_neg, x_scaled_negated, x_scaled);
        
        // Extend reciprocal to i64 for multiplication
        // recip is u32, extend to i64 (will be positive)
        let recip_u64 = builder.ins().uextend(types::I64, recip);
        // Multiply: x_scaled_abs (i64, positive) * recip_u64 (i64, positive)
        let mul_result = builder.ins().imul(x_scaled_abs, recip_u64);
        let mul_result_2x = builder.ins().imul(mul_result, two_i64);
        let quotient = builder.ins().ushr(mul_result_2x, shift_const);

        // Apply sign: if x_scaled and guess have different signs, negate
        let guess_is_neg = builder.ins().icmp(IntCC::SignedLessThan, current_guess, zero_i64);
        let result_is_neg = builder.ins().bxor(x_scaled_is_neg, guess_is_neg);
        let quotient_negated = builder.ins().ineg(quotient);
        let x_div_guess = builder.ins().select(result_is_neg, quotient_negated, quotient);

        // 4. Newton-Raphson step: guess_new = (guess + x_scaled / guess) >> 1
        let sum = builder.ins().iadd(current_guess, x_div_guess);
        current_guess = builder.ins().sshr(sum, shift_1_const);

        // Ensure guess doesn't become zero
        let is_zero_guess = builder.ins().icmp(IntCC::Equal, current_guess, zero_i64);
        current_guess = builder.ins().select(is_zero_guess, one_i64, current_guess);
    }

    // Final scaling: result = guess >> 8
    let shift_8_const = builder.ins().iconst(types::I64, 8);
    let result_i64 = builder.ins().sshr(current_guess, shift_8_const);
    let result = builder.ins().ireduce(types::I32, result_i64);

    // Handle edge cases: if input was zero or negative, return 0
    let final_result = builder.ins().select(is_invalid, zero, result);

    // Return result
    builder.ins().return_(&[final_result]);
    
    // Seal all blocks before finalizing
    builder.seal_all_blocks();
    builder.finalize();

    // Define function in module
    module
        .module_mut_internal()
        .define_function(func_id, &mut ctx)
        .map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                alloc::format!("Failed to define builtin function '{}': {}", FUNC_NAME, e),
            )
        })?;

    // Update the function in fns map
    let function = ctx.func.clone();
    module.add_function_to_fns(FUNC_NAME, sig, function, func_id);

    module.module_internal().clear_context(&mut ctx);

    Ok(func_id)
}

