//! Binary compilation for emulator execution
//!
//! Compiles GLSL to machine code and combines with bootstrap

use anyhow::Result;
use cranelift_codegen::isa::lookup;
use lp_glsl::FixedPointFormat;

/// Compile GLSL source to binary code for riscv32 emulator
/// Returns combined binary: bootstrap code + test function code
pub fn compile_to_binary(
    glsl_source: &str,
    fixed_point_format: Option<FixedPointFormat>,
    _bootstrap_code: &[u8], // Will be regenerated with correct address
) -> Result<Vec<u8>> {
    // Build riscv32 ISA
    use cranelift_codegen::settings;
    use target_lexicon::Triple;
    let triple = Triple {
        architecture: target_lexicon::Architecture::Riscv32(
            target_lexicon::Riscv32Architecture::Riscv32imac,
        ),
        vendor: target_lexicon::Vendor::Unknown,
        operating_system: target_lexicon::OperatingSystem::None_,
        environment: target_lexicon::Environment::Unknown,
        binary_format: target_lexicon::BinaryFormat::Elf,
    };
    let flag_builder = settings::builder();
    let flags = cranelift_codegen::settings::Flags::new(flag_builder);
    let isa = lookup(triple)
        .map_err(|e| anyhow::anyhow!("Failed to lookup ISA: {}", e))?
        .finish(flags)
        .map_err(|e| anyhow::anyhow!("Failed to build ISA: {}", e))?;

    // Compile GLSL to code bytes using the new compile_to_code_bytes method
    let mut compiler = lp_glsl::Compiler::new();
    compiler.set_fixed_point_format(fixed_point_format);
    
    let test_func_code = compiler.compile_to_code_bytes(glsl_source, isa.as_ref())
        .map_err(|e| anyhow::anyhow!("GLSL compilation failed: {}", e))?;

    // Generate initial bootstrap to estimate size
    use crate::execution::bootstrap::generate_bootstrap;
    use crate::execution::backend::ReturnType;
    let initial_bootstrap = generate_bootstrap(0, ReturnType::Float, fixed_point_format)?;
    
    // Calculate test function address (after bootstrap, aligned to 4 bytes)
    let mut test_func_addr = initial_bootstrap.len() as u32;
    // Align to 4-byte boundary
    test_func_addr = (test_func_addr + 3) & !3;

    // Regenerate bootstrap with correct test function address
    let bootstrap_code = generate_bootstrap(test_func_addr, ReturnType::Float, fixed_point_format)?;

    // For now, create a simple binary: bootstrap + test function
    // TODO: Properly link bootstrap with test function address
    let mut binary = Vec::new();
    binary.extend_from_slice(&bootstrap_code);

    // Align test function to 4-byte boundary
    while binary.len() % 4 != 0 {
        binary.push(0);
    }

    binary.extend_from_slice(&test_func_code);

    Ok(binary)
}
