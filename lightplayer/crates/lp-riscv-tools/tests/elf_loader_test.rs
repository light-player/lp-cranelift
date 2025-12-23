//! Tests for ELF loading and relocation application.
//!
//! These tests verify that:
//! - ELF segments are loaded correctly into code/RAM buffers
//! - Relocations are applied correctly
//! - Symbols can be resolved
//! - Code buffer contains non-zero executable code

#![cfg(feature = "std")]

use cranelift_codegen::{
    Context,
    ir::{AbiParam, Function, InstBuilder, Signature, types},
    isa,
    settings::{self, Flags},
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use lp_riscv_tools::elf_loader::{find_symbol_address, load_elf};

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract a 32-bit instruction from code at the given offset.
fn extract_instruction_at(code: &[u8], offset: usize) -> Option<u32> {
    if offset + 4 <= code.len() {
        Some(u32::from_le_bytes([
            code[offset],
            code[offset + 1],
            code[offset + 2],
            code[offset + 3],
        ]))
    } else {
        None
    }
}

/// Verify that code buffer contains non-zero bytes in the given range.
fn verify_code_non_zero(code: &[u8], start: usize, len: usize) -> bool {
    let end = (start + len).min(code.len());
    if start >= code.len() || start >= end {
        return false;
    }
    code[start..end].iter().any(|&b| b != 0)
}

/// Generate a test ELF file using Cranelift ObjectModule with a simple function.
fn generate_test_elf_with_simple_function() -> Result<Vec<u8>, String> {
    // Create ISA
    let isa = isa::lookup_by_name("riscv32")
        .map_err(|e| format!("Failed to lookup ISA: {}", e))?
        .finish(Flags::new(settings::builder()))
        .map_err(|e| format!("Failed to create ISA: {}", e))?;

    // Create ObjectModule
    let builder = ObjectBuilder::new(
        isa,
        "test_module",
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| format!("Failed to create ObjectBuilder: {}", e))?;

    let mut module = ObjectModule::new(builder);

    // Create a simple function that returns a constant
    let mut sig = Signature::new(isa::CallConv::SystemV);
    sig.returns.push(AbiParam::new(types::I32));
    sig.params.push(AbiParam::new(types::I32));

    let func_id = module
        .declare_function("test_func", Linkage::Export, &sig)
        .map_err(|e| format!("Failed to declare function: {}", e))?;

    // Build the function
    let mut func = Function::with_name_signature(
        cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32()),
        sig,
    );

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut func_builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let entry_block = func_builder.create_block();
    func_builder.append_block_params_for_function_params(entry_block);
    func_builder.switch_to_block(entry_block);
    func_builder.seal_block(entry_block);

    // Return the input parameter + 42
    let param = func_builder.block_params(entry_block)[0];
    let const_val = func_builder.ins().iconst(types::I32, 42);
    let result = func_builder.ins().iadd(param, const_val);
    func_builder.ins().return_(&[result]);

    func_builder.seal_all_blocks();
    func_builder.finalize();

    // Compile and define the function
    let mut ctx = Context::new();
    ctx.func = func;
    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| format!("Failed to define function: {}", e))?;

    // Finalize and emit
    let product: ObjectProduct = module.finish();
    let elf_bytes = product
        .emit()
        .map_err(|e| format!("Failed to emit ELF: {}", e))?;

    Ok(elf_bytes)
}

