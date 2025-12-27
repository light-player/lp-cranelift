//! Emulator codegen - build executable from GlModule<ObjectModule>

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
    use object::{Object, ObjectSection};

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
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Failed to define function '{}': {}", name, e),
                )
            })?;

        // Capture V-Code and disassembly if available (only in std builds)
        #[cfg(feature = "std")]
        {
            if let Some(compiled_code) = ctx.compiled_code() {
                // Get disassembly from compiled_code
                let disasm = compiled_code.vcode.as_ref().map(|s| s.clone());

                // Try to generate disassembly using ISA capabilities if available
                let disasm = if let Some(ref disasm_str) = disasm {
                    Some(disasm_str.clone())
                } else {
                    // Try to disassemble using the ISA if available
                    let module_ref = gl_module.module_internal();
                    let isa = module_ref.isa();
                    #[cfg(feature = "disas")]
                    {
                        if let Ok(cs) = isa.to_capstone() {
                            if let Ok(disasm_str) =
                                compiled_code.disassemble(Some(&ctx.func.params), &cs)
                            {
                                Some(disasm_str)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    #[cfg(not(feature = "disas"))]
                    {
                        None
                    }
                };

                if let Some(ref disasm_str) = disasm {
                    all_disasm_parts.push(format!("// function {}:\n{}", name, disasm_str));
                }

                // For V-Code, use the disassembly as a placeholder
                // TODO: Capture actual pre-regalloc V-Code if needed
                if let Some(ref disasm_str) = disasm {
                    all_vcode_parts.push(format!("// function {}:\n{}", name, disasm_str));
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
    let elf_bytes = product
        .emit()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to emit ELF: {}", e)))?;

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

    // 5. Create emulator
    let binary = load_info.code;
    let mut emulator = Riscv32Emulator::new(binary.clone(), vec![0; options.max_memory])
        .with_max_instructions(options.max_instructions)
        .with_log_level(LogLevel::Instructions);

    // 6. Set up stack and PC
    emulator.set_register(Gpr::Sp, options.max_memory as i32);
    emulator.set_pc(0);

    // 7. Create GlslEmulatorModule
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
        trap_source_info: Vec::new(), // Phase 1: empty
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
