//! Function linking: rebuild functions with remapped FuncRefs for a new module
//!
//! This module handles the process of taking functions from ClifModule (which were
//! compiled with FuncRefs pointing to a temporary module) and rebuilding them for
//! a new module (JITModule, ObjectModule, etc.) with FuncRefs pointing to the
//! new module's FuncIds.
//!
//! This is similar to the fixed-point transformation in `transform/fixed32/rewrite.rs`,
//! but simpler - we just remap FuncRefs without changing types.
#![allow(dead_code)]

use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::Function;
use cranelift_module::{FuncId, Module};
use hashbrown::HashMap;

use alloc::{string::String, vec::Vec};

/// Rebuild a function for a new module, remapping FuncRefs to point to new FuncIds
///
/// This creates a new function with the same signature and copies all instructions,
/// remapping FuncRefs in call instructions to point to the new module's FuncIds.
pub fn rebuild_function_for_module<M: Module>(
    old_func: &Function,
    module: &mut M,
    func_id_to_name: &HashMap<u32, String>,
    name_to_id: &HashMap<String, FuncId>,
    _new_func_id: FuncId,
) -> Result<Function, GlslError> {
    use cranelift_codegen::ir::{ExternalName, FuncRef};

    // 1. Create new function with same signature, preserving the original function name
    let mut new_func = Function::with_name_signature(
        old_func.name.clone(),
        old_func.signature.clone(),
    );

    // 2. Build mapping from old FuncRef to new FuncRef BEFORE creating builder
    // (to avoid borrow conflicts with new_func)
    let mut func_ref_to_func_id: Vec<(FuncRef, u32)> = Vec::new();
    for (old_func_ref, old_ext_func) in old_func.dfg.ext_funcs.iter() {
        if let ExternalName::User(user_name_ref) = old_ext_func.name {
            // Extract old FuncId from user_named_funcs
            let user_named_funcs = old_func.params.user_named_funcs();
            let old_func_id = if let Some(user_name) = user_named_funcs.get(user_name_ref) {
                user_name.index
            } else {
                // user_named_funcs is empty - match by signature
                let old_sig = &old_func.dfg.signatures[old_ext_func.signature];
                let mut found = false;
                let mut matched_func_id = None;

                // Try to match by comparing signatures
                // We need to find which function in func_id_to_name has a matching signature
                for (func_id_val, func_name) in func_id_to_name.iter() {
                    if let Some(new_func_id) = name_to_id.get(func_name) {
                        let decl = module.declarations().get_function_decl(*new_func_id);
                        // Compare signatures - they should match exactly
                        if decl.signature.params.len() == old_sig.params.len()
                            && decl.signature.returns.len() == old_sig.returns.len()
                        {
                            let params_match =
                                decl.signature.params.iter().zip(old_sig.params.iter()).all(
                                    |(new_param, old_param)| {
                                        new_param.value_type == old_param.value_type
                                            && new_param.purpose == old_param.purpose
                                    },
                                );
                            let returns_match = decl
                                .signature
                                .returns
                                .iter()
                                .zip(old_sig.returns.iter())
                                .all(|(new_ret, old_ret)| {
                                    new_ret.value_type == old_ret.value_type
                                        && new_ret.purpose == old_ret.purpose
                                });

                            if params_match && returns_match {
                                matched_func_id = Some(*func_id_val);
                                found = true;
                                break;
                            }
                        }
                    }
                }

                if !found {
                    // Provide more detailed error message with available signatures
                    let available_sigs: Vec<String> = func_id_to_name
                        .iter()
                        .filter_map(|(func_id_val, func_name)| {
                            name_to_id.get(func_name).map(|new_func_id| {
                                let decl = module.declarations().get_function_decl(*new_func_id);
                                format!(
                                    "  {} (FuncId {}): {:?}",
                                    func_name, func_id_val, decl.signature
                                )
                            })
                        })
                        .collect();

                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "Could not match FuncRef to FuncId - signature matching failed.\n\
                            Looking for signature: {:?}\n\
                            Available signatures:\n{}",
                            old_sig,
                            available_sigs.join("\n")
                        ),
                    ));
                }
                matched_func_id.unwrap()
            };
            func_ref_to_func_id.push((old_func_ref, old_func_id));
        }
    }

    // 3. Create FuncRefs in new module context (before creating builder to avoid borrow conflicts)
    let mut func_ref_map: HashMap<FuncRef, FuncRef> = HashMap::new();
    for (old_func_ref, old_func_id) in &func_ref_to_func_id {
        // Look up function name and get new FuncId
        let callee_name = func_id_to_name.get(old_func_id).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                format!(
                    "Could not find function name for old FuncId {}",
                    old_func_id
                ),
            )
        })?;
        let new_callee_func_id = name_to_id.get(callee_name).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                format!("Could not find new FuncId for function '{}'", callee_name),
            )
        })?;

        // Create new FuncRef in new module context
        let new_func_ref = module.declare_func_in_func(*new_callee_func_id, &mut new_func);
        func_ref_map.insert(*old_func_ref, new_func_ref);
    }

    // 4. Clone the function and remap FuncRefs directly
    // This is simpler than using the instruction copying pipeline
    let mut func_clone = old_func.clone();
    
    // Remap FuncRefs in call instructions
    let blocks: Vec<_> = func_clone.layout.blocks().collect();
    let mut insts_to_update: Vec<(cranelift_codegen::ir::Inst, FuncRef)> = Vec::new();
    for block in &blocks {
        for inst in func_clone.layout.block_insts(*block) {
            let inst_data = &func_clone.dfg.insts[inst];
            if let cranelift_codegen::ir::InstructionData::Call { func_ref, .. } = inst_data {
                if let Some(&new_func_ref) = func_ref_map.get(func_ref) {
                    insts_to_update.push((inst, new_func_ref));
                }
            }
        }
    }

    // Update the instructions
    for (inst, new_func_ref) in insts_to_update {
        let inst_data = &mut func_clone.dfg.insts[inst];
        if let cranelift_codegen::ir::InstructionData::Call { opcode, args, .. } = inst_data.clone() {
            *inst_data = cranelift_codegen::ir::InstructionData::Call {
                opcode,
                func_ref: new_func_ref,
                args,
            };
        }
    }

    // Replace the function we built with the cloned and remapped one
    new_func = func_clone;

    Ok(new_func)
}