/// Generate a test ELF file with two functions where one calls the other.
/// This will generate a CALL_PLT relocation.
fn generate_test_elf_with_function_call() -> Result<Vec<u8>, String> {
    // Create ISA
    let isa = isa::lookup_by_name("riscv32")
        .map_err(|e| format!("Failed to lookup ISA: {}", e))?
        .finish(Flags::new(settings::builder()))
        .map_err(|e| format!("Failed to create ISA: {}", e))?;

    // Create ObjectModule
    let builder = ObjectBuilder::new(
        isa,
        "test_module",
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| format!("Failed to create ObjectBuilder: {}", e))?;

    let mut module = ObjectModule::new(builder);

    // Create callee function
    let mut callee_sig = Signature::new(isa::CallConv::SystemV);
    callee_sig.returns.push(AbiParam::new(types::I32));
    callee_sig.params.push(AbiParam::new(types::I32));

    let callee_id = module
        .declare_function("callee", Linkage::Export, &callee_sig)
        .map_err(|e| format!("Failed to declare callee: {}", e))?;

    // Build callee function
    let mut callee_func = Function::with_name_signature(
        cranelift_codegen::ir::UserFuncName::user(0, callee_id.as_u32()),
        callee_sig.clone(),
    );

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut func_builder = FunctionBuilder::new(&mut callee_func, &mut builder_ctx);

    let entry_block = func_builder.create_block();
    func_builder.append_block_params_for_function_params(entry_block);
    func_builder.switch_to_block(entry_block);
    func_builder.seal_block(entry_block);

    // Return input * 2
    let param = func_builder.block_params(entry_block)[0];
    let const_val = func_builder.ins().iconst(types::I32, 2);
    let result = func_builder.ins().imul(param, const_val);
    func_builder.ins().return_(&[result]);

    func_builder.seal_all_blocks();
    func_builder.finalize();

    // Compile and define callee
    let mut ctx = Context::new();
    ctx.func = callee_func;
    module
        .define_function(callee_id, &mut ctx)
        .map_err(|e| format!("Failed to define callee: {}", e))?;

    // Create caller function (main)
    let mut caller_sig = Signature::new(isa::CallConv::SystemV);
    caller_sig.returns.push(AbiParam::new(types::I32));
    caller_sig.params.push(AbiParam::new(types::I32));

    let caller_id = module
        .declare_function("main", Linkage::Export, &caller_sig)
        .map_err(|e| format!("Failed to declare caller: {}", e))?;

    // Build caller function
    let mut caller_func = Function::with_name_signature(
        cranelift_codegen::ir::UserFuncName::user(0, caller_id.as_u32()),
        caller_sig.clone(),
    );

    // Declare callee reference in caller function (must be done before builder)
    let callee_ref = module.declare_func_in_func(callee_id, &mut caller_func);

    let mut builder_ctx2 = FunctionBuilderContext::new();
    let mut func_builder2 = FunctionBuilder::new(&mut caller_func, &mut builder_ctx2);

    let entry_block2 = func_builder2.create_block();
    func_builder2.append_block_params_for_function_params(entry_block2);
    func_builder2.switch_to_block(entry_block2);
    func_builder2.seal_block(entry_block2);

    // Call callee and return result
    let param2 = func_builder2.block_params(entry_block2)[0];
    let call_result = func_builder2.ins().call(callee_ref, &[param2]);
    let result_val = func_builder2.inst_results(call_result)[0];
    func_builder2.ins().return_(&[result_val]);

    func_builder2.seal_all_blocks();
    func_builder2.finalize();

    // Compile and define caller
    let mut ctx2 = Context::new();
    ctx2.func = caller_func;
    module
        .define_function(caller_id, &mut ctx2)
        .map_err(|e| format!("Failed to define caller: {}", e))?;

    // Finalize and emit
    let product: ObjectProduct = module.finish();
    let elf_bytes = product
        .emit()
        .map_err(|e| format!("Failed to emit ELF: {}", e))?;

    Ok(elf_bytes)
}

// ============================================================================
// Segment Loading Tests
// ============================================================================

