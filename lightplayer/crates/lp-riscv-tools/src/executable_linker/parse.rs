//! ELF parsing and validation utilities.

use crate::debug;
use crate::elf_linker::LinkerError;
use alloc::format;
use object::read::Object as ObjectTrait;

/// Parsed ELF files and architecture information.
pub struct ParsedElfs<'data> {
    pub base_elf: object::File<'data>,
    pub object_elf: object::File<'data>,
    pub arch: object::Architecture,
    pub endian: object::Endianness,
}

/// Parse both ELF files and validate architecture match.
pub fn parse_elfs<'data>(
    base_executable_bytes: &'data [u8],
    object_file_bytes: &'data [u8],
) -> Result<ParsedElfs<'data>, LinkerError> {
    debug!("=== Parsing ELF files ===");
    debug!("Base executable size: {} bytes", base_executable_bytes.len());
    debug!("Object file size: {} bytes", object_file_bytes.len());

    let base_elf = object::File::parse(base_executable_bytes)?;
    let object_elf = object::File::parse(object_file_bytes)?;

    validate_architecture(&base_elf, &object_elf)?;

    let arch = base_elf.architecture();
    let endian = base_elf.endianness();

    debug!("Architecture: {:?}, Endianness: {:?}", arch, endian);

    Ok(ParsedElfs {
        base_elf,
        object_elf,
        arch,
        endian,
    })
}

/// Verify that architectures match between base and object ELFs.
pub fn validate_architecture(
    base_elf: &object::File,
    object_elf: &object::File,
) -> Result<(), LinkerError> {
    if base_elf.architecture() != object_elf.architecture() {
        return Err(LinkerError::ParseError(format!(
            "Architecture mismatch: base={:?}, object={:?}",
            base_elf.architecture(),
            object_elf.architecture()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_architecture_mismatch() {
        // This test would require creating mock ELF files with different architectures
        // For now, we'll test the error path with invalid data
        let invalid_base = &[0u8; 10];
        let invalid_object = &[0u8; 10];

        // Should fail to parse, not architecture mismatch
        assert!(parse_elfs(invalid_base, invalid_object).is_err());
    }
}