// ============================================================================
// Backend linking functions
// ============================================================================

use crate::backend::ir::ClifModule;

use alloc::format as alloc_format;
/// Options for emulator execution
#[cfg(feature = "emulator")]
pub(crate) struct EmulatorOptions {
    pub max_memory: usize,
    pub stack_size: usize,
    pub max_instructions: u64,
}

/// Format CLIF IR from a ClifModule by setting correct function names
/// This is used for the original module before transformations
#[cfg(feature = "emulator")]
fn format_clif_from_module(module: &ClifModule) -> Result<String, crate::error::GlslError> {
    use crate::error::GlslError;
    use cranelift_codegen::write_function;

    let mut clif_ir = String::new();

    // Create a mapping from function names to sequential FuncIds
    // This mimics the order that would be assigned during linking
    let mut name_to_func_id = HashMap::new();
    let mut next_func_id = 0u32;

    // User functions get sequential IDs in iteration order
    // Sort by name for deterministic output
    let mut user_funcs: Vec<_> = module.user_functions().iter().collect();
    user_funcs.sort_by_key(|(name, _)| *name);

    for (name, _) in &user_funcs {
        name_to_func_id.insert((*name).clone(), next_func_id);
        next_func_id += 1;
    }
    // Main function gets the next ID
    name_to_func_id.insert(String::from("main"), next_func_id);

    // Format user functions with correct names (in sorted order for deterministic output)
    for (name, func) in &user_funcs {
        clif_ir.push_str(&format!("; function {}:\n", name));

        // Clone the function and set the correct name
        let mut func_clone = (*func).clone();
        use cranelift_codegen::ir::{ExternalName, UserFuncName};
        func_clone.name = UserFuncName::testcase(name.as_bytes());

        // Update external function references to use testcase names
        let user_named_funcs: std::collections::HashMap<_, _> = func_clone
            .params
            .user_named_funcs()
            .iter()
            .map(|(k, v)| (k, v.clone()))
            .collect();
        for (_, ext_func) in func_clone.dfg.ext_funcs.iter_mut() {
            if let ExternalName::User(user_name_ref) = &ext_func.name {
                // Look up the function name from the user_named_funcs
                if let Some(user_name) = user_named_funcs.get(user_name_ref) {
                    // The user_name.index should correspond to the func_id
                    // Look up the name in the mapping
                    if let Some(func_name) = module.func_id_to_name(user_name.index) {
                        ext_func.name = ExternalName::testcase(func_name.as_bytes());
                    }
                }
            }
        }

        let mut buf = String::new();
        write_function(&mut buf, &func_clone).map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                format!("failed to write function '{}': {}", name, e),
            )
        })?;
        clif_ir.push_str(&buf);
        clif_ir.push('\n');
    }

    // Format main function with correct name
    clif_ir.push_str("; function main:\n");
    let mut main_func_clone = module.main_function().clone();
    use cranelift_codegen::ir::{ExternalName, UserFuncName};
    main_func_clone.name = UserFuncName::testcase("main".as_bytes());

    // Update external function references to use testcase names
    let user_named_funcs: std::collections::HashMap<_, _> = main_func_clone
        .params
        .user_named_funcs()
        .iter()
        .map(|(k, v)| (k, v.clone()))
        .collect();
    for (_, ext_func) in main_func_clone.dfg.ext_funcs.iter_mut() {
        if let ExternalName::User(user_name_ref) = &ext_func.name {
            // Look up the function name from the user_named_funcs
            if let Some(user_name) = user_named_funcs.get(user_name_ref) {
                // The user_name.index should correspond to the func_id
                // Look up the name in the mapping
                if let Some(func_name) = module.func_id_to_name(user_name.index) {
                    ext_func.name = ExternalName::testcase(func_name.as_bytes());
                }
            }
        }
    }

    let mut buf = String::new();
    write_function(&mut buf, &main_func_clone).map_err(|_e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            format!("failed to write main function: {}", "main"),
        )
    })?;
    clif_ir.push_str(&buf);

    Ok(clif_ir)
}

