//! JIT-compiled GLSL module implementation
//!
//! This module provides the JIT execution backend for GLSL functions.

use crate::error::GlslError;
use crate::exec::executable::GlslExecutable;
use crate::exec::glsl_value::GlslValue;
use crate::frontend::semantic::functions::FunctionSignature;
use cranelift_codegen::ir::types;
use hashbrown::HashMap;
use lp_jit_util::call_structreturn;

use alloc::{format, string::String, vec::Vec};

/// JIT-compiled GLSL module (executes on host or embedded)
/// Works in both std and no_std (JITModule supports no_std)
pub struct GlslJitModule {
    #[allow(dead_code)]
    pub(crate) jit_module: cranelift_jit::JITModule,
    pub(crate) function_ptrs: HashMap<String, *const u8>,
    pub(crate) signatures: HashMap<String, FunctionSignature>,
    // Store Cranelift signatures for proper function calling with arguments
    pub(crate) cranelift_signatures: HashMap<String, cranelift_codegen::ir::Signature>,
    pub(crate) call_conv: cranelift_codegen::isa::CallConv,
    pub(crate) pointer_type: cranelift_codegen::ir::Type,
}

impl GlslJitModule {
    /// Validate that only "main" function is being called
    fn validate_main_only(&self, name: &str) -> Result<(), GlslError> {
        use crate::error::ErrorCode;
        if name != "main" {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!("Only 'main' function is supported, got '{}'", name),
            ));
        }
        Ok(())
    }

    /// Validate that no arguments are provided (for methods that don't support args yet)
    fn validate_no_args(&self, args: &[GlslValue], method_name: &str) -> Result<(), GlslError> {
        use crate::error::ErrorCode;
        if !args.is_empty() {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "{}: functions with arguments not yet supported (got {} args)",
                    method_name,
                    args.len()
                ),
            ));
        }
        Ok(())
    }

    // Helper to convert GlslValue to calling convention arguments for JIT
    // This is a simplified version - full implementation would need platform-specific code
    fn glsl_value_to_jit_args(
        &self,
        value: &GlslValue,
        sig: &cranelift_codegen::ir::Signature,
        arg_idx: &mut usize,
    ) -> Result<Vec<u64>, GlslError> {
        use crate::error::ErrorCode;

        let mut args = Vec::new();

        match value {
            GlslValue::I32(v) => {
                if *arg_idx >= sig.params.len() {
                    return Err(GlslError::new(ErrorCode::E0400, "Too many arguments"));
                }
                let param_ty = sig.params[*arg_idx].value_type;
                match param_ty {
                    types::I32 => args.push(*v as u64),
                    types::I64 => args.push(*v as u64), // Sign-extend
                    _ => {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            format!("Type mismatch: expected {:?}, got I32", param_ty),
                        ));
                    }
                }
                *arg_idx += 1;
            }
            GlslValue::F32(v) => {
                if *arg_idx >= sig.params.len() {
                    return Err(GlslError::new(ErrorCode::E0400, "Too many arguments"));
                }
                let param_ty = sig.params[*arg_idx].value_type;
                match param_ty {
                    types::F32 => args.push(v.to_bits() as u64),
                    types::I32 => args.push(*v as i32 as u64), // Fixed-point
                    _ => {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            format!("Type mismatch: expected {:?}, got F32", param_ty),
                        ));
                    }
                }
                *arg_idx += 1;
            }
            GlslValue::Bool(v) => {
                if *arg_idx >= sig.params.len() {
                    return Err(GlslError::new(ErrorCode::E0400, "Too many arguments"));
                }
                let param_ty = sig.params[*arg_idx].value_type;
                match param_ty {
                    types::I8 => args.push(if *v { 1 } else { 0 } as u64),
                    types::I32 => args.push(if *v { 1 } else { 0 } as u64),
                    _ => {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            format!("Type mismatch: expected {:?}, got Bool", param_ty),
                        ));
                    }
                }
                *arg_idx += 1;
            }
            // Vectors and matrices need special handling - for now, return error
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    "Vector and matrix arguments not yet supported in JIT calls",
                ));
            }
        }

        Ok(args)
    }
}

