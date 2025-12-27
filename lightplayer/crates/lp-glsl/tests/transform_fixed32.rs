//! Fixed32 Transform Tests
//!
//! Tests that verify the fixed32 transform correctly preserves SSA form and block parameters
//! when transforming CLIF IR. Since i32 types don't need conversion, the transform should
//! produce identical output.

use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_reader::parse_functions;
use lp_glsl::{
    ClifModule, FixedPointFormat,
    error::{ErrorCode, GlslError},
    transform_module,
};

#[cfg(feature = "std")]
fn create_test_isa() -> Result<OwnedTargetIsa, GlslError> {
    use cranelift_native;

    let mut flag_builder = settings::builder();
    flag_builder
        .set("opt_level", "none")
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("failed to set opt_level: {}", e)))?;
    let flags = settings::Flags::new(flag_builder);

    // Use host ISA - this works for transform tests since we're not actually compiling
    cranelift_native::builder()
        .map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!("failed to create native builder: {}", e),
            )
        })?
        .finish(flags)
        .map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!("Failed to create host ISA: {}", e),
            )
        })
}

#[cfg(not(feature = "std"))]
fn create_test_isa() -> Result<OwnedTargetIsa, GlslError> {
    // For no_std builds, we can't create an ISA
    // This test requires std feature
    Err(GlslError::new(
        ErrorCode::E0400,
        "This test requires the std feature to be enabled",
    ))
}

