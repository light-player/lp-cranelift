//! Address layout calculation utilities.

use crate::debug;
use object::read::{Object as ObjectTrait, ObjectSection};

/// Calculate the highest address used in the base executable.
pub fn calculate_highest_base_address(base_elf: &object::File) -> Result<u64, crate::elf_linker::LinkerError> {
    let mut highest_base_address = 0u64;
    for section in base_elf.sections() {
        let address = section.address();
        if address > highest_base_address {
            highest_base_address = address;
        }
        // Also check section size
        if let Ok(data) = section.data() {
            let end_address = address + data.len() as u64;
            if end_address > highest_base_address {
                highest_base_address = end_address;
            }
        }
    }
    Ok(highest_base_address)
}

/// Calculate where object sections should start (after base executable, aligned to 16 bytes).
pub fn calculate_object_section_start(highest_base_address: u64) -> u64 {
    (highest_base_address + 15) & !15
}

/// Calculate address layout for linking.
pub fn calculate_layout(base_elf: &object::File) -> Result<(u64, u64), crate::elf_linker::LinkerError> {
    let highest_base_address = calculate_highest_base_address(base_elf)?;
    debug!("Highest base address: 0x{:x}", highest_base_address);

    let object_section_start = calculate_object_section_start(highest_base_address);
    debug!("Object sections will start at: 0x{:x}", object_section_start);

    Ok((highest_base_address, object_section_start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_object_section_start() {
        // Test alignment to 16 bytes
        assert_eq!(calculate_object_section_start(0x1000), 0x1000);
        assert_eq!(calculate_object_section_start(0x1001), 0x1010);
        assert_eq!(calculate_object_section_start(0x100F), 0x1010);
        assert_eq!(calculate_object_section_start(0x1010), 0x1010);
        assert_eq!(calculate_object_section_start(0x1011), 0x1020);
    }
}

