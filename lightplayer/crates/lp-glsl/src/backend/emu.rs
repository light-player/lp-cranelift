//! Emulator-based GLSL module implementation
//!
//! This module provides the RISC-V 32-bit emulator execution backend for GLSL functions.
//! Requires `emulator` feature flag to be enabled.

use crate::backend::executable::GlslExecutable;
use crate::backend::glsl_value::GlslValue;
use crate::error::GlslError;
use crate::semantic::functions::FunctionSignature;
use hashbrown::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

/// Emulator-based GLSL module (executes in RISC-V emulator)
/// Requires `emulator` feature flag to be enabled
/// Currently only supports calling `main` with no arguments
#[cfg(feature = "emulator")]
pub struct GlslEmulatorModule {
    pub(crate) emulator: lp_riscv_tools::emu::emulator::Riscv32Emulator,
    pub(crate) signatures: HashMap<String, FunctionSignature>,
    // Store Cranelift signatures for proper function calling with arguments
    pub(crate) cranelift_signatures: HashMap<String, cranelift_codegen::ir::Signature>,
    pub(crate) binary: Vec<u8>,
    pub(crate) main_address: u32,
}

// Helper function to convert GlslValue to DataValue
#[cfg(feature = "emulator")]
fn glsl_value_to_data_value(value: &GlslValue) -> Vec<cranelift_codegen::data_value::DataValue> {
    use cranelift_codegen::data_value::DataValue;
    match value {
        GlslValue::I32(v) => vec![DataValue::I32(*v)],
        GlslValue::F32(v) => {
            // For fixed-point, F32 is represented as I32
            // TODO: Handle actual float when emulator supports it
            vec![DataValue::I32(*v as i32)]
        }
        GlslValue::Bool(v) => vec![DataValue::I8(if *v { 1 } else { 0 })],
        GlslValue::Vec2(v) => v.iter().map(|&f| DataValue::I32(f as i32)).collect(),
        GlslValue::Vec3(v) => v.iter().map(|&f| DataValue::I32(f as i32)).collect(),
        GlslValue::Vec4(v) => v.iter().map(|&f| DataValue::I32(f as i32)).collect(),
        GlslValue::Mat2x2(m) => m
            .iter()
            .flatten()
            .map(|&f| DataValue::I32(f as i32))
            .collect(),
        GlslValue::Mat3x3(m) => m
            .iter()
            .flatten()
            .map(|&f| DataValue::I32(f as i32))
            .collect(),
        GlslValue::Mat4x4(m) => m
            .iter()
            .flatten()
            .map(|&f| DataValue::I32(f as i32))
            .collect(),
    }
}

#[cfg(feature = "emulator")]
impl GlslEmulatorModule {
    /// Validate that only "main" function is being called
    fn validate_main_only(name: &str) -> Result<(), GlslError> {
        use crate::error::ErrorCode;
        if name != "main" {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!("Only 'main' function is supported, got '{}'", name),
            ));
        }
        Ok(())
    }

    /// Validate that no arguments are provided (panics for emulator)
    fn validate_no_args(args: &[GlslValue]) {
        if !args.is_empty() {
            panic!(
                "Emulator only supports calling main with no arguments (got {} args)",
                args.len()
            );
        }
    }
}

