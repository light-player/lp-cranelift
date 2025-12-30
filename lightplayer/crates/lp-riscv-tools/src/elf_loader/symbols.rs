//! Symbol map building for relocations.

use crate::debug;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use hashbrown::HashMap;
use object::{Object, ObjectSymbol, SymbolSection};
use super::memory::is_ram_address;

/// Build a comprehensive symbol map for relocations.
/// 
/// Returns a map from symbol name to address (offset for ROM symbols, absolute for RAM symbols).
pub fn build_symbol_map(obj: &object::File, text_base: u64) -> HashMap<String, u32> {
    debug!("=== Building symbol map for relocations ===");
    debug!("text_section_base: 0x{:x}", text_base);
    
    let mut symbol_map: HashMap<String, u32> = HashMap::new();
    
    // First pass: collect all symbols, preferring defined ones
    let mut defined_symbols: Vec<(String, u32, SymbolSection)> = Vec::new();
    let mut undefined_symbols: Vec<(String, u32)> = Vec::new();
    
    for symbol in obj.symbols() {
        if let Ok(name) = symbol.name() {
            if name.is_empty() {
                continue; // Skip unnamed symbols
            }
            
            let addr = symbol.address();
            let symbol_section = symbol.section();
            let is_defined = symbol_section != SymbolSection::Undefined;
            
            // Calculate offset/address
            // For RAM symbols, use absolute address
            // For ROM symbols, use offset relative to text base
            let offset = if is_ram_address(addr) {
                // RAM symbol - use absolute address
                addr as u32
            } else if addr >= text_base {
                // ROM symbol - relative to text base
                (addr - text_base) as u32
            } else {
                // Other ROM sections - use as-is
                addr as u32
            };
            
            if is_defined {
                defined_symbols.push((name.to_string(), offset, symbol_section));
            } else {
                undefined_symbols.push((name.to_string(), offset));
            }
        }
    }
    
    // Add defined symbols first
    // If there are duplicates, keep the one with the higher address
    for (name, offset, section) in defined_symbols {
        if let Some(&existing_offset) = symbol_map.get(&name) {
            if offset > existing_offset {
                symbol_map.insert(name.clone(), offset);
                debug!("  Symbol '{}': replacing offset 0x{:x} with 0x{:x} (higher address), section={:?}", 
                       name, existing_offset, offset, section);
            } else {
                debug!("  Symbol '{}': keeping existing offset 0x{:x} (new: 0x{:x}), section={:?}", 
                       name, existing_offset, offset, section);
            }
        } else {
            symbol_map.insert(name.clone(), offset);
            debug!("  Symbol '{}': offset=0x{:x}, section={:?} (defined)", 
                   name, offset, section);
        }
    }
    
    // Add undefined symbols only if not already present
    for (name, offset) in undefined_symbols {
        if !symbol_map.contains_key(&name) {
            symbol_map.insert(name.clone(), offset);
            debug!("  Symbol '{}': offset=0x{:x} (undefined)", name, offset);
        } else {
            debug!("  Symbol '{}': skipping undefined (already have defined)", name);
        }
    }
    
    debug!("Symbol map contains {} entries", symbol_map.len());
    symbol_map
}

/// Find the address of a symbol by name (relative to text section base).
///
/// Returns the offset from the text section base address.
pub fn find_symbol_address(
    obj: &object::File,
    symbol_name: &str,
    text_section_base: u64,
) -> Result<u32, String> {
    for symbol in obj.symbols() {
        if symbol.kind() == object::SymbolKind::Text {
            if let Ok(name) = symbol.name() {
                if name == symbol_name {
                    let addr = symbol.address();
                    // In ELF object files, symbol addresses are section-relative (often 0x0 for start of section)
                    // If addr is already >= text_section_base, it's an absolute address - use it directly
                    // Otherwise, it's section-relative, so we use it as-is (it's already the offset)
                    if addr >= text_section_base {
                        return Ok((addr - text_section_base) as u32);
                    } else {
                        // Section-relative address - use it directly as the offset
                        return Ok(addr as u32);
                    }
                }
            }
        }
    }
    Err(format!("Symbol '{}' not found", symbol_name))
}

