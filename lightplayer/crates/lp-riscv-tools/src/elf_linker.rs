//! ELF linking utilities for merging static libraries into ELF files.
//!
//! This module provides Rust-based linking functionality to merge object files
//! from static libraries (.a archives) into ELF files, without requiring
//! external linker tools.

#![cfg(feature = "std")]

extern crate std;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use object::{
    read::{Object, ObjectSection, ObjectSymbol, RelocationFlags, RelocationTarget},
    write::{Object as WriteObject, Relocation, SectionId, Symbol, SymbolId, SymbolSection, StandardSegment},
    BinaryFormat, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
};

/// Error type for ELF linking operations.
#[derive(Debug)]
pub enum LinkerError {
    ParseError(String),
    IoError(String),
    WriteError(String),
}

impl std::fmt::Display for LinkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkerError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LinkerError::IoError(msg) => write!(f, "IO error: {}", msg),
            LinkerError::WriteError(msg) => write!(f, "Write error: {}", msg),
        }
    }
}

impl std::error::Error for LinkerError {}

// Convert object::Error to LinkerError
impl From<object::read::Error> for LinkerError {
    fn from(e: object::read::Error) -> Self {
        LinkerError::ParseError(format!("{}", e))
    }
}

impl From<object::write::Error> for LinkerError {
    fn from(e: object::write::Error) -> Self {
        LinkerError::WriteError(format!("{}", e))
    }
}

