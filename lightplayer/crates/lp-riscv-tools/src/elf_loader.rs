//! ELF file loader for RISC-V emulator.
//!
//! This module provides utilities to load RISC-V ELF files into the emulator's memory.

#![cfg(feature = "std")]

extern crate std;

use alloc::{format, string::{String, ToString}, vec, vec::Vec};

/// Information extracted from an ELF file for emulator loading.
pub struct ElfLoadInfo {
    /// Code/ROM region (starts at address 0)
    pub code: Vec<u8>,
    /// RAM region (starts at 0x80000000)
    pub ram: Vec<u8>,
    /// Entry point address
    pub entry_point: u32,
}

/// Load a RISC-V ELF file and extract code and data sections for the emulator.
///
/// This function:
/// - Parses the ELF file
/// - Extracts loadable segments
/// - Splits them into ROM (low addresses) and RAM (high addresses)
/// - Returns the entry point address
pub fn load_elf(elf_data: &[u8]) -> Result<ElfLoadInfo, String> {
    use elf::abi::PT_LOAD;
    use elf::endian::LittleEndian;
    use elf::ElfBytes;

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
    if let Some(segments) = elf.segments() {
        for segment in segments.iter().filter(|s| s.p_type == PT_LOAD) {
            let vaddr = segment.p_vaddr as u32;
            let filesz = segment.p_filesz as usize;
            let memsz = segment.p_memsz as usize;

            // Get segment data
            let data = elf_data
                .get(segment.p_offset as usize..(segment.p_offset as usize + filesz))
                .ok_or_else(|| format!("Segment data out of bounds"))?;

            if vaddr < RAM_START {
                // ROM region
                let offset = vaddr as usize;
                if offset < code.len() && offset + filesz <= code.len() {
                    // Copy file data
                    code[offset..offset + filesz].copy_from_slice(data);
                    // Rest is zero-initialized (for .bss-like segments)
                } else if filesz > 0 {
                    return Err(format!(
                        "Segment data out of bounds: vaddr=0x{:x}, size={}, code_len={}",
                        vaddr, memsz, code.len()
                    ));
                }
            } else {
                // RAM region
                let offset = (vaddr - RAM_START) as usize;
                if offset < ram.len() && offset + filesz <= ram.len() {
                    // Copy file data
                    ram[offset..offset + filesz].copy_from_slice(data);
                    // Rest is zero-initialized
                } else if filesz > 0 {
                    return Err(format!(
                        "Segment data out of bounds: vaddr=0x{:x}, size={}, ram_len={}",
                        vaddr, memsz, ram.len()
                    ));
                }
            }
        }
    }

    Ok(ElfLoadInfo {
        code,
        ram,
        entry_point,
    })
}

