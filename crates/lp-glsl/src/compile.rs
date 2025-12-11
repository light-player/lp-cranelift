//! Compilation pipeline functions
//!
//! This module provides internal, reusable compilation functions that can be
//! used by different backends (JIT, emulator, etc.)

use crate::backend::executable::{DecimalFormat, GlslOptions, RunMode};
use crate::compiler::glsl_compiler::GlslCompiler;
use crate::error::GlslError;
use crate::ir::ClifModule;
use crate::transform::fixed32::{FixedPointFormat, transform_module};

use cranelift_codegen::isa::OwnedTargetIsa;

#[cfg(feature = "std")]
use cranelift_native;

#[cfg(not(feature = "std"))]
use alloc::format as alloc_format;
#[cfg(feature = "std")]
use std::format as alloc_format;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};
#[cfg(feature = "std")]
use std::{boxed::Box, string::String};

/// Compile GLSL to CLIF module (internal, reusable)
/// This is the core compilation step that can be reused for different backends
pub fn compile_glsl_to_clif(source: &str, options: &GlslOptions) -> Result<ClifModule, GlslError> {
    options.validate()?;

    let mut compiler = GlslCompiler::new();

    // Determine ISA based on run mode
    let isa = match &options.run_mode {
        #[cfg(feature = "std")]
        RunMode::HostJit => create_host_isa()?,
        #[cfg(all(feature = "emulator", feature = "std"))]
        RunMode::Emulator { .. } => create_riscv32_isa()?,
        #[cfg(not(feature = "emulator"))]
        RunMode::Emulator { .. } => {
            return Err(GlslError::new(
                crate::error::ErrorCode::E0400,
                "Emulator mode requires 'emulator' feature flag",
            ));
        }
        #[cfg(all(not(feature = "std"), feature = "emulator"))]
        RunMode::HostJit => {
            return Err(GlslError::new(
                crate::error::ErrorCode::E0400,
                "HostJit mode requires 'std' feature flag",
            ));
        }
    };

    // Compile to CLIF
    let mut module = compiler.compile_to_clif_module(source, isa)?;

    // Apply transformations
    match options.decimal_format {
        DecimalFormat::Fixed32 => {
            module = transform_module(&module, FixedPointFormat::Fixed16x16)?;
        }
        DecimalFormat::Fixed64 => {
            return Err(GlslError::new(
                crate::error::ErrorCode::E0400,
                "Fixed64 not yet supported",
            ));
        }
        DecimalFormat::Float => {
            // No transformation needed
        }
    }

    Ok(module)
}

/// Link CLIF module for JIT execution
/// Works in both std and no_std (JITModule supports no_std)
pub fn link_glsl_for_jit(
    module: ClifModule,
) -> Result<crate::backend::jit::GlslJitModule, GlslError> {
    use crate::backend::jit::GlslJitModule;
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

    let name_to_id = module.link_into(&mut jit_module, Linkage::Export)?;

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
/// Requires `emulator` feature flag to be enabled
#[cfg(feature = "emulator")]
pub fn link_glsl_for_emulator(
    module: ClifModule,
    emulator_options: &EmulatorOptions,
) -> Result<crate::backend::emu::GlslEmulatorModule, GlslError> {
    use crate::backend::emu::GlslEmulatorModule;
    use cranelift_codegen::{Context, control::ControlPlane};
    use hashbrown::HashMap;
    use lp_riscv_tools::Gpr;
    use lp_riscv_tools::emu::emulator::Riscv32Emulator;

    // Compile to binary
    let binary = compile_clif_to_binary(&module)?;

    // Create emulator
    let ram_size = emulator_options.max_memory;
    let mut emulator = Riscv32Emulator::new(binary.clone(), vec![0; ram_size])
        .with_max_instructions(emulator_options.max_instructions);

    // Set up stack pointer (stack starts at top of RAM, grows downward)
    let stack_base = ram_size as u32;
    emulator.set_register(Gpr::Sp, stack_base as i32);
    emulator.set_pc(0);

    // Extract signatures (both GLSL and Cranelift)
    // Note: We only support calling main at address 0x00 for now
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

    Ok(GlslEmulatorModule {
        emulator,
        signatures,
        cranelift_signatures,
        binary,
    })
}

/// Options for emulator execution
#[cfg(feature = "emulator")]
struct EmulatorOptions {
    max_memory: usize,
    stack_size: usize,
    max_instructions: u64,
}

/// Compile CLIF module to binary for emulator execution
/// Uses ObjectModule to properly handle function call relocations
#[cfg(feature = "emulator")]
fn compile_clif_to_binary(module: &ClifModule) -> Result<Vec<u8>, GlslError> {
    use cranelift_module::Linkage;
    use cranelift_object::{ObjectBuilder, ObjectModule};

    // Create ObjectModule for proper linking with relocations
    let isa = module.isa();
    let mut isa_builder = cranelift_codegen::isa::Builder::from_target_isa(isa);
    let flags = isa.flags().clone();
    let owned_isa = isa_builder.finish(flags).map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to recreate ISA: {:?}", e),
        )
    })?;

    let builder = ObjectBuilder::new(
        owned_isa,
        "glsl_module",
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to create ObjectBuilder: {:?}", e),
        )
    })?;

    let mut object_module = ObjectModule::new(builder);

    // Link all functions into the ObjectModule (handles relocations)
    module
        .link_into(&mut object_module, Linkage::Export)
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                alloc_format!("failed to link functions: {}", e),
            )
        })?;

    // Finish the module and get the object file
    let object_product = object_module.finish();
    let object_bytes = object_product.emit().map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to emit object: {:?}", e),
        )
    })?;

    // Parse the object file to extract the .text section (code)
    use object::{Object, ObjectSection};
    let obj = object::File::parse(&object_bytes[..]).map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to parse object file: {:?}", e),
        )
    })?;

    // Find the .text section
    let mut code_bytes = Vec::new();
    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            if let Ok(data) = section.data() {
                code_bytes.extend_from_slice(data);
            }
        }
    }

    if code_bytes.is_empty() {
        return Err(GlslError::new(
            crate::error::ErrorCode::E0400,
            "No .text section found in object file",
        ));
    }

    Ok(code_bytes)
}

