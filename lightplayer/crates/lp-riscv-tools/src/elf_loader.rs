//! ELF file loader for RISC-V emulator.
//!
//! This module provides utilities to load RISC-V ELF files into the emulator's memory.
//! It handles both segment loading and relocation application.

#![cfg(feature = "std")]

extern crate std;

use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use hashbrown::HashMap;
use object::{Object, ObjectSection, ObjectSymbol};

/// Information extracted from an ELF file for emulator loading.
pub struct ElfLoadInfo {
    /// Code/ROM region (starts at address 0) with relocations applied
    pub code: Vec<u8>,
    /// RAM region (starts at 0x80000000)
    pub ram: Vec<u8>,
    /// Entry point address
    pub entry_point: u32,
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

/// Apply a single relocation to code bytes.
fn apply_single_relocation(
    reloc: &object::Relocation,
    reloc_offset: u64,
    section_load_addr: u64,
    code_bytes: &mut [u8],
    symbol_map: &HashMap<String, u32>,
    obj: &object::File,
) -> Result<(), String> {
    use object::{RelocationFlags, RelocationTarget};

    use crate::debug;
    
    // Get target symbol address
    // IMPORTANT: Symbol indices can be wrong after linking due to symbol reordering.
    // We resolve by name from the symbol_map, which prefers defined symbols.
    let target_addr = match reloc.target() {
        RelocationTarget::Symbol(sym_idx) => {
            if let Ok(sym) = obj.symbol_by_index(sym_idx) {
                if let Ok(name) = sym.name() {
                    debug!("  Relocation targets symbol '{}' (index {})", name, sym_idx.0);
                    
                    // Look up by name in symbol_map - this ensures we get the right symbol
                    // even if the index is wrong due to symbol reordering
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

    // Note: Address 0 is valid if the text section starts at 0 and the function is at offset 0
    // The validation that rejected address 0 was too strict - we only need to check that
    // the symbol was found in the map, which we already did above

    // Calculate PC-relative offset
    // reloc_offset is relative to section start, so PC = section_load_addr + reloc_offset
    // target_addr from symbol_map is an offset relative to text_section_base
    // We need to get the text_section_base to convert target_addr to absolute address
    // But wait - section_load_addr IS the text section base! So:
    let pc = (section_load_addr + reloc_offset) as u32;
    // target_addr is relative to text_section_base (which equals section_load_addr for text section)
    // So absolute target = section_load_addr + target_addr
    let target_absolute = (section_load_addr + target_addr as u64) as u32;
    let pcrel = target_absolute
        .wrapping_sub(pc)
        .wrapping_add(reloc.addend() as u32);
    
    debug!("  PC=0x{:x}, target_absolute=0x{:x}, pcrel=0x{:x} (signed: {})", 
           pc, target_absolute, pcrel, pcrel as i32);

    // Determine relocation type from flags
    let reloc_offset = reloc_offset as usize;
    if reloc_offset >= code_bytes.len() {
        return Err(format!("Relocation offset {} out of bounds", reloc_offset));
    }

    match reloc.flags() {
        RelocationFlags::Elf { r_type } => {
            match r_type {
                object::elf::R_RISCV_CALL_PLT => {
                    // RISC-V CALL_PLT: auipc+jalr pair
                    // This is equivalent to R_RISCV_PCREL_HI20 + R_RISCV_PCREL_LO12_I
                    if reloc_offset + 8 > code_bytes.len() {
                        return Err(format!(
                            "CALL_PLT relocation at offset {} requires 8 bytes, but only {} available",
                            reloc_offset,
                            code_bytes.len() - reloc_offset
                        ));
                    }

                    // Read the two instructions
                    let auipc_bytes = &code_bytes[reloc_offset..reloc_offset + 4];
                    let jalr_bytes = &code_bytes[reloc_offset + 4..reloc_offset + 8];
                    let auipc_word = u32::from_le_bytes([
                        auipc_bytes[0],
                        auipc_bytes[1],
                        auipc_bytes[2],
                        auipc_bytes[3],
                    ]);
                    let jalr_word = u32::from_le_bytes([
                        jalr_bytes[0],
                        jalr_bytes[1],
                        jalr_bytes[2],
                        jalr_bytes[3],
                    ]);

                    // Apply RISC-V CALL_PLT patching (see cranelift/jit/src/compiled_blob.rs)
                    // Split pcrel into hi20 and lo12
                    // pcrel is a signed value stored as u32 (two's complement)
                    // The formula rounds to nearest 4KB boundary for hi20
                    let hi20 = pcrel.wrapping_add(0x800) & 0xFFFFF000;
                    let lo12 = pcrel.wrapping_sub(hi20) & 0xFFF;

                    // Patch auipc (bits [31:12] contain the immediate)
                    let auipc_bytes = &mut code_bytes[reloc_offset..reloc_offset + 4];
                    let patched_auipc = (auipc_word & 0xFFF) | hi20;
                    auipc_bytes.copy_from_slice(&patched_auipc.to_le_bytes());

                    // Patch jalr (bits [31:20] contain the immediate)
                    let jalr_bytes = &mut code_bytes[reloc_offset + 4..reloc_offset + 8];
                    let patched_jalr = (jalr_word & 0xFFFFF) | (lo12 << 20);
                    jalr_bytes.copy_from_slice(&patched_jalr.to_le_bytes());
                }
                object::elf::R_RISCV_PCREL_HI20 => {
                    // RISC-V PC-relative high 20 bits
                    if reloc_offset + 4 > code_bytes.len() {
                        return Err(format!(
                            "PCREL_HI20 relocation at offset {} requires 4 bytes",
                            reloc_offset
                        ));
                    }
                    let inst_bytes = &mut code_bytes[reloc_offset..reloc_offset + 4];
                    let inst_word = u32::from_le_bytes([
                        inst_bytes[0],
                        inst_bytes[1],
                        inst_bytes[2],
                        inst_bytes[3],
                    ]);
                    let hi20 = pcrel.wrapping_add(0x800) & 0xFFFFF000;
                    let patched = (inst_word & 0xFFF) | hi20;
                    inst_bytes.copy_from_slice(&patched.to_le_bytes());
                }
                object::elf::R_RISCV_PCREL_LO12_I => {
                    // RISC-V PC-relative low 12 bits (immediate)
                    if reloc_offset + 4 > code_bytes.len() {
                        return Err(format!(
                            "PCREL_LO12_I relocation at offset {} requires 4 bytes",
                            reloc_offset
                        ));
                    }
                    let inst_bytes = &mut code_bytes[reloc_offset..reloc_offset + 4];
                    let inst_word = u32::from_le_bytes([
                        inst_bytes[0],
                        inst_bytes[1],
                        inst_bytes[2],
                        inst_bytes[3],
                    ]);
                    // For LO12_I, we need the low 12 bits of the offset
                    // This is typically used with a preceding HI20 relocation
                    let lo12 = pcrel & 0xFFF;
                    let patched = (inst_word & 0xFFFFF) | (lo12 << 20);
                    inst_bytes.copy_from_slice(&patched.to_le_bytes());
                }
                object::elf::R_RISCV_32 => {
                    // RISC-V 32-bit absolute relocation
                    // Write the absolute target address directly
                    if reloc_offset + 4 > code_bytes.len() {
                        return Err(format!(
                            "R_RISCV_32 relocation at offset {} requires 4 bytes",
                            reloc_offset
                        ));
                    }
                    // target_addr is relative to section base, so absolute address is:
                    // section_load_addr + target_addr
                    let absolute_addr = (section_load_addr + target_addr as u64) as u32;
                    let reloc_bytes = &mut code_bytes[reloc_offset..reloc_offset + 4];
                    reloc_bytes.copy_from_slice(&absolute_addr.to_le_bytes());
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

/// Apply relocations to code bytes using the object crate's relocation API.
fn apply_relocations(
    obj: &object::File,
    code_bytes: &mut [u8],
    text_section_id: object::SectionIndex,
    text_section_base: u64,
) -> Result<(), String> {
    use crate::debug;
    
    // Build comprehensive symbol map (name -> address)
    // We need to handle duplicate symbols - prefer defined ones over undefined ones
    let mut symbol_map: HashMap<String, u32> = HashMap::new();

    debug!("=== Building symbol map for relocations ===");
    debug!("text_section_base: 0x{:x}", text_section_base);

    // First pass: collect all symbols, preferring defined ones
    // Strategy: First collect all defined symbols, then add undefined ones only if not already present
    let mut defined_symbols: Vec<(String, u32, object::SymbolSection)> = Vec::new();
    let mut undefined_symbols: Vec<(String, u32)> = Vec::new();
    
    for symbol in obj.symbols() {
        if let Ok(name) = symbol.name() {
            if name.is_empty() {
                continue; // Skip unnamed symbols
            }

            let addr = symbol.address();
            let symbol_section = symbol.section();
            let is_defined = symbol_section != object::SymbolSection::Undefined;

            // Calculate offset
            let offset = if addr >= text_section_base {
                (addr - text_section_base) as u32
            } else {
                addr as u32
            };

            if is_defined {
                defined_symbols.push((name.to_string(), offset, symbol_section));
            } else {
                undefined_symbols.push((name.to_string(), offset));
            }
        }
    }
    
    // Add all defined symbols first
    // If there are duplicates, keep the one with the highest (most recent) address
    // This handles cases where the same symbol appears multiple times (e.g., from different object files)
    for (name, offset, section) in defined_symbols {
        if let Some(&existing_offset) = symbol_map.get(&name) {
            // Keep the one with the higher address (more likely to be correct after linking)
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
            debug!("  Symbol '{}': addr=0x{:x}, offset=0x{:x}, section={:?} (defined)", 
                   name, text_section_base + offset as u64, offset, section);
        }
    }
    
    // Add undefined symbols only if not already present
    for (name, offset) in undefined_symbols {
        if !symbol_map.contains_key(&name) {
            symbol_map.insert(name.clone(), offset);
            debug!("  Symbol '{}': addr=0x{:x}, offset=0x{:x}, section=Undefined", 
                   name, text_section_base + offset as u64, offset);
        } else {
            debug!("  Symbol '{}': skipping undefined (already have defined)", name);
        }
    }
    
    debug!("Symbol map contains {} entries", symbol_map.len());

    // Find text section and apply relocations
    debug!("=== Applying relocations ===");
    for section in obj.sections() {
        if section.index() == text_section_id {
            let section_load_addr = section.address();
            debug!("Text section load address: 0x{:x}", section_load_addr);
            let mut reloc_count = 0;
            for (reloc_offset, reloc) in section.relocations() {
                reloc_count += 1;
                debug!("Relocation {} at offset 0x{:x}", reloc_count, reloc_offset);
                apply_single_relocation(
                    &reloc,
                    reloc_offset,
                    section_load_addr,
                    code_bytes,
                    &symbol_map,
                    obj,
                )
                .map_err(|e| {
                    // Add context about which symbol failed
                    use object::RelocationTarget;
                    match reloc.target() {
                        RelocationTarget::Symbol(sym_idx) => {
                            if let Ok(sym) = obj.symbol_by_index(sym_idx) {
                                if let Ok(name) = sym.name() {
                                    return format!(
                                        "{} (symbol: '{}', offset: 0x{:x})",
                                        e, name, reloc_offset
                                    );
                                }
                            }
                            format!(
                                "{} (symbol index: {}, offset: 0x{:x})",
                                e, sym_idx.0, reloc_offset
                            )
                        }
                        _ => format!("{} (offset: 0x{:x})", e, reloc_offset),
                    }
                })?;
            }
            debug!("Applied {} relocations", reloc_count);
            break;
        }
    }

    Ok(())
}

/// Load a RISC-V ELF file and extract code and data sections for the emulator.
///
/// This function:
/// - Parses the ELF file
/// - Extracts loadable segments
/// - Splits them into ROM (low addresses) and RAM (high addresses)
/// - Applies relocations to code sections
/// - Returns the entry point address
pub fn load_elf(elf_data: &[u8]) -> Result<ElfLoadInfo, String> {
    use elf::ElfBytes;
    use elf::abi::PT_LOAD;
    use elf::endian::LittleEndian;

    // Parse ELF
    let elf = ElfBytes::<LittleEndian>::minimal_parse(elf_data)
        .map_err(|e| format!("Failed to parse ELF: {}", e))?;

    // Verify it's RISC-V 32-bit
    if elf.ehdr.e_machine != 0xf3 {
        // EM_RISCV
        return Err(format!(
            "Not a RISC-V ELF (machine type: 0x{:x})",
            elf.ehdr.e_machine
        ));
    }
    if elf.ehdr.class != elf::file::Class::ELF32 {
        return Err("Not a 32-bit ELF".to_string());
    }

    let entry_point = elf.ehdr.e_entry as u32;

    // Split address ranges:
    // ROM/Code: 0x00000000 - 0x7FFFFFFF
    // RAM: 0x80000000 - 0xFFFFFFFF
    const RAM_START: u32 = 0x80000000;

    // Allocate buffers - we'll determine the size based on the highest address used
    let mut max_rom_addr: u32 = 0;
    let mut max_ram_addr: u32 = 0;

    // First pass: determine buffer sizes
    if let Some(segments) = elf.segments() {
        for segment in segments.iter().filter(|s| s.p_type == PT_LOAD) {
            let vaddr = segment.p_vaddr as u32;
            let memsz = segment.p_memsz as u32;
            let end_addr = vaddr.saturating_add(memsz);

            if vaddr < RAM_START {
                max_rom_addr = max_rom_addr.max(end_addr);
            } else {
                // RAM address - need to track offset from RAM_START
                let ram_offset = vaddr - RAM_START;
                let ram_end = ram_offset.saturating_add(memsz);
                max_ram_addr = max_ram_addr.max(ram_end);
            }
        }
    }

    // Allocate buffers (at least 4KB each to be safe)
    // Add extra padding to ROM for potential PC-relative loads at the end
    let rom_size = (max_rom_addr.max(4096) + 4096) as usize; // Add 4KB padding
    let ram_size = max_ram_addr.max(512 * 1024) as usize; // At least 512KB for heap
    let mut code = vec![0u8; rom_size];
    let mut ram = vec![0u8; ram_size];

    // Second pass: copy segment data
    use crate::debug;
    debug!("=== Loading segments ===");
    if let Some(segments) = elf.segments() {
        let loadable_segments: Vec<_> = segments.iter().filter(|s| s.p_type == PT_LOAD).collect();
        debug!("Found {} loadable segments", loadable_segments.len());
        
        for (idx, segment) in loadable_segments.iter().enumerate() {
            let vaddr = segment.p_vaddr as u32;
            let filesz = segment.p_filesz as usize;
            let memsz = segment.p_memsz as usize;

            debug!("  Segment {}: vaddr=0x{:x}, filesz={}, memsz={}, offset=0x{:x}", 
                   idx, vaddr, filesz, memsz, segment.p_offset);

            // Get segment data
            let data = elf_data
                .get(segment.p_offset as usize..(segment.p_offset as usize + filesz))
                .ok_or_else(|| format!("Segment data out of bounds"))?;

            if vaddr < RAM_START {
                // ROM region
                let offset = vaddr as usize;
                debug!("    -> Copying {} bytes to code[0x{:x}..0x{:x}]", filesz, offset, offset + filesz);
                if offset < code.len() && offset + filesz <= code.len() {
                    // Copy file data
                    code[offset..offset + filesz].copy_from_slice(data);
                    // Rest is zero-initialized (for .bss-like segments)
                    debug!("    -> Copied successfully");
                } else if filesz > 0 {
                    return Err(format!(
                        "Segment data out of bounds: vaddr=0x{:x}, size={}, code_len={}",
                        vaddr,
                        memsz,
                        code.len()
                    ));
                }
            } else {
                // RAM region
                let offset = (vaddr - RAM_START) as usize;
                debug!("    -> Copying {} bytes to ram[0x{:x}..0x{:x}]", filesz, offset, offset + filesz);
                if offset < ram.len() && offset + filesz <= ram.len() {
                    // Copy file data
                    ram[offset..offset + filesz].copy_from_slice(data);
                    // Rest is zero-initialized
                    debug!("    -> Copied successfully");
                } else if filesz > 0 {
                    return Err(format!(
                        "Segment data out of bounds: vaddr=0x{:x}, size={}, ram_len={}",
                        vaddr,
                        memsz,
                        ram.len()
                    ));
                }
            }
        }
    } else {
        debug!("No segments found");
    }

    // Apply relocations to code sections using the object crate
    // Parse with object crate to access sections and relocations
    let obj = object::File::parse(elf_data)
        .map_err(|e| format!("Failed to parse ELF with object crate: {}", e))?;

    // Load section data into code/RAM buffers
    // Object files use sections, not segments
    // Only load executable sections (Text) and data sections, skip metadata
    debug!("=== Loading sections ===");
    let mut section_count = 0;
    // Track current offset for sections with address 0x0 (object files)
    // Place them sequentially to avoid overwriting
    let mut next_code_offset = 0u64;
    let mut next_ram_offset = 0u64;
    // Track where we actually loaded the text section (for relocations)
    let mut actual_text_section_base = 0u64;
    
    for section in obj.sections() {
        // Skip metadata sections (symbol tables, string tables, etc.)
        let section_kind = section.kind();
        if section_kind == object::SectionKind::Metadata
            || section_kind == object::SectionKind::Other
        {
            continue;
        }

        if let Ok(name) = section.name() {
            if let Ok(data) = section.data() {
                if data.is_empty() {
                    continue;
                }

                let section_addr = section.address();
                section_count += 1;
                debug!("  Section '{}': addr=0x{:x}, size={}, kind={:?}", 
                       name, section_addr, data.len(), section_kind);

                if section_kind == object::SectionKind::Text || section_addr < RAM_START as u64 {
                    // ROM/Code region
                    // If address is 0x0 (common in object files), place sequentially
                    let offset = if section_addr == 0 {
                        let offset = next_code_offset;
                        next_code_offset += data.len() as u64;
                        offset as usize
                    } else {
                        section_addr as usize
                    };
                    // Track where we loaded the text section
                    if section_kind == object::SectionKind::Text {
                        actual_text_section_base = offset as u64;
                    }
                    debug!("    -> Copying {} bytes to code[0x{:x}..0x{:x}]", data.len(), offset, offset + data.len());
                    if offset + data.len() <= code.len() {
                        code[offset..offset + data.len()].copy_from_slice(data);
                        debug!("    -> Copied successfully");
                    } else {
                        // Extend code buffer if needed
                        let needed_size = offset + data.len();
                        debug!("    -> Extending code buffer from {} to {}", code.len(), needed_size);
                        code.resize(needed_size.max(code.len()), 0);
                        code[offset..offset + data.len()].copy_from_slice(data);
                        debug!("    -> Copied successfully");
                    }
                } else {
                    // RAM region
                    // If address is 0x0 (common in object files), place sequentially
                    let offset = if section_addr == 0 {
                        let offset = next_ram_offset;
                        next_ram_offset += data.len() as u64;
                        offset as usize
                    } else {
                        (section_addr - RAM_START as u64) as usize
                    };
                    debug!("    -> Copying {} bytes to ram[0x{:x}..0x{:x}]", data.len(), offset, offset + data.len());
                    if offset + data.len() <= ram.len() {
                        ram[offset..offset + data.len()].copy_from_slice(data);
                        debug!("    -> Copied successfully");
                    } else {
                        // Extend RAM buffer if needed
                        let needed_size = offset + data.len();
                        debug!("    -> Extending ram buffer from {} to {}", ram.len(), needed_size);
                        ram.resize(needed_size.max(ram.len()), 0);
                        ram[offset..offset + data.len()].copy_from_slice(data);
                        debug!("    -> Copied successfully");
                    }
                }
            }
        }
    }
    debug!("Loaded {} sections", section_count);

    // Find the .text section and apply relocations
    // Use the actual load address where we placed the section (not the ELF address)
    let mut text_section_id = None;
    for section in obj.sections() {
        if section.kind() == object::SectionKind::Text {
            text_section_id = Some(section.index());
            break;
        }
    }

    // Apply relocations if we found a text section
    // Use actual_text_section_base (where we loaded it) instead of section.address()
    if let Some(text_id) = text_section_id {
        apply_relocations(&obj, &mut code, text_id, actual_text_section_base)?;
    }

    Ok(ElfLoadInfo {
        code,
        ram,
        entry_point,
    })
}
