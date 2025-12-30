//! ELF patching to convert relocatable object files to executables.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::debug;
use crate::elf_linker::LinkerError;
use object::{
    elf::{FileHeader32, FileHeader64, SectionHeader32, SectionHeader64, ET_EXEC},
    endian::{Endian, Endianness},
    read::elf::FileHeader as FileHeaderTrait,
    write::SectionId,
};

/// Patch the ELF file to be an executable with correct section addresses.
/// Also patches __data_source_start symbol to point to the actual .data section LMA.
pub fn patch_elf_to_executable(
    bytes: &mut Vec<u8>,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    endian: Endianness,
    data_section_lma: Option<u64>,
) -> Result<(), LinkerError> {
    debug!("=== Patching ELF to executable ===");
    
    if bytes.len() < 20 {
        return Err(LinkerError::ParseError("ELF file too small".to_string()));
    }
    
    // Check ELF magic
    if &bytes[0..4] != b"\x7fELF" {
        return Err(LinkerError::ParseError("Not an ELF file".to_string()));
    }
    
    // Determine if 32-bit or 64-bit (EI_CLASS: 1=32-bit, 2=64-bit)
    let is_64 = bytes[4] == 2;
    debug!("ELF format: {}bit", if is_64 { 64 } else { 32 });
    
    match endian {
        Endianness::Little => {
            use object::endian::LittleEndian;
            if is_64 {
                patch_elf64_to_executable::<LittleEndian>(bytes, base_section_map, data_section_lma)
            } else {
                patch_elf32_to_executable::<LittleEndian>(bytes, base_section_map, data_section_lma)
            }
        }
        Endianness::Big => {
            use object::endian::BigEndian;
            if is_64 {
                patch_elf64_to_executable::<BigEndian>(bytes, base_section_map, data_section_lma)
            } else {
                patch_elf32_to_executable::<BigEndian>(bytes, base_section_map, data_section_lma)
            }
        }
    }
}

fn patch_elf64_to_executable<E: Endian>(
    bytes: &mut Vec<u8>,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    data_section_lma: Option<u64>,
) -> Result<(), LinkerError> {
    let e = E::default();
    
    // Parse ELF header and extract needed values
    let (e_shoff, e_shentsize, e_shnum) = {
        let header = FileHeader64::<E>::parse(&bytes[..])
            .map_err(|e| LinkerError::ParseError(format!("Failed to parse ELF header: {}", e)))?;
        
        // Get section header table info before dropping header
        let e_shoff = header.e_shoff.get(e) as usize;
        let e_shentsize = header.e_shentsize.get(e) as usize;
        let e_shnum = header.e_shnum.get(e) as usize;
        
        (e_shoff, e_shentsize, e_shnum)
    };
    
    if e_shoff == 0 || e_shnum == 0 {
        debug!("No section headers to patch");
        return Ok(());
    }
    
    debug!("Section header table: offset=0x{:x}, size={}, count={}", e_shoff, e_shentsize, e_shnum);
    
    // Patch e_type to ET_EXEC
    {
        let header_mut: &mut FileHeader64<E> =
            object::from_bytes_mut(&mut bytes[..])
                .map_err(|_| LinkerError::ParseError("Failed to get mutable header".to_string()))?.0;
        
        let old_e_type = header_mut.e_type.get(e);
        header_mut.e_type.set(e, ET_EXEC);
        debug!("Changed e_type from {} (ET_REL) to {} (ET_EXEC)", old_e_type, ET_EXEC);
    }
    
    // Parse sections to get names and collect section indices to patch
    let patches: Vec<(usize, u64)> = {
        let header = FileHeader64::<E>::parse(&bytes[..])
            .map_err(|e| LinkerError::ParseError(format!("Failed to parse ELF header: {}", e)))?;
        let sections = header.sections(e, &bytes[..])
            .map_err(|e| LinkerError::ParseError(format!("Failed to parse sections: {}", e)))?;
        
        // Collect section indices and addresses to patch (to avoid borrow checker issues)
        let mut patches: Vec<(usize, u64)> = Vec::new();
        for (name, &(_, address)) in base_section_map {
            if let Some((section_index, _section_header)) = sections.section_by_name(e, name.as_bytes()) {
                let sh_offset = e_shoff + section_index.0 * e_shentsize;
                if sh_offset + e_shentsize <= bytes.len() {
                    patches.push((sh_offset, address));
                    debug!("  Will patch section '{}' at offset 0x{:x} to address 0x{:x}", name, sh_offset, address);
                } else {
                    debug!("Warning: Section header for '{}' out of bounds, skipping", name);
                }
            }
        }
        patches
    };
    
    // Now patch the section addresses (sections is dropped, so we can mutate bytes)
    for (sh_offset, address) in &patches {
        let sh_mut: &mut SectionHeader64<E> =
            object::from_bytes_mut(&mut bytes[*sh_offset..])
                .map_err(|_| LinkerError::ParseError("Failed to get mutable section header".to_string()))?.0;
        
        let old_addr = sh_mut.sh_addr.get(e);
        sh_mut.sh_addr.set(e, *address);
        debug!("  Patched section at offset 0x{:x}: sh_addr from 0x{:x} to 0x{:x}", sh_offset, old_addr, address);
    }
    
    let patched_count = patches.len();
    
    debug!("Patched {} section addresses", patched_count);
    
    // TODO: Patch __data_source_start symbol if .data section LMA is provided
    // This is complex and requires parsing symbol tables. For now, skip it.
    if data_section_lma.is_some() {
        debug!("Note: __data_source_start symbol patching not yet implemented");
    }
    
    Ok(())
}