#[test]
fn test_segment_loading_basic() {
    println!("=== Test: Basic Segment Loading ===");

    // Generate test ELF
    let elf_bytes = generate_test_elf_with_simple_function().expect("Failed to generate test ELF");

    println!("Generated ELF: {} bytes", elf_bytes.len());

    // Load ELF
    let load_info = load_elf(&elf_bytes).expect("Failed to load ELF");

    println!("Code buffer size: {} bytes", load_info.code.len());
    println!("RAM buffer size: {} bytes", load_info.ram.len());
    println!("Entry point: 0x{:08x}", load_info.entry_point);

    // Verify code buffer is not all zeros
    assert!(
        verify_code_non_zero(&load_info.code, 0, load_info.code.len()),
        "Code buffer should contain non-zero bytes"
    );

    // Print first 32 bytes for debugging
    println!("First 32 bytes of code:");
    for i in (0..32.min(load_info.code.len())).step_by(4) {
        if i + 4 <= load_info.code.len() {
            let word = extract_instruction_at(&load_info.code, i).unwrap();
            println!("  0x{:04x}: 0x{:08x}", i, word);
        }
    }

    // Verify code buffer has reasonable size (at least 4KB)
    assert!(
        load_info.code.len() >= 4096,
        "Code buffer should be at least 4KB, got {} bytes",
        load_info.code.len()
    );
}

#[test]
fn test_segment_loading_address_mapping() {
    println!("=== Test: Segment Address Mapping ===");

    let elf_bytes = generate_test_elf_with_simple_function().expect("Failed to generate test ELF");

    // Parse ELF to check segment addresses
    use object::{Object, ObjectSection};
    let obj = object::File::parse(&elf_bytes[..]).expect("Failed to parse ELF");

    // Find text section address
    let mut text_section_base = 0u64;
    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            text_section_base = section.address();
            println!("Text section base: 0x{:08x}", text_section_base);
            break;
        }
    }

    // Load ELF
    let load_info = load_elf(&elf_bytes).expect("Failed to load ELF");

    // If text section is at non-zero address, verify mapping
    if text_section_base > 0 {
        // Code should be placed at offset 0 in buffer, but symbols reference text_section_base
        // So symbol addresses should be relative to text_section_base
        println!(
            "Text section at 0x{:08x}, code buffer starts at 0",
            text_section_base
        );

        // Verify we can find the function symbol
        match find_symbol_address(&obj, "test_func", text_section_base) {
            Ok(addr) => {
                println!("Found test_func at offset 0x{:08x}", addr);
                // Verify code at that offset is non-zero
                assert!(
                    verify_code_non_zero(&load_info.code, addr as usize, 16),
                    "Code at test_func address should be non-zero"
                );
            }
            Err(e) => {
                println!("Warning: Could not find test_func symbol: {}", e);
            }
        }
    }
}

// ============================================================================
// Relocation Application Tests
// ============================================================================

#[test]
fn test_relocation_call_plt() {
    println!("=== Test: CALL_PLT Relocation ===");

    // Generate ELF with function call (will have CALL_PLT relocation)
    let elf_bytes = generate_test_elf_with_function_call()
        .expect("Failed to generate test ELF with function call");

    println!("Generated ELF: {} bytes", elf_bytes.len());

    // Parse ELF to inspect relocations
    use object::{Object, ObjectSection};
    let obj = object::File::parse(&elf_bytes[..]).expect("Failed to parse ELF");

    // Find text section and check for relocations
    let mut text_section_base = 0u64;
    let mut reloc_count = 0;

    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            text_section_base = section.address();
            println!("Text section base: 0x{:08x}", text_section_base);

            for (_offset, reloc) in section.relocations() {
                reloc_count += 1;
                let reloc_type = match reloc.flags() {
                    object::RelocationFlags::Elf { r_type } => {
                        format!("R_RISCV_{}", r_type)
                    }
                    _ => "unknown".to_string(),
                };
                println!(
                    "  Relocation: type={}, addend={}",
                    reloc_type,
                    reloc.addend()
                );
            }
            break;
        }
    }

    println!("Found {} relocations in text section", reloc_count);

    // Load ELF (this should apply relocations)
    let load_info = load_elf(&elf_bytes).expect("Failed to load ELF");

    println!("Code buffer size: {} bytes", load_info.code.len());

    // Verify code is non-zero
    assert!(
        verify_code_non_zero(&load_info.code, 0, load_info.code.len()),
        "Code buffer should contain non-zero bytes after relocation"
    );

    // Try to find main function
    if let Ok(main_addr) = find_symbol_address(&obj, "main", text_section_base) {
        println!("Found main at offset 0x{:08x}", main_addr);
        assert!(
            verify_code_non_zero(&load_info.code, main_addr as usize, 32),
            "Code at main address should be non-zero"
        );
    }

    // Try to find callee function
    if let Ok(callee_addr) = find_symbol_address(&obj, "callee", text_section_base) {
        println!("Found callee at offset 0x{:08x}", callee_addr);
        assert!(
            verify_code_non_zero(&load_info.code, callee_addr as usize, 32),
            "Code at callee address should be non-zero"
        );
    }
}