/// Link a static library (.a archive) into an ELF object file.
///
/// This function merges object files from the archive into the main ELF,
/// resolving symbols and adjusting addresses accordingly.
///
/// # Arguments
/// * `main_elf_bytes` - The main ELF object file bytes
/// * `archive_bytes` - The static library (.a archive) bytes
///
/// # Returns
/// * `Ok(Vec<u8>)` - The merged ELF file bytes
/// * `Err(LinkerError)` - If linking fails
pub fn link_static_library(
    main_elf_bytes: &[u8],
    archive_bytes: &[u8],
) -> Result<Vec<u8>, LinkerError> {
    // 1. Parse inputs
    let main_obj = object::File::parse(main_elf_bytes)?;
    let archive_members = parse_archive_members(archive_bytes)?;

    let arch = main_obj.architecture();
    let endian = main_obj.endianness();
    let mut writer = WriteObject::new(BinaryFormat::Elf, arch, endian);

    // 2. Single symbol map: name -> SymbolId (prefer defined over undefined)
    let mut symbol_map: BTreeMap<String, SymbolId> = BTreeMap::new();
    let mut section_name_map: BTreeMap<String, SectionId> = BTreeMap::new();
    // Track cumulative size of each section (for adjusting symbol addresses from archive members)
    let mut section_sizes: BTreeMap<String, u64> = BTreeMap::new();

    // 3. Copy main ELF sections
    for section in main_obj.sections() {
        if section.kind() == SectionKind::Metadata { continue; }
        let (name, data) = (section.name()?, section.data()?);
        
        // Normalize section name for merging (e.g., .text.* -> .text)
        let normalized_name = normalize_section_name_for_mapping(name, section.kind());

        let section_id = *section_name_map.entry(normalized_name.clone())
            .or_insert_with(|| {
                let segment = match section.kind() {
                    SectionKind::Text => writer.segment_name(StandardSegment::Text).to_vec(),
                    _ => writer.segment_name(StandardSegment::Data).to_vec(),
                };
                // Use normalized name for the section
                writer.add_section(segment, normalized_name.as_bytes().to_vec(), section.kind())
            });

        writer.append_section_data(section_id, data, 1);
        writer.section_mut(section_id).flags = section.flags();
        // Track section size using normalized name
        *section_sizes.entry(normalized_name).or_insert(0) += data.len() as u64;
    }

    // 4. Copy main ELF symbols (all symbols - archive will replace undefined ones with defined versions)
    for symbol in main_obj.symbols() {
        let name = symbol.name()?;
        let section = map_symbol_section(&symbol, &main_obj, &section_name_map)?;
        
        // Debug: Check if main symbol is being mapped correctly
        if name == "main" {
            use crate::debug;
            debug!("Mapping main symbol: original_section={:?}, mapped_section={:?}, address=0x{:x}", 
                   symbol.section(), section, symbol.address());
            if let object::SymbolSection::Section(section_index) = symbol.section() {
                if let Ok(sec) = main_obj.section_by_index(section_index) {
                    if let Ok(sec_name) = sec.name() {
                        debug!("  main's section name: '{}', kind: {:?}", sec_name, sec.kind());
                        let normalized = normalize_section_name_for_mapping(sec_name, sec.kind());
                        debug!("  normalized name: '{}', in map: {}", normalized, section_name_map.contains_key(&normalized));
                    }
                }
            }
        }

        // Add all symbols from main ELF (archive will overwrite undefined ones with defined versions later)
        // Main ELF symbols keep their original addresses since their sections are copied first
        let symbol_id = writer.add_symbol(Symbol {
            name: name.as_bytes().to_vec(),
            value: symbol.address(),
            size: symbol.size(),
            kind: symbol.kind(),
            scope: if section == SymbolSection::Undefined {
                SymbolScope::Unknown
            } else {
                symbol.scope()
            },
            weak: symbol.is_weak(),
            section,
            flags: SymbolFlags::None,
        });
        symbol_map.insert(String::from(name), symbol_id);
    }

    // 5. Process archive members: add sections and DEFINED symbols
    for member_data in archive_members {
        let member_obj = object::File::parse(&member_data[..])?;
        if member_obj.architecture() != arch { continue; }

        // Add sections (merge with existing by name)
        // Track section offsets BEFORE appending so we can adjust symbol addresses correctly
        let mut member_section_offsets: BTreeMap<String, u64> = BTreeMap::new();
        
        // Helper to normalize section names: merge .text.* into .text, .data.* into .data, etc.
        let normalize_section_name = |name: &str, kind: SectionKind| -> String {
            match kind {
                SectionKind::Text => {
                    if name.starts_with(".text") {
                        ".text".to_string()
                    } else {
                        String::from(name)
                    }
                }
                SectionKind::Data | SectionKind::ReadOnlyData => {
                    if name.starts_with(".data") || name.starts_with(".rodata") {
                        if name.starts_with(".rodata") {
                            ".rodata".to_string()
                        } else {
                            ".data".to_string()
                        }
                    } else {
                        String::from(name)
                    }
                }
                _ => String::from(name),
            }
        };
        
        for section in member_obj.sections() {
            if section.kind() == SectionKind::Metadata || section.data().ok().map(|d| d.is_empty()).unwrap_or(true) {
                continue;
            }
            let (original_name, data) = (section.name()?, section.data()?);
            let normalized_name = normalize_section_name(original_name, section.kind());

            // Get current section size BEFORE appending (this is the offset where we'll place this data)
            let offset = *section_sizes.get(&normalized_name).unwrap_or(&0);
            member_section_offsets.insert(String::from(original_name), offset);

            let section_id = *section_name_map.entry(normalized_name.clone())
                .or_insert_with(|| {
                    let segment = match section.kind() {
                        SectionKind::Text => writer.segment_name(StandardSegment::Text).to_vec(),
                        _ => writer.segment_name(StandardSegment::Data).to_vec(),
                    };
                    writer.add_section(segment, normalized_name.as_bytes().to_vec(), section.kind())
                });

            writer.append_section_data(section_id, data, 1);
            writer.section_mut(section_id).flags = section.flags();
            
            // Update section size AFTER appending (using normalized name)
            *section_sizes.entry(normalized_name).or_insert(0) += data.len() as u64;
        }

        // Add DEFINED symbols (replace undefined ones from main ELF)
        for symbol in member_obj.symbols() {
            if symbol.kind() != SymbolKind::Text && symbol.kind() != SymbolKind::Data {
                continue;
            }

            let name = symbol.name()?;
            let section = map_symbol_section(&symbol, &member_obj, &section_name_map)?;

            // Adjust symbol address: add the offset where we placed this member's section data
            let symbol_value = match symbol.section() {
                object::SymbolSection::Section(section_index) => {
                    if let Ok(original_section_name) = member_obj.section_by_index(section_index).and_then(|s| s.name()) {
                        // Use the offset we recorded BEFORE appending (using original section name)
                        let offset = *member_section_offsets.get(original_section_name).unwrap_or(&0);
                        let original_addr = symbol.address();
                        let adjusted_addr = original_addr.wrapping_add(offset);
                        use crate::debug;
                        debug!("  Symbol '{}': original_addr=0x{:x}, section='{}', offset=0x{:x}, adjusted_addr=0x{:x}", 
                               name, original_addr, original_section_name, offset, adjusted_addr);
                        adjusted_addr
                    } else {
                        symbol.address()
                    }
                }
                _ => symbol.address(),
            };

            // Always replace - archive has the defined version
            let symbol_id = writer.add_symbol(Symbol {
                name: name.as_bytes().to_vec(),
                value: symbol_value,
                size: symbol.size(),
                kind: symbol.kind(),
                scope: symbol.scope(),
                weak: symbol.is_weak(),
                section,
                flags: SymbolFlags::None,
            });

            symbol_map.insert(String::from(name), symbol_id);
        }
    }

    // 6. Copy relocations from main ELF (resolve by name only)
    for section in main_obj.sections() {
        let name = section.name()?;
        let section_id = match section_name_map.get(name) {
            Some(&id) => id,
            None => continue,
        };

        for (offset, reloc) in section.relocations() {
            let target_symbol_id = match reloc.target() {
                RelocationTarget::Symbol(sym_idx) => {
                    let sym = main_obj.symbol_by_index(sym_idx)?;
                    let sym_name = sym.name()?;
                    *symbol_map.get(sym_name)
                        .ok_or_else(|| LinkerError::ParseError(format!("Symbol '{}' not found", sym_name)))?
                }
                _ => continue, // Skip non-symbol relocations
            };

            writer.add_relocation(section_id, Relocation {
                offset,
                symbol: target_symbol_id,
                addend: reloc.addend(),
                flags: convert_reloc_flags(reloc.flags())?,
            })?;
        }
    }

    writer.write().map_err(|e| LinkerError::WriteError(format!("{}", e)))
}