/// Compile and JIT execute GLSL
/// Works in both std and no_std environments
pub fn glsl_jit(
    source: &str,
    options: GlslOptions,
) -> Result<Box<dyn crate::backend::executable::GlslExecutable>, GlslError> {
    let module = compile_glsl_to_clif(source, &options)?;
    let jit_module = link_glsl_for_jit(module)?;
    Ok(Box::new(jit_module))
}

/// Compile and execute GLSL in RISC-V 32-bit emulator
/// Requires `emulator` feature flag to be enabled
#[cfg(feature = "emulator")]
pub fn glsl_emu_riscv32(
    source: &str,
    options: GlslOptions,
) -> Result<std::boxed::Box<dyn crate::backend::executable::GlslExecutable>, GlslError> {
    let module = compile_glsl_to_clif(source, &options)?;

    let emulator_options = match &options.run_mode {
        RunMode::Emulator {
            max_memory,
            stack_size,
            max_instructions,
            ..
        } => EmulatorOptions {
            max_memory: *max_memory,
            stack_size: *stack_size,
            max_instructions: *max_instructions,
        },
        _ => {
            return Err(GlslError::new(
                crate::error::ErrorCode::E0400,
                "Invalid run mode for emulator",
            ));
        }
    };

    let emu_module = link_glsl_for_emulator(module, &emulator_options)?;
    Ok(std::boxed::Box::new(emu_module))
}

/// Create host ISA for JIT compilation
#[cfg(feature = "std")]
fn create_host_isa() -> Result<OwnedTargetIsa, GlslError> {
    use cranelift_codegen::settings::{self, Configurable};

    let mut flag_builder = settings::builder();
    flag_builder.set("is_pic", "false").map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to set is_pic: {}", e),
        )
    })?;
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                alloc_format!("failed to set use_colocated_libcalls: {}", e),
            )
        })?;
    flag_builder
        .set("enable_multi_ret_implicit_sret", "true")
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                alloc_format!("failed to set enable_multi_ret_implicit_sret: {}", e),
            )
        })?;

    let flags = settings::Flags::new(flag_builder);
    let isa_builder = cranelift_native::builder().map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            format!("host machine is not supported: {}", e),
        )
    })?;
    isa_builder.finish(flags).map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to create host ISA: {}", e),
        )
    })
}

/// Create RISC-V 32-bit ISA for emulator compilation
#[cfg(feature = "emulator")]
fn create_riscv32_isa() -> Result<OwnedTargetIsa, GlslError> {
    use cranelift_codegen::isa::riscv32::isa_builder;
    use cranelift_codegen::settings::{self, Configurable};
    use target_lexicon::{
        Architecture, BinaryFormat, Environment, OperatingSystem, Riscv32Architecture, Triple,
        Vendor,
    };

    let mut flag_builder = settings::builder();
    flag_builder.set("is_pic", "false").map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to set is_pic: {}", e),
        )
    })?;
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                alloc_format!("failed to set use_colocated_libcalls: {}", e),
            )
        })?;
    flag_builder
        .set("enable_multi_ret_implicit_sret", "true")
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                alloc_format!("failed to set enable_multi_ret_implicit_sret: {}", e),
            )
        })?;

    let flags = settings::Flags::new(flag_builder);
    let triple = Triple {
        architecture: Architecture::Riscv32(Riscv32Architecture::Riscv32imac),
        vendor: Vendor::Unknown,
        operating_system: OperatingSystem::None_,
        environment: Environment::Unknown,
        binary_format: BinaryFormat::Elf,
    };

    isa_builder(triple).finish(flags).map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to create riscv32 ISA: {}", e),
        )
    })
}