impl GlslExecutable for GlslJitModule {
    fn call_void(&mut self, name: &str, args: &[GlslValue]) -> Result<(), GlslError> {
        use crate::error::ErrorCode;

        self.validate_main_only(name)?;

        let func_ptr = self.function_ptrs.get(name).ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, format!("Function '{}' not found", name))
        })?;

        // Get the actual Cranelift signature for this function
        let sig = self.cranelift_signatures.get(name).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0101,
                format!("Function signature for '{}' not found", name),
            )
        })?;

        // Handle no-argument case (most common)
        if args.is_empty() {
            let func: unsafe extern "C" fn() = unsafe { core::mem::transmute(*func_ptr) };
            unsafe { func() };
            return Ok(());
        }

        // For now, only support simple cases (1-2 scalar arguments)
        // Full implementation would need platform-specific inline assembly
        if args.len() > 2 {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "JIT calls with more than 2 arguments not yet supported (got {} args)",
                    args.len()
                ),
            ));
        }

        // Convert arguments
        let mut arg_idx = 0;
        let mut jit_args = Vec::new();
        for arg in args {
            jit_args.extend(self.glsl_value_to_jit_args(arg, sig, &mut arg_idx)?);
        }

        // Call based on number of arguments
        match jit_args.len() {
            0 => {
                let func: unsafe extern "C" fn() = unsafe { core::mem::transmute(*func_ptr) };
                unsafe { func() };
            }
            1 => {
                let func: unsafe extern "C" fn(u64) = unsafe { core::mem::transmute(*func_ptr) };
                unsafe { func(jit_args[0]) };
            }
            2 => {
                let func: unsafe extern "C" fn(u64, u64) =
                    unsafe { core::mem::transmute(*func_ptr) };
                unsafe { func(jit_args[0], jit_args[1]) };
            }
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!("Too many arguments for simple JIT call: {}", jit_args.len()),
                ));
            }
        }

        Ok(())
    }

    fn call_i32(&mut self, name: &str, args: &[GlslValue]) -> Result<i32, GlslError> {
        use crate::error::ErrorCode;

        self.validate_main_only(name)?;

        let func_ptr = self.function_ptrs.get(name).ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, format!("Function '{}' not found", name))
        })?;

        // Get the actual Cranelift signature for this function
        let sig = self.cranelift_signatures.get(name).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0101,
                format!("Function signature for '{}' not found", name),
            )
        })?;

        // Handle no-argument case
        if args.is_empty() {
            let func: unsafe extern "C" fn() -> i32 = unsafe { core::mem::transmute(*func_ptr) };
            return Ok(unsafe { func() });
        }

        // For now, only support simple cases (1-2 scalar arguments)
        if args.len() > 2 {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "JIT calls with more than 2 arguments not yet supported (got {} args)",
                    args.len()
                ),
            ));
        }

        // Convert arguments
        let mut arg_idx = 0;
        let mut jit_args = Vec::new();
        for arg in args {
            jit_args.extend(self.glsl_value_to_jit_args(arg, sig, &mut arg_idx)?);
        }

        // Call based on number of arguments
        let result = match jit_args.len() {
            1 => {
                let func: unsafe extern "C" fn(u64) -> i32 =
                    unsafe { core::mem::transmute(*func_ptr) };
                unsafe { func(jit_args[0]) }
            }
            2 => {
                let func: unsafe extern "C" fn(u64, u64) -> i32 =
                    unsafe { core::mem::transmute(*func_ptr) };
                unsafe { func(jit_args[0], jit_args[1]) }
            }
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!("Too many arguments for simple JIT call: {}", jit_args.len()),
                ));
            }
        };

        Ok(result)
    }

    fn call_f32(&mut self, name: &str, args: &[GlslValue]) -> Result<f32, GlslError> {
        use crate::error::ErrorCode;

        self.validate_main_only(name)?;

        let func_ptr = self.function_ptrs.get(name).ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, format!("Function '{}' not found", name))
        })?;

        // Get the actual Cranelift signature for this function
        let sig = self.cranelift_signatures.get(name).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0101,
                format!("Function signature for '{}' not found", name),
            )
        })?;

        // Check return type: I32 means fixed-point, F32 means native float
        let return_type = sig
            .returns
            .first()
            .map(|r| r.value_type)
            .unwrap_or(types::F32);

        // Handle no-argument case
        if args.is_empty() {
            if return_type == types::I32 {
                // Fixed-point: call as i32, convert to f32
                let func: unsafe extern "C" fn() -> i32 =
                    unsafe { core::mem::transmute(*func_ptr) };
                let fixed_result = unsafe { func() };
                return Ok(fixed_result as f32 / 65536.0);
            } else {
                // Native float: call as f32
                let func: unsafe extern "C" fn() -> f32 =
                    unsafe { core::mem::transmute(*func_ptr) };
                return Ok(unsafe { func() });
            }
        }

        // For now, only support simple cases (1-2 scalar arguments)
        if args.len() > 2 {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "JIT calls with more than 2 arguments not yet supported (got {} args)",
                    args.len()
                ),
            ));
        }

        // Convert arguments
        let mut arg_idx = 0;
        let mut jit_args = Vec::new();
        for arg in args {
            jit_args.extend(self.glsl_value_to_jit_args(arg, sig, &mut arg_idx)?);
        }

        // Call based on number of arguments and return type
        let result = if return_type == types::I32 {
            // Fixed-point: call as i32, convert to f32
            match jit_args.len() {
                1 => {
                    let func: unsafe extern "C" fn(u64) -> i32 =
                        unsafe { core::mem::transmute(*func_ptr) };
                    let fixed_result = unsafe { func(jit_args[0]) };
                    fixed_result as f32 / 65536.0
                }
                2 => {
                    let func: unsafe extern "C" fn(u64, u64) -> i32 =
                        unsafe { core::mem::transmute(*func_ptr) };
                    let fixed_result = unsafe { func(jit_args[0], jit_args[1]) };
                    fixed_result as f32 / 65536.0
                }
                _ => {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!("Too many arguments for simple JIT call: {}", jit_args.len()),
                    ));
                }
            }
        } else {
            // Native float: call as f32
            match jit_args.len() {
                1 => {
                    let func: unsafe extern "C" fn(u64) -> f32 =
                        unsafe { core::mem::transmute(*func_ptr) };
                    unsafe { func(jit_args[0]) }
                }
                2 => {
                    let func: unsafe extern "C" fn(u64, u64) -> f32 =
                        unsafe { core::mem::transmute(*func_ptr) };
                    unsafe { func(jit_args[0], jit_args[1]) }
                }
                _ => {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!("Too many arguments for simple JIT call: {}", jit_args.len()),
                    ));
                }
            }
        };

        Ok(result)
    }

    fn call_bool(&mut self, name: &str, args: &[GlslValue]) -> Result<bool, GlslError> {
        use crate::error::ErrorCode;

        self.validate_main_only(name)?;

        let func_ptr = self.function_ptrs.get(name).ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, format!("Function '{}' not found", name))
        })?;

        // Get the actual Cranelift signature for this function
        let sig = self.cranelift_signatures.get(name).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0101,
                format!("Function signature for '{}' not found", name),
            )
        })?;

        // Handle no-argument case
        if args.is_empty() {
            let func: unsafe extern "C" fn() -> i8 = unsafe { core::mem::transmute(*func_ptr) };
            return Ok(unsafe { func() != 0 });
        }

        // For now, only support simple cases (1-2 scalar arguments)
        if args.len() > 2 {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "JIT calls with more than 2 arguments not yet supported (got {} args)",
                    args.len()
                ),
            ));
        }

        // Convert arguments
        let mut arg_idx = 0;
        let mut jit_args = Vec::new();
        for arg in args {
            jit_args.extend(self.glsl_value_to_jit_args(arg, sig, &mut arg_idx)?);
        }

        // Call based on number of arguments
        let result = match jit_args.len() {
            1 => {
                let func: unsafe extern "C" fn(u64) -> i8 =
                    unsafe { core::mem::transmute(*func_ptr) };
                unsafe { func(jit_args[0]) }
            }
            2 => {
                let func: unsafe extern "C" fn(u64, u64) -> i8 =
                    unsafe { core::mem::transmute(*func_ptr) };
                unsafe { func(jit_args[0], jit_args[1]) }
            }
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!("Too many arguments for simple JIT call: {}", jit_args.len()),
                ));
            }
        };

        Ok(result != 0)
    }

    fn call_vec(
        &mut self,
        name: &str,
        args: &[GlslValue],
        dim: usize,
    ) -> Result<Vec<f32>, GlslError> {
        use crate::error::ErrorCode;

        self.validate_main_only(name)?;
        self.validate_no_args(args, "call_vec")?;

        let func_ptr = self.function_ptrs.get(name).ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, format!("Function '{}' not found", name))
        })?;

        // Use struct return for vectors (multiple f32s returned via pointer)
        // Calculate buffer size in bytes
        let buffer_size = dim * core::mem::size_of::<f32>();
        let mut buffer = vec![0.0f32; dim];
        unsafe {
            call_structreturn(
                *func_ptr,
                buffer.as_mut_ptr(),
                buffer_size,
                self.call_conv,
                self.pointer_type,
            )
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("StructReturn call failed for vec{}: {}", dim, e),
                )
            })?;
        }
        Ok(buffer)
    }

    fn call_mat(
        &mut self,
        name: &str,
        args: &[GlslValue],
        rows: usize,
        cols: usize,
    ) -> Result<Vec<f32>, GlslError> {
        use crate::error::ErrorCode;

        self.validate_main_only(name)?;
        self.validate_no_args(args, "call_mat")?;

        let func_ptr = self.function_ptrs.get(name).ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, format!("Function '{}' not found", name))
        })?;

        // Use struct return for matrices (column-major, rows*cols f32s)
        let count = rows * cols;
        let buffer_size = count * core::mem::size_of::<f32>();
        let mut buffer = vec![0.0f32; count];
        unsafe {
            call_structreturn(
                *func_ptr,
                buffer.as_mut_ptr(),
                buffer_size,
                self.call_conv,
                self.pointer_type,
            )
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("StructReturn call failed for mat{}x{}: {}", rows, cols, e),
                )
            })?;
        }
        Ok(buffer)
    }

    fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.signatures.get(name)
    }

    fn list_functions(&self) -> Vec<String> {
        self.signatures.keys().cloned().collect()
    }
}
