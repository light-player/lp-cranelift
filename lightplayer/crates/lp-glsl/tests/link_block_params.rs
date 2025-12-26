//! Test to verify that block parameters are preserved during function linking

use cranelift_reader::parse_functions;
use lp_glsl::{ClifModule, error::{ErrorCode, GlslError}};
use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_codegen::settings;

#[cfg(feature = "std")]
fn create_test_isa() -> Result<OwnedTargetIsa, GlslError> {
    use cranelift_native;
    use cranelift_codegen::settings::{self, Configurable};

    let mut flag_builder = settings::builder();
    flag_builder.set("opt_level", "none").map_err(|e| {
        GlslError::new(ErrorCode::E0400, format!("failed to set opt_level: {}", e))
    })?;
    let flags = settings::Flags::new(flag_builder);

    cranelift_native::builder()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("failed to create native builder: {}", e)))?
        .finish(flags)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("failed to create ISA: {}", e)))
}

#[cfg(not(feature = "std"))]
fn create_test_isa() -> Result<OwnedTargetIsa, GlslError> {
    Err(GlslError::new(
        ErrorCode::E0400,
        "This test requires the std feature to be enabled",
    ))
}

#[test]
fn test_link_preserves_block_params() {
    // Parse a simple function with block parameters
    let clif_input = r#"
function %test_block_params() -> i32 system_v {
block0:
    v0 = iconst.i32 0
    v1 = iconst.i32 0
    jump block1(v0, v1)

block1(v2: i32, v3: i32):
    v4 = iadd v2, v3
    v5 = iconst.i32 1
    v6 = iadd v3, v5
    v7 = iconst.i32 2
    v8 = icmp sge v6, v7
    v9 = iconst.i8 1
    v10 = iconst.i8 0
    v11 = select v8, v9, v10
    brif v11, block4, block5(v6, v4)

block4:
    jump block2(v6, v4)

block5(v13: i32, v22: i32):
    jump block2(v13, v22)

block2(v12: i32, v21: i32):
    v16 = iconst.i32 5
    v17 = icmp slt v12, v16
    v18 = iconst.i8 1
    v19 = iconst.i8 0
    v20 = select v17, v18, v19
    brif v20, block1(v21, v12), block3

block3:
    return v21
}
"#;

    // Parse the function
    let mut functions = parse_functions(clif_input)
        .expect("Failed to parse CLIF IR");

    assert_eq!(functions.len(), 1, "Expected exactly one function");
    let old_func = functions.remove(0);

    // Verify the original function has block parameters
    // In Cranelift, block parameters are inferred from jump instructions, not declared
    // So we need to find blocks by looking at which blocks are targets of jumps with arguments
    let old_blocks: Vec<_> = old_func.layout.blocks().collect();

    // Find block1 - it's the target of the entry block's jump with 2 arguments
    let entry_block = old_func.layout.entry_block().unwrap();
    let mut old_block1 = None;
    for inst in old_func.layout.block_insts(entry_block) {
        if let cranelift_codegen::ir::InstructionData::Jump { destination, .. } = &old_func.dfg.insts[inst] {
            let target = destination.block(&old_func.dfg.value_lists);
            let args: Vec<_> = destination.args(&old_func.dfg.value_lists)
                .filter_map(|arg| arg.as_value())
                .collect();
            if args.len() == 2 {
                old_block1 = Some(target);
                break;
            }
        }
    }

    // Find block2 - it's targeted by jumps from block4 and block5 with 2 arguments each
    let mut old_block2 = None;
    for &block in &old_blocks {
        if Some(block) == old_block1 {
            continue;
        }
        // Check if this block is targeted by jumps with 2 arguments
        let mut jump_count = 0;
        for &other_block in &old_blocks {
            for inst in old_func.layout.block_insts(other_block) {
                if let cranelift_codegen::ir::InstructionData::Jump { destination, .. } = &old_func.dfg.insts[inst] {
                    let target = destination.block(&old_func.dfg.value_lists);
                    if target == block {
                        let args: Vec<_> = destination.args(&old_func.dfg.value_lists)
                            .filter_map(|arg| arg.as_value())
                            .collect();
                        if args.len() == 2 {
                            jump_count += 1;
                        }
                    }
                }
            }
        }
        if jump_count >= 2 {
            old_block2 = Some(block);
            break;
        }
    }

    let old_block1 = old_block1.expect("block1 should exist");
    let old_block1_params = old_func.dfg.block_params(old_block1);
    assert_eq!(old_block1_params.len(), 2, "block1 should have 2 parameters. Got: {}", old_block1_params.len());

    let old_block2 = old_block2.expect("block2 should exist");
    let old_block2_params = old_func.dfg.block_params(old_block2);

    // Print debug info
    println!("Original function blocks and param counts:");
    for (i, &block) in old_blocks.iter().enumerate() {
        let params = old_func.dfg.block_params(block);
        println!("  block{}: {} params", i, params.len());
    }

    // Format the original function to see what it looks like
    use cranelift_codegen::write_function;
    let mut original_buf = String::new();
    write_function(&mut original_buf, &old_func).unwrap();
    println!("Original function CLIF:\n{}", original_buf);

    // In Cranelift, block parameters are inferred from jumps, not declarations
    // The CLIF parser may not set up block parameters from the declaration
    // But the jumps should still have arguments - let's verify that
    let mut jumps_to_block2_with_args = 0;
    for &block in &old_blocks {
        for inst in old_func.layout.block_insts(block) {
            if let cranelift_codegen::ir::InstructionData::Jump { destination, .. } = &old_func.dfg.insts[inst] {
                let target = destination.block(&old_func.dfg.value_lists);
                if target == old_block2 {
                    let args: Vec<_> = destination.args(&old_func.dfg.value_lists)
                        .filter_map(|arg| arg.as_value())
                        .collect();
                    println!("Jump to block2 from block{:?} has {} arguments", block, args.len());
                    if args.len() == 2 {
                        jumps_to_block2_with_args += 1;
                    }
                }
            }
        }
    }

    println!("Found {} jumps to block2 with 2 arguments", jumps_to_block2_with_args);

    // The key test: after linking, block2 should have 2 parameters
    // (because the original CLIF shows block2(v12: i32, v21: i32) and jumps to it with 2 args)

    // Create a ClifModule with this function
    let isa = create_test_isa().expect("Failed to create ISA");
    let module = ClifModule::builder()
        .set_function_registry(lp_glsl::semantic::functions::FunctionRegistry::new())
        .set_source_text(String::from("test"))
        .set_isa(isa)
        .set_main_function(old_func.clone())
        .build()
        .expect("Failed to create ClifModule");

    // Link the module to a JITModule (this is what tests block param preservation)
    use cranelift_jit::{JITBuilder, JITModule};
    use cranelift_module::Linkage;
    
    let isa_for_jit = module.isa();
    let isa_builder = cranelift_codegen::isa::Builder::from_target_isa(isa_for_jit);
    let flags = isa_for_jit.flags().clone();
    let owned_isa = isa_builder.finish(flags).expect("Failed to recreate ISA");
    
    let builder = JITBuilder::with_isa(owned_isa, cranelift_module::default_libcall_names());
    let mut jit_module = JITModule::new(builder);
    
    let (_name_to_id, linked_clif, _traps) = module
        .link_into(&mut jit_module, Linkage::Export)
        .expect("Failed to link into JIT module");

    // Print for debugging first
    println!("Linked CLIF:\n{}", linked_clif);

    // Parse the linked CLIF to verify block parameters
    let mut linked_functions = parse_functions(&linked_clif)
        .expect("Failed to parse linked CLIF IR");

    assert_eq!(linked_functions.len(), 1, "Expected exactly one function in linked output");
    let linked_func = linked_functions.remove(0);

    // Verify block parameters are preserved
    let linked_blocks: Vec<_> = linked_func.layout.blocks().collect();

    // Find block1 (should be the second block, after entry block)
    let linked_block1 = linked_blocks.get(1).expect("block1 should exist");
    let linked_block1_params = linked_func.dfg.block_params(*linked_block1);
    assert_eq!(
        linked_block1_params.len(),
        2,
        "block1 should still have 2 parameters after linking. Got: {}\n\nLinked CLIF:\n{}",
        linked_block1_params.len(),
        &linked_clif
    );

    // Find block2 (should be the third block)
    let linked_block2 = linked_blocks.get(2).expect("block2 should exist");
    let linked_block2_params = linked_func.dfg.block_params(*linked_block2);
    assert_eq!(
        linked_block2_params.len(),
        2,
        "block2 should still have 2 parameters after linking. Got: {}\n\nLinked CLIF:\n{}",
        linked_block2_params.len(),
        &linked_clif
    );
}

