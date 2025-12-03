//! Compile CLIF to RISC-V32 and verify output
//!
//! This module provides functionality to:
//! - Parse CLIF (Cranelift IR) text
//! - Compile to RISC-V32 machine code using Cranelift
//! - Disassemble the generated code
//! - Verify output using filecheck patterns
//!
//! ## Error Handling
//!
//! This module uses two error types:
//! - `CompileError` - Internal error type with detailed variants for different failure modes
//! - `String` - External API error type for test functions, providing simple error messages

use std::sync::Arc;

use cranelift_codegen::ir::Function;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::{isa, Context};
use cranelift_control::ControlPlane;
use cranelift_reader::parse_functions;
use lp_riscv_tools::disasm;

/// Result of compiling a CLIF function
#[derive(Debug)]
pub struct CompiledCode {
    /// Machine code bytes
    pub code: Vec<u8>,
    /// Disassembled text
    pub disassembly: String,
    /// Function name
    pub func_name: String,
}

/// Errors that can occur during compilation
#[derive(Debug)]
pub enum CompileError {
    /// Failed to parse CLIF
    Parse(String),
    /// Failed to create ISA
    Isa(String),
    /// Failed to compile
    Compile(String),
    /// Failed to disassemble
    Disasm(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Parse(msg) => write!(f, "Parse error: {}", msg),
            CompileError::Isa(msg) => write!(f, "ISA creation error: {}", msg),
            CompileError::Compile(msg) => write!(f, "Compile error: {}", msg),
            CompileError::Disasm(msg) => write!(f, "Disassembly error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

/// Create a RISC-V32 ISA builder with standard settings
pub fn create_riscv32_isa() -> Result<Arc<dyn isa::TargetIsa>, CompileError> {
    let mut flag_builder = settings::builder();
    
    // Set optimization level
    flag_builder
        .set("opt_level", "speed")
        .map_err(|e| CompileError::Isa(format!("Failed to set opt_level: {}", e)))?;
    
    // Disable verifier for faster compilation in tests
    // The verifier is useful for development but slows down test runs
    flag_builder
        .set("enable_verifier", "false")
        .map_err(|e| CompileError::Isa(format!("Failed to disable verifier: {}", e)))?;
    
    let isa_builder = isa::lookup_by_name("riscv32")
        .map_err(|e| CompileError::Isa(format!("Failed to lookup riscv32: {}", e)))?;
    
    let flags = settings::Flags::new(flag_builder);
    
    isa_builder
        .finish(flags)
        .map_err(|e| CompileError::Isa(format!("Failed to finish ISA: {}", e)))
}

/// Parse CLIF text and return the functions
pub fn parse_clif(clif_text: &str) -> Result<Vec<Function>, CompileError> {
    parse_functions(clif_text)
        .map_err(|e| CompileError::Parse(format!("Failed to parse CLIF: {}", e)))
}

/// Compile a single CLIF function to RISC-V32 machine code
pub fn compile_function(func: Function, isa: &dyn isa::TargetIsa) -> Result<CompiledCode, CompileError> {
    let func_name = func.name.to_string();
    
    // Create compilation context
    let mut ctx = Context::new();
    ctx.func = func;
    
    // Compile with control plane
    let mut ctrl_plane = ControlPlane::default();
    let compiled = ctx
        .compile(isa, &mut ctrl_plane)
        .map_err(|e| CompileError::Compile(format!("Compilation failed: {:?}", e)))?;
    
    // Extract machine code
    let code = compiled.code_buffer().to_vec();
    
    // Disassemble
    let disassembly = disasm::disassemble_code(&code);
    
    Ok(CompiledCode {
        code,
        disassembly,
        func_name,
    })
}

/// Compile CLIF text to RISC-V32 machine code
pub fn compile_clif(clif_text: &str) -> Result<Vec<CompiledCode>, CompileError> {
    // Parse CLIF
    let funcs = parse_clif(clif_text)?;
    
    if funcs.is_empty() {
        return Err(CompileError::Parse("No functions found in CLIF".to_string()));
    }
    
    // Create ISA
    let isa = create_riscv32_isa()?;
    
    // Compile all functions
    let mut results = Vec::new();
    for func in funcs {
        results.push(compile_function(func, &*isa)?);
    }
    
    Ok(results)
}

/// Extract filecheck patterns from CLIF text
///
/// Patterns are lines starting with `;` followed by filecheck directives
/// like `; check:`, `; nextln:`, etc.
pub fn extract_filecheck_patterns(clif_text: &str) -> Option<String> {
    let mut patterns = Vec::new();
    
    for line in clif_text.lines() {
        let trimmed = line.trim();
        
        // Check if this is a comment line with a filecheck directive
        if trimmed.starts_with(';') {
            let comment_content = trimmed[1..].trim();
            
            // Check for filecheck directives
            if comment_content.starts_with("check:")
                || comment_content.starts_with("check-NOT:")
                || comment_content.starts_with("nextln:")
                || comment_content.starts_with("sameln:")
                || comment_content.starts_with("CHECK:")
                || comment_content.starts_with("CHECK-NOT:")
                || comment_content.starts_with("CHECK-NEXT:")
                || comment_content.starts_with("CHECK-SAME:")
            {
                patterns.push(comment_content.to_string());
            }
        }
    }
    
    if patterns.is_empty() {
        None
    } else {
        Some(patterns.join("\n"))
    }
}

/// Run a compile test with filecheck verification
pub fn run_compile_test(clif_text: &str) -> Result<(), String> {
    // Compile
    let compiled = compile_clif(clif_text).map_err(|e| e.to_string())?;
    
    // Extract filecheck patterns
    if let Some(patterns) = extract_filecheck_patterns(clif_text) {
        // Run filecheck on each compiled function
        for result in &compiled {
            crate::filecheck::match_filecheck(&result.disassembly, &patterns)?;
        }
    }
    
    Ok(())
}

/// Run an execution test that compiles CLIF and runs it in the emulator
///
/// # Arguments
/// * `clif_text` - CLIF source code
/// * `setup_fn` - Function to set up emulator state (registers, memory) before execution
/// * `verify_fn` - Function to verify emulator state after execution
pub fn run_execution_test<S, V>(
    clif_text: &str,
    setup_fn: S,
    verify_fn: V,
) -> Result<(), String>
where
    S: FnOnce(&mut lp_riscv_tools::emu::emulator::Riscv32Emulator),
    V: FnOnce(&lp_riscv_tools::emu::emulator::Riscv32Emulator) -> Result<(), String>,
{
    use lp_riscv_tools::emu::emulator::Riscv32Emulator;
    
    // Compile
    let compiled = compile_clif(clif_text).map_err(|e| e.to_string())?;
    
    if compiled.is_empty() {
        return Err("No functions compiled".to_string());
    }
    
    // Use first compiled function
    let compiled_code = &compiled[0];
    
    // Create emulator with code and some RAM
    let mut emu = Riscv32Emulator::new(
        compiled_code.code.clone(),
        vec![0; 4096], // 4KB RAM
    );
    
    // PC starts at 0 (where code is loaded)
    emu.set_pc(0);
    
    // Run setup function
    setup_fn(&mut emu);
    
    // Execute until return or halt
    let max_steps = 1000;
    let mut seen_pcs = std::collections::HashSet::new();
    seen_pcs.insert(emu.get_pc());
    
    for _ in 0..max_steps {
        use lp_riscv_tools::emu::emulator::StepResult;
        
        match emu.step() {
            Ok(StepResult::Continue) => {
                let pc = emu.get_pc();
                // If we've seen this PC before, we're likely in an infinite loop
                // This typically happens when the function returns and PC goes back to ra (0)
                if !seen_pcs.insert(pc) {
                    break;
                }
            }
            Ok(StepResult::Halted) => {
                // Hit an EBREAK or similar, stop execution
                break;
            }
            Ok(StepResult::Syscall(_)) => {
                // Syscall encountered, for simple tests we can stop here
                break;
            }
            Err(e) => {
                return Err(format!("Emulator error: {:?}", e));
            }
        }
    }
    
    // Run verification
    verify_fn(&emu)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_function() {
        let clif = r#"
function %add(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}
"#;
        
        let funcs = parse_clif(clif).unwrap();
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name.to_string(), "%add");
    }
    
    #[test]
    fn test_extract_filecheck_patterns() {
        let clif = r#"
function %add(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}

; check: add
; check-NOT: addw
"#;
        
        let patterns = extract_filecheck_patterns(clif).unwrap();
        assert!(patterns.contains("check: add"));
        assert!(patterns.contains("check-NOT: addw"));
    }
    
    #[test]
    fn test_create_isa() {
        let isa = create_riscv32_isa();
        assert!(isa.is_ok());
    }
    
    // File-based tests - Arithmetic operations
    
    #[test]
    fn test_iadd() {
        let content = include_str!("../filetests/riscv32/iadd.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_isub() {
        let content = include_str!("../filetests/riscv32/isub.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_imul() {
        let content = include_str!("../filetests/riscv32/imul.clif");
        run_compile_test(content).unwrap();
    }
    
    // Division and remainder
    
    #[test]
    fn test_udiv() {
        let content = include_str!("../filetests/riscv32/udiv.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_sdiv() {
        let content = include_str!("../filetests/riscv32/sdiv.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_urem() {
        let content = include_str!("../filetests/riscv32/urem.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_srem() {
        let content = include_str!("../filetests/riscv32/srem.clif");
        run_compile_test(content).unwrap();
    }
    
    // Shift operations
    
    #[test]
    fn test_ishl() {
        let content = include_str!("../filetests/riscv32/ishl.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_ushr() {
        let content = include_str!("../filetests/riscv32/ushr.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_sshr() {
        let content = include_str!("../filetests/riscv32/sshr.clif");
        run_compile_test(content).unwrap();
    }
    
    // Bitwise operations
    
    #[test]
    fn test_band() {
        let content = include_str!("../filetests/riscv32/band.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_bor() {
        let content = include_str!("../filetests/riscv32/bor.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_bxor() {
        let content = include_str!("../filetests/riscv32/bxor.clif");
        run_compile_test(content).unwrap();
    }
    
    // Constants and memory operations
    
    #[test]
    fn test_iconst() {
        let content = include_str!("../filetests/riscv32/iconst.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_load() {
        let content = include_str!("../filetests/riscv32/load.clif");
        run_compile_test(content).unwrap();
    }
    
    #[test]
    fn test_store() {
        let content = include_str!("../filetests/riscv32/store.clif");
        run_compile_test(content).unwrap();
    }
    
    // Execution tests - verify code actually runs correctly
    
    #[test]
    fn test_iadd_disasm() {
        // Debug test to see what code is generated
        let clif = include_str!("../filetests/riscv32/iadd.clif");
        let compiled = compile_clif(clif).unwrap();
        println!("Generated code for iadd:");
        println!("{}", compiled[0].disassembly);
        println!("Code bytes: {:02x?}", &compiled[0].code);
    }
    
    #[test]
    fn test_iadd_execution() {
        use lp_riscv_tools::Gpr;
        let clif = include_str!("../filetests/riscv32/iadd.clif");
        
        run_execution_test(
            clif,
            |emu| {
                // Set up arguments: a0 = 10, a1 = 32
                emu.set_register(Gpr::A0, 10);
                emu.set_register(Gpr::A1, 32);
                emu.set_register(Gpr::Ra, 0);  // Return to 0 will halt
            },
            |emu| {
                // Verify result: a0 should be 10 + 32 = 42
                let result = emu.get_register(Gpr::A0);
                if result == 42 {
                    Ok(())
                } else {
                    Err(format!("Expected a0 = 42, got {}", result))
                }
            }
        ).unwrap();
    }
    
    #[test]
    fn test_isub_execution() {
        use lp_riscv_tools::Gpr;
        let clif = include_str!("../filetests/riscv32/isub.clif");
        
        run_execution_test(
            clif,
            |emu| {
                emu.set_register(Gpr::A0, 100);
                emu.set_register(Gpr::A1, 42);
                emu.set_register(Gpr::Ra, 0);
            },
            |emu| {
                let result = emu.get_register(Gpr::A0);
                if result == 58 {
                    Ok(())
                } else {
                    Err(format!("Expected a0 = 58, got {}", result))
                }
            }
        ).unwrap();
    }
    
    #[test]
    fn test_imul_execution() {
        use lp_riscv_tools::Gpr;
        let clif = include_str!("../filetests/riscv32/imul.clif");
        
        run_execution_test(
            clif,
            |emu| {
                emu.set_register(Gpr::A0, 6);
                emu.set_register(Gpr::A1, 7);
                emu.set_register(Gpr::Ra, 0);
            },
            |emu| {
                let result = emu.get_register(Gpr::A0);
                if result == 42 {
                    Ok(())
                } else {
                    Err(format!("Expected a0 = 42, got {}", result))
                }
            }
        ).unwrap();
    }
    
    #[test]
    fn test_band_execution() {
        use lp_riscv_tools::Gpr;
        let clif = include_str!("../filetests/riscv32/band.clif");
        
        run_execution_test(
            clif,
            |emu| {
                emu.set_register(Gpr::A0, 0b1111);  // 15
                emu.set_register(Gpr::A1, 0b1010);  // 10
                emu.set_register(Gpr::Ra, 0);
            },
            |emu| {
                let result = emu.get_register(Gpr::A0);
                if result == 10 {
                    Ok(())
                } else {
                    Err(format!("Expected a0 = 10, got {}", result))
                }
            }
        ).unwrap();
    }
    
    #[test]
    fn test_bor_execution() {
        use lp_riscv_tools::Gpr;
        let clif = include_str!("../filetests/riscv32/bor.clif");
        
        run_execution_test(
            clif,
            |emu| {
                emu.set_register(Gpr::A0, 0b1100);  // 12
                emu.set_register(Gpr::A1, 0b0011);  // 3
                emu.set_register(Gpr::Ra, 0);
            },
            |emu| {
                let result = emu.get_register(Gpr::A0);
                if result == 15 {
                    Ok(())
                } else {
                    Err(format!("Expected a0 = 15, got {}", result))
                }
            }
        ).unwrap();
    }
    
    #[test]
    fn test_bxor_execution() {
        use lp_riscv_tools::Gpr;
        let clif = include_str!("../filetests/riscv32/bxor.clif");
        
        run_execution_test(
            clif,
            |emu| {
                emu.set_register(Gpr::A0, 0b1111);  // 15
                emu.set_register(Gpr::A1, 0b1010);  // 10
                emu.set_register(Gpr::Ra, 0);
            },
            |emu| {
                let result = emu.get_register(Gpr::A0);
                if result == 5 {
                    Ok(())
                } else {
                    Err(format!("Expected a0 = 5, got {}", result))
                }
            }
        ).unwrap();
    }
    
    #[test]
    fn test_ishl_execution() {
        use lp_riscv_tools::Gpr;
        let clif = include_str!("../filetests/riscv32/ishl.clif");
        
        run_execution_test(
            clif,
            |emu| {
                emu.set_register(Gpr::A0, 3);
                emu.set_register(Gpr::A1, 4);
                emu.set_register(Gpr::Ra, 0);
            },
            |emu| {
                let result = emu.get_register(Gpr::A0);
                if result == 48 {
                    Ok(())
                } else {
                    Err(format!("Expected a0 = 48, got {}", result))
                }
            }
        ).unwrap();
    }
    
    #[test]
    fn test_ushr_execution() {
        use lp_riscv_tools::Gpr;
        let clif = include_str!("../filetests/riscv32/ushr.clif");
        
        run_execution_test(
            clif,
            |emu| {
                emu.set_register(Gpr::A0, 64);
                emu.set_register(Gpr::A1, 2);
                emu.set_register(Gpr::Ra, 0);
            },
            |emu| {
                let result = emu.get_register(Gpr::A0);
                if result == 16 {
                    Ok(())
                } else {
                    Err(format!("Expected a0 = 16, got {}", result))
                }
            }
        ).unwrap();
    }
    
    #[test]
    fn test_iconst_execution() {
        use lp_riscv_tools::Gpr;
        let clif = include_str!("../filetests/riscv32/iconst.clif");
        
        run_execution_test(
            clif,
            |emu| {
                emu.set_register(Gpr::Ra, 0);
            },
            |emu| {
                let result = emu.get_register(Gpr::A0);
                if result == 42 {
                    Ok(())
                } else {
                    Err(format!("Expected a0 = 42, got {}", result))
                }
            }
        ).unwrap();
    }
}

