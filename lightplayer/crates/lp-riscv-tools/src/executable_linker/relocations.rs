//! Relocation copying logic for base executable and object file.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use crate::debug;
use crate::elf_linker::LinkerError;
use object::{
    read::{Object as ObjectTrait, ObjectSection, ObjectSymbol, RelocationFlags, RelocationTarget},
    write::{Object as WriteObject, Relocation, SectionId},
};

/// Convert relocation flags from read format to write format.
pub fn convert_reloc_flags(flags: RelocationFlags) -> Result<object::write::RelocationFlags, LinkerError> {
    match flags {
        RelocationFlags::Elf { r_type } => Ok(object::write::RelocationFlags::Elf { r_type }),
        _ => Err(LinkerError::ParseError(format!(
            "Unsupported relocation flags: {:?}",
            flags
        ))),
    }
}

/// Copy relocations from base executable.
pub fn copy_base_relocations(
    base_elf: &object::File,
    writer: &mut WriteObject,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    symbol_map: &BTreeMap<String, object::write::SymbolId>,
) -> Result<(), LinkerError> {
    debug!("Copying relocations from base executable...");
    let mut total_base_relocs = 0;
    for section in base_elf.sections() {
        let section_name = section.name()?;
        let has_relocs = section.relocations().next().is_some();
        if has_relocs {
            debug!("  Base section '{}' has relocations", section_name);
        }
        if let Some(&(section_id, _)) = base_section_map.get(section_name) {
            let mut reloc_count = 0;
            for (offset, reloc) in section.relocations() {
                total_base_relocs += 1;
                reloc_count += 1;
                
                // Get symbol name for debugging
                let symbol_name = match reloc.target() {
                    RelocationTarget::Symbol(sym_idx) => {
                        if let Ok(sym) = base_elf.symbol_by_index(sym_idx) {
                            sym.name().unwrap_or("<unnamed>").to_string()
                        } else {
                            format!("symbol_index_{}", sym_idx.0)
                        }
                    }
                    _ => "<unknown>".to_string(),
                };
                
                // Get relocation type for debugging
                let r_type = match reloc.flags() {
                    RelocationFlags::Elf { r_type } => r_type,
                    _ => 0,
                };
                let r_type_str = match r_type {
                    1 => "R_RISCV_32",
                    17 => "R_RISCV_CALL_PLT",
                    19 => "R_RISCV_GOT_HI20",
                    20 => "R_RISCV_PCREL_HI20",
                    21 => "R_RISCV_PCREL_LO12_I",
                    24 => "R_RISCV_PCREL_LO12_I",
                    _ => "R_RISCV_UNKNOWN",
                };
                
                debug!("  Base relocation {} in section '{}' at offset 0x{:x}: type={} ({}), target='{}', addend={}", 
                       reloc_count, section_name, offset, r_type, r_type_str, symbol_name, reloc.addend());
                
                // Resolve relocation target
                let target_symbol_id = match reloc.target() {
                    RelocationTarget::Symbol(sym_idx) => {
                        if let Ok(sym) = base_elf.symbol_by_index(sym_idx) {
                            if let Ok(target_name) = sym.name() {
                                symbol_map.get(target_name).copied()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(target_symbol_id) = target_symbol_id {
                    debug!("    -> Mapped to symbol ID in writer, adding relocation");
                    // Add relocation
                    writer.add_relocation(
                        section_id,
                        Relocation {
                            offset,
                            symbol: target_symbol_id,
                            addend: reloc.addend(),
                            flags: convert_reloc_flags(reloc.flags())?,
                        },
                    )?;
                } else {
                    debug!("    -> WARNING: Target symbol '{}' not found in symbol map, skipping relocation", symbol_name);
                    debug!("      (This relocation will NOT be applied at load time!)");
                }
            }
            if reloc_count > 0 {
                debug!("  Copied {} relocations from base section '{}'", reloc_count, section_name);
            }
        } else {
            // Check if section has relocations but wasn't in our map
            let has_relocs = section.relocations().next().is_some();
            if has_relocs {
                debug!("  WARNING: Base section '{}' has relocations but is not in base_section_map!", section_name);
            }
        }
    }
    debug!("Total base relocations found: {}", total_base_relocs);
    Ok(())
}

/// Copy relocations from object file.
pub fn copy_object_relocations(
    object_elf: &object::File,
    writer: &mut WriteObject,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    object_section_map: &BTreeMap<String, (SectionId, u64)>,
    symbol_map: &BTreeMap<String, object::write::SymbolId>,
    base_text_section_size: u64,
) -> Result<(), LinkerError> {
    debug!("Copying relocations from object file...");
    for section in object_elf.sections() {
        let section_name = section.name()?;
        // Skip debug sections
        if section_name.starts_with(".debug_") || section_name.starts_with(".zdebug_") {
            continue;
        }

        // Find section ID in our maps
        let (section_id, section_offset) = if let Some(&(id, _addr)) = object_section_map.get(section_name) {
            // If we merged .text, use the base .text section size as the offset
            let offset = if section_name == ".text" && base_section_map.contains_key(".text") {
                base_text_section_size
            } else {
                0
            };
            (id, offset)
        } else {
            continue;
        };

        let mut reloc_count = 0;
        for (offset, reloc) in section.relocations() {
            reloc_count += 1;
            // Adjust offset if we merged the section
            let adjusted_offset = offset + section_offset;

            // Get relocation type for debugging
            let r_type_str = match reloc.flags() {
                RelocationFlags::Elf { r_type } => {
                    match r_type {
                        1 => "R_RISCV_32",
                        17 => "R_RISCV_CALL_PLT",
                        19 => "R_RISCV_GOT_HI20",
                        20 => "R_RISCV_PCREL_HI20",
                        21 => "R_RISCV_PCREL_LO12_I",
                        _ => &format!("R_RISCV_{}", r_type),
                    }
                }
                _ => "unknown",
            };

            // Resolve relocation target
            let target_symbol_id = match reloc.target() {
                RelocationTarget::Symbol(sym_idx) => {
                    if let Ok(sym) = object_elf.symbol_by_index(sym_idx) {
                        if let Ok(target_name) = sym.name() {
                            debug!("  Object relocation #{} in section '{}' at offset 0x{:x} (adjusted: 0x{:x}): type={}, target='{}', addend={}", 
                                   reloc_count, section_name, offset, adjusted_offset, r_type_str, target_name, reloc.addend());
                            
                            // Don't skip local labels - they're needed for label-based relocations (like R_RISCV_PCREL_LO12_I)
                            // These labels point to instructions (like auipc) and are referenced by relocations
                            // We should have copied them in copy_object_symbols
                            let found = symbol_map.get(target_name).copied();
                            if found.is_none() {
                                debug!("    -> WARNING: Target symbol '{}' not found in symbol map", target_name);
                                if target_name.starts_with(".L") {
                                    debug!("    -> This is a label symbol - it should have been copied in copy_object_symbols");
                                }
                            } else {
                                debug!("    -> Mapped to symbol ID in writer");
                            }
                            found
                        } else {
                            debug!("    -> WARNING: Symbol has no name");
                            None
                        }
                    } else {
                        debug!("    -> WARNING: Invalid symbol index {}", sym_idx.0);
                        None
                    }
                }
                _ => {
                    debug!("    -> WARNING: Non-symbol relocation target");
                    None
                },
            };

            if let Some(target_symbol_id) = target_symbol_id {
                // Add relocation with adjusted offset
                debug!("    -> Adding relocation to writer");
                writer.add_relocation(
                    section_id,
                    Relocation {
                        offset: adjusted_offset,
                        symbol: target_symbol_id,
                        addend: reloc.addend(),
                        flags: convert_reloc_flags(reloc.flags())?,
                    },
                )?;
            } else {
                debug!("    -> SKIPPING relocation (no valid target symbol)");
            }
        }
        if reloc_count > 0 {
            debug!("  Processed {} relocations from object section '{}'", reloc_count, section_name);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_reloc_flags_elf() {
        // Test that ELF relocation flags are converted correctly
        let flags = RelocationFlags::Elf { r_type: 1 };
        let converted = convert_reloc_flags(flags).unwrap();
        match converted {
            object::write::RelocationFlags::Elf { r_type } => {
                assert_eq!(r_type, 1);
            }
            _ => panic!("Expected Elf relocation flags"),
        }
    }
}

