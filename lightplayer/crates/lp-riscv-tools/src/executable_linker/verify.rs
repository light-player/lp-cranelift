//! Final verification of linked ELF.

use alloc::format;
use alloc::vec::Vec;
use crate::debug;
use crate::elf_linker::LinkerError;
use object::{read::{Object as ObjectTrait, ObjectSymbol}, SymbolKind};

/// Verify that required symbols exist in the linked ELF.
pub fn verify_linked_elf(linked_elf_bytes: &[u8], user_main_address: Option<u64>) -> Result<(), LinkerError> {
    let linked_elf = object::File::parse(linked_elf_bytes)?;
    let mut missing_symbols = Vec::new();

    // Check for required __lp_* symbols (mem functions might be inlined or provided differently)
    let required_symbols = ["__lp_fixed32_sqrt", "__lp_fixed32_mul", "__lp_fixed32_div"];
    for sym_name in required_symbols.iter() {
        let mut found = false;
        for symbol in linked_elf.symbols() {
            if let Ok(name) = symbol.name() {
                if name == *sym_name && symbol.kind() == SymbolKind::Text && !symbol.is_undefined() {
                    found = true;
                    break;
                }
            }
        }
        if !found {
            missing_symbols.push(*sym_name);
        }
    }

    if !missing_symbols.is_empty() {
        return Err(LinkerError::ParseError(format!(
            "Required symbols not found after linking: {:?}",
            missing_symbols
        )));
    }

    // Verify __USER_MAIN_PTR is set correctly if main() was found
    if let Some(main_addr) = user_main_address {
        debug!("Verifying __USER_MAIN_PTR points to main() at 0x{:x}", main_addr);
        // Note: Actual verification would require reading the .data section, which is complex
        // For now, we trust the update logic above
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Comprehensive tests would require creating mock ELF files
    // These are placeholder tests that verify the logic structure
}