/// Link CLIF module for JIT execution
/// Works in both std and no_std (JITModule supports no_std)
pub fn link_glsl_for_jit(
    module: ClifModule,
) -> Result<crate::exec::jit::GlslJitModule, crate::error::GlslError> {
    use crate::exec::jit::GlslJitModule;
    use crate::error::GlslError;
    // JITModule supports no_std, so we can use it unconditionally
    use cranelift_jit::{JITBuilder, JITModule};
    use cranelift_module::Linkage;
    use hashbrown::HashMap;

    // Recreate the ISA from the TargetIsa reference
    use cranelift_codegen::isa;
    let isa_builder = isa::Builder::from_target_isa(module.isa());
    // Copy flags from the original ISA
    let flags = module.isa().flags().clone();
    let isa = isa_builder.finish(flags).map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to recreate ISA: {:?}", e),
        )
    })?;

    let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    let mut jit_module = JITModule::new(builder);

    let (name_to_id, _clif_ir, _traps) = module.link_into(&mut jit_module, Linkage::Export)?;

    jit_module.finalize_definitions().map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to finalize JIT module: {}", e),
        )
    })?;

    // Build function pointer map
    let mut function_ptrs = HashMap::new();
    for (name, func_id) in &name_to_id {
        let ptr = jit_module.get_finalized_function(*func_id);
        function_ptrs.insert(name.clone(), ptr);
    }

    // Extract signatures (both GLSL and Cranelift)
    let mut signatures = HashMap::new();
    let mut cranelift_signatures = HashMap::new();

    for (name, func) in module.user_functions() {
        // Store Cranelift signature for argument handling
        cranelift_signatures.insert(name.clone(), func.signature.clone());

        // Get GLSL signature from ClifModule
        let glsl_sig = module.glsl_signature(name).ok_or_else(|| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                format!("GLSL signature for function '{}' not found", name),
            )
        })?;
        signatures.insert(name.clone(), glsl_sig.clone());
    }

    // Store main function's Cranelift signature
    cranelift_signatures.insert(
        String::from("main"),
        module.main_function().signature.clone(),
    );

    // Get main function's GLSL signature from ClifModule
    let main_glsl_sig = module.glsl_signature("main").ok_or_else(|| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            "GLSL signature for 'main' not found",
        )
    })?;
    signatures.insert(String::from("main"), main_glsl_sig.clone());

    Ok(GlslJitModule {
        jit_module,
        function_ptrs,
        signatures,
        cranelift_signatures,
        call_conv: module.isa().default_call_conv(),
        pointer_type: module.isa().pointer_type(),
    })
}

