//! Emulator codegen - build executable from GlModule<ObjectModule>

#[cfg(feature = "emulator")]
#[allow(unused_imports)]
mod builtins_lib {
    include!(concat!(env!("OUT_DIR"), "/lp_builtins_lib.rs"));
    // For backward compatibility, alias the old name
    pub const LP_BUILTINS_LIB_BYTES: &[u8] = LP_BUILTINS_EXE_BYTES;
}

#[cfg(feature = "emulator")]
use crate::backend::module::gl_module::GlModule;
#[cfg(feature = "emulator")]
use crate::error::{ErrorCode, GlslError};
#[cfg(feature = "emulator")]
use crate::exec::emu::GlslEmulatorModule;
#[cfg(feature = "emulator")]
use crate::frontend::src_loc::GlSourceMap;
#[cfg(feature = "emulator")]
use alloc::string::String;
#[cfg(feature = "emulator")]
use alloc::vec::Vec;
#[cfg(feature = "emulator")]
use cranelift_module::Module;
#[cfg(feature = "emulator")]
use cranelift_object::ObjectModule;
#[cfg(feature = "emulator")]
use hashbrown::HashMap;

/// Emulator execution options
#[derive(Debug, Clone)]
pub struct EmulatorOptions {
    /// Maximum memory size in bytes (RAM)
    pub max_memory: usize,
    /// Stack size in bytes (stored for future use)
    #[allow(unused)]
    pub stack_size: usize,
    /// Maximum instruction count before timeout
    pub max_instructions: u64,
}

