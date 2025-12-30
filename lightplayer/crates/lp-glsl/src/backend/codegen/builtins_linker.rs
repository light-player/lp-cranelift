//! Builtins library linking and verification utilities

#[cfg(all(feature = "std", feature = "emulator"))]
extern crate std;

#[cfg(all(feature = "std", feature = "emulator"))]
use crate::error::{ErrorCode, GlslError};
#[cfg(all(feature = "std", feature = "emulator"))]
use alloc::vec::Vec;

/// Link builtins executable into ELF and verify symbols are defined
///
/// # Arguments
/// * `elf_bytes` - The ELF object file bytes to link into the executable
/// * `builtins_exe_bytes` - The lp-builtins-app executable bytes
///
/// # Returns
/// * `Ok(Vec<u8>)` - The linked ELF file bytes with builtins
/// * `Err(GlslError)` - If linking or verification fails
#[cfg(all(feature = "std", feature = "emulator"))]
pub fn link_and_verify_builtins(
    elf_bytes: &[u8],
    builtins_exe_bytes: &[u8],
) -> Result<Vec<u8>, GlslError> {
    use crate::backend::builtins::registry::BuiltinId;
    use object::{File as ObjectFile, Object, ObjectSymbol, SymbolKind, SymbolSection};

    crate::debug!("=== Linking object file into builtins executable ===");
    crate::debug!("Builtins executable size: {} bytes", builtins_exe_bytes.len());
    crate::debug!("Object file size: {} bytes", elf_bytes.len());

    if builtins_exe_bytes.is_empty() {
        return Err(GlslError::new(
            ErrorCode::E0400,
            "lp-builtins-app executable is empty or not available. \
             Build it with: scripts/build-builtins.sh",
        ));
    }

    // Link the object file into the executable
    crate::debug!("Attempting to link object file into executable...");
    let linked_elf = lp_riscv_tools::executable_linker::link_into_executable(builtins_exe_bytes, elf_bytes)
        .map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!(
                    "Failed to link object file into executable: {}. \
                     Ensure lp-builtins-app is correctly compiled.",
                    e
                ),
            )
        })?;

    crate::debug!("Linking succeeded!");

    // Verify that builtin symbols are present and defined in the linked ELF
    let obj = ObjectFile::parse(&linked_elf[..]).map_err(|e| {
        GlslError::new(
            ErrorCode::E0400,
            format!("Failed to parse linked ELF: {}", e),
        )
    })?;

    let mut missing_symbols = Vec::new();
    let mut undefined_symbols = Vec::new();

    crate::debug!("Checking for builtin symbols in linked ELF...");

    // Debug: Count symbols (but don't print them all - too verbose!)
    let mut symbol_count = 0;
    let mut text_symbols = 0;
    for symbol in obj.symbols() {
        symbol_count += 1;
        if symbol.kind() == SymbolKind::Text {
            text_symbols += 1;
        }
    }
    crate::debug!("ELF contains {} total symbols ({} text symbols)", symbol_count, text_symbols);

    for builtin in BuiltinId::all() {
        let symbol_name = builtin.name();
        crate::debug!("Checking for builtin symbol: {}", symbol_name);
        let mut found_defined = false;
        let mut found_undefined = false;

        for symbol in obj.symbols() {
            if let Ok(name) = symbol.name() {
                                if name == symbol_name && symbol.kind() == SymbolKind::Text {
                    crate::debug!(
                        "  Found symbol {}: section={:?} address=0x{:x}",
                        name,
                        symbol.section(),
                        symbol.address()
                    );
                                    if symbol.section() == SymbolSection::Undefined {
                        found_undefined = true;
                        crate::debug!("    -> Symbol is UNDEFINED");
                    } else {
                        found_defined = true;
                        crate::debug!("    -> Symbol is DEFINED");
                        break;
                    }
                }
            }
        }

        if found_undefined && !found_defined {
            crate::debug!("  -> Adding to undefined_symbols: {}", symbol_name);
            undefined_symbols.push(symbol_name);
        } else if !found_defined {
            crate::debug!("  -> Adding to missing_symbols: {}", symbol_name);
            missing_symbols.push(symbol_name);
        } else {
            crate::debug!("  -> Symbol {} is properly defined", symbol_name);
        }
    }

    crate::debug!("undefined_symbols: {:?}", undefined_symbols);
    crate::debug!("missing_symbols: {:?}", missing_symbols);

    if !undefined_symbols.is_empty() {
        return Err(GlslError::new(
            ErrorCode::E0400,
            format!(
                "Builtin symbols are undefined after linking: {:?}. \
                 These symbols were declared but not resolved by the linker. \
                 Ensure lp-builtins library is built and linked correctly.",
                undefined_symbols
            ),
        ));
    }

    if !missing_symbols.is_empty() {
        return Err(GlslError::new(
            ErrorCode::E0400,
            format!(
                "Builtin symbols not found after linking: {:?}. \
                 Ensure lp-builtins library is built and contains these symbols.",
                missing_symbols
            ),
        ));
    }

    Ok(linked_elf)
}

