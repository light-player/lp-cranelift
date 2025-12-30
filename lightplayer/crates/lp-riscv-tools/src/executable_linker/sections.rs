//! Section copying logic for base executable and object file.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::debug;
use crate::elf_linker::LinkerError;
use object::{
    SectionKind,
    read::{Object as ObjectTrait, ObjectSection},
    write::{Object as WriteObject, SectionId, StandardSegment},
};

/// Information about a deferred .data section that needs to be written later.
pub struct DeferredDataSection {
    pub name: String,
    pub data: Vec<u8>,
    pub address: u64,
}

/// Result of copying base sections.
pub struct BaseSectionsResult {
    pub base_text_section_size: u64,
    pub deferred_data_section: Option<DeferredDataSection>,
}

/// Copy all sections from base executable.
/// Returns information about deferred .data section and base .text section size.
pub fn copy_base_sections(
    base_elf: &object::File,
    writer: &mut WriteObject,
    base_section_map: &mut BTreeMap<String, (SectionId, u64)>,
) -> Result<BaseSectionsResult, LinkerError> {
    debug!("Copying sections from base executable...");
    let mut base_text_section_size = 0u64;
    let mut deferred_data_section: Option<DeferredDataSection> = None;

    for section in base_elf.sections() {
        let section_name = section.name()?;
        debug!("  Base section: {} at 0x{:x}", section_name, section.address());

        let section_data = section.data()?;

        // Defer .data section - we'll write it later with updated __USER_MAIN_PTR
        // Note: .data section has > RAM AT > ROM, so the data is loaded from ROM but lives at RAM address
        // We need to update the data that will be loaded from ROM (the section.data() we get here)
        if section_name == ".data" {
            // Store the section data (this is what gets loaded from ROM)
            // Also store the file offset if available to verify we're updating the right place
            let file_offset = section.file_range().map(|r| r.0).unwrap_or(0);
            debug!("  Deferring .data section: vaddr=0x{:x}, file_offset=0x{:x}, size={}", 
                   section.address(), file_offset, section_data.len());
            deferred_data_section = Some(DeferredDataSection {
                name: String::from(section_name),
                data: section_data.to_vec(),
                address: section.address(),
            });
            // Still create the section, but don't write data yet
            let segment = writer.segment_name(StandardSegment::Data).to_vec();
            let section_id = writer.add_section(segment, section_name.as_bytes().to_vec(), section.kind());
            writer.section_mut(section_id).flags = section.flags();
            base_section_map.insert(section_name.to_string(), (section_id, section.address()));
            continue;
        }

        let segment = match section.kind() {
            SectionKind::Text => writer.segment_name(StandardSegment::Text).to_vec(),
            _ => writer.segment_name(StandardSegment::Data).to_vec(),
        };

        let section_id = writer.add_section(segment, section_name.as_bytes().to_vec(), section.kind());

        // Write section data
        if !section_data.is_empty() {
            writer.append_section_data(section_id, section_data, 1);
            if section_name == ".text" {
                base_text_section_size = section_data.len() as u64;
            }
        }

        writer.section_mut(section_id).flags = section.flags();

        base_section_map.insert(section_name.to_string(), (section_id, section.address()));
    }

    Ok(BaseSectionsResult {
        base_text_section_size,
        deferred_data_section,
    })
}

/// Copy sections from object file.
/// Handles merging .text sections and keeping other sections separate.
pub fn copy_object_sections(
    object_elf: &object::File,
    writer: &mut WriteObject,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    object_section_map: &mut BTreeMap<String, (SectionId, u64)>,
    object_section_start: u64,
) -> Result<(), LinkerError> {
    debug!("Copying sections from object file...");
    let mut current_object_address = object_section_start;

    for section in object_elf.sections() {
        let section_name = section.name()?;
        debug!("  Object section: {} (original: 0x{:x})", section_name, section.address());

        // Skip debug sections
        if section_name.starts_with(".debug_") || section_name.starts_with(".zdebug_") {
            continue;
        }

        let section_data = section.data()?;

        // For .text sections, merge into the existing .text section
        // For other sections, keep them separate
        let (_final_section_name, section_id) = if section_name == ".text" && base_section_map.contains_key(".text") {
            // Merge into existing .text section
            let &(existing_section_id, existing_address) = base_section_map.get(".text").unwrap();
            // Get current size of .text section to append after it
            let current_size = writer.section(existing_section_id).data().len() as u64;
            let append_address = existing_address + current_size;
            debug!("    Merging into .text section at offset 0x{:x}", append_address);
            (String::from(".text"), existing_section_id)
        } else if base_section_map.contains_key(section_name) {
            // Append suffix to keep separate for non-text sections
            let final_name = format!("{}.user", section_name);
            let segment = match section.kind() {
                SectionKind::Text => writer.segment_name(StandardSegment::Text).to_vec(),
                _ => writer.segment_name(StandardSegment::Data).to_vec(),
            };
            let new_section_id = writer.add_section(segment, final_name.as_bytes().to_vec(), section.kind());
            (final_name, new_section_id)
        } else {
            // New section, create it
            let segment = match section.kind() {
                SectionKind::Text => writer.segment_name(StandardSegment::Text).to_vec(),
                _ => writer.segment_name(StandardSegment::Data).to_vec(),
            };
            let new_section_id = writer.add_section(segment, section_name.as_bytes().to_vec(), section.kind());
            (String::from(section_name), new_section_id)
        };

        // Align address for new sections (merged .text uses existing address)
        if section_name != ".text" || !base_section_map.contains_key(".text") {
            current_object_address = (current_object_address + 15) & !15;
        }

        // Write section data
        if !section_data.is_empty() {
            writer.append_section_data(section_id, section_data, 1);
            writer.section_mut(section_id).flags = section.flags();

            // Update address tracking
            if section_name == ".text" && base_section_map.contains_key(".text") {
                // Already merged, address is tracked by base section
            } else {
                current_object_address += section_data.len() as u64;
            }
        }

        object_section_map.insert(String::from(section_name), (section_id, current_object_address));
        debug!("    Placed at: 0x{:x}", current_object_address);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Comprehensive tests would require creating mock ELF files
    // These are placeholder tests that verify the logic structure
}

