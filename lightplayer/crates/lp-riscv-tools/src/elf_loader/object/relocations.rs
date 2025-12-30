//! Relocation application for object files.

extern crate alloc;

use crate::debug;
use alloc::string::{String, ToString};
use hashbrown::HashMap;
use ::object::{Object, ObjectSection};

use super::sections::ObjectSectionPlacement;
use super::super::relocations::{
    analyze_relocations, apply_relocations_phase2, SectionAddressInfo, BufferSlice,
};
use super::super::memory::RAM_START;

/// Apply relocations for object file.
///
/// Applies all relocations in the object file using the merged symbol map
/// (containing both base executable and object file symbols).
///
/// # Arguments
///
/// * `obj` - The object file containing relocations
/// * `code` - Mutable reference to code buffer
/// * `ram` - Mutable reference to RAM buffer
/// * `merged_symbol_map` - Merged symbol map (base + object)
/// * `section_placement` - Information about where object file sections were placed
///
/// # Returns
///
/// Ok(()) if relocations were applied successfully, or an error message.
pub fn apply_object_relocations(
    obj: &::object::File,
    code: &mut [u8],
    ram: &mut [u8],
    merged_symbol_map: &HashMap<String, u32>,
    section_placement: &ObjectSectionPlacement,
) -> Result<(), String> {
    debug!("=== Applying object file relocations ===");

    // Phase 1: Analyze relocations (this works with section-relative addresses)
    let (relocations, got_tracker, _section_addrs) = analyze_relocations(
        obj,
        code,
        ram,
        merged_symbol_map,
    )?;

    // Build adjusted section address map for object file sections
    // Object file sections are placed at specific addresses, so we need to adjust
    // the section addresses based on where they were actually placed
    let mut adjusted_section_addrs: HashMap<String, SectionAddressInfo> = HashMap::new();

    for section in obj.sections() {
        let section_name = match section.name() {
            Ok(name) => name,
            Err(_) => continue,
        };

        // Skip debug sections
        if section_name.starts_with(".debug_") || section_name.starts_with(".zdebug_") {
            continue;
        }

        let section_kind = section.kind();
        let _section_size = section.size() as usize;

        // Only process loadable sections
        match section_kind {
            ::object::SectionKind::Text
            | ::object::SectionKind::Data
            | ::object::SectionKind::ReadOnlyData
            | ::object::SectionKind::ReadOnlyString
            | ::object::SectionKind::UninitializedData => {}
            _ => continue,
        }

        match section_name {
            ".text" => {
                // .text section: VMA and LMA are the same, in code buffer
                let vma = section_placement.text_start as u64;
                let lma = vma;
                adjusted_section_addrs.insert(
                    section_name.to_string(),
                    SectionAddressInfo {
                        vma,
                        lma,
                        buffer: BufferSlice::Rom {
                            offset: section_placement.text_start as usize,
                        },
                    },
                );
                debug!("  Section '.text': VMA=0x{:x}, LMA=0x{:x}, offset={}",
                       vma, lma, section_placement.text_start);
            }
            ".data" => {
                // .data section: VMA in RAM, LMA same as VMA (already copied to RAM)
                let vma = (RAM_START as u64) + section_placement.data_start as u64;
                let lma = vma; // For object files, .data is already in RAM
                adjusted_section_addrs.insert(
                    section_name.to_string(),
                    SectionAddressInfo {
                        vma,
                        lma,
                        buffer: BufferSlice::Ram {
                            offset: section_placement.data_start as usize,
                        },
                    },
                );
                debug!("  Section '.data': VMA=0x{:x}, LMA=0x{:x}, offset={}",
                       vma, lma, section_placement.data_start);
            }
            ".rodata" => {
                // .rodata section: placed in code buffer after .text
                // Calculate placement (after .text, aligned)
                let rodata_start = section_placement.text_start + section_placement.text_size as u32;
                let rodata_start_aligned = (rodata_start + 3) & !3;
                let vma = rodata_start_aligned as u64;
                let lma = vma;
                adjusted_section_addrs.insert(
                    section_name.to_string(),
                    SectionAddressInfo {
                        vma,
                        lma,
                        buffer: BufferSlice::Rom {
                            offset: rodata_start_aligned as usize,
                        },
                    },
                );
                debug!("  Section '.rodata': VMA=0x{:x}, LMA=0x{:x}, offset={}",
                       vma, lma, rodata_start_aligned);
            }
            ".bss" => {
                // .bss section: placed in RAM buffer after .data
                let bss_start = section_placement.data_start + section_placement.data_size as u32;
                let bss_start_aligned = (bss_start + 3) & !3;
                let vma = (RAM_START as u64) + bss_start_aligned as u64;
                let lma = vma;
                adjusted_section_addrs.insert(
                    section_name.to_string(),
                    SectionAddressInfo {
                        vma,
                        lma,
                        buffer: BufferSlice::Ram {
                            offset: bss_start_aligned as usize,
                        },
                    },
                );
                debug!("  Section '.bss': VMA=0x{:x}, LMA=0x{:x}, offset={}",
                       vma, lma, bss_start_aligned);
            }
            _ => {
                // Other sections: skip for now
                debug!("  Skipping section '{}' for relocation", section_name);
            }
        }
    }

    // Phase 2: Apply relocations with adjusted section addresses
    apply_relocations_phase2(
        &relocations,
        &got_tracker,
        &adjusted_section_addrs,
        code,
        ram,
        merged_symbol_map,
    )?;

    debug!("=== Object file relocations applied successfully ===");
    Ok(())
}

