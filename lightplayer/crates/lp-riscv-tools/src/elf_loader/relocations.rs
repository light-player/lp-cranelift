
//! Relocation application for ELF loading.

use crate::debug;
use alloc::format;
use alloc::string::{String, ToString};
use hashbrown::HashMap;
use object::{Object, ObjectSection, ObjectSymbol, RelocationFlags, RelocationTarget};
use super::memory::{RAM_START, is_rom_address, is_ram_address, ram_address_to_offset};

/// Apply all relocations from the ELF to ROM and RAM buffers.
pub fn apply_relocations(
    obj: &object::File,
    rom: &mut [u8],
    ram: &mut [u8],
    symbol_map: &HashMap<String, u32>,
) -> Result<(), String> {
    debug!("=== Applying relocations to all sections ===");
    
    // Build section VMA map (same logic as in sections.rs)
    let mut section_vma_map: HashMap<String, u64> = HashMap::new();
    for symbol in obj.symbols() {
        if let Ok(name) = symbol.name() {
            let symbol_section = symbol.section();
            if let Some(section_idx) = symbol_section.index() {
                if let Ok(section) = obj.section_by_index(section_idx) {
                    if let Ok(section_name) = section.name() {
                        let section_addr = section.address();
                        let symbol_addr = symbol.address();
                        if section_addr == 0 && is_ram_address(symbol_addr) && section.kind() == object::SectionKind::Data {
                            section_vma_map.entry(section_name.to_string())
                                .and_modify(|vma| {
                                    if symbol_addr < *vma {
                                        *vma = symbol_addr;
                                    }
                                })
                                .or_insert(symbol_addr);
                        }
                    }
                }
            }
        }
    }
    
    // Track sequential offsets for sections with address 0
    // Also track LMA addresses for .data sections (needed for relocations)
    let mut next_rom_offset = 0u64;
    let mut section_lma_map: HashMap<String, u64> = HashMap::new();
    
    // Find __data_source_start symbol to determine .data section LMA
    // Look it up in the symbol map we built (it might be there even if not in obj.symbols())
    let data_source_start: Option<u64> = symbol_map.get("__data_source_start")
        .map(|&addr| addr as u64)
        .or_else(|| {
            // Also try obj.symbols() as fallback
            for symbol in obj.symbols() {
                if let Ok(name) = symbol.name() {
                    if name == "__data_source_start" {
                        let addr = symbol.address();
                        debug!("Found __data_source_start symbol at 0x{:x} for .data LMA", addr);
                        return Some(addr);
                    }
                }
            }
            None
        });
    if let Some(addr) = data_source_start {
        debug!("Using __data_source_start at 0x{:x} for .data LMA", addr);
    } else {
        debug!("__data_source_start not found, will use sequential placement for .data");
    }
    
    // First pass: determine LMA addresses (where sections are actually loaded in ROM)
    for section in obj.sections() {
        let section_name = section.name().unwrap_or("<unnamed>");
        let section_kind = section.kind();
        let section_addr = section.address();
        
        // Skip non-loadable sections
        match section_kind {
            object::SectionKind::Text | 
            object::SectionKind::Data | 
            object::SectionKind::ReadOnlyData |
            object::SectionKind::ReadOnlyString | // .rodata sections
            object::SectionKind::UninitializedData => {}
            _ => continue,
        }
        
        if let Ok(data) = section.data() {
            if data.is_empty() {
                continue;
            }
            
            // Determine VMA (same logic as sections.rs)
            let vma = if section_addr == 0 {
                if let Some(&ram_vma) = section_vma_map.get(&section_name.to_string()) {
                    ram_vma
                } else {
                    let current = next_rom_offset;
                    next_rom_offset = (current + data.len() as u64 + 3) & !3;
                    current
                }
            } else {
                section_addr
            };
            
            // For .data sections with RAM VMA, LMA is in ROM
            if is_ram_address(vma) && section_kind == object::SectionKind::Data {
                let lma = if section_name == ".data" {
                    // Use __data_source_start if available (it's the authoritative source)
                    // TEMPORARY FIX: Use 0xa18 if __data_source_start not found (match sections.rs)
                    // The sections.rs code will have loaded .data at this address
                    // We need to match that address here
                    if let Some(lma_addr) = data_source_start {
                        lma_addr
                    } else {
                        // TEMPORARY: hardcode 0xa18 to match sections.rs
                        0xa18
                    }
                } else if section_addr == 0 {
                    // Sequential placement in ROM
                    let current = next_rom_offset;
                    next_rom_offset = (current + data.len() as u64 + 3) & !3;
                    current
                } else {
                    // Use section address as LMA (though this shouldn't happen for RAM sections)
                    section_addr
                };
                section_lma_map.insert(section_name.to_string(), lma);
            }
        }
    }
    
    // Reset next_rom_offset for actual relocation application
    next_rom_offset = 0u64;
    
    for section in obj.sections() {
        let section_name = section.name().unwrap_or("<unnamed>");
        let section_kind = section.kind();
        let section_addr = section.address();
        
        // Skip non-loadable sections
        match section_kind {
            object::SectionKind::Text | 
            object::SectionKind::Data | 
            object::SectionKind::ReadOnlyData |
            object::SectionKind::ReadOnlyString | // .rodata sections
            object::SectionKind::UninitializedData => {}
            _ => continue,
        }
        
        // Determine VMA (same logic as sections.rs)
        let vma = if section_addr == 0 {
            if let Some(&ram_vma) = section_vma_map.get(&section_name.to_string()) {
                ram_vma
            } else {
                let current = next_rom_offset;
                if let Ok(data) = section.data() {
                    next_rom_offset = (current + data.len() as u64 + 3) & !3;
                }
                current
            }
        } else {
            section_addr
        };
        
        // Check if this section has relocations
        let mut reloc_count = 0;
        for (reloc_offset, _reloc) in section.relocations() {
            reloc_count += 1;
        }
        
        if reloc_count > 0 {
            debug!("Section '{}' (kind: {:?}, LMA: 0x{:x}, VMA: 0x{:x}) has {} relocations", 
                   section_name, section_kind, section_addr, vma, reloc_count);
            
            if is_rom_address(vma) {
                // ROM section - apply relocations to code buffer
                debug!("  -> Applying relocations to CODE buffer");
                let load_addr = section_addr;
                
                for (reloc_offset, reloc) in section.relocations() {
                    debug!("  Relocation at offset 0x{:x} in section '{}'", reloc_offset, section_name);
                    
                    // Get symbol name for debugging
                    let symbol_name = match reloc.target() {
                        RelocationTarget::Symbol(sym_idx) => {
                            if let Ok(sym) = obj.symbol_by_index(sym_idx) {
                                sym.name().unwrap_or("<unnamed>").to_string()
                            } else {
                                format!("symbol_index_{}", sym_idx.0)
                            }
                        }
                        _ => "<unknown>".to_string(),
                    };
                    debug!("    -> Targets symbol: '{}'", symbol_name);
                    
                    apply_single_relocation(
                        &reloc,
                        reloc_offset,
                        load_addr,
                        rom,
                        symbol_map,
                        obj,
                    )
                    .map_err(|e| format!("Failed to apply relocation in section '{}' at offset 0x{:x} (target: '{}'): {}", 
                                         section_name, reloc_offset, symbol_name, e))?;
                }
            } else if is_ram_address(vma) {
                // RAM section - for .data sections with "> RAM AT > ROM", apply relocations to ROM copy (LMA)
                if section_kind == object::SectionKind::Data {
                    // Get LMA from map
                    if let Some(&lma) = section_lma_map.get(&section_name.to_string()) {
                        debug!("  -> Applying relocations to ROM buffer (LMA=0x{:x}) for .data section", lma);
                        let load_addr = lma;
                        
                        for (reloc_offset, reloc) in section.relocations() {
                            debug!("  Relocation at offset 0x{:x} in section '{}'", reloc_offset, section_name);
                            
                            // Get symbol name for debugging
                            let symbol_name = match reloc.target() {
                                RelocationTarget::Symbol(sym_idx) => {
                                    if let Ok(sym) = obj.symbol_by_index(sym_idx) {
                                        sym.name().unwrap_or("<unnamed>").to_string()
                                    } else {
                                        format!("symbol_index_{}", sym_idx.0)
                                    }
                                }
                                _ => "<unknown>".to_string(),
                            };
                            debug!("    -> Targets symbol: '{}'", symbol_name);
                            
                            // Check if this is modifying __USER_MAIN_PTR
                            if symbol_name == "__USER_MAIN_PTR" || symbol_name == "_user_main" || symbol_name == "main" {
                                debug!("    -> *** IMPORTANT: This relocation affects __USER_MAIN_PTR or main() ***");
                                let rom_offset = lma as usize + reloc_offset as usize;
                                if rom_offset + 4 <= rom.len() {
                                    let old_value = u32::from_le_bytes([
                                        rom[rom_offset],
                                        rom[rom_offset + 1],
                                        rom[rom_offset + 2],
                                        rom[rom_offset + 3],
                                    ]);
                                    debug!("    -> Old value at ROM offset 0x{:x} (LMA 0x{:x}): 0x{:x}", 
                                           rom_offset, lma + reloc_offset, old_value);
                                }
                            }
                            
                            // Apply relocation to ROM copy
                            let rom_slice = &mut rom[lma as usize..];
                            apply_single_relocation(
                                &reloc,
                                reloc_offset,
                                load_addr, // Use LMA, not VMA
                                rom_slice,
                                symbol_map,
                                obj,
                            )
                            .map_err(|e| format!("Failed to apply relocation in section '{}' at offset 0x{:x} (target: '{}'): {}", 
                                             section_name, reloc_offset, symbol_name, e))?;
                            
                            // Check new value
                            if symbol_name == "__USER_MAIN_PTR" || symbol_name == "_user_main" || symbol_name == "main" {
                                let rom_offset = lma as usize + reloc_offset as usize;
                                if rom_offset + 4 <= rom.len() {
                                    let new_value = u32::from_le_bytes([
                                        rom[rom_offset],
                                        rom[rom_offset + 1],
                                        rom[rom_offset + 2],
                                        rom[rom_offset + 3],
                                    ]);
                                    debug!("    -> New value at ROM offset 0x{:x} (LMA 0x{:x}): 0x{:x}", 
                                           rom_offset, lma + reloc_offset, new_value);
                                }
                            }
                        }
                        continue; // Skip the RAM relocation code below
                    }
                }
                
                // Other RAM sections (shouldn't happen, but handle it)
                debug!("  -> Applying relocations to RAM buffer");
                let ram_offset = ram_address_to_offset(vma);
                let load_addr = vma;
                
                for (reloc_offset, reloc) in section.relocations() {
                    debug!("  Relocation at offset 0x{:x} in section '{}'", reloc_offset, section_name);
                    
                    // Get symbol name for debugging
                    let symbol_name = match reloc.target() {
                        RelocationTarget::Symbol(sym_idx) => {
                            if let Ok(sym) = obj.symbol_by_index(sym_idx) {
                                sym.name().unwrap_or("<unnamed>").to_string()
                            } else {
                                format!("symbol_index_{}", sym_idx.0)
                            }
                        }
                        _ => "<unknown>".to_string(),
                    };
                    debug!("    -> Targets symbol: '{}'", symbol_name);
                    
                    // Check if this is modifying __USER_MAIN_PTR
                    if symbol_name == "__USER_MAIN_PTR" || symbol_name == "_user_main" || symbol_name == "main" {
                        debug!("    -> *** IMPORTANT: This relocation affects __USER_MAIN_PTR or main() ***");
                        let data_offset = ram_offset + reloc_offset as usize;
                        if data_offset + 4 <= ram.len() {
                            let old_value = u32::from_le_bytes([
                                ram[data_offset],
                                ram[data_offset + 1],
                                ram[data_offset + 2],
                                ram[data_offset + 3],
                            ]);
                            debug!("    -> Old value at RAM offset 0x{:x} (addr 0x{:x}): 0x{:x}", 
                                   data_offset, vma + reloc_offset, old_value);
                        }
                    }
                    
                    // Apply the relocation - create slice starting at section start
                    if ram_offset + reloc_offset as usize + 4 > ram.len() {
                        return Err(format!("Relocation offset 0x{:x} in RAM section '{}' out of bounds (ram.len()={})", 
                                          ram_offset + reloc_offset as usize, section_name, ram.len()));
                    }
                    
                    // Create a slice starting at the section start (ram_offset)
                    // The relocation offset is relative to the section start
                    let ram_slice = &mut ram[ram_offset..];
                    
                    apply_single_relocation(
                        &reloc,
                        reloc_offset, // Offset relative to section start
                        load_addr,
                        ram_slice,
                        symbol_map,
                        obj,
                    )
                    .map_err(|e| format!("Failed to apply relocation in section '{}' at offset 0x{:x} (target: '{}'): {}", 
                                         section_name, reloc_offset, symbol_name, e))?;
                    
                    // Check the new value if this was __USER_MAIN_PTR
                    if symbol_name == "__USER_MAIN_PTR" || symbol_name == "_user_main" || symbol_name == "main" {
                        let data_offset = ram_offset + reloc_offset as usize;
                        if data_offset + 4 <= ram.len() {
                            let new_value = u32::from_le_bytes([
                                ram[data_offset],
                                ram[data_offset + 1],
                                ram[data_offset + 2],
                                ram[data_offset + 3],
                            ]);
                            debug!("    -> New value at RAM offset 0x{:x} (addr 0x{:x}): 0x{:x}", 
                                   data_offset, vma + reloc_offset, new_value);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Apply a single relocation to a buffer.
fn apply_single_relocation(
    reloc: &object::Relocation,
    reloc_offset: u64,
    section_load_addr: u64,
    buffer: &mut [u8],
    symbol_map: &HashMap<String, u32>,
    obj: &object::File,
) -> Result<(), String> {
    // Get target symbol address
    let target_addr = match reloc.target() {
        RelocationTarget::Symbol(sym_idx) => {
            if let Ok(sym) = obj.symbol_by_index(sym_idx) {
                if let Ok(name) = sym.name() {
                    debug!("  Relocation targets symbol '{}' (index {})", name, sym_idx.0);
                    
                    // Look up by name in symbol_map
                    symbol_map.get(name).copied()
                } else {
                    debug!("  Relocation targets unnamed symbol (index {})", sym_idx.0);
                    None
                }
            } else {
                debug!("  Relocation targets invalid symbol index {}", sym_idx.0);
                None
            }
        }
        _ => {
            debug!("  Relocation has non-symbol target");
            None
        }
    };
    
    let target_addr = target_addr.ok_or_else(|| {
        let symbol_name = match reloc.target() {
            RelocationTarget::Symbol(sym_idx) => {
                if let Ok(sym) = obj.symbol_by_index(sym_idx) {
                    sym.name().unwrap_or("<unnamed>").to_string()
                } else {
                    format!("symbol_index_{}", sym_idx.0)
                }
            }
            _ => "<unknown>".to_string(),
        };
        format!(
            "Could not resolve relocation target '{}' at offset 0x{:x}. Symbol not found in symbol map.",
            symbol_name, reloc_offset
        )
    })?;
    
    // Calculate PC-relative offset
    let pc = (section_load_addr + reloc_offset) as u32;
    // target_addr from symbol_map: for ROM it's relative to text_base (which is 0x0), for RAM it's absolute
    // For R_RISCV_32 (absolute relocation), we need the absolute address
    // For PC-relative relocations, we need the absolute address to calculate the offset
    let target_absolute = if target_addr >= RAM_START {
        // RAM symbol - already absolute
        target_addr
    } else {
        // ROM symbol - target_addr is relative to text_base (0x0), so it's already absolute
        // Don't add section_load_addr - that would be wrong
        target_addr
    };
    let pcrel = target_absolute
        .wrapping_sub(pc)
        .wrapping_add(reloc.addend() as u32);
    
    debug!("  PC=0x{:x}, target_absolute=0x{:x}, pcrel=0x{:x} (signed: {})", 
           pc, target_absolute, pcrel, pcrel as i32);
    
    // Determine relocation type from flags
    let reloc_offset = reloc_offset as usize;
    if reloc_offset >= buffer.len() {
        return Err(format!("Relocation offset {} out of bounds", reloc_offset));
    }
    
    match reloc.flags() {
        RelocationFlags::Elf { r_type } => {
            match r_type {
                object::elf::R_RISCV_CALL_PLT => {
                    // RISC-V CALL_PLT: auipc+jalr pair
                    if reloc_offset + 8 > buffer.len() {
                        return Err(format!(
                            "CALL_PLT relocation at offset {} requires 8 bytes, but only {} available",
                            reloc_offset,
                            buffer.len() - reloc_offset
                        ));
                    }
                    
                    // Read the two instructions
                    let auipc_bytes = &buffer[reloc_offset..reloc_offset + 4];
                    let jalr_bytes = &buffer[reloc_offset + 4..reloc_offset + 8];
                    
                    let auipc_word = u32::from_le_bytes([auipc_bytes[0], auipc_bytes[1], auipc_bytes[2], auipc_bytes[3]]);
                    let jalr_word = u32::from_le_bytes([jalr_bytes[0], jalr_bytes[1], jalr_bytes[2], jalr_bytes[3]]);
                    
                    // Extract immediate from auipc (bits [31:12])
                    let auipc_imm = (auipc_word >> 12) & 0xFFFFF;
                    // Extract immediate from jalr (bits [31:20])
                    let jalr_imm = (jalr_word >> 20) & 0xFFF;
                    
                    // Calculate the full 32-bit offset
                    let offset_hi20 = auipc_imm as i32;
                    let offset_lo12 = jalr_imm as i32;
                    // Sign-extend hi20
                    let offset_hi20_signed = ((offset_hi20 << 12) as i32) >> 12;
                    let full_offset = (offset_hi20_signed << 12) | offset_lo12;
                    
                    // Calculate new offset
                    let new_offset = pcrel;
                    let new_hi20 = ((new_offset >> 12) + ((new_offset & 0x800) != 0) as u32) & 0xFFFFF;
                    let new_lo12 = new_offset & 0xFFF;
                    
                    // Patch auipc instruction
                    let new_auipc = (auipc_word & 0xFFF) | (new_hi20 << 12);
                    let auipc_bytes = &mut buffer[reloc_offset..reloc_offset + 4];
                    auipc_bytes.copy_from_slice(&new_auipc.to_le_bytes());
                    
                    // Patch jalr instruction
                    let new_jalr = (jalr_word & 0xFFFFF) | (new_lo12 << 20);
                    let jalr_bytes = &mut buffer[reloc_offset + 4..reloc_offset + 8];
                    jalr_bytes.copy_from_slice(&new_jalr.to_le_bytes());
                }
                object::elf::R_RISCV_PCREL_HI20 => {
                    // RISC-V PC-relative high 20 bits
                    if reloc_offset + 4 > buffer.len() {
                        return Err(format!(
                            "PCREL_HI20 relocation at offset {} requires 4 bytes",
                            reloc_offset
                        ));
                    }
                    let inst_bytes = &mut buffer[reloc_offset..reloc_offset + 4];
                    let inst_word = u32::from_le_bytes([
                        inst_bytes[0],
                        inst_bytes[1],
                        inst_bytes[2],
                        inst_bytes[3],
                    ]);
                    // Extract the high 20 bits of the PC-relative offset
                    let hi20 = ((pcrel >> 12) + ((pcrel & 0x800) != 0) as u32) & 0xFFFFF;
                    let patched = (inst_word & 0xFFF) | (hi20 << 12);
                    inst_bytes.copy_from_slice(&patched.to_le_bytes());
                }
                object::elf::R_RISCV_PCREL_LO12_I => {
                    // RISC-V PC-relative low 12 bits (immediate)
                    if reloc_offset + 4 > buffer.len() {
                        return Err(format!(
                            "PCREL_LO12_I relocation at offset {} requires 4 bytes",
                            reloc_offset
                        ));
                    }
                    let inst_bytes = &mut buffer[reloc_offset..reloc_offset + 4];
                    let inst_word = u32::from_le_bytes([
                        inst_bytes[0],
                        inst_bytes[1],
                        inst_bytes[2],
                        inst_bytes[3],
                    ]);
                    // For LO12_I, we need the low 12 bits of the offset
                    let lo12 = pcrel & 0xFFF;
                    let patched = (inst_word & 0xFFFFF) | (lo12 << 20);
                    inst_bytes.copy_from_slice(&patched.to_le_bytes());
                }
                object::elf::R_RISCV_32 => {
                    // RISC-V 32-bit absolute relocation
                    if reloc_offset + 4 > buffer.len() {
                        return Err(format!(
                            "R_RISCV_32 relocation at offset {} requires 4 bytes",
                            reloc_offset
                        ));
                    }
                    // Write the absolute target address directly
                    let reloc_bytes = &mut buffer[reloc_offset..reloc_offset + 4];
                    reloc_bytes.copy_from_slice(&target_absolute.to_le_bytes());
                }
                _ => {
                    return Err(format!(
                        "Unsupported relocation type {} at offset 0x{:x}. Supported types: R_RISCV_CALL_PLT, R_RISCV_PCREL_HI20, R_RISCV_PCREL_LO12_I, R_RISCV_32",
                        r_type, reloc_offset
                    ));
                }
            }
        }
        _ => {
            return Err(format!(
                "Unsupported relocation format at offset 0x{:x}",
                reloc_offset
            ));
        }
    }
    
    Ok(())
}

