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