/// Link CLIF module for emulator execution
/// Generate VCode and disassembly for all functions in a ClifModule for debugging purposes
#[cfg(feature = "emulator")]
fn generate_vcode_and_disassembly(
    module: &ClifModule,
) -> Result<(Option<String>, Option<String>), crate::error::GlslError> {
    let isa = module.isa();
    let mut vcode_output = String::new();
    let mut disassembly_output = String::new();

    // Process user functions
    for (func_name, func) in module.user_functions() {
        if let Ok((vcode, disasm)) = generate_function_vcode_and_disassembly(func, isa) {
            if !vcode_output.is_empty() {
                vcode_output.push_str(&format!("\n// function {}:\n", func_name));
            } else {
                vcode_output.push_str(&format!("// function {}:\n", func_name));
            }
            vcode_output.push_str(&vcode);

            if !disassembly_output.is_empty() {
                disassembly_output.push_str(&format!("\n// function {}:\n", func_name));
            } else {
                disassembly_output.push_str(&format!("// function {}:\n", func_name));
            }
            disassembly_output.push_str(&disasm);
        }
    }

    // Process main function
    if let Ok((vcode, disasm)) =
        generate_function_vcode_and_disassembly(module.main_function(), isa)
    {
        if !vcode_output.is_empty() {
            vcode_output.push_str("\n// function main:\n");
        } else {
            vcode_output.push_str("// function main:\n");
        }
        vcode_output.push_str(&vcode);

        if !disassembly_output.is_empty() {
            disassembly_output.push_str("\n// function main:\n");
        } else {
            disassembly_output.push_str("// function main:\n");
        }
        disassembly_output.push_str(&disasm);
    }

    Ok((
        if vcode_output.is_empty() {
            None
        } else {
            Some(vcode_output)
        },
        if disassembly_output.is_empty() {
            None
        } else {
            Some(disassembly_output)
        },
    ))
}

/// Generate VCode and disassembly for a single function
#[cfg(feature = "emulator")]
fn generate_function_vcode_and_disassembly(
    func: &cranelift_codegen::ir::Function,
    isa: &dyn cranelift_codegen::isa::TargetIsa,
) -> Result<(String, String), crate::error::GlslError> {
    use crate::error::GlslError;
    use cranelift_codegen::Context;
    use cranelift_control::ControlPlane;

    let params = func.params.clone();
    let mut comp_ctx = Context::for_function(func.clone());

    // Request disassembly results
    comp_ctx.set_disasm(true);

    let compiled_code = comp_ctx
        .compile(isa, &mut ControlPlane::default())
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                format!("Failed to compile function for disassembly: {:?}", e),
            )
        })?;

    let vcode = compiled_code.vcode.as_ref().ok_or_else(|| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            "No VCode available after compilation",
        )
    })?;

    // Generate disassembly using Capstone (RISC-V only for now)
    let cs = isa.to_capstone().map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            format!("Failed to create Capstone disassembler: {}", e),
        )
    })?;
    let dis = compiled_code.disassemble(Some(&params), &cs).map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            format!("Capstone disassembly failed: {}", e),
        )
    })?;

    Ok((vcode.clone(), dis))
}

