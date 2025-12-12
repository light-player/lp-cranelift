//! Execution logic for running GLSL executables.

use anyhow::Result;
use lp_glsl::semantic::types::Type;
use lp_glsl::{GlslExecutable, GlslValue};

/// Execute main() and return the result as a GlslValue.
pub fn execute_main(executable: &mut dyn GlslExecutable) -> Result<GlslValue> {
    // Try to get the signature to determine return type
    let sig = executable
        .get_function_signature("main")
        .ok_or_else(|| anyhow::anyhow!("main function not found"))?;

    // Helper to add emulator state to error if available
    fn format_error(e: lp_glsl::error::GlslError, executable: &dyn GlslExecutable) -> anyhow::Error {
        let error_msg = format!("{}", e);
        if let Some(state) = executable.format_emulator_state() {
            anyhow::anyhow!("{}{}", error_msg, state)
        } else {
            anyhow::anyhow!("{}", error_msg)
        }
    }

    // Call main() based on return type
    match &sig.return_type {
        Type::Float => executable
            .call_f32("main", &[])
            .map(GlslValue::F32)
            .map_err(|e| format_error(e, executable)),
        Type::Int => executable
            .call_i32("main", &[])
            .map(GlslValue::I32)
            .map_err(|e| format_error(e, executable)),
        Type::Bool => executable
            .call_bool("main", &[])
            .map(GlslValue::Bool)
            .map_err(|e| format_error(e, executable)),
        Type::Vec2 => executable
            .call_vec("main", &[], 2)
            .map(|v| GlslValue::Vec2([v[0], v[1]]))
            .map_err(|e| format_error(e, executable)),
        Type::Vec3 => executable
            .call_vec("main", &[], 3)
            .map(|v| GlslValue::Vec3([v[0], v[1], v[2]]))
            .map_err(|e| format_error(e, executable)),
        Type::Vec4 => executable
            .call_vec("main", &[], 4)
            .map(|v| GlslValue::Vec4([v[0], v[1], v[2], v[3]]))
            .map_err(|e| format_error(e, executable)),
        Type::Mat2 => executable
            .call_mat("main", &[], 2, 2)
            .map(|v| {
                // Convert flat array to 2x2 matrix (column-major)
                GlslValue::Mat2x2([[v[0], v[2]], [v[1], v[3]]])
            })
            .map_err(|e| format_error(e, executable)),
        Type::Mat3 => executable
            .call_mat("main", &[], 3, 3)
            .map(|v| {
                // Convert flat array to 3x3 matrix (column-major)
                GlslValue::Mat3x3([[v[0], v[3], v[6]], [v[1], v[4], v[7]], [v[2], v[5], v[8]]])
            })
            .map_err(|e| format_error(e, executable)),
        Type::Mat4 => executable
            .call_mat("main", &[], 4, 4)
            .map(|v| {
                // Convert flat array to 4x4 matrix (column-major)
                GlslValue::Mat4x4([
                    [v[0], v[4], v[8], v[12]],
                    [v[1], v[5], v[9], v[13]],
                    [v[2], v[6], v[10], v[14]],
                    [v[3], v[7], v[11], v[15]],
                ])
            })
            .map_err(|e| format_error(e, executable)),
        _ => anyhow::bail!("unsupported return type: {:?}", sig.return_type),
    }
}