fn patch_elf32_to_executable<E: Endian>(
    bytes: &mut Vec<u8>,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    data_section_lma: Option<u64>,
) -> Result<(), LinkerError> {
    let e = E::default();
    
    // Parse ELF header and extract needed values
    let (e_shoff, e_shentsize, e_shnum) = {
        let header = FileHeader32::<E>::parse(&bytes[..])
            .map_err(|e| LinkerError::ParseError(format!("Failed to parse ELF header: {}", e)))?;
        
        // Get section header table info before dropping header
        let e_shoff = header.e_shoff.get(e) as usize;
        let e_shentsize = header.e_shentsize.get(e) as usize;
        let e_shnum = header.e_shnum.get(e) as usize;
        
        (e_shoff, e_shentsize, e_shnum)
    };
    
    if e_shoff == 0 || e_shnum == 0 {
        debug!("No section headers to patch");
        return Ok(());
    }
    
    debug!("Section header table: offset=0x{:x}, size={}, count={}", e_shoff, e_shentsize, e_shnum);
    
    // Patch e_type to ET_EXEC
    {
        let header_mut: &mut FileHeader32<E> =
            object::from_bytes_mut(&mut bytes[..])
                .map_err(|_| LinkerError::ParseError("Failed to get mutable header".to_string()))?.0;
        
        let old_e_type = header_mut.e_type.get(e);
        header_mut.e_type.set(e, ET_EXEC);
        debug!("Changed e_type from {} (ET_REL) to {} (ET_EXEC)", old_e_type, ET_EXEC);
    }
    
    // Parse sections to get names and collect section indices to patch
    let patches: Vec<(usize, u64)> = {
        let header = FileHeader32::<E>::parse(&bytes[..])
            .map_err(|e| LinkerError::ParseError(format!("Failed to parse ELF header: {}", e)))?;
        let sections = header.sections(e, &bytes[..])
            .map_err(|e| LinkerError::ParseError(format!("Failed to parse sections: {}", e)))?;
        
        // Collect section indices and addresses to patch (to avoid borrow checker issues)
        let mut patches: Vec<(usize, u64)> = Vec::new();
        for (name, &(_, address)) in base_section_map {
            if let Some((section_index, _section_header)) = sections.section_by_name(e, name.as_bytes()) {
                let sh_offset = e_shoff + section_index.0 * e_shentsize;
                if sh_offset + e_shentsize <= bytes.len() {
                    patches.push((sh_offset, address));
                    debug!("  Will patch section '{}' at offset 0x{:x} to address 0x{:x}", name, sh_offset, address);
                } else {
                    debug!("Warning: Section header for '{}' out of bounds, skipping", name);
                }
            }
        }
        patches
    };
    
    // Now patch the section addresses (sections is dropped, so we can mutate bytes)
    for (sh_offset, address) in &patches {
        let sh_mut: &mut SectionHeader32<E> =
            object::from_bytes_mut(&mut bytes[*sh_offset..])
                .map_err(|_| LinkerError::ParseError("Failed to get mutable section header".to_string()))?.0;
        
        let old_addr = sh_mut.sh_addr.get(e) as u64;
        sh_mut.sh_addr.set(e, *address as u32);
        debug!("  Patched section at offset 0x{:x}: sh_addr from 0x{:x} to 0x{:x}", sh_offset, old_addr, address);
    }
    
    let patched_count = patches.len();
    
    debug!("Patched {} section addresses", patched_count);
    
    // TODO: Patch __data_source_start symbol if .data section LMA is provided
    // This is complex and requires parsing symbol tables. For now, skip it.
    if data_section_lma.is_some() {
        debug!("Note: __data_source_start symbol patching not yet implemented");
    }
    
    Ok(())
}

