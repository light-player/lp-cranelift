//! Emulator codegen - build executable from GlModule<ObjectModule>

use crate::backend2::module::gl_module::GlModule;
use crate::exec::emu::GlslEmulatorModule;
use crate::error::{ErrorCode, GlslError};
use crate::frontend::src_loc::GlSourceMap;
use crate::frontend::src_loc_manager::SourceLocManager;
use cranelift_object::ObjectModule;
use hashbrown::HashMap;
use alloc::vec::Vec;

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
    gl_module: GlModule<ObjectModule>,
    options: &EmulatorOptions,
) -> Result<GlslEmulatorModule, GlslError> {
    use lp_riscv_tools::emu::emulator::Riscv32Emulator;
    use lp_riscv_tools::Gpr;
    use lp_riscv_tools::elf_loader::{find_symbol_address, load_elf};
    use object::{Object, ObjectSection};
    
    // 1. Finish module and get object file
    let product = gl_module.module.finish();
    let elf_bytes = product.emit()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to emit ELF: {}", e)))?;

    // 2. Load ELF and find main address
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
    let main_address = find_symbol_address(&obj, "main", text_section_base)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to find main address: {}", e)))?;

    // 3. Create emulator
    let binary = load_info.code;
    let mut emulator = Riscv32Emulator::new(binary.clone(), vec![0; options.max_memory])
        .with_max_instructions(options.max_instructions);

    // 4. Set up stack and PC
    emulator.set_register(Gpr::Sp, options.max_memory as i32);
    emulator.set_pc(0);

    // 5. Build signatures (minimal for Phase 1)
    let signatures = HashMap::new();
    let mut cranelift_signatures = HashMap::new();
    for (name, gl_func) in &gl_module.fns {
        cranelift_signatures.insert(name.clone(), gl_func.clif_sig.clone());
        // Minimal GLSL signature for Phase 1
    }

    // 6. Create GlslEmulatorModule
    // Note: Some fields are required by GlslEmulatorModule but not needed for Phase 1
    // Use minimal/default values for now
    Ok(GlslEmulatorModule {
        emulator,
        signatures,
        cranelift_signatures,
        binary,
        main_address,
        transformed_clif: None,  // Phase 1: not needed
        original_clif: None,     // Phase 1: not needed
        vcode: None,             // Phase 1: not needed
        disassembly: None,       // Phase 1: not needed
        trap_source_info: Vec::new(),  // Phase 1: empty
        source_text: None,       // Phase 1: not needed
        source_file_path: None,  // Phase 1: not needed
        source_loc_manager: SourceLocManager::new(),
        source_map: GlSourceMap::new(),
        next_buffer_addr: 0x80000000,  // Default RAM start
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend2::target::Target;
    use crate::backend2::module::builder::build_simple_function;
    use cranelift_codegen::ir::{types, AbiParam, Signature, InstBuilder};
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
        }).unwrap();
        
        // Build executable
        let options = EmulatorOptions {
            max_memory: 1024 * 1024,
            stack_size: 64 * 1024,
            max_instructions: 10000,
        };
        
        let mut executable = build_emu_executable(gl_module, &options).unwrap();
        // main_address will be set by find_symbol_address
        // Note: main_address can be 0 if the function is at the start of the text section
        
        // Actually call the function and verify it returns 42
        let result = executable.call_i32("main", &[]).unwrap();
        assert_eq!(result, 42);
    }
}
