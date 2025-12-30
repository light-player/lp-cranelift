//! Symbol map building for object files.

extern crate alloc;

use crate::debug;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use hashbrown::HashMap;
use ::object::{Object, ObjectSection, ObjectSymbol, SymbolSection};

use super::super::memory::RAM_START;

/// Build symbol map for object file with adjusted addresses.
///
/// Creates a symbol map from object file symbols, adjusting their addresses
/// based on where sections were placed in memory.
///
/// # Arguments
///
/// * `obj` - The object file to build symbol map from
/// * `text_placement` - Address where .text section was placed
/// * `data_placement` - Offset where .data section was placed (relative to RAM_START)
///
/// # Returns
///
/// Symbol map mapping symbol names to their final addresses.
pub fn build_object_symbol_map(
    obj: &::object::File,
    text_placement: u32,
    data_placement: u32,
) -> HashMap<String, u32> {
    debug!("=== Building object file symbol map ===");
    debug!("Text placement: 0x{:x}, Data placement offset: 0x{:x}", text_placement, data_placement);

    let mut symbol_map: HashMap<String, u32> = HashMap::new();

    // First pass: collect all symbols, preferring defined ones
    let mut defined_symbols: Vec<(String, u32, SymbolSection)> = Vec::new();
    let mut undefined_symbols: Vec<(String, u32)> = Vec::new();

    for symbol in obj.symbols() {
        if let Ok(name) = symbol.name() {
            if name.is_empty() {
                continue; // Skip unnamed symbols
            }

            let symbol_addr = symbol.address();
            let symbol_section = symbol.section();
            let is_defined = symbol_section != SymbolSection::Undefined;

            // Determine which section this symbol belongs to and adjust address
            let final_addr = if !is_defined {
                // Undefined symbol - keep address as-is (will be resolved via merge)
                symbol_addr as u32
            } else {
                // Defined symbol - need to find which section it belongs to
                let section_name = if let Some(section_idx) = symbol_section.index() {
                    if let Ok(section) = obj.section_by_index(section_idx) {
                        section.name().ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                match section_name {
                    Some(".text") => {
                        // .text section symbol: adjust by text_placement
                        // symbol_addr is section-relative offset
                        let adjusted = text_placement.wrapping_add(symbol_addr as u32);
                        debug!("  Symbol '{}': .text section, offset=0x{:x}, final=0x{:x}", 
                               name, symbol_addr, adjusted);
                        adjusted
                    }
                    Some(".data") => {
                        // .data section symbol: adjust by RAM_START + data_placement
                        // symbol_addr is section-relative offset
                        let adjusted = RAM_START.wrapping_add(data_placement).wrapping_add(symbol_addr as u32);
                        debug!("  Symbol '{}': .data section, offset=0x{:x}, final=0x{:x}", 
                               name, symbol_addr, adjusted);
                        adjusted
                    }
                    Some(".rodata") => {
                        // .rodata section symbol: placed in code buffer after .text
                        // For now, place after .text (we'd need to track .rodata placement)
                        // This is a simplification - in practice .rodata might be placed differently
                        let adjusted = text_placement.wrapping_add(symbol_addr as u32);
                        debug!("  Symbol '{}': .rodata section, offset=0x{:x}, final=0x{:x} (simplified)", 
                               name, symbol_addr, adjusted);
                        adjusted
                    }
                    Some(".bss") => {
                        // .bss section symbol: placed in RAM buffer after .data
                        // For now, place after .data (we'd need to track .bss placement)
                        let adjusted = RAM_START.wrapping_add(data_placement).wrapping_add(symbol_addr as u32);
                        debug!("  Symbol '{}': .bss section, offset=0x{:x}, final=0x{:x} (simplified)", 
                               name, symbol_addr, adjusted);
                        adjusted
                    }
                    _ => {
                        // Unknown section or no section - use address as-is
                        debug!("  Symbol '{}': unknown section '{:?}', using address as-is: 0x{:x}", 
                               name, section_name, symbol_addr);
                        symbol_addr as u32
                    }
                }
            };

            if is_defined {
                defined_symbols.push((name.to_string(), final_addr, symbol_section));
            } else {
                undefined_symbols.push((name.to_string(), final_addr));
            }
        }
    }

    // Add defined symbols first
    // If there are duplicates, keep the one with the higher address
    for (name, addr, section) in defined_symbols {
        if let Some(&existing_addr) = symbol_map.get(&name) {
            if addr > existing_addr {
                symbol_map.insert(name.clone(), addr);
                debug!("  Symbol '{}': replacing address 0x{:x} with 0x{:x} (higher address), section={:?}", 
                       name, existing_addr, addr, section);
            } else {
                debug!("  Symbol '{}': keeping existing address 0x{:x} (new: 0x{:x}), section={:?}", 
                       name, existing_addr, addr, section);
            }
        } else {
            symbol_map.insert(name.clone(), addr);
            debug!("  Symbol '{}': address=0x{:x}, section={:?} (defined)", 
                   name, addr, section);
        }
    }

    // Add undefined symbols only if not already present
    for (name, addr) in undefined_symbols {
        if !symbol_map.contains_key(&name) {
            symbol_map.insert(name.clone(), addr);
            debug!("  Symbol '{}': address=0x{:x} (undefined)", name, addr);
        } else {
            debug!("  Symbol '{}': skipping undefined (already have defined)", name);
        }
    }

    debug!("Object symbol map contains {} entries", symbol_map.len());
    symbol_map
}

/// Merge base and object symbol maps.
///
/// Combines symbol maps, with base symbols taking precedence over object symbols.
///
/// # Arguments
///
/// * `base_map` - Base executable's symbol map
/// * `obj_map` - Object file's symbol map
///
/// # Returns
///
/// Merged symbol map with base symbols taking precedence.
pub fn merge_symbol_maps(
    base_map: &HashMap<String, u32>,
    obj_map: &HashMap<String, u32>,
) -> HashMap<String, u32> {
    // TODO: Phase 6 - Implement symbol map merging
    // For now, just return base map to allow compilation
    let mut merged = base_map.clone();
    for (name, addr) in obj_map {
        merged.entry(name.clone()).or_insert(*addr);
    }
    merged
}