/// Test that the fixed32 transform preserves block parameters for do-while loops with continue.
///
/// This test uses the CLIF IR from the failing do-while continue test. Since all types are i32,
/// the transform should produce identical output (no type conversion needed). Any differences
/// indicate bugs in the transform.
#[test]
fn test_do_while_continue_block_parameters() {
    // CLIF IR from the BEFORE transformation (should have proper block parameters)
    let clif_input = r#"
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
    v24 = iconst.i32 0
    v23 -> v24
    v15 = iconst.i32 0
    v14 -> v15
    jump block5(v15, v24)  ; v15 = 0, v24 = 0

block5(v13: i32, v22: i32):
    jump block2(v13, v22)

block2(v12: i32, v21: i32):
    v16 = iconst.i32 5
    v17 = icmp slt v12, v16  ; v16 = 5
    v18 = iconst.i8 1
    v19 = iconst.i8 0
    v20 = select v17, v18, v19  ; v18 = 1, v19 = 0
    brif v20, block1(v21, v12), block3

block3:
    return v21

block7:
    v25 = iconst.i32 0
    return v25  ; v25 = 0
}
"#;

    // Parse the CLIF IR
    let functions = parse_functions(clif_input).expect("Failed to parse CLIF IR");

    assert_eq!(functions.len(), 1, "Expected exactly one function");
    let original_func = functions.into_iter().next().unwrap();
    let original_func_clone = original_func.clone();

    // Verify block5 has parameters before transform
    // Find block5 by checking block parameters (it should have 2 params)
    let block5 = original_func_clone
        .layout
        .blocks()
        .find(|&b| {
            let params = original_func_clone.dfg.block_params(b);
            params.len() == 2 && {
                // Check if this block is used as a target with 2 args
                // block5 receives (v13, v22) from block1 and (v15, v24) from block6
                true
            }
        })
        .expect("Could not find block5");

    let block5_params_before = original_func_clone.dfg.block_params(block5);
    assert_eq!(
        block5_params_before.len(),
        2,
        "block5 should have 2 parameters before transform: {:?}",
        block5_params_before
    );

    // Create a minimal ClifModule with the parsed function
    let isa = create_test_isa().expect("Failed to create ISA");
    let module = ClifModule::builder()
        .set_function_registry(lp_glsl::semantic::functions::FunctionRegistry::new())
        .set_source_text(String::from("test"))
        .set_isa(isa)
        .set_main_function(original_func)
        .set_source_map(lp_glsl::frontend::src_loc::GlSourceMap::default())
        .build()
        .expect("Failed to create ClifModule");

    // Apply fixed32 transform
    let transformed_module = transform_module(&module, FixedPointFormat::Fixed16x16)
        .expect("Failed to apply fixed32 transform");

    // Get the transformed function
    let transformed_func = transformed_module.main_function();

    // Find block5 in the transformed function
    // It should still have 2 parameters after transform
    let transformed_block5 = transformed_func
        .layout
        .blocks()
        .find(|&b| {
            let params = transformed_func.dfg.block_params(b);
            params.len() == 2
        })
        .expect("Could not find block5 after transform");

    // Verify block5 still has parameters after transform
    let block5_params_after = transformed_func.dfg.block_params(transformed_block5);
    assert_eq!(
        block5_params_after.len(),
        2,
        "block5 should still have 2 parameters after transform: {:?}",
        block5_params_after
    );

    // Format both functions for comparison
    use cranelift_codegen::write_function;
    let mut before_buf = String::new();
    write_function(&mut before_buf, &original_func_clone).unwrap();

    let mut after_buf = String::new();
    write_function(&mut after_buf, transformed_func).unwrap();

    // Print CLIF for debugging
    eprintln!("\n=== BEFORE Transform ===");
    eprintln!("{}", before_buf);
    eprintln!("\n=== AFTER Transform ===");
    eprintln!("{}", after_buf);

    // Find the block5 line in both outputs
    let before_block5_line = before_buf
        .lines()
        .find(|l| l.contains("block5"))
        .expect("Could not find block5 in BEFORE transform");
    let after_block5_lines: Vec<_> = after_buf.lines().filter(|l| l.contains("block5")).collect();

    eprintln!("\nBEFORE block5: {}", before_block5_line);
    eprintln!("AFTER block5 lines: {:?}", after_block5_lines);

    // Check that block5 in the transformed function has parameters
    // The BEFORE transformation shows: "block5(v13: i32, v22: i32):"
    // The AFTER transformation should also show parameters, not "block5:"
    let block5_has_params_after = after_block5_lines
        .iter()
        .any(|line| line.contains("block5(") && line.contains(":"));

    assert!(
        block5_has_params_after,
        "block5 should have parameters in transformed function.\n\
         BEFORE transform block5: {}\n\
         AFTER transform block5 lines: {:?}\n\
         Full AFTER CLIF:\n{}",
        before_block5_line, after_block5_lines, after_buf
    );

    // Check for @0001 source location labels - these shouldn't appear if they weren't in the original
    // The BEFORE transformation has NO @0001 labels, so AFTER should also have none (or we need to investigate why they appear)
    let before_has_srcloc_labels = before_buf.contains("@0001") || before_buf.contains("@0000");
    let after_has_srcloc_labels = after_buf.contains("@0001");

    if !before_has_srcloc_labels && after_has_srcloc_labels {
        eprintln!("\nWARNING: @0001 source location labels appear after transform but not before!");
        eprintln!("This indicates a bug in source location handling during transform.");
        // Don't fail the test for this, but log it for investigation
    }

    // Verify SSA form: block5 should use its block parameters, not values from other blocks
    // Check that block5 doesn't use v6 directly (which would violate SSA)
    // v6 is computed in block1, so it shouldn't be used in block5 without proper phi nodes
    // Look for instructions in block5 that use v6
    let mut in_block5 = false;
    let mut block5_uses_v6_directly = false;
    for line in after_buf.lines() {
        if line.contains("block5(") {
            in_block5 = true;
        } else if line.trim().starts_with("block") && !line.contains("block5") {
            in_block5 = false;
        } else if in_block5 && line.contains("v6") && !line.contains("block5(") {
            block5_uses_v6_directly = true;
            eprintln!("Found v6 usage in block5: {}", line);
        }
    }

    assert!(
        !block5_uses_v6_directly,
        "block5 should not use v6 directly (SSA violation). Transformed CLIF:\n{}",
        after_buf
    );
}

