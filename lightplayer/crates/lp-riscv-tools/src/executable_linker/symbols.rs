//! Symbol copying logic for base executable and object file.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use crate::debug;
use crate::elf_linker::LinkerError;
use object::{
    SectionKind, SymbolFlags, SymbolKind,
    read::{Object as ObjectTrait, ObjectSection, ObjectSymbol},
    write::{Object as WriteObject, SectionId, Symbol, SymbolId, SymbolSection},
};

/// Copy symbols from base executable.
pub fn copy_base_symbols(
    base_elf: &object::File,
    writer: &mut WriteObject,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    symbol_map: &mut BTreeMap<String, SymbolId>,
) -> Result<(), LinkerError> {
    debug!("Copying symbols from base executable...");
    for symbol in base_elf.symbols() {
        if let Ok(name) = symbol.name() {
            // Skip local labels
            if name.starts_with(".L") {
                continue;
            }

            // Skip special symbols that the writer doesn't support
            if name.starts_with("$") {
                debug!("  Skipping special symbol: {}", name);
                continue;
            }

            // Map symbol section and determine appropriate symbol kind
            let (section, symbol_kind) = match map_base_symbol_section(
                &symbol,
                base_elf,
                base_section_map,
                name,
            ) {
                Ok(result) => result,
                Err(_) => continue, // Skip symbols in metadata sections
            };

            // For Unknown symbols (linker script symbols), skip adding to writer
            // They're already in the base executable and the writer has issues with them
            if symbol.kind() == SymbolKind::Unknown {
                debug!("  Linker script symbol {} at 0x{:x} - skipping (not adding to writer)", name, symbol.address());
                continue;
            }

            let symbol_id = writer.add_symbol(Symbol {
                name: name.as_bytes().to_vec(),
                value: symbol.address(),
                size: symbol.size(),
                kind: symbol_kind,
                scope: symbol.scope(),
                weak: symbol.is_weak(),
                section,
                flags: SymbolFlags::None,
            });

            symbol_map.insert(name.to_string(), symbol_id);
            debug!("  Base symbol: {} at 0x{:x}", name, symbol.address());
        }
    }
    Ok(())
}

/// Map a base symbol's section and determine its symbol kind.
fn map_base_symbol_section(
    symbol: &dyn ObjectSymbol,
    base_elf: &object::File,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    name: &str,
) -> Result<(SymbolSection, SymbolKind), LinkerError> {
    match symbol.section() {
        object::SymbolSection::Section(idx) => {
            if let Ok(section_obj) = base_elf.section_by_index(idx) {
                if let Ok(section_name) = section_obj.name() {
                    let section_kind = section_obj.kind();

                    // Skip symbols in metadata sections (like .symtab, .strtab, .shstrtab) - these are not actual symbols
                    if section_name == ".symtab" || section_name == ".strtab" || section_name == ".shstrtab" {
                        // This should have been filtered out earlier, but handle gracefully
                        return Err(LinkerError::ParseError(format!(
                            "Symbol {} in metadata section {} should have been skipped",
                            name, section_name
                        )));
                    }

                    if let Some(&(section_id, _)) = base_section_map.get(section_name) {
                        // For Unknown symbols (linker script symbols), use SymbolSection::None
                        // to avoid section kind mismatch issues
                        if symbol.kind() == SymbolKind::Unknown {
                            debug!("  Mapping linker script symbol {} (Unknown -> Data, SymbolSection::None)", name);
                            Ok((SymbolSection::None, SymbolKind::Data))
                        } else {
                            // Map Unknown symbols to match the section kind
                            let kind = match section_kind {
                                SectionKind::Text => SymbolKind::Text,
                                SectionKind::Data | SectionKind::ReadOnlyData => SymbolKind::Data,
                                SectionKind::UninitializedData => SymbolKind::Data, // BSS symbols
                                _ => symbol.kind(), // Use original kind for other sections
                            };
                            Ok((SymbolSection::Section(section_id), kind))
                        }
                    } else {
                        // Section not found in map - use None and map Unknown to Data
                        let kind = if symbol.kind() == SymbolKind::Unknown {
                            debug!("  Mapping linker script symbol {} (Unknown -> Data, section {} not in map)", name, section_name);
                            SymbolKind::Data
                        } else {
                            symbol.kind()
                        };
                        Ok((SymbolSection::None, kind))
                    }
                } else {
                    let kind = if symbol.kind() == SymbolKind::Unknown {
                        SymbolKind::Data
                    } else {
                        symbol.kind()
                    };
                    Ok((SymbolSection::None, kind))
                }
            } else {
                let kind = if symbol.kind() == SymbolKind::Unknown {
                    SymbolKind::Data
                } else {
                    symbol.kind()
                };
                Ok((SymbolSection::None, kind))
            }
        }
        object::SymbolSection::Undefined => {
            Ok((SymbolSection::Undefined, symbol.kind()))
        }
        _ => {
            // No section - map Unknown to Data
            let kind = if symbol.kind() == SymbolKind::Unknown {
                debug!("  Mapping linker script symbol {} (Unknown -> Data, no section)", name);
                SymbolKind::Data
            } else {
                symbol.kind()
            };
            Ok((SymbolSection::None, kind))
        }
    }
}

