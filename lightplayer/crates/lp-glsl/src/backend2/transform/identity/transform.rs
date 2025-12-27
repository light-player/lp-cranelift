//! Identity transform implementation

use crate::backend2::transform::pipeline::{Transform, TransformContext};
use crate::backend2::transform::shared::{copy_instruction, transform_function_body};
use crate::error::GlslError;
use cranelift_codegen::ir::{Function, Signature};

/// Identity transform - copies functions exactly without modification
pub struct IdentityTransform;

impl Transform for IdentityTransform {
    fn transform_signature(&self, sig: &Signature) -> Signature {
        sig.clone()
    }

    fn transform_function<M: cranelift_module::Module>(
        &self,
        old_func: &Function,
        _ctx: &mut TransformContext<'_, M>,
    ) -> Result<Function, GlslError> {
        // Get transformed signature
        let new_sig = self.transform_signature(&old_func.signature);

        // Use shared transform function with identity callbacks
        transform_function_body(
            old_func,
            new_sig,
            // Instruction transformation: copy instructions exactly
            |old_func, old_inst, builder, value_map, stack_slot_map, block_map| {
                copy_instruction(
                    old_func,
                    old_inst,
                    builder,
                    value_map,
                    stack_slot_map,
                    block_map,
                )
            },
            // Type mapping: identity (no conversion)
            |t| t,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend2::transform::pipeline::TransformContext;
    use alloc::string::String;
    use alloc::vec::Vec;
    use cranelift_codegen::write_function;
    use cranelift_reader::parse_functions;

    /// Normalize CLIF strings for comparison
    fn normalize_clif(clif: &str) -> String {
        clif.lines()
            .map(|line| {
                let line = if let Some(comment_pos) = line.find(';') {
                    &line[..comment_pos]
                } else {
                    line
                };
                line.trim()
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Parse CLIF input, transform it, and return CLIF strings for comparison
    fn parse_and_transform(clif_input: &str) -> (String, String) {
        // Parse the CLIF IR
        let functions = parse_functions(clif_input).expect("Failed to parse CLIF IR");
        assert_eq!(functions.len(), 1, "Expected exactly one function");
        let original_func = functions.into_iter().next().unwrap();
        let original_func_clone = original_func.clone();

        // Format the parsed function (before transformation)
        let mut parsed_buf = String::new();
        write_function(&mut parsed_buf, &original_func_clone).unwrap();

        // Transform the function directly
        let transform = IdentityTransform;
        // Create a dummy context (not used for identity transform)
        let mut dummy_module =
            crate::backend2::module::gl_module::GlModule::<cranelift_jit::JITModule>::new_jit(
                crate::backend2::target::Target::host_jit().unwrap(),
            )
            .unwrap();
        let mut transform_ctx = TransformContext {
            module: &mut dummy_module,
            func_ref_map: hashbrown::HashMap::new(),
        };
        let transformed_func = transform
            .transform_function(&original_func, &mut transform_ctx)
            .expect("Failed to apply identity transform");

        // Format the transformed function
        let mut transformed_buf = String::new();
        write_function(&mut transformed_buf, &transformed_func).unwrap();

        (parsed_buf, transformed_buf)
    }

    /// Assert that identity transform produces identical CLIF output
    fn assert_identity_transform(message: &str, clif_input: &str) {
        let (parsed_buf, transformed_buf) = parse_and_transform(clif_input);

        let normalized_parsed = normalize_clif(&parsed_buf);
        let normalized_transformed = normalize_clif(&transformed_buf);

        assert_eq!(
            normalized_parsed, normalized_transformed,
            "{}\n\
         PARSED:\n{}\n\n\
         TRANSFORMED:\n{}",
            message, parsed_buf, transformed_buf
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_identity_transform_simple() {
        assert_identity_transform(
            "Identity transform should produce identical CLIF",
            r#"
function %add(i32, i32) -> i32 system_v {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}
"#,
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_identity_transform_block_order() {
        assert_identity_transform(
            "Identity transform should preserve block order",
            r#"
function %test(i32) -> i32 system_v {
block0(v0: i32):
    jump block1

block1:
    jump block2

block2:
    return v0
}
"#,
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_identity_transform_block_params() {
        assert_identity_transform(
            "Identity transform should preserve block parameters",
            r#"
function %test(i32) -> i32 system_v {
block0(v0: i32):
    jump block1(v0)

block1(v1: i32):
    return v1
}
"#,
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_identity_transform_stack_slots() {
        assert_identity_transform(
            "Identity transform should preserve stack slots",
            r#"
function %test(i32) -> i32 system_v {
ss0 = explicit_slot 4, align = 4
block0(v0: i32):
    return v0
}
"#,
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_identity_transform_multi_function() {
        // Test with multiple functions - parse each separately
        assert_identity_transform(
            "Identity transform should preserve add function",
            r#"
function %add(i32, i32) -> i32 system_v {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}
"#,
        );

        assert_identity_transform(
            "Identity transform should preserve multiply function",
            r#"
function %multiply(i32, i32) -> i32 system_v {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}
"#,
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_complex_clif() {
        // Test with multiple functions - parse each separately
        assert_identity_transform(
            "Identity transform should preserve add function",
            r#"
function %test_continue_do_while_loop_after_first() -> i32 system_v {
block0:
    v0 = iconst.i32 0
    v1 = iconst.i32 0
    jump block1(v0, v1)  ; v0 = 0, v1 = 0

block1(v2: i32, v3: i32):
    v4 = iadd v2, v3
    v5 = iconst.i32 1
    v6 = iadd v3, v5  ; v5 = 1
    v7 = iconst.i32 2
    v8 = icmp sge v6, v7  ; v7 = 2
    v9 = iconst.i8 1
    v10 = iconst.i8 0
    v11 = select v8, v9, v10  ; v9 = 1, v10 = 0
    brif v11, block4, block5(v6, v4)

block4:
    jump block2(v6, v4)

block6:
    v16 = iconst.i32 0
    v17 = iconst.i32 0
    jump block5(v17, v16)  ; v17 = 0, v16 = 0

block5(v12: i32, v13: i32):
    jump block2(v12, v13)

block2(v14: i32, v15: i32):
    v18 = iconst.i32 5
    v19 = icmp slt v14, v18  ; v18 = 5
    v20 = iconst.i8 1
    v21 = iconst.i8 0
    v22 = select v19, v20, v21  ; v20 = 1, v21 = 0
    brif v22, block1(v15, v14), block3

block3:
    return v15

block7:
    v23 = iconst.i32 0
    return v23  ; v23 = 0
}
"#,
        );
    }
}