/// Build emulator executable from GlModule<ObjectModule>
/// Called by GlModule<ObjectModule>::build_executable()
#[cfg(feature = "emulator")]
pub fn build_emu_executable(
    mut gl_module: GlModule<ObjectModule>,
    options: &EmulatorOptions,
    original_clif: Option<String>,
    transformed_clif: Option<String>,
) -> Result<GlslEmulatorModule, GlslError> {
    use lp_riscv_tools::Gpr;
    use lp_riscv_tools::elf_loader::{find_symbol_address, load_elf};
    use lp_riscv_tools::emu::LogLevel;
    use lp_riscv_tools::emu::emulator::Riscv32Emulator;
    use object::{Object, ObjectSection, ObjectSymbol};

    // Builtin functions are already declared when the module was created

    // 1. Define all functions (compile them)
    // Collect function data first to avoid borrowing conflicts
    let funcs: Vec<(
        String,
        cranelift_codegen::ir::Function,
        cranelift_module::FuncId,
    )> = gl_module
        .fns
        .iter()
        .map(|(name, gl_func)| (name.clone(), gl_func.function.clone(), gl_func.func_id))
        .collect();

    // Collect V-Code and disassembly for all functions
    #[cfg(feature = "std")]
    let mut all_vcode_parts: Vec<String> = Vec::new();
    #[cfg(feature = "std")]
    let mut all_disasm_parts: Vec<String> = Vec::new();

    // Collect trap information: (function_name, trap_offset, trap_code, srcloc)
    let mut trap_info: Vec<(
        String,
        u32,
        cranelift_codegen::ir::TrapCode,
        cranelift_codegen::ir::SourceLoc,
    )> = Vec::new();

    for (name, func, func_id) in funcs {
        // Create context using immutable borrow
        let mut ctx = {
            let module_ref = gl_module.module_internal();
            module_ref.make_context()
        };
        ctx.func = func;

        // Enable disassembly for debugging (only in std builds)
        #[cfg(feature = "std")]
        ctx.set_disasm(true);

        // Define function using mutable borrow
        gl_module
            .module_mut_internal()
            .define_function(func_id, &mut ctx)
            .map_err(|e| {
                // TODO: This is a hacky way to get the verifier error and it should be improved
                // Check if this is a verifier error by checking the error message
                // If it is, verify the function again to get detailed error messages
                let error_str = format!("{}", e);
                let error_msg = if error_str.contains("Verifier errors") {
                    // It's a verifier error - verify the function again to get detailed errors
                    use cranelift_codegen::verifier::verify_function;
                    let module_ref = gl_module.module_internal();
                    let isa = module_ref.isa();

                    if let Err(verifier_errors) = verify_function(&ctx.func, isa) {
                        // Format verifier errors with the function IR for context
                        #[cfg(feature = "std")]
                        {
                            use cranelift_codegen::print_errors::pretty_verifier_error;
                            format!(
                                "Failed to define function '{}': Verifier errors\n\n{}",
                                name,
                                pretty_verifier_error(&ctx.func, None, verifier_errors)
                            )
                        }
                        #[cfg(not(feature = "std"))]
                        {
                            format!(
                                "Failed to define function '{}': Verifier errors\n\n{}",
                                name, verifier_errors
                            )
                        }
                    } else {
                        // Fallback if verification somehow succeeds
                        format!("Failed to define function '{}': {}", name, e)
                    }
                } else {
                    format!("Failed to define function '{}': {}", name, e)
                };

                let mut error = GlslError::new(ErrorCode::E0400, error_msg);

                // Add CLIF IR (before and after transformation) if available
                // Only show both if they're different
                match (&original_clif, &transformed_clif) {
                    (Some(original), Some(transformed)) if original != transformed => {
                        error = error.with_note(format!(
                            "=== CLIF IR (BEFORE transformation) ===\n{}",
                            original
                        ));
                        error = error.with_note(format!(
                            "=== CLIF IR (AFTER transformation) ===\n{}",
                            transformed
                        ));
                    }
                    (Some(ir), Some(_)) => {
                        // They're the same, just show one
                        error = error.with_note(format!("=== CLIF IR ===\n{}", ir));
                    }
                    (Some(ir), None) => {
                        error = error.with_note(format!("=== CLIF IR ===\n{}", ir));
                    }
                    (None, Some(ir)) => {
                        error = error.with_note(format!("=== CLIF IR ===\n{}", ir));
                    }
                    (None, None) => {
                        // No CLIF IR available
                    }
                }

                error
            })?;

        // Capture V-Code and disassembly if available (only in std builds)
        // Also collect trap information
        #[cfg(feature = "std")]
        {
            if let Some(compiled_code) = ctx.compiled_code() {
                // Get VCode (intermediate representation)
                let vcode = compiled_code.vcode.as_ref().map(|s| s.clone());

                // Collect trap information from this function
                // Traps are stored with offsets relative to the start of the function
                for trap in compiled_code.buffer.traps() {
                    // Get source location if available
                    let srcloc = ctx.func.params.base_srcloc();
                    trap_info.push((name.clone(), trap.offset, trap.code, srcloc));
                }

                // Try to generate actual RISC-V disassembly using Capstone first (preferred)
                // This gives us real assembly instructions, not VCode pseudo-instructions
                let disasm = {
                    #[cfg(feature = "emulator")]
                    {
                        let module_ref = gl_module.module_internal();
                        let isa = module_ref.isa();
                        if let Ok(cs) = isa.to_capstone() {
                            if let Ok(disasm_str) =
                                compiled_code.disassemble(Some(&ctx.func.params), &cs)
                            {
                                Some(disasm_str)
                            } else {
                                // Fall back to VCode if Capstone disassembly fails
                                vcode.clone()
                            }
                        } else {
                            // Fall back to VCode if Capstone isn't available
                            vcode.clone()
                        }
                    }
                    #[cfg(not(feature = "emulator"))]
                    {
                        // Fall back to VCode if emulator feature isn't enabled
                        vcode.clone()
                    }
                };

                // Store actual disassembly (RISC-V assembly)
                if let Some(ref disasm_str) = disasm {
                    all_disasm_parts.push(format!("// function {}:\n{}", name, disasm_str));
                }

                // Store VCode separately (intermediate representation)
                if let Some(ref vcode_str) = vcode {
                    all_vcode_parts.push(format!("// function {}:\n{}", name, vcode_str));
                }
            }
        }
        #[cfg(not(feature = "std"))]
        {
            // Even without std, we need to collect trap information
            if let Some(compiled_code) = ctx.compiled_code() {
                for trap in compiled_code.buffer.traps() {
                    let srcloc = ctx.func.params.base_srcloc();
                    trap_info.push((name.clone(), trap.offset, trap.code, srcloc));
                }
            }
        }

        // Clear context using immutable borrow
        {
            let module_ref = gl_module.module_internal();
            module_ref.clear_context(&mut ctx);
        }
    }

    // Combine all V-Code and disassembly parts
    #[cfg(feature = "std")]
    let vcode = if all_vcode_parts.is_empty() {
        None
    } else {
        Some(all_vcode_parts.join("\n\n"))
    };
    #[cfg(feature = "std")]
    let disassembly = if all_disasm_parts.is_empty() {
        None
    } else {
        Some(all_disasm_parts.join("\n\n"))
    };
    #[cfg(not(feature = "std"))]
    let vcode = None;
    #[cfg(not(feature = "std"))]
    let disassembly = None;

    // 2. Build signatures and extract metadata before moving gl_module
    let signatures = gl_module.glsl_signatures.clone();
    let mut cranelift_signatures = HashMap::new();
    for (name, gl_func) in &gl_module.fns {
        cranelift_signatures.insert(name.clone(), gl_func.clif_sig.clone());
    }
    // Extract source_loc_manager and source_map using mem::replace to avoid partial moves
    use crate::frontend::src_loc_manager::SourceLocManager;
    let source_loc_manager =
        core::mem::replace(&mut gl_module.source_loc_manager, SourceLocManager::new());
    let source_map = core::mem::replace(&mut gl_module.source_map, GlSourceMap::new());

    // 3. Finish module and get object file
    let product = gl_module.into_module().finish();
    let mut elf_bytes = product
        .emit()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to emit ELF: {}", e)))?;

    // Debug: Check symbols BEFORE linking
    crate::debug!("=== Symbols BEFORE linking ===");
    if let Ok(obj) = object::File::parse(&elf_bytes[..]) {
        use crate::backend::builtins::registry::BuiltinId;
        for builtin in BuiltinId::all() {
            let symbol_name = builtin.name();
            crate::debug!("Looking for builtin: {}", symbol_name);
            let mut found = false;
            for symbol in obj.symbols() {
                if let Ok(name) = symbol.name() {
                    if name == symbol_name {
                        found = true;
                        crate::debug!(
                            "  Found {}: kind={:?} section={:?} address=0x{:x}",
                            name,
                            symbol.kind(),
                            symbol.section(),
                            symbol.address()
                        );
                    }
                }
            }
            if !found {
                crate::debug!("  NOT FOUND: {}", symbol_name);
            }
        }
    }

    // 3.5 Link builtins static library into the ELF if available
    #[cfg(feature = "std")]
    {
        // Use compile-time embedded library bytes
        let builtins_lib_bytes = builtins_lib::LP_BUILTINS_LIB_BYTES;
        elf_bytes = crate::backend::codegen::builtins_linker::link_and_verify_builtins(
            &elf_bytes,
            builtins_lib_bytes,
        )?;
    }

    // 4. Load ELF and find main address
    let load_info = load_elf(&elf_bytes)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to load ELF: {}", e)))?;
    let obj = object::File::parse(&elf_bytes[..])
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to parse ELF: {}", e)))?;

    // Find text section base address
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
            format!("Failed to find main address: {}", e),
        )
    })?;

    // 5. Collect trap information: convert function-relative offsets to absolute addresses
    // Build a map of function names to their addresses in the binary (relative to text section base)
    let mut func_addresses: HashMap<String, u32> = HashMap::new();
    for symbol in obj.symbols() {
        if let Ok(name) = symbol.name() {
            if symbol.kind() == object::SymbolKind::Text {
                let address = symbol.address();
                if address >= text_section_base {
                    // Address relative to text section base (matches find_symbol_address logic)
                    let offset = (address - text_section_base) as u32;
                    func_addresses.insert(String::from(name), offset);
                }
            }
        }
    }

    // Convert trap offsets to absolute addresses (relative to code buffer start, which is 0)
    let mut traps: Vec<(u32, cranelift_codegen::ir::TrapCode)> = Vec::new();
    let mut trap_source_info: Vec<(
        u32,
        cranelift_codegen::ir::TrapCode,
        cranelift_codegen::ir::SourceLoc,
        String,
    )> = Vec::new();

    for (func_name, trap_offset, trap_code, srcloc) in trap_info {
        if let Some(&func_addr) = func_addresses.get(&func_name) {
            // Trap offset is relative to function start, so add it to function address
            // Both are offsets from the start of the code buffer (which starts at address 0)
            let absolute_addr = func_addr + trap_offset;
            traps.push((absolute_addr, trap_code));
            trap_source_info.push((absolute_addr, trap_code, srcloc, func_name));
        }
    }

    // 6. Create emulator with trap information
    let binary = load_info.code;
    let mut emulator =
        Riscv32Emulator::with_traps(binary.clone(), vec![0; options.max_memory], &traps)
            .with_max_instructions(options.max_instructions)
            .with_log_level(LogLevel::Instructions);

    // 7. Set up stack and PC
    emulator.set_register(Gpr::Sp, options.max_memory as i32);
    // Note: PC will be set by call_function to main_address, but initialize it here for safety
    emulator.set_pc(main_address);

    // 8. Create GlslEmulatorModule
    // Preserve metadata from GlModule
    Ok(GlslEmulatorModule {
        emulator,
        signatures,
        cranelift_signatures,
        binary,
        main_address,
        transformed_clif,
        original_clif,
        vcode,
        disassembly,
        trap_source_info,
        source_loc_manager,
        source_map,
        next_buffer_addr: 0x80000000, // Default RAM start
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::module::gl_module::GlModule;
    use crate::backend::module::test_helpers::test_helpers::build_simple_function;
    use crate::backend::target::Target;
    use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature, types};
    use cranelift_codegen::isa::CallConv;
    use cranelift_module::Linkage;

    #[test]
    #[cfg(feature = "emulator")]
    fn test_build_emu_executable() {
        use crate::exec::executable::GlslExecutable;

        let target = Target::riscv32_emulator().unwrap();
        let mut gl_module = GlModule::new_object(target).unwrap();

        // Build a simple function that returns 42
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I32));

        build_simple_function(&mut gl_module, "main", Linkage::Export, sig, |builder| {
            let val = builder.ins().iconst(types::I32, 42);
            builder.ins().return_(&[val]);
            Ok(())
        })
        .unwrap();

        // Build executable
        let options = EmulatorOptions {
            max_memory: 1024 * 1024,
            stack_size: 64 * 1024,
            max_instructions: 10000,
        };

        let mut executable = build_emu_executable(gl_module, &options, None, None).unwrap();
        // main_address will be set by find_symbol_address
        // Note: main_address can be 0 if the function is at the start of the text section

        // Actually call the function and verify it returns 42
        let result = executable.call_i32("main", &[]).unwrap();
        assert_eq!(result, 42);
    }
}