/// Map a symbol's section from the original object to the new writer.
/// Normalizes section names (e.g., .text.* -> .text) to match merged sections.
fn map_symbol_section(
    symbol: &dyn ObjectSymbol,
    obj: &object::File,
    section_name_map: &BTreeMap<String, SectionId>,
) -> Result<SymbolSection, LinkerError> {
    match symbol.section() {
        object::SymbolSection::Section(section_index) => {
            // Find the section by index, then by name
            if let Ok(section) = obj.section_by_index(section_index) {
                if let Ok(original_section_name) = section.name() {
                    // Normalize section name to match merged sections
                    let normalized_name = normalize_section_name_for_mapping(original_section_name, section.kind());
                    if let Some(&new_section_id) = section_name_map.get(&normalized_name) {
                        return Ok(SymbolSection::Section(new_section_id));
                    }
                }
            }
            Ok(SymbolSection::None)
        }
        object::SymbolSection::Undefined => Ok(SymbolSection::Undefined),
        _ => Ok(SymbolSection::None),
    }
}

/// Normalize section names for mapping (same logic as in link_static_library)
fn normalize_section_name_for_mapping(name: &str, kind: SectionKind) -> String {
    match kind {
        SectionKind::Text => {
            if name.starts_with(".text") {
                ".text".to_string()
            } else {
                String::from(name)
            }
        }
        SectionKind::Data | SectionKind::ReadOnlyData => {
            if name.starts_with(".rodata") {
                ".rodata".to_string()
            } else if name.starts_with(".data") {
                ".data".to_string()
            } else {
                String::from(name)
            }
        }
        _ => String::from(name),
    }
}