/// Requires `emulator` feature flag to be enabled
#[cfg(feature = "emulator")]
pub fn link_glsl_for_emulator(
    original_module: ClifModule,
    transformed_module: ClifModule,
    emulator_options: &EmulatorOptions,
    source_text: Option<String>,
    source_file_path: Option<String>,
) -> Result<crate::exec::emu::GlslEmulatorModule, crate::error::GlslError> {
    use crate::exec::emu::GlslEmulatorModule;
    use crate::error::GlslError;
    use hashbrown::HashMap;
    use lp_riscv_tools::Gpr;
    use lp_riscv_tools::elf_loader::{find_symbol_address, load_elf};
    use lp_riscv_tools::emu::emulator::Riscv32Emulator;

    // Format CLIF from original module (before transformations)
    let original_clif = format_clif_from_module(&original_module)?;

    // Build ObjectModule from transformed module to get ELF and CLIF
    let (elf_bytes, transformed_clif, trap_info) = transformed_module.build_object_module()?;

    // Load ELF and apply relocations
    let load_info = load_elf(&elf_bytes)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("ELF load failed: {}", e)))?;

    // Parse ELF to find main function address
    use object::{Object, ObjectSection};
    let obj = object::File::parse(&elf_bytes[..]).map_err(|e| {
        GlslError::new(
            ErrorCode::E0400,
            alloc_format!("Failed to parse ELF for symbol lookup: {:?}", e),
        )
    })?;

    // Find text section base for symbol address calculation
    let mut text_section_base = 0u64;
    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            text_section_base = section.address();
            break;
        }
    }

    // Find main function address
    let main_address = find_symbol_address(&obj, "main", text_section_base).map_err(|e| {
        GlslError::new(
            ErrorCode::E0400,
            format!("Failed to find main symbol: {}", e),
        )
    })?;

    let binary = load_info.code;

    // Map trap offsets to absolute addresses and preserve source location information
    let mut traps = Vec::new();
    let mut trap_source_info = Vec::new();

    // Map trap offsets to absolute addresses using ELF function addresses
    for (func_name, func_traps) in &trap_info {
        // Find function address in ELF
        let func_address = if func_name == "main" {
            main_address
        } else {
            find_symbol_address(&obj, func_name, text_section_base).map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Failed to find function '{}' symbol: {}", func_name, e),
                )
            })?
        };

        for trap in func_traps {
            // Calculate absolute offset: function_address + relative_trap_offset
            let absolute_offset = u32::from(func_address) + trap.offset;
            traps.push((absolute_offset, trap.code));

            // Store source location information for error reporting
            trap_source_info.push((absolute_offset, trap.code, trap.srcloc, func_name.clone()));
        }
    }

    // Create emulator with trap information
    let ram_size = emulator_options.max_memory;
    use lp_riscv_tools::emu::LogLevel;
    let mut emulator = Riscv32Emulator::with_traps(binary.clone(), vec![0; ram_size], &traps)
        .with_max_instructions(emulator_options.max_instructions)
        .with_log_level(LogLevel::Instructions);

    // Set up stack pointer (stack starts at top of RAM, grows downward)
    let stack_base = ram_size as u32;
    emulator.set_register(Gpr::Sp, stack_base as i32);
    emulator.set_pc(0);

    // Extract signatures (both GLSL and Cranelift) from transformed module
    let mut signatures = HashMap::new();
    let mut cranelift_signatures = HashMap::new();

    for (name, func) in transformed_module.user_functions() {
        // Store Cranelift signature for argument handling
        cranelift_signatures.insert(name.clone(), func.signature.clone());

        // Get GLSL signature from ClifModule
        let glsl_sig = transformed_module.glsl_signature(name).ok_or_else(|| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                format!("GLSL signature for function '{}' not found", name),
            )
        })?;
        signatures.insert(name.clone(), glsl_sig.clone());
    }

    // Store main function's Cranelift signature
    let main_sig = transformed_module.main_function().signature.clone();
    cranelift_signatures.insert(String::from("main"), main_sig);

    // Get main function's GLSL signature from ClifModule
    let main_glsl_sig = transformed_module.glsl_signature("main").ok_or_else(|| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            "GLSL signature for 'main' not found",
        )
    })?;
    signatures.insert(String::from("main"), main_glsl_sig.clone());

    // Generate VCode and disassembly for debugging
    let (vcode, disassembly) = generate_vcode_and_disassembly(&transformed_module)?;

    // Extract source location manager and source map from transformed module
    let source_loc_manager = transformed_module.source_loc_manager().clone();
    let source_map = transformed_module.source_map().clone();

    // DEFAULT_RAM_START is 0x80000000 (from lp-riscv-tools/src/emu/memory.rs)
    const DEFAULT_RAM_START: u32 = 0x80000000;

    Ok(GlslEmulatorModule {
        emulator,
        signatures,
        cranelift_signatures,
        binary,
        main_address,
        transformed_clif: Some(transformed_clif),
        original_clif: Some(original_clif),
        vcode,
        disassembly,
        trap_source_info,
        source_text,
        source_file_path,
        source_loc_manager,
        source_map,
        next_buffer_addr: DEFAULT_RAM_START,
    })
}

/// Compile CLIF module to ELF object file for emulator execution
/// Uses ObjectModule to properly handle function call relocations
/// Returns the ELF bytes
#[cfg(feature = "emulator")]
fn compile_clif_to_elf(module: &ClifModule) -> Result<Vec<u8>, crate::error::GlslError> {
    // Use the new build_object_module method which returns (elf_bytes, clif_ir, traps)
    // We only need the ELF bytes here
    let (elf_bytes, _clif_ir, _traps) = module.build_object_module()?;
    Ok(elf_bytes)
}
