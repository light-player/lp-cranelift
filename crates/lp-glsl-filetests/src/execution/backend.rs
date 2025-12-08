//! Execution backend trait definition

use anyhow::Result;
use lp_glsl::FixedPointFormat;

/// Compiled code ready for execution
pub enum CompiledCode {
    /// Native JIT function pointer (for host execution)
    NativeJit {
        code_ptr: *const u8,
        return_type: ReturnType,
    },
    /// Binary code for emulator execution
    EmulatorBinary {
        binary: Vec<u8>,
        return_type: ReturnType,
    },
}

/// Return type of the compiled function
#[derive(Debug, Clone, Copy)]
pub enum ReturnType {
    Int,
    Bool,
    Float,
    I64,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

/// Trait for execution backends
pub trait ExecutionBackend {
    /// Execute code that returns a float
    fn execute_float(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<f32>;

    /// Execute code that returns an i32
    fn execute_int(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i32>;

    /// Execute code that returns an i64
    fn execute_i64(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i64>;

    /// Execute code that returns a bool
    fn execute_bool(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i8>;

    /// Execute code that returns vec2
    fn execute_vec2(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32)>;

    /// Execute code that returns vec3
    fn execute_vec3(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32)>;

    /// Execute code that returns vec4
    fn execute_vec4(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32)>;

    /// Execute code that returns mat2
    fn execute_mat2(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32)>;

    /// Execute code that returns mat3
    fn execute_mat3(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32, f32, f32, f32, f32, f32)>;

    /// Execute code that returns mat4
    fn execute_mat4(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
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
    )>;
}
