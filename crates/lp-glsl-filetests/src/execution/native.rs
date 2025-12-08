//! Native JIT execution backend

use super::backend::{CompiledCode, ExecutionBackend};
use anyhow::Result;
use lp_glsl::FixedPointFormat;
use lp_jit_util::call_structreturn;

/// Native JIT execution backend (uses existing JIT compilation)
pub struct NativeJitBackend;

impl ExecutionBackend for NativeJitBackend {
    fn execute_float(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<f32> {
        match code {
            CompiledCode::NativeJit { code_ptr, .. } => {
                if let Some(format) = fixed_point_format {
                    match format {
                        lp_glsl::FixedPointFormat::Fixed16x16 => {
                            let func: fn() -> i32 = unsafe { std::mem::transmute(*code_ptr) };
                            Ok(func() as f32 / 65536.0)
                        }
                        lp_glsl::FixedPointFormat::Fixed32x32 => {
                            let func: fn() -> i64 = unsafe { std::mem::transmute(*code_ptr) };
                            Ok((func() as f64 / 4294967296.0) as f32)
                        }
                    }
                } else {
                    let func: fn() -> f32 = unsafe { std::mem::transmute(*code_ptr) };
                    Ok(func())
                }
            }
            _ => anyhow::bail!("NativeJitBackend requires NativeJit compiled code"),
        }
    }

    fn execute_int(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i32> {
        match code {
            CompiledCode::NativeJit { code_ptr, .. } => {
                let func: fn() -> i32 = unsafe { std::mem::transmute(*code_ptr) };
                Ok(func())
            }
            _ => anyhow::bail!("NativeJitBackend requires NativeJit compiled code"),
        }
    }

    fn execute_i64(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i64> {
        match code {
            CompiledCode::NativeJit { code_ptr, .. } => {
                let func: fn() -> i64 = unsafe { std::mem::transmute(*code_ptr) };
                Ok(func())
            }
            _ => anyhow::bail!("NativeJitBackend requires NativeJit compiled code"),
        }
    }

    fn execute_bool(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i8> {
        match code {
            CompiledCode::NativeJit { code_ptr, .. } => {
                let func: fn() -> i8 = unsafe { std::mem::transmute(*code_ptr) };
                Ok(func())
            }
            _ => anyhow::bail!("NativeJitBackend requires NativeJit compiled code"),
        }
    }

    fn execute_vec2(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32)> {
        match code {
            CompiledCode::NativeJit {
                code_ptr,
                return_type: _,
            } => {
                use cranelift_codegen::settings;
                let call_conv = cranelift_codegen::isa::CallConv::triple_default(
                    cranelift_native::builder()
                        .unwrap()
                        .finish(cranelift_codegen::settings::Flags::new(settings::builder()))
                        .unwrap()
                        .triple(),
                );
                let pointer_type = cranelift_codegen::ir::types::I64; // Assume 64-bit host

                let mut buffer = [0.0f32; 2];
                unsafe {
                    call_structreturn(*code_ptr, buffer.as_mut_ptr(), 2, call_conv, pointer_type)?;
                }
                Ok((buffer[0], buffer[1]))
            }
            _ => anyhow::bail!("NativeJitBackend requires NativeJit compiled code"),
        }
    }

    fn execute_vec3(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32)> {
        match code {
            CompiledCode::NativeJit { code_ptr, .. } => {
                use cranelift_codegen::settings;
                let call_conv = cranelift_codegen::isa::CallConv::triple_default(
                    cranelift_native::builder()
                        .unwrap()
                        .finish(cranelift_codegen::settings::Flags::new(settings::builder()))
                        .unwrap()
                        .triple(),
                );
                let pointer_type = cranelift_codegen::ir::types::I64;

                let mut buffer = [0.0f32; 3];
                unsafe {
                    call_structreturn(*code_ptr, buffer.as_mut_ptr(), 3, call_conv, pointer_type)?;
                }
                Ok((buffer[0], buffer[1], buffer[2]))
            }
            _ => anyhow::bail!("NativeJitBackend requires NativeJit compiled code"),
        }
    }

    fn execute_vec4(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32)> {
        match code {
            CompiledCode::NativeJit { code_ptr, .. } => {
                use cranelift_codegen::settings;
                let call_conv = cranelift_codegen::isa::CallConv::triple_default(
                    cranelift_native::builder()
                        .unwrap()
                        .finish(cranelift_codegen::settings::Flags::new(settings::builder()))
                        .unwrap()
                        .triple(),
                );
                let pointer_type = cranelift_codegen::ir::types::I64;

                let mut buffer = [0.0f32; 4];
                unsafe {
                    call_structreturn(*code_ptr, buffer.as_mut_ptr(), 4, call_conv, pointer_type)?;
                }
                Ok((buffer[0], buffer[1], buffer[2], buffer[3]))
            }
            _ => anyhow::bail!("NativeJitBackend requires NativeJit compiled code"),
        }
    }

    fn execute_mat2(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32)> {
        self.execute_vec4(code, _fixed_point_format)
    }

    fn execute_mat3(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32, f32, f32, f32, f32, f32)> {
        match code {
            CompiledCode::NativeJit { code_ptr, .. } => {
                use cranelift_codegen::settings;
                let call_conv = cranelift_codegen::isa::CallConv::triple_default(
                    cranelift_native::builder()
                        .unwrap()
                        .finish(cranelift_codegen::settings::Flags::new(settings::builder()))
                        .unwrap()
                        .triple(),
                );
                let pointer_type = cranelift_codegen::ir::types::I64;

                let mut buffer = [0.0f32; 9];
                unsafe {
                    call_structreturn(*code_ptr, buffer.as_mut_ptr(), 9, call_conv, pointer_type)?;
                }
                Ok((
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7], buffer[8],
                ))
            }
            _ => anyhow::bail!("NativeJitBackend requires NativeJit compiled code"),
        }
    }

    fn execute_mat4(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
    )> {
        match code {
            CompiledCode::NativeJit { code_ptr, .. } => {
                use cranelift_codegen::settings;
                let call_conv = cranelift_codegen::isa::CallConv::triple_default(
                    cranelift_native::builder()
                        .unwrap()
                        .finish(cranelift_codegen::settings::Flags::new(settings::builder()))
                        .unwrap()
                        .triple(),
                );
                let pointer_type = cranelift_codegen::ir::types::I64;

                let mut buffer = [0.0f32; 16];
                unsafe {
                    call_structreturn(*code_ptr, buffer.as_mut_ptr(), 16, call_conv, pointer_type)?;
                }
                Ok((
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12],
                    buffer[13], buffer[14], buffer[15],
                ))
            }
            _ => anyhow::bail!("NativeJitBackend requires NativeJit compiled code"),
        }
    }
}
