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
    for section in base_elf.sections() {
        let section_name = section.name()?;
        if let Some(&(section_id, _)) = base_section_map.get(section_name) {
            let mut reloc_count = 0;
            for (offset, reloc) in section.relocations() {
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
                
                debug!("  Base relocation {} in section '{}' at offset 0x{:x} targets '{}'", 
                       reloc_count, section_name, offset, symbol_name);
                
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
                    debug!("    -> Mapped to symbol ID in writer");
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
                    debug!("    -> WARNING: Target symbol not found in symbol map, skipping relocation");
                }
            }
            if reloc_count > 0 {
                debug!("  Copied {} relocations from base section '{}'", reloc_count, section_name);
            }
        }
    }
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

        for (offset, reloc) in section.relocations() {
            // Adjust offset if we merged the section
            let adjusted_offset = offset + section_offset;

            // Resolve relocation target
            let target_symbol_id = match reloc.target() {
                RelocationTarget::Symbol(sym_idx) => {
                    if let Ok(sym) = object_elf.symbol_by_index(sym_idx) {
                        if let Ok(target_name) = sym.name() {
                            // Skip local labels
                            if target_name.starts_with(".L") {
                                continue;
                            }
                            let found = symbol_map.get(target_name).copied();
                            if found.is_none() {
                                debug!("  Warning: Relocation target '{}' not found in symbol map (offset: 0x{:x})", target_name, adjusted_offset);
                            } else {
                                debug!("  Relocation at offset 0x{:x} (adjusted from 0x{:x}) targets '{}'", adjusted_offset, offset, target_name);
                            }
                            found
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
                // Add relocation with adjusted offset
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
                debug!("  Warning: Relocation at offset 0x{:x} target not found in symbol map - skipping", adjusted_offset);
            }
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