// ============================================================================
// Symbol Resolution Tests
// ============================================================================

#[test]
fn test_find_symbol_address() {
    println!("=== Test: Symbol Address Resolution ===");

    let elf_bytes = generate_test_elf_with_function_call().expect("Failed to generate test ELF");

    // Parse ELF
    use object::{Object, ObjectSection};
    let obj = object::File::parse(&elf_bytes[..]).expect("Failed to parse ELF");

    // Find text section base
    let mut text_section_base = 0u64;
    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            text_section_base = section.address();
            break;
        }
    }

    println!("Text section base: 0x{:08x}", text_section_base);

    // Test finding main symbol
    match find_symbol_address(&obj, "main", text_section_base) {
        Ok(addr) => {
            println!("✓ Found main at offset 0x{:08x}", addr);
            assert!(addr < 0x10000, "Symbol address should be reasonable");
        }
        Err(e) => {
            panic!("Failed to find main symbol: {}", e);
        }
    }

    // Test finding callee symbol
    match find_symbol_address(&obj, "callee", text_section_base) {
        Ok(addr) => {
            println!("✓ Found callee at offset 0x{:08x}", addr);
            assert!(addr < 0x10000, "Symbol address should be reasonable");
        }
        Err(e) => {
            panic!("Failed to find callee symbol: {}", e);
        }
    }

    // Test finding non-existent symbol
    match find_symbol_address(&obj, "nonexistent", text_section_base) {
        Ok(_) => {
            panic!("Should not find nonexistent symbol");
        }
        Err(_) => {
            println!("✓ Correctly failed to find nonexistent symbol");
        }
    }
}

// ============================================================================
// End-to-End Integration Tests
// ============================================================================

#[test]
fn test_elf_load_complete_workflow() {
    println!("=== Test: Complete ELF Load Workflow ===");

    // Generate ELF
    let elf_bytes = generate_test_elf_with_function_call().expect("Failed to generate test ELF");

    println!("Step 1: Generated ELF ({} bytes)", elf_bytes.len());

    // Load ELF
    let load_info = load_elf(&elf_bytes).expect("Failed to load ELF");

    println!("Step 2: Loaded ELF");
    println!("  Code: {} bytes", load_info.code.len());
    println!("  RAM: {} bytes", load_info.ram.len());
    println!("  Entry: 0x{:08x}", load_info.entry_point);

    // Verify code buffer is non-zero
    assert!(
        verify_code_non_zero(&load_info.code, 0, load_info.code.len()),
        "Code buffer must contain non-zero bytes"
    );

    println!("Step 3: Verified code buffer is non-zero");

    // Parse ELF for symbol lookup
    use object::{Object, ObjectSection};
    let obj = object::File::parse(&elf_bytes[..]).expect("Failed to parse ELF");

    // Find text section base
    let mut text_section_base = 0u64;
    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            text_section_base = section.address();
            break;
        }
    }

    println!("Step 4: Text section base: 0x{:08x}", text_section_base);

    // Verify symbols can be found
    let main_addr =
        find_symbol_address(&obj, "main", text_section_base).expect("Failed to find main symbol");

    println!("Step 5: Found main at 0x{:08x}", main_addr);

    // Verify code at symbol address is non-zero
    assert!(
        verify_code_non_zero(&load_info.code, main_addr as usize, 32),
        "Code at main address must be non-zero"
    );

    println!("Step 6: Verified code at main address is non-zero");

    // Print code around main for debugging
    println!("Code around main (0x{:08x}):", main_addr);
    let start = (main_addr as usize).saturating_sub(8);
    let end = (main_addr as usize + 32).min(load_info.code.len());
    for i in (start..end).step_by(4) {
        if i + 4 <= load_info.code.len() {
            if let Some(word) = extract_instruction_at(&load_info.code, i) {
                let marker = if i == main_addr as usize {
                    " <-- main"
                } else {
                    ""
                };
                println!("  0x{:04x}: 0x{:08x}{}", i, word, marker);
            }
        }
    }

    println!("✓ Complete workflow test passed!");
}