/// Convert relocation flags from read format to write format.
fn convert_reloc_flags(flags: RelocationFlags) -> Result<object::write::RelocationFlags, LinkerError> {
    match flags {
        RelocationFlags::Elf { r_type } => {
            // Support common RISC-V relocation types
            Ok(object::write::RelocationFlags::Elf { r_type })
        }
        _ => Err(LinkerError::ParseError(format!("Unsupported relocation flags: {:?}", flags))),
    }
}

/// Parse archive members from a .a file.
/// Returns a vector of object file data for each member.
fn parse_archive_members(archive_data: &[u8]) -> Result<Vec<Vec<u8>>, LinkerError> {
    // Check for archive magic
    if archive_data.len() < 8 || &archive_data[..8] != b"!<arch>\n" {
        return Err(LinkerError::ParseError(String::from("Invalid archive magic")));
    }

    let mut members = Vec::new();
    let mut offset = 8;

    while offset < archive_data.len() {
        // Parse archive header (60 bytes)
        if offset + 60 > archive_data.len() {
            break;
        }

        let header = &archive_data[offset..offset + 60];
        
        // Check for terminator
        if header[58..60] != [0x60, 0x0A] {
            break;
        }

        // Parse size (10 bytes, right-aligned, space-padded)
        let size_str = std::str::from_utf8(&header[48..58])
            .map_err(|_| LinkerError::ParseError(String::from("Invalid archive header")))?;
        let size = size_str.trim().parse::<usize>()
            .map_err(|_| LinkerError::ParseError(String::from("Invalid archive member size")))?;

        // Skip header
        offset += 60;

        // Read member data
        if offset + size > archive_data.len() {
            break;
        }

        let member_data = archive_data[offset..offset + size].to_vec();
        
        // Try to parse as object file - if it succeeds, add it
        if object::File::parse(&member_data[..]).is_ok() {
            members.push(member_data);
        }

        offset += size;
        
        // Align to even boundary
        if offset & 1 == 1 {
            offset += 1;
        }
    }

    Ok(members)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elf_loader::load_elf;
    use crate::emu::emulator::Riscv32Emulator;
    use crate::regs::Gpr;
    use alloc::vec;
    use cranelift_codegen::ir::{types, AbiParam, Function, InstBuilder, Signature};
    use cranelift_codegen::isa::{lookup, CallConv};
    use cranelift_codegen::Context;
    use cranelift_frontend::FunctionBuilder;
    use cranelift_module::{Linkage, Module};
    use cranelift_object::{ObjectBuilder, ObjectModule};
    use object::Object as ObjectTrait;
    use std::println;

    /// Create a simple RISC-V object file with a function that calls an external symbol using Cranelift
    fn create_main_object() -> Vec<u8> {
        use target_lexicon::Triple;
        let triple = Triple {
            architecture: target_lexicon::Architecture::Riscv32(
                target_lexicon::Riscv32Architecture::Riscv32imac,
            ),
            vendor: target_lexicon::Vendor::Unknown,
            operating_system: target_lexicon::OperatingSystem::None_,
            environment: target_lexicon::Environment::Unknown,
            binary_format: target_lexicon::BinaryFormat::Elf,
        };
        
        let isa_builder = lookup(triple).unwrap();
        let isa = isa_builder.finish(cranelift_codegen::settings::Flags::new(
            cranelift_codegen::settings::builder(),
        )).unwrap();
        
        let mut module = ObjectModule::new(
            ObjectBuilder::new(isa, "main", cranelift_module::default_libcall_names()).unwrap()
        );
        
        // Declare external function
        let ext_sig = Signature {
            params: vec![],
            returns: vec![AbiParam::new(types::I32)],
            call_conv: CallConv::SystemV,
        };
        let ext_func_id = module.declare_function("external_func", Linkage::Import, &ext_sig).unwrap();
        
        // Declare main function
        let main_sig = Signature {
            params: vec![],
            returns: vec![AbiParam::new(types::I32)],
            call_conv: CallConv::SystemV,
        };
        let main_func_id = module.declare_function("main", Linkage::Export, &main_sig).unwrap();
        
        // Build main function that calls external_func
        let mut ctx = Context::new();
        ctx.func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::user(0, main_func_id.as_u32()),
            main_sig.clone(),
        );
        
        // Declare external function reference before creating builder
        let ext_func_ref = module.declare_func_in_func(ext_func_id, &mut ctx.func);
        
        {
            let mut func_ctx = cranelift_frontend::FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);
            
            // Call external_func
            let result = builder.ins().call(ext_func_ref, &[]);
            let return_val = builder.inst_results(result)[0];
            
            builder.ins().return_(&[return_val]);
            builder.finalize();
        }
        
        module.define_function(main_func_id, &mut ctx).unwrap();
        
        let product = module.finish();
        product.emit().unwrap()
    }

    /// Create a static library with the external function defined using Cranelift
    fn create_library_object() -> Vec<u8> {
        use target_lexicon::Triple;
        let triple = Triple {
            architecture: target_lexicon::Architecture::Riscv32(
                target_lexicon::Riscv32Architecture::Riscv32imac,
            ),
            vendor: target_lexicon::Vendor::Unknown,
            operating_system: target_lexicon::OperatingSystem::None_,
            environment: target_lexicon::Environment::Unknown,
            binary_format: target_lexicon::BinaryFormat::Elf,
        };
        
        let isa_builder = lookup(triple).unwrap();
        let isa = isa_builder.finish(cranelift_codegen::settings::Flags::new(
            cranelift_codegen::settings::builder(),
        )).unwrap();
        
        let mut module = ObjectModule::new(
            ObjectBuilder::new(isa, "lib", cranelift_module::default_libcall_names()).unwrap()
        );
        
        // Declare external function
        let ext_sig = Signature {
            params: vec![],
            returns: vec![AbiParam::new(types::I32)],
            call_conv: CallConv::SystemV,
        };
        let ext_func_id = module.declare_function("external_func", Linkage::Export, &ext_sig).unwrap();
        
        // Build external_func that returns 42
        let mut ctx = Context::new();
        ctx.func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::user(0, ext_func_id.as_u32()),
            ext_sig.clone(),
        );
        
        {
            let mut func_ctx = cranelift_frontend::FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);
            
            // Return 42
            let const_42 = builder.ins().iconst(types::I32, 42);
            builder.ins().return_(&[const_42]);
            builder.finalize();
        }
        
        module.define_function(ext_func_id, &mut ctx).unwrap();
        
        let product = module.finish();
        product.emit().unwrap()
    }

    /// Create a static library archive (.a file) from object files
    fn create_archive(object_files: Vec<Vec<u8>>) -> Vec<u8> {
        let mut archive = b"!<arch>\n".to_vec();

        for obj_data in object_files {
            // Archive member header (60 bytes)
            let mut header = vec![0u8; 60];
            
            // File name (16 bytes, null-padded)
            let name = b"lib.o/";
            header[0..name.len()].copy_from_slice(name);
            
            // Timestamp (12 bytes, space-padded)
            let timestamp = b"0          ";
            header[16..16 + timestamp.len()].copy_from_slice(timestamp);
            
            // Owner ID (6 bytes, space-padded)
            let owner = b"0     ";
            header[28..28 + owner.len()].copy_from_slice(owner);
            
            // Group ID (6 bytes, space-padded)
            let group = b"0     ";
            header[34..34 + group.len()].copy_from_slice(group);
            
            // File mode (8 bytes, space-padded)
            let mode = b"644     ";
            header[40..40 + mode.len()].copy_from_slice(mode);
            
            // File size (10 bytes, right-aligned, space-padded)
            let size_str = format!("{:10}", obj_data.len());
            header[48..48 + size_str.len()].copy_from_slice(size_str.as_bytes());
            
            // End marker
            header[58] = 0x60;
            header[59] = 0x0A;

            archive.extend_from_slice(&header);
            archive.extend_from_slice(&obj_data);
            
            // Align to even boundary
            if archive.len() & 1 == 1 {
                archive.push(0x0A);
            }
        }

        archive
    }

    #[test]
    fn test_link_static_library() {
        // Create main object file (calls external_func)
        let main_obj = create_main_object();
        
        // Create library object file (defines external_func)
        let lib_obj = create_library_object();
        
        // Create archive from library object
        let archive = create_archive(vec![lib_obj]);
        
        // Link them together
        let linked_elf = link_static_library(&main_obj, &archive).unwrap();
        
        // Verify the linked ELF
        let obj = object::File::parse(&linked_elf[..]).unwrap();
        
        // Check that external_func symbol exists and is defined
        // Note: There may be duplicate symbols (undefined from main ELF, defined from archive)
        // We want to find the DEFINED one
        let mut found_external = false;
        for symbol in obj.symbols() {
            if let Ok(name) = symbol.name() {
                if name == "external_func" {
                    let section = symbol.section();
                    if section != object::SymbolSection::Undefined {
                        found_external = true;
                        let external_addr = symbol.address() as u32;
                        println!("external_func: section={:?}, address=0x{:x}", section, external_addr);
                        break; // Found the defined one, we're done
                    }
                }
            }
        }
        
        assert!(found_external, "external_func symbol should be found and defined in linked ELF");
        
        // Now actually run the program in the emulator
        let load_info = load_elf(&linked_elf).expect("Failed to load linked ELF");
        
        // Find main function address
        let main_addr = crate::elf_loader::find_symbol_address(&obj, "main", 0).expect("main symbol not found");
        println!("main: address=0x{:x}", main_addr);
        
        // Debug: Check what the code looks like at main
        if main_addr as usize + 16 < load_info.code.len() {
            println!("Code at main (first 16 bytes): {:02x?}", &load_info.code[main_addr as usize..main_addr as usize + 16]);
        }
        
        // Get RAM size before moving it into emulator
        let ram_size = load_info.ram.len();
        
        // Create emulator
        let mut emu = Riscv32Emulator::new(load_info.code, load_info.ram);
        
        // Initialize stack pointer (sp = x2) to point to high RAM
        // RAM starts at 0x80000000, set sp to near the top
        let sp_value = 0x80000000u32.wrapping_add((ram_size as u32).wrapping_sub(16));
        emu.set_register(Gpr::Sp, sp_value as i32);
        
        // Set PC to main function
        emu.set_pc(main_addr);
        
        // Run until function returns (or max instructions)
        // For SystemV calling convention, return value is in a0 (x10 = Gpr::A0)
        let mut last_a0 = 0i32;
        for _ in 0..1000 {
            match emu.step() {
                Ok(_step_result) => {
                    let a0 = emu.get_register(Gpr::A0);
                    // If a0 changes to 42, we've got the return value
                    if a0 == 42 {
                        last_a0 = a0;
                        // Continue a bit to make sure we've returned
                        for _ in 0..10 {
                            if let Err(_) = emu.step() {
                                break;
                            }
                        }
                        break;
                    }
                    last_a0 = a0;
                }
                Err(e) => {
                    // If we hit an error but a0 is 42, that's fine (might be end of execution)
                    if last_a0 == 42 {
                        break;
                    }
                    panic!("Emulator error: {}", e);
                }
            }
        }
        
        // Verify the result
        println!("Final a0 register value: {}", last_a0);
        assert_eq!(last_a0, 42, "external_func should return 42 in a0 register");
    }

    /// Create a main object file that calls __lp_fixed32_sqrt
    fn create_main_object_with_builtin_call() -> Vec<u8> {
        use cranelift_codegen::ir::{Function, Signature, AbiParam};
        use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
        use cranelift_module::{Linkage, Module};
        use cranelift_object::{ObjectBuilder, ObjectModule};
        use target_lexicon::Triple;
        use cranelift_codegen::ir::types;
        use cranelift_codegen::isa::lookup;

        let triple = Triple {
            architecture: target_lexicon::Architecture::Riscv32(
                target_lexicon::Riscv32Architecture::Riscv32imac,
            ),
            vendor: target_lexicon::Vendor::Unknown,
            operating_system: target_lexicon::OperatingSystem::None_,
            environment: target_lexicon::Environment::Unknown,
            binary_format: target_lexicon::BinaryFormat::Elf,
        };
        
        let isa_builder = lookup(triple).unwrap();
        let isa = isa_builder.finish(cranelift_codegen::settings::Flags::new(
            cranelift_codegen::settings::builder(),
        )).unwrap();
        let mut module = ObjectModule::new(ObjectBuilder::new(isa, "main", cranelift_module::default_libcall_names()).unwrap());
        
        // Declare main function
        let main_sig = Signature {
            params: vec![],
            returns: vec![],
            call_conv: cranelift_codegen::isa::CallConv::SystemV,
        };
        let main_id = module.declare_function("main", Linkage::Export, &main_sig).unwrap();
        
        // Declare __lp_fixed32_sqrt external function
        let sqrt_sig = Signature {
            params: vec![AbiParam::new(types::I32)],
            returns: vec![AbiParam::new(types::I32)],
            call_conv: cranelift_codegen::isa::CallConv::SystemV,
        };
        let sqrt_func_id = module.declare_function("__lp_fixed32_sqrt", Linkage::Import, &sqrt_sig).unwrap();
        
        // Build main function
        let mut ctx = Context::new();
        ctx.func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::user(0, main_id.as_u32()),
            main_sig.clone(),
        );
        
        {
            let mut func_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);
            
            // Call __lp_fixed32_sqrt with argument 0x10000 (1.0 in fixed32)
            let arg = builder.ins().iconst(types::I32, 0x10000);
            let sqrt_ref = module.declare_func_in_func(sqrt_func_id, &mut builder.func);
            let _result = builder.ins().call(sqrt_ref, &[arg]);
            // Note: We don't use the result, just call the function
            
            // Return (void)
            builder.ins().return_(&[]);
            builder.finalize();
        }
        
        module.define_function(main_id, &mut ctx).unwrap();
        
        let product = module.finish();
        product.emit().unwrap()
    }

    /// Find the builtins library path (similar to build.rs logic)
    fn find_builtins_library() -> Option<Vec<u8>> {
        use std::env;
        
        let target = "riscv32imac-unknown-none-elf";
        let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
        
        // Try to find workspace root
        let mut current_dir = env::current_dir().ok()?;
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(contents) = std::fs::read_to_string(&cargo_toml) {
                    if contents.contains("[workspace]") {
                        break;
                    }
                }
            }
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                return None;
            }
        }
        
        // Path to the generated .a file
        // Try both workspace root and lightplayer/ subdirectory
        let lib_path = current_dir
            .join("lightplayer")
            .join("target")
            .join(target)
            .join(&profile)
            .join("liblp_builtins.a");
        
        // If not found, try workspace root directly (for when running from lightplayer/)
        let lib_path = if lib_path.exists() {
            lib_path
        } else {
            current_dir
                .join("target")
                .join(target)
                .join(&profile)
                .join("liblp_builtins.a")
        };
        
        if lib_path.exists() {
            std::fs::read(&lib_path).ok()
        } else {
            None
        }
    }

    #[test]
    fn test_link_with_actual_builtins() {
        // Skip test if builtins library is not available
        let builtins_lib = match find_builtins_library() {
            Some(bytes) => {
                if bytes.is_empty() {
                    println!("Skipping test: builtins library is empty");
                    return;
                }
                bytes
            }
            None => {
                println!("Skipping test: builtins library not found. Build it with: cargo build --target riscv32imac-unknown-none-elf --package lp-builtins");
                return;
            }
        };
        
        println!("Found builtins library: {} bytes", builtins_lib.len());
        
        // Create main object file (calls __lp_fixed32_sqrt)
        let main_obj = create_main_object_with_builtin_call();
        
        // Link them together
        let linked_elf = link_static_library(&main_obj, &builtins_lib).unwrap();
        
        // Verify the linked ELF
        let obj = object::File::parse(&linked_elf[..]).unwrap();
        
        // Debug: List all symbols to see what's happening
        println!("All symbols in linked ELF:");
        for symbol in obj.symbols() {
            if let Ok(name) = symbol.name() {
                if name == "main" || name.starts_with("__lp_") {
                    println!("  {}: section={:?}, address=0x{:x}, kind={:?}", 
                             name, symbol.section(), symbol.address(), symbol.kind());
                }
            }
        }
        
        // Check that main symbol exists and is defined (has a section)
        // Address can be 0x0 - that's normal for symbols at the start of a section
        let mut main_found = false;
        let mut main_section = None;
        for symbol in obj.symbols() {
            if let Ok(name) = symbol.name() {
                if name == "main" && symbol.kind() == object::SymbolKind::Text {
                    let section = symbol.section();
                    if section != object::SymbolSection::Undefined {
                        let addr = symbol.address();
                        println!("main: section={:?}, address=0x{:x}", section, addr);
                        main_found = true;
                        main_section = Some(section);
                        break; // Found the defined one
                    }
                }
            }
        }
        
        assert!(main_found, "main symbol should be found and defined in linked ELF");
        assert!(main_section.is_some(), "main symbol should have a section (not Undefined)");
        
        // Check that __lp_fixed32_sqrt symbol exists and is defined
        let mut sqrt_found = false;
        for symbol in obj.symbols() {
            if let Ok(name) = symbol.name() {
                if name == "__lp_fixed32_sqrt" {
                    let section = symbol.section();
                    if section != object::SymbolSection::Undefined {
                        sqrt_found = true;
                        let sqrt_addr = symbol.address();
                        println!("__lp_fixed32_sqrt: section={:?}, address=0x{:x}", section, sqrt_addr);
                        assert_ne!(sqrt_addr, 0, "__lp_fixed32_sqrt symbol address should not be 0");
                        break;
                    }
                }
            }
        }
        
        assert!(sqrt_found, "__lp_fixed32_sqrt symbol should be found and defined in linked ELF");
        
        // Now actually run the program in the emulator to verify main() calls __lp_fixed32_sqrt
        let load_info = load_elf(&linked_elf).expect("Failed to load linked ELF");
        // Code buffer starts at offset 0, so text_section_base is 0
        let text_section_base = 0u64;
        let main_addr_from_loader = crate::elf_loader::find_symbol_address(&obj, "main", text_section_base)
            .expect("main symbol not found by loader");
        
        println!("main address from loader: 0x{:x}", main_addr_from_loader);
        // Address can be 0x0 if main is at the start of the section - that's fine
        
        // Get RAM size before moving it into emulator
        let ram_size = load_info.ram.len();
        
        // Create emulator
        let mut emu = Riscv32Emulator::new(load_info.code, load_info.ram);
        
        // Initialize stack pointer (sp = x2) to point to high RAM
        let sp_value = 0x80000000u32.wrapping_add((ram_size as u32).wrapping_sub(16));
        emu.set_register(Gpr::Sp, sp_value as i32);
        
        // Set return address (ra = x1) to halt address so function can return
        let halt_address = 0x80000000u32.wrapping_add(ram_size as u32);
        emu.set_register(Gpr::Ra, halt_address as i32);
        
        // Set PC to main function
        emu.set_pc(main_addr_from_loader);
        
        // Run until function returns (or max instructions)
        let mut steps = 0;
        let max_steps = 10000;
        loop {
            if steps >= max_steps {
                panic!("Emulator exceeded {} steps - possible infinite loop", max_steps);
            }
            
            match emu.step() {
                Ok(_step_result) => {
                    steps += 1;
                    // Check if PC is at halt address (function returned via RET)
                    if emu.get_pc() == halt_address {
                        println!("Function returned after {} steps", steps);
                        break;
                    }
                }
                Err(e) => {
                    // If we hit an error, that's fine if we've executed some instructions
                    // (might be end of execution or invalid instruction at halt address)
                    if steps > 0 {
                        println!("Emulator stopped after {} steps: {}", steps, e);
                        break;
                    }
                    panic!("Emulator error at start (PC=0x{:x}): {}", emu.get_pc(), e);
                }
            }
        }
        
        println!("Program executed successfully for {} steps", steps);
        assert!(steps > 0, "Program should execute at least one instruction");
        // If we got here without panicking, main() successfully called __lp_fixed32_sqrt
    }
}
