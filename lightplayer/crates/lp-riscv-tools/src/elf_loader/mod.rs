//! Modular ELF file loader for RISC-V emulator.
//!
//! This module provides utilities to load RISC-V ELF files into the emulator's memory.
//! It handles section loading and relocation application.

#![cfg(feature = "std")]

mod memory;
mod parse;
mod layout;
mod sections;
mod symbols;
mod relocations;

use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use object::{Object, ObjectSection};
use crate::debug;

// Re-export public types and functions
pub use symbols::find_symbol_address;

/// Information extracted from an ELF file for emulator loading.
pub struct ElfLoadInfo {
    /// Code/ROM region (starts at address 0) with relocations applied
    pub code: Vec<u8>,
    /// RAM region (starts at 0x80000000)
    pub ram: Vec<u8>,
    /// Entry point address
    pub entry_point: u32,
}

/// Load a RISC-V ELF file and extract code and data sections for the emulator.
///
/// This function:
/// - Parses the ELF file using the object crate
/// - Validates it's RISC-V 32-bit
/// - Calculates memory layout (ROM vs RAM)
/// - Loads sections into appropriate buffers
/// - Builds symbol map for relocations
/// - Applies relocations to all sections (code and data)
/// - Returns the entry point address
pub fn load_elf(elf_data: &[u8]) -> Result<ElfLoadInfo, String> {
    debug!("=== Loading ELF file ===");
    
    // Step 1: Parse ELF
    let obj = parse::parse_elf(elf_data)?;
    
    // Step 2: Validate ELF
    parse::validate_elf(&obj)?;
    
    // Step 3: Extract entry point
    let entry_point = parse::extract_entry_point(&obj);
    
    // Step 4: Calculate memory layout
    let layout = layout::calculate_memory_layout(&obj, entry_point)?;
    
    // Step 5: Allocate buffers
    let mut code = vec![0u8; layout.rom_size];
    let mut ram = vec![0u8; layout.ram_size];
    
    // Step 6: Load sections
    sections::load_sections(&obj, &mut code, &mut ram)?;
    
    // Step 7: Build symbol map
    // Find text section base for symbol address calculation
    let mut text_base = 0u64;
    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            text_base = section.address();
            break;
        }
    }
    let symbol_map = symbols::build_symbol_map(&obj, text_base);
    
    // Step 8: Apply relocations
    relocations::apply_relocations(&obj, &mut code, &mut ram, &symbol_map)?;
    
    debug!("=== ELF loading complete ===");
    debug!("Code size: {} bytes, RAM size: {} bytes, Entry point: 0x{:x}", 
           code.len(), ram.len(), entry_point);
    
    Ok(ElfLoadInfo {
        code,
        ram,
        entry_point,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emu::{Riscv32Emulator, LogLevel, StepResult};
    use crate::Gpr;
    use std::println;

    /// Find the builtins executable path (similar to executable_linker logic)
    fn find_builtins_executable() -> Option<Vec<u8>> {
        use std::env;

        let target = "riscv32imac-unknown-none-elf";

        // Try to find workspace root
        let mut current_dir = env::current_dir().ok()?;
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(contents) = std::fs::read_to_string(&cargo_toml) {
                    if contents.contains("[workspace]") {
                        break;
                    }
                }
            }
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                return None;
            }
        }

        // Try both debug and release profiles
        for profile in ["debug", "release"].iter() {
            // Path to the executable
            // Try both workspace root and lightplayer/ subdirectory
            let exe_path = current_dir
                .join("lightplayer")
                .join("target")
                .join(target)
                .join(profile)
                .join("lp-builtins-app");

            // If not found, try workspace root directly (for when running from lightplayer/)
            let exe_path = if exe_path.exists() {
                exe_path
            } else {
                current_dir
                    .join("target")
                    .join(target)
                    .join(profile)
                    .join("lp-builtins-app")
            };

            if exe_path.exists() {
                return std::fs::read(&exe_path).ok();
            }
        }

        None
    }

    #[test]
    fn test_load_and_run_bootstrap_app() {
        // Find the bootstrap app executable
        let builtins_exe = match find_builtins_executable() {
            Some(bytes) => {
                if bytes.is_empty() {
                    println!("Skipping test: builtins executable is empty");
                    return;
                }
                bytes
            }
            None => {
                println!(
                    "Skipping test: builtins executable not found. Build it with: scripts/build-builtins.sh"
                );
                return;
            }
        };

        // Load the ELF
        let load_info = match load_elf(&builtins_exe) {
            Ok(info) => info,
            Err(e) => {
                panic!("Failed to load bootstrap app ELF: {}", e);
            }
        };

        // Get RAM size before moving it into emulator
        let ram_size = load_info.ram.len();

        // Create emulator with instruction-level logging
        let mut emu = Riscv32Emulator::new(load_info.code, load_info.ram)
            .with_log_level(LogLevel::Instructions);

        // Initialize stack pointer (sp = x2) to point to high RAM
        let sp_value = 0x80000000u32.wrapping_add((ram_size as u32).wrapping_sub(16));
        emu.set_register(Gpr::Sp, sp_value as i32);

        // Set return address (ra = x1) to halt address so function can return
        let halt_address = 0x80000000u32.wrapping_add(ram_size as u32);
        emu.set_register(Gpr::Ra, halt_address as i32);

        // Set PC to entry point
        emu.set_pc(load_info.entry_point);

        // Run until halt, panic, or max steps
        let mut steps = 0;
        let max_steps = 10000;
        loop {
            if steps >= max_steps {
                println!("\n=== Emulator exceeded {} steps - possible infinite loop ===", max_steps);
                println!("PC: 0x{:x}", emu.get_pc());
                println!("\n=== Emulator State ===");
                println!("{}", emu.dump_state());
                println!("\n=== Debug Info ===");
                println!("{}", emu.format_debug_info(Some(emu.get_pc()), 50));
                panic!("Emulator exceeded {} steps", max_steps);
            }

            match emu.step() {
                Ok(step_result) => {
                    steps += 1;
                    let pc_after = emu.get_pc();

                    // Handle panic result
                    if let StepResult::Panic(panic_info) = step_result {
                        println!("\n=== Panic Detected ===");
                        println!("Panic message: {}", panic_info.message);
                        if let Some(ref file) = panic_info.file {
                            if let Some(line) = panic_info.line {
                                println!("  at {}:{}", file, line);
                            } else {
                                println!("  at {}", file);
                            }
                        } else if let Some(line) = panic_info.line {
                            println!("  at line {}", line);
                        } else {
                            println!("  (no file/line information available, PC: 0x{:x})", panic_info.pc);
                        }
                        println!("PC: 0x{:x}", panic_info.pc);
                        println!("\n=== Emulator State ===");
                        println!("{}", emu.dump_state());
                        println!("\n=== Debug Info ===");
                        println!("{}", emu.format_debug_info(Some(emu.get_pc()), 50));
                        panic!("Panic occurred in bootstrap app: {}", panic_info.message);
                    }

                    // Handle halt result
                    if let StepResult::Halted = step_result {
                        break;
                    }

                    // Check if PC is at halt address (function returned via RET)
                    if pc_after == halt_address {
                        break;
                    }
                }
                Err(e) => {
                    println!("\n=== Emulator Error ===");
                    println!("Error: {}", e);
                    println!("Steps executed: {}", steps);
                    println!("PC: 0x{:x}", emu.get_pc());
                    println!("\n=== Emulator State ===");
                    println!("{}", emu.dump_state());
                    println!("\n=== Debug Info ===");
                    println!("{}", emu.format_debug_info(Some(emu.get_pc()), 50));
                    panic!("Emulator error: {}", e);
                }
            }
        }

        assert!(steps > 0, "Bootstrap app should execute at least one instruction");
    }
}