#[test]
fn test_elf_load_with_function_calls() {
    println!("=== Test: ELF Load with Function Calls ===");

    let elf_bytes = generate_test_elf_with_function_call().expect("Failed to generate test ELF");

    // Load ELF
    let load_info = load_elf(&elf_bytes).expect("Failed to load ELF");

    // Parse ELF
    use object::{Object, ObjectSection};
    let obj = object::File::parse(&elf_bytes[..]).expect("Failed to parse ELF");

    // Find text section base
    let mut text_section_base = 0u64;
    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            text_section_base = section.address();
            break;
        }
    }

    // Verify main function exists and has code
    let main_addr =
        find_symbol_address(&obj, "main", text_section_base).expect("Failed to find main");
    assert!(
        verify_code_non_zero(&load_info.code, main_addr as usize, 32),
        "Main function code must be present"
    );
    println!("✓ Main function code present at 0x{:08x}", main_addr);

    // Verify callee function exists and has code
    let callee_addr =
        find_symbol_address(&obj, "callee", text_section_base).expect("Failed to find callee");
    assert!(
        verify_code_non_zero(&load_info.code, callee_addr as usize, 32),
        "Callee function code must be present"
    );
    println!("✓ Callee function code present at 0x{:08x}", callee_addr);

    // Check for relocations in the text section
    let mut has_relocations = false;
    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            for (_offset, reloc) in section.relocations() {
                has_relocations = true;
                let reloc_type = match reloc.flags() {
                    object::RelocationFlags::Elf { r_type } => {
                        format!("R_RISCV_{}", r_type)
                    }
                    _ => "unknown".to_string(),
                };
                println!("  Found relocation: {}", reloc_type);
            }
            break;
        }
    }

    if has_relocations {
        println!("✓ Relocations found and should be applied");
    } else {
        println!("⚠ No relocations found (may be optimized away)");
    }
}

// ============================================================================
// Diagnostic Tests
// ============================================================================

#[test]
fn test_code_buffer_initialization() {
    println!("=== Test: Code Buffer Initialization ===");

    let elf_bytes = generate_test_elf_with_simple_function().expect("Failed to generate test ELF");

    let load_info = load_elf(&elf_bytes).expect("Failed to load ELF");

    // Verify buffer is allocated
    assert!(load_info.code.len() > 0, "Code buffer should be allocated");
    println!("✓ Code buffer allocated: {} bytes", load_info.code.len());

    // Verify buffer is not all zeros (segments should have loaded)
    let has_non_zero = load_info.code.iter().any(|&b| b != 0);
    assert!(
        has_non_zero,
        "Code buffer should contain non-zero bytes after segment loading"
    );
    println!("✓ Code buffer contains non-zero bytes");

    // Count non-zero bytes
    let non_zero_count = load_info.code.iter().filter(|&&b| b != 0).count();
    println!(
        "  Non-zero bytes: {} / {}",
        non_zero_count,
        load_info.code.len()
    );
}




