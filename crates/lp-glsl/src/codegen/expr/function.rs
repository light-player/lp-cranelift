use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError, source_span_to_location};
use crate::semantic::type_check::{is_matrix_type_name, is_vector_type_name};
use crate::semantic::types::Type as GlslType;
use cranelift_codegen::ir::InstBuilder;
use glsl::syntax::Expr;

use super::coercion;
use super::constructor;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub fn translate_function_call(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<cranelift_codegen::ir::Value>, GlslType), GlslError> {
    let Expr::FunCall(func_ident, args, span) = expr else {
        unreachable!("translate_function_call called on non-call");
    };

    let func_name = match func_ident {
        glsl::syntax::FunIdentifier::Identifier(ident) => &ident.name,
        _ => {
            return Err(GlslError::new(
                ErrorCode::E0400,
                "complex function identifiers not yet supported",
            ));
        }
    };

    // Check if it's a type constructor
    if is_vector_type_name(func_name) {
        return constructor::translate_vector_constructor(ctx, func_name, args, span.clone());
    }

    if is_matrix_type_name(func_name) {
        return constructor::translate_matrix_constructor(ctx, func_name, args);
    }

    // Check if it's a built-in function
    if crate::semantic::builtins::is_builtin_function(func_name) {
        return translate_builtin_call_expr(ctx, func_name, args, span.clone());
    }

    // User-defined function
    translate_user_function_call(ctx, func_name, args, span.clone())
}

fn translate_builtin_call_expr(
    ctx: &mut CodegenContext,
    name: &str,
    args: &[glsl::syntax::Expr],
    call_span: glsl::syntax::SourceSpan,
) -> Result<(Vec<cranelift_codegen::ir::Value>, GlslType), GlslError> {
    // Translate all arguments
    let mut translated_args = Vec::new();
    let mut arg_types = Vec::new();

    for arg in args {
        let (vals, ty) = ctx.translate_expr_typed(arg)?;
        translated_args.push((vals, ty.clone()));
        arg_types.push(ty);
    }

    // Validate builtin call before codegen
    match crate::semantic::builtins::check_builtin_call(name, &arg_types) {
        Ok(_return_type) => {
            // Validation passed, proceed with codegen
        }
        Err(err_msg) => {
            // Convert validation error to GlslError
            let error = GlslError::new(crate::error::ErrorCode::E0114, err_msg)
                .with_location(source_span_to_location(&call_span));
            return Err(ctx.add_span_to_error(error, &call_span));
        }
    }

    // Delegate to built-in implementation and add span to any errors
    match ctx.translate_builtin_call(name, translated_args) {
        Ok(result) => Ok(result),
        Err(mut error) => {
            // Add location and span_text if not already present
            if error.location.is_none() {
                error = error.with_location(source_span_to_location(&call_span));
            }
            Err(ctx.add_span_to_error(error, &call_span))
        }
    }
}

