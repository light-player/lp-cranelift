//! Object file loading for RISC-V emulator.
//!
//! This module provides utilities to load relocatable object files into the emulator
//! after a base executable has been loaded.

#![cfg(feature = "std")]

mod layout;
mod sections;
mod symbols;
mod relocations;

#[cfg(test)]
mod tests;

// Re-export public types and functions
pub use layout::calculate_object_layout;
pub use sections::load_object_sections;
pub use symbols::{build_object_symbol_map, merge_symbol_maps};
pub use relocations::apply_object_relocations;

extern crate alloc;

use crate::debug;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;

use super::parse;
use crate::elf_loader::memory::RAM_START;

/// Information about a loaded object file.
pub struct ObjectLoadInfo {
    /// Address of 'main' function if found
    pub main_address: Option<u32>,
    /// Object file's symbol map (with adjusted addresses)
    pub symbol_map: HashMap<String, u32>,
    /// Start address where .text section was placed
    pub text_start: u32,
    /// Start offset where .data section was placed (relative to RAM_START)
    pub data_start: u32,
}

/// Load a relocatable object file into the emulator after a base executable has been loaded.
///
/// This function:
/// - Parses the object file
/// - Calculates where to place sections (after base executable)
/// - Loads sections into the code/ram buffers
/// - Builds symbol map with adjusted addresses
/// - Merges symbol map with base executable's symbol map
/// - Applies relocations using the merged symbol map
/// - Updates the caller's symbol map with merged result
///
/// # Arguments
///
/// * `obj_file_bytes` - The object file bytes to load
/// * `code` - Mutable reference to code buffer (will be extended if needed)
/// * `ram` - Mutable reference to RAM buffer (will be extended if needed)
/// * `symbol_map` - Mutable reference to base executable's symbol map (will be updated with merged map)
///
/// # Returns
///
/// Information about the loaded object file, or an error if loading fails.
pub fn load_object_file(
    obj_file_bytes: &[u8],
    code: &mut Vec<u8>,
    ram: &mut Vec<u8>,
    symbol_map: &mut HashMap<String, u32>,
) -> Result<ObjectLoadInfo, String> {
    debug!("=== Loading object file ===");

    // Step 1: Parse object file
    let obj = parse::parse_elf(obj_file_bytes)?;
    
    // Step 2: Validate object file
    parse::validate_elf(&obj)?;

    // Step 3: Get base executable end addresses from buffer sizes
    // The buffers contain the base executable, so their current sizes indicate where it ends
    let base_code_end = code.len() as u32;
    let base_ram_end = ram.len() as u32;
    
    debug!("Base executable: code_end=0x{:x}, ram_end=0x{:x}", base_code_end, base_ram_end);

    // Step 4: Calculate layout for object file sections
    let layout = calculate_object_layout(&obj, base_code_end, base_ram_end)?;

    // Step 5: Load object file sections into buffers
    let section_placement = load_object_sections(&obj, code, ram, &layout)?;

    // Step 6: Build object file's symbol map with adjusted addresses
    let obj_symbol_map = build_object_symbol_map(
        &obj,
        layout.text_placement,
        layout.data_placement,
    );

    // Step 7: Merge symbol maps (base takes precedence)
    let merged_symbol_map = merge_symbol_maps(symbol_map, &obj_symbol_map);

    // Step 8: Apply relocations using merged symbol map
    apply_object_relocations(
        &obj,
        code,
        ram,
        &merged_symbol_map,
        &section_placement,
    )?;

    // Step 9: Update caller's symbol map with merged result
    *symbol_map = merged_symbol_map.clone();

    // Step 10: Find 'main' symbol if present in object file and update __USER_MAIN_PTR
    // Use object file's main (not base executable's main) for __USER_MAIN_PTR
    let main_address = obj_symbol_map.get("main").copied();
    
    if let Some(main_addr) = main_address {
        debug!("Found 'main' symbol in object file at 0x{:x}", main_addr);
        
        // Update __USER_MAIN_PTR in RAM if it exists (from base executable)
        if let Some(&user_main_ptr_addr) = merged_symbol_map.get("__USER_MAIN_PTR") {
            // Calculate RAM offset (__USER_MAIN_PTR is in RAM)
            if user_main_ptr_addr >= RAM_START {
                let ram_offset = (user_main_ptr_addr - RAM_START) as usize;
                
                // Ensure RAM buffer is large enough
                if ram_offset + 4 <= ram.len() {
                    // Write main address as little-endian u32
                    ram[ram_offset..ram_offset + 4].copy_from_slice(&main_addr.to_le_bytes());
                    debug!("Updated __USER_MAIN_PTR at 0x{:x} (RAM offset 0x{:x}) to point to main() at 0x{:x}",
                           user_main_ptr_addr, ram_offset, main_addr);
                } else {
                    debug!("Warning: __USER_MAIN_PTR at 0x{:x} is out of RAM buffer bounds (len={})",
                           user_main_ptr_addr, ram.len());
                }
            } else {
                debug!("Warning: __USER_MAIN_PTR at 0x{:x} is not in RAM region", user_main_ptr_addr);
            }
        } else {
            debug!("__USER_MAIN_PTR symbol not found, skipping update");
        }
    } else {
        debug!("No 'main' symbol found in object file");
    }

    debug!("=== Object file loading complete ===");
    debug!("Object file: .text at 0x{:x}, .data at offset 0x{:x}", 
           section_placement.text_start, section_placement.data_start);

    Ok(ObjectLoadInfo {
        main_address,
        symbol_map: obj_symbol_map,
        text_start: section_placement.text_start,
        data_start: section_placement.data_start,
    })
}