#[cfg(feature = "emulator")]
impl GlslExecutable for GlslEmulatorModule {
    fn call_void(&mut self, name: &str, args: &[GlslValue]) -> Result<(), GlslError> {
        use crate::error::ErrorCode;

        Self::validate_main_only(name)?;
        Self::validate_no_args(args);

        // Main is always at address 0x00
        const MAIN_ENTRY: u32 = 0x00;

        // Get the actual Cranelift signature for main
        let sig = self.cranelift_signatures.get("main").ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, "Function signature for 'main' not found")
        })?;

        // Call main via emulator (no arguments)
        let _results = self
            .emulator
            .call_function(MAIN_ENTRY, &[], sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Emulator execution failed: {}", e),
                )
            })?;

        Ok(())
    }

    fn call_i32(&mut self, name: &str, args: &[GlslValue]) -> Result<i32, GlslError> {
        use crate::error::ErrorCode;

        Self::validate_main_only(name)?;
        Self::validate_no_args(args);

        // Get the actual Cranelift signature for main
        let sig = self.cranelift_signatures.get("main").ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, "Function signature for 'main' not found")
        })?;

        // Call main via emulator (no arguments) at its actual address
        let results = self
            .emulator
            .call_function(self.main_address, &[], sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Emulator execution failed: {}", e),
                )
            })?;

        // Extract i32 return value
        match results.first() {
            Some(cranelift_codegen::data_value::DataValue::I32(v)) => Ok(*v),
            _ => Err(GlslError::new(
                ErrorCode::E0400,
                "Expected i32 return value",
            )),
        }
    }

    fn call_f32(&mut self, name: &str, args: &[GlslValue]) -> Result<f32, GlslError> {
        use crate::error::ErrorCode;

        Self::validate_main_only(name)?;
        Self::validate_no_args(args);

        // Get the actual Cranelift signature for main
        let sig = self.cranelift_signatures.get("main").ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, "Function signature for 'main' not found")
        })?;

        // Call main via emulator (no arguments) at its actual address
        let results = self
            .emulator
            .call_function(self.main_address, &[], sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Emulator execution failed: {}", e),
                )
            })?;

        // Extract i32 return value and convert from fixed-point
        match results.first() {
            Some(cranelift_codegen::data_value::DataValue::I32(v)) => {
                // TODO: Convert from fixed-point to f32 when needed
                // For now, treat as fixed-point
                Ok(*v as f32 / crate::codegen::constants::FIXED16X16_SCALE)
            }
            _ => Err(GlslError::new(
                ErrorCode::E0400,
                "Expected i32 return value",
            )),
        }
    }

    fn call_bool(&mut self, name: &str, args: &[GlslValue]) -> Result<bool, GlslError> {
        use crate::error::ErrorCode;

        Self::validate_main_only(name)?;
        Self::validate_no_args(args);

        // Get the actual Cranelift signature for main
        let sig = self.cranelift_signatures.get("main").ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, "Function signature for 'main' not found")
        })?;

        // Call main via emulator (no arguments) at its actual address
        let results = self
            .emulator
            .call_function(self.main_address, &[], sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Emulator execution failed: {}", e),
                )
            })?;

        // Extract i8 return value
        match results.first() {
            Some(cranelift_codegen::data_value::DataValue::I8(v)) => Ok(*v != 0),
            _ => Err(GlslError::new(ErrorCode::E0400, "Expected i8 return value")),
        }
    }

    fn call_vec(
        &mut self,
        name: &str,
        args: &[GlslValue],
        dim: usize,
    ) -> Result<Vec<f32>, GlslError> {
        use crate::error::ErrorCode;

        Self::validate_main_only(name)?;
        Self::validate_no_args(args);

        // Get the actual Cranelift signature for main
        let sig = self.cranelift_signatures.get("main").ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, "Function signature for 'main' not found")
        })?;

        // Call main via emulator (no arguments) at its actual address
        let results = self
            .emulator
            .call_function(self.main_address, &[], sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Emulator execution failed: {}", e),
                )
            })?;

        // Convert results from fixed-point i32 to f32
        let mut vec_result = Vec::with_capacity(dim);
        for result in results.iter().take(dim) {
            match result {
                cranelift_codegen::data_value::DataValue::I32(v) => {
                    vec_result.push(*v as f32 / crate::codegen::constants::FIXED16X16_SCALE)
                }
                _ => {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        "Expected i32 return values",
                    ));
                }
            }
        }
        Ok(vec_result)
    }

    fn call_mat(
        &mut self,
        name: &str,
        args: &[GlslValue],
        rows: usize,
        cols: usize,
    ) -> Result<Vec<f32>, GlslError> {
        use crate::error::ErrorCode;

        Self::validate_main_only(name)?;
        Self::validate_no_args(args);

        // Get the actual Cranelift signature for main
        let sig = self.cranelift_signatures.get("main").ok_or_else(|| {
            GlslError::new(ErrorCode::E0101, "Function signature for 'main' not found")
        })?;

        // Call main via emulator (no arguments) at its actual address
        let results = self
            .emulator
            .call_function(self.main_address, &[], sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Emulator execution failed: {}", e),
                )
            })?;

        // Convert results from fixed-point i32 to f32
        let count = rows * cols;
        let mut mat_result = Vec::with_capacity(count);
        for result in results.iter().take(count) {
            match result {
                cranelift_codegen::data_value::DataValue::I32(v) => {
                    mat_result.push(*v as f32 / crate::codegen::constants::FIXED16X16_SCALE)
                }
                _ => {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        "Expected i32 return values",
                    ));
                }
            }
        }
        Ok(mat_result)
    }

    fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.signatures.get(name)
    }

    fn list_functions(&self) -> Vec<String> {
        self.signatures.keys().cloned().collect()
    }

    #[cfg(feature = "std")]
    fn format_emulator_state(&self) -> Option<String> {
        let state_dump = self.emulator.dump_state();
        let debug_info = self.emulator.format_debug_info(None, 100);
        Some(format!(
            "\n=== Emulator State ===\n{}\n\n=== Debug Info ===\n{}",
            state_dump, debug_info
        ))
    }
}
