//! Handling of __USER_MAIN_PTR relocation.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use crate::debug;
use crate::elf_linker::LinkerError;
use object::{
    read::{Object as ObjectTrait, ObjectSymbol},
    write::{Object as WriteObject, Relocation, SectionId},
};
use super::sections::DeferredDataSection;

/// Write the deferred .data section and add relocation for __USER_MAIN_PTR.
pub fn write_deferred_data_section_and_relocation(
    base_elf: &object::File,
    writer: &mut WriteObject,
    base_section_map: &BTreeMap<String, (SectionId, u64)>,
    symbol_map: &BTreeMap<String, object::write::SymbolId>,
    deferred_data_section: Option<DeferredDataSection>,
    user_main_address: Option<u64>,
) -> Result<(), LinkerError> {
    if let Some(deferred) = deferred_data_section {
        if let Some(&(data_section_id, _)) = base_section_map.get(&deferred.name) {
            // DEBUG: Check what's in the .data section before writing
            debug!("DEBUG: .data section size: {} bytes", deferred.data.len());
            if deferred.data.len() >= 4 {
                let first_word = u32::from_le_bytes([
                    deferred.data[0],
                    deferred.data[1],
                    deferred.data[2],
                    deferred.data[3],
                ]);
                debug!("DEBUG: First word in .data section: 0x{:x} (expected 0xDEADBEEF for __USER_MAIN_PTR)", first_word);
            }
            if deferred.data.len() >= 8 {
                let second_word = u32::from_le_bytes([
                    deferred.data[4],
                    deferred.data[5],
                    deferred.data[6],
                    deferred.data[7],
                ]);
                debug!("DEBUG: Second word in .data section: 0x{:x}", second_word);
            }
            
            // Write the .data section data (unchanged - relocations will update it)
            if !deferred.data.is_empty() {
                writer.append_section_data(data_section_id, &deferred.data, 1);
            }

            // TEMPORARILY DISABLED: Add relocation for __USER_MAIN_PTR to point to our main() function
            // This is disabled to verify that DEADBEEF sentinel is present and to debug what's modifying it
            debug!("DEBUG: Relocation for __USER_MAIN_PTR is DISABLED - sentinel value should remain 0xDEADBEEF");
            if let Some(main_addr) = user_main_address {
                debug!("DEBUG: Would set __USER_MAIN_PTR to main() at 0x{:x}, but relocation is disabled", main_addr);
                // DISABLED: add_user_main_relocation(
                //     base_elf,
                //     writer,
                //     data_section_id,
                //     symbol_map,
                //     deferred.address,
                //     main_addr,
                // )?;
            } else {
                debug!("Warning: user main() not found, __USER_MAIN_PTR will remain 0");
            }
        }
    }
    Ok(())
}

/// Add relocation for __USER_MAIN_PTR to point to user main() function.
fn add_user_main_relocation(
    base_elf: &object::File,
    writer: &mut WriteObject,
    data_section_id: SectionId,
    symbol_map: &BTreeMap<String, object::write::SymbolId>,
    data_section_address: u64,
    main_addr: u64,
) -> Result<(), LinkerError> {
    debug!("Adding relocation for __USER_MAIN_PTR to point to main() at 0x{:x}", main_addr);

    // Find __USER_MAIN_PTR symbol
    let ptr_symbol = base_elf.symbols()
        .find(|s| s.name().map_or(false, |n| n == "__USER_MAIN_PTR"));

    if let Some(ptr_symbol) = ptr_symbol {
        let ptr_address = ptr_symbol.address();

        // Calculate offset within .data section
        let offset = if ptr_address >= 0x80000000 {
            // Absolute RAM address - calculate offset from .data section start
            (ptr_address - data_section_address) as u64
        } else {
            // Section-relative address
            ptr_address.wrapping_sub(data_section_address)
        };

        // Get the _user_main symbol we added earlier when processing object file symbols
        let user_main_symbol_id = symbol_map.get("_user_main")
            .ok_or_else(|| LinkerError::ParseError("_user_main symbol not found in symbol map - user main() was not found".to_string()))?;

        // Add RISC-V 32-bit relocation: R_RISCV_32 (absolute 32-bit address)
        // This will write the address of _user_main directly to __USER_MAIN_PTR at load time
        writer.add_relocation(
            data_section_id,
            Relocation {
                offset,
                symbol: *user_main_symbol_id,
                addend: 0, // No addend needed - we want the exact address
                flags: object::write::RelocationFlags::Elf {
                    r_type: 1, // R_RISCV_32 - absolute 32-bit address
                },
            },
        )?;

        debug!("  Added relocation for __USER_MAIN_PTR at offset 0x{:x} (address 0x{:x}) -> main() at 0x{:x}", 
               offset, ptr_address, main_addr);
    } else {
        debug!("Warning: __USER_MAIN_PTR symbol not found, skipping relocation");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Comprehensive tests would require creating mock ELF files
    // These are placeholder tests that verify the logic structure
}