/// Result of copying object symbols.
pub struct ObjectSymbolsResult {
    pub user_main_address: Option<u64>,
}

/// Copy symbols from object file (adjust addresses).
pub fn copy_object_symbols(
    object_elf: &object::File,
    writer: &mut WriteObject,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    object_section_map: &BTreeMap<String, (SectionId, u64)>,
    symbol_map: &mut BTreeMap<String, SymbolId>,
    base_text_section_size: u64,
) -> Result<ObjectSymbolsResult, LinkerError> {
    debug!("Copying symbols from object file...");
    let mut user_main_address = None;

    for symbol in object_elf.symbols() {
        if let Ok(name) = symbol.name() {
            // Don't skip local labels - they're needed for label-based relocations (like R_RISCV_PCREL_LO12_I)
            // These labels point to instructions (like auipc) and are referenced by relocations

            // Find which section this symbol belongs to
            let section_name: String = match symbol.section() {
                object::SymbolSection::Section(idx) => {
                    object_elf.section_by_index(idx)
                        .ok()
                        .and_then(|s| s.name().ok())
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                }
                _ => String::default(),
            };

            let (adjusted_address, section) = if let Some(&(section_id, section_address)) = object_section_map.get(&section_name) {
                // Calculate offset within the section
                let section_base = object_elf.section_by_name(&section_name)
                    .map(|s| s.address())
                    .unwrap_or(0);
                let section_offset_in_object = symbol.address().wrapping_sub(section_base);

                // If we merged .text sections, adjust by the base .text section size
                let final_address = if section_name == ".text" && base_section_map.contains_key(".text") {
                    // Symbol is in merged .text section - adjust by base section size
                    let base_text_addr = base_section_map.get(".text").unwrap().1;
                    base_text_addr + base_text_section_size + section_offset_in_object
                } else {
                    // Symbol is in a separate section - use section_address
                    section_address + section_offset_in_object
                };

                // Map symbol section
                let section = SymbolSection::Section(section_id);

                // Track main() address
                if name == "main" && symbol.kind() == SymbolKind::Text {
                    user_main_address = Some(final_address);
                    debug!("  Found user main() at 0x{:x} (adjusted from 0x{:x})", final_address, symbol.address());
                }

                (final_address, section)
            } else {
                // Undefined symbol - keep as is
                (symbol.address(), SymbolSection::Undefined)
            };

            // For "main", we don't add it to the symbol map as "main" (base executable's main takes precedence)
            // but we add it as "_user_main" so we can reference it in relocations
            // For label symbols (starting with .L), always add them (they're local to the object file)
            // For other symbols, only add if not already present (base executable symbols take precedence)
            let is_label = name.starts_with(".L");
            if name == "main" {
                // Add user main() as "_user_main" so we can reference it in relocations
                let user_main_symbol_id = writer.add_symbol(Symbol {
                    name: b"_user_main".to_vec(),
                    value: adjusted_address,
                    size: symbol.size(),
                    kind: symbol.kind(),
                    scope: symbol.scope(),
                    weak: symbol.is_weak(),
                    section,
                    flags: SymbolFlags::None,
                });
                symbol_map.insert("_user_main".to_string(), user_main_symbol_id);
                debug!("  Found user main() at 0x{:x} (added as _user_main for relocations)", adjusted_address);
            } else if is_label || !symbol_map.contains_key(name) {
                // Always add label symbols (they're local to this object file)
                // For other symbols, only add if not already present
                // Map label symbols to Text kind (they point to instructions)
                let symbol_kind = if is_label && (symbol.kind() == SymbolKind::Label || symbol.kind() == SymbolKind::Unknown) {
                    SymbolKind::Text
                } else {
                    symbol.kind()
                };
                let symbol_id = writer.add_symbol(Symbol {
                    name: name.as_bytes().to_vec(),
                    value: adjusted_address,
                    size: symbol.size(),
                    kind: symbol_kind,
                    scope: symbol.scope(),
                    weak: symbol.is_weak(),
                    section,
                    flags: SymbolFlags::None,
                });
                symbol_map.insert(name.to_string(), symbol_id);
                if is_label {
                    debug!("  Object label symbol: {} at 0x{:x} (adjusted from 0x{:x})", name, adjusted_address, symbol.address());
                } else {
                    debug!("  Object symbol: {} at 0x{:x} (adjusted from 0x{:x})", name, adjusted_address, symbol.address());
                }
            }
        }
    }

    Ok(ObjectSymbolsResult { user_main_address })
}

#[cfg(test)]
mod tests {
    // Note: Comprehensive tests would require creating mock ELF files
    // These are placeholder tests that verify the logic structure
}