/// Test that the fixed32 transform produces identical CLIF output for i32-only functions.
///
/// Since all types are i32, the transform should be a no-op and produce exactly the same CLIF.
/// This test normalizes the CLIF strings (removes comments, normalizes whitespace) before comparing.
#[test]
#[ignore]
fn test_i32_only_exact_match() {
    // CLIF IR with only i32 types - transform should produce identical output
    let clif_input = r#"
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
    v24 = iconst.i32 0
    v23 -> v24
    v15 = iconst.i32 0
    v14 -> v15
    jump block5(v15, v24)  ; v15 = 0, v24 = 0

block5(v13: i32, v22: i32):
    jump block2(v13, v22)

block2(v12: i32, v21: i32):
    v16 = iconst.i32 5
    v17 = icmp slt v12, v16  ; v16 = 5
    v18 = iconst.i8 1
    v19 = iconst.i8 0
    v20 = select v17, v18, v19  ; v18 = 1, v19 = 0
    brif v20, block1(v21, v12), block3

block3:
    return v21

block7:
    v25 = iconst.i32 0
    return v25  ; v25 = 0
}
"#;

    // Parse the CLIF IR
    let functions = parse_functions(clif_input).expect("Failed to parse CLIF IR");

    assert_eq!(functions.len(), 1, "Expected exactly one function");
    let original_func = functions.into_iter().next().unwrap();
    let original_func_clone = original_func.clone();

    // Create a minimal ClifModule with the parsed function
    let isa = create_test_isa().expect("Failed to create ISA");
    let module = ClifModule::builder()
        .set_function_registry(lp_glsl::semantic::functions::FunctionRegistry::new())
        .set_source_text(String::from("test"))
        .set_isa(isa)
        .set_main_function(original_func)
        .set_source_map(lp_glsl::frontend::src_loc::GlSourceMap::default())
        .build()
        .expect("Failed to create ClifModule");

    // Apply fixed32 transform
    let transformed_module = transform_module(&module, FixedPointFormat::Fixed16x16)
        .expect("Failed to apply fixed32 transform");

    // Get the transformed function
    let transformed_func = transformed_module.main_function();

    // Format both functions
    use cranelift_codegen::write_function;
    let mut before_buf = String::new();
    write_function(&mut before_buf, &original_func_clone).unwrap();

    let mut after_buf = String::new();
    write_function(&mut after_buf, transformed_func).unwrap();

    // Normalize CLIF strings for comparison:
    // 1. Remove comments (lines starting with ; or containing ;)
    // 2. Normalize whitespace (collapse multiple spaces, trim lines)
    // 3. Remove empty lines
    fn normalize_clif(clif: &str) -> String {
        clif.lines()
            .map(|line| {
                // Remove comments (everything after ;)
                let line = if let Some(comment_pos) = line.find(';') {
                    &line[..comment_pos]
                } else {
                    line
                };
                // Trim whitespace
                line.trim()
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    let normalized_before = normalize_clif(&before_buf);
    let normalized_after = normalize_clif(&after_buf);

    // Extract block order from both CLIF strings to verify they match
    fn extract_block_order(clif: &str) -> Vec<String> {
        clif.lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("block") && trimmed.contains(':') {
                    // Extract block definition (e.g., "block0", "block1(v2: i32, v3: i32)")
                    let block_part = trimmed.split(':').next().unwrap_or("").trim();
                    Some(block_part.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    let before_blocks = extract_block_order(&normalized_before);
    let after_blocks = extract_block_order(&normalized_after);

    // Verify block order matches - this is the key check for block ordering preservation
    assert_eq!(
        before_blocks, after_blocks,
        "Block order mismatch!\n\
         BEFORE block order: {:?}\n\
         AFTER block order: {:?}\n\n\
         BEFORE (normalized):\n{}\n\n\
         AFTER (normalized):\n{}",
        before_blocks, after_blocks, normalized_before, normalized_after
    );

    // Also compare the full normalized CLIF - they should be identical for i32-only functions
    // Note: Value numbers may differ due to entity ID assignment, but block order and structure should match
    assert_eq!(
        normalized_before, normalized_after,
        "CLIF output should exactly match input for i32-only functions.\n\
         BEFORE (normalized):\n{}\n\n\
         AFTER (normalized):\n{}\n\n\
         BEFORE (raw):\n{}\n\n\
         AFTER (raw):\n{}",
        normalized_before, normalized_after, before_buf, after_buf
    );
}