fn translate_user_function_call(
    ctx: &mut CodegenContext,
    name: &str,
    args: &[glsl::syntax::Expr],
    call_span: glsl::syntax::SourceSpan,
) -> Result<(Vec<cranelift_codegen::ir::Value>, GlslType), GlslError> {
    // Translate arguments and collect types first (requires mutable borrow)
    let mut arg_vals_flat = Vec::new();
    let mut arg_types = Vec::new();

    for arg in args {
        let (vals, ty) = ctx.translate_expr_typed(arg)?;
        arg_vals_flat.extend(vals);
        arg_types.push(ty);
    }

    // Now get function IDs and registry (immutable borrow)
    let func_ids = ctx
        .function_ids
        .as_ref()
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, "function IDs not set (internal error)"))?;
    let func_registry = ctx.function_registry.ok_or_else(|| {
        GlslError::new(
            ErrorCode::E0400,
            "function registry not set (internal error)",
        )
    })?;

    // Lookup function signature - this will return E0114 if no match
    let func_id = func_ids.get(name).ok_or_else(|| {
        let error = GlslError::undefined_function(name);
        if error.location.is_none() {
            error.with_location(crate::error::source_span_to_location(&call_span))
        } else {
            error
        }
    })?;
    let func_sig = match func_registry.lookup_function(name, &arg_types) {
        Ok(sig) => sig,
        Err(mut error) => {
            // Ensure error has location
            if error.location.is_none() {
                error = error.with_location(crate::error::source_span_to_location(&call_span));
            }
            return Err(ctx.add_span_to_error(error, &call_span));
        }
    };

    // Validate that all arguments can be coerced BEFORE declaring the function
    // This prevents function context pollution if coercion fails
    for (param, arg_ty) in func_sig.parameters.iter().zip(&arg_types) {
        let arg_base = if arg_ty.is_vector() {
            arg_ty.vector_base_type().unwrap()
        } else {
            arg_ty.clone()
        };
        let param_base = if param.ty.is_vector() {
            param.ty.vector_base_type().unwrap()
        } else {
            param.ty.clone()
        };

        // Check if coercion is possible
        if arg_base != param_base
            && !crate::semantic::type_check::can_implicitly_convert(&arg_base, &param_base)
        {
            // Calculate expected parameter count for error message
            let expected_count: usize = func_sig
                .parameters
                .iter()
                .map(|p| {
                    if p.ty.is_vector() {
                        p.ty.component_count().unwrap()
                    } else {
                        1
                    }
                })
                .sum();
            let error = GlslError::new(
                ErrorCode::E0400,
                format!(
                    "function parameter mismatch: expected {} block parameters, got 0",
                    expected_count
                ),
            )
            .with_location(crate::error::source_span_to_location(&call_span))
            .with_note(format!(
                "function `{}` expects parameter of type `{:?}`, got `{:?}`",
                name, param.ty, arg_ty
            ));
            return Err(ctx.add_span_to_error(error, &call_span));
        }
    }

    // Import the function into the current function to get a FuncRef
    let func_ref = ctx.module.declare_func_in_func(*func_id, ctx.builder.func);

    // Get the callee signature and check if it uses StructReturn
    // Extract all needed information before any mutations
    let (uses_struct_return, buffer_size_opt, pointer_type) = {
        let ext_func_data = &ctx.builder.func.dfg.ext_funcs[func_ref];
        let callee_sig_ref = ext_func_data.signature;
        let callee_sig = &ctx.builder.func.dfg.signatures[callee_sig_ref];

        let uses_sret = callee_sig
            .params
            .iter()
            .any(|p| p.purpose == cranelift_codegen::ir::ArgumentPurpose::StructReturn);

        // Calculate buffer size if needed
        let buf_size = if uses_sret {
            let element_count = if func_sig.return_type.is_vector() {
                func_sig.return_type.component_count().unwrap()
            } else if func_sig.return_type.is_matrix() {
                func_sig.return_type.matrix_element_count().unwrap()
            } else {
                0 // Shouldn't happen
            };
            Some((element_count * 4) as u32) // 4 bytes per f32
        } else {
            None
        };

        let ptr_ty = ctx.module.isa().pointer_type();

        (uses_sret, buf_size, ptr_ty)
    };

    // If StructReturn, allocate stack slot for return buffer (now we can mutate)
    let return_buffer_ptr = if let Some(buffer_size) = buffer_size_opt {
        // Use 4-byte alignment for f32 values (align_shift: 2, since 2^2 = 4)
        // This ensures proper alignment for f32 loads/stores
        let slot =
            ctx.builder
                .func
                .create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                    cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                    buffer_size,
                    2u8, // align_shift: 2 = 4-byte alignment (for f32)
                ));
        Some(ctx.builder.ins().stack_addr(pointer_type, slot, 0))
    } else {
        None
    };

    // Type check and prepare arguments (with implicit conversions)
    // Build call args according to callee signature (which includes StructReturn if used)
    // Note: StructReturn is added FIRST in our signature builder (like cranelift-examples)
    let mut call_args = Vec::new();

    // First, add StructReturn parameter if present (it's FIRST in the signature)
    if uses_struct_return {
        if let Some(buffer_ptr) = return_buffer_ptr {
            call_args.push(buffer_ptr);
        } else {
            return Err(GlslError::new(
                ErrorCode::E0400,
                "StructReturn parameter required but no buffer allocated",
            ));
        }
    }

    // Then add all normal parameters (expanded from GLSL params)
    let mut arg_val_idx = 0;
    // Iterate through GLSL parameters and expand them to match the callee signature
    for (glsl_param_idx, param) in func_sig.parameters.iter().enumerate() {
        let arg_ty = &arg_types[glsl_param_idx];

        // Get the component count for this parameter
        let component_count = if param.ty.is_vector() {
            param.ty.component_count().unwrap()
        } else if param.ty.is_matrix() {
            param.ty.matrix_element_count().unwrap()
        } else {
            1
        };

        // Get base types for coercion
        let arg_base = if arg_ty.is_vector() {
            arg_ty.vector_base_type().unwrap()
        } else {
            arg_ty.clone()
        };
        let param_base = if param.ty.is_vector() {
            param.ty.vector_base_type().unwrap()
        } else {
            param.ty.clone()
        };

        // Process each component of this parameter
        for _ in 0..component_count {
            if arg_val_idx >= arg_vals_flat.len() {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!(
                        "Not enough argument values for parameter {}",
                        glsl_param_idx
                    ),
                ));
            }

            let arg_val = arg_vals_flat[arg_val_idx];
            let converted = coercion::coerce_to_type_with_location(
                ctx,
                arg_val,
                &arg_base,
                &param_base,
                Some(call_span.clone()),
            )?;
            call_args.push(converted);
            arg_val_idx += 1;
        }
    }

    // Make the function call
    let call_inst = ctx.builder.ins().call(func_ref, &call_args);

    // Get return values or load from buffer
    let return_vals = if uses_struct_return {
        // Load values from return buffer
        let buffer_ptr = return_buffer_ptr.unwrap();
        let element_count = if func_sig.return_type.is_vector() {
            func_sig.return_type.component_count().unwrap()
        } else if func_sig.return_type.is_matrix() {
            func_sig.return_type.matrix_element_count().unwrap()
        } else {
            return Err(GlslError::new(
                ErrorCode::E0400,
                "StructReturn used but return type is not composite",
            ));
        };
        let mut loaded_vals = Vec::new();
        for i in 0..element_count {
            let offset = (i * 4) as i32; // 4 bytes per f32
            let val = ctx.builder.ins().load(
                cranelift_codegen::ir::types::F32,
                cranelift_codegen::ir::MemFlags::trusted(),
                buffer_ptr,
                offset,
            );
            loaded_vals.push(val);
        }
        loaded_vals
    } else {
        // Get return values from call instruction
        ctx.builder.inst_results(call_inst).to_vec()
    };

    // Package return value(s)
    if func_sig.return_type == GlslType::Void {
        Ok((vec![], GlslType::Void))
    } else if func_sig.return_type.is_vector() {
        let count = func_sig.return_type.component_count().unwrap();
        Ok((return_vals[0..count].to_vec(), func_sig.return_type.clone()))
    } else if func_sig.return_type.is_matrix() {
        let count = func_sig.return_type.matrix_element_count().unwrap();
        Ok((return_vals[0..count].to_vec(), func_sig.return_type.clone()))
    } else {
        Ok((vec![return_vals[0]], func_sig.return_type.clone()))
    }
}
