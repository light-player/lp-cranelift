//! ELF linking utilities for merging static libraries into ELF files.
//!
//! This module provides Rust-based linking functionality to merge object files
//! from static libraries (.a archives) into ELF files, without requiring
//! external linker tools.

#![cfg(feature = "std")]

extern crate std;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use object::{
    read::{Object, ObjectSection, ObjectSymbol},
    write::{Object as WriteObject, SectionId, Symbol, SymbolId, SymbolSection, StandardSegment},
    BinaryFormat, SectionKind, SymbolFlags, SymbolKind,
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
    // 1. Parse main ELF
    let main_obj = object::File::parse(main_elf_bytes)
        .map_err(|e| LinkerError::ParseError(format!("Failed to parse main ELF: {}", e)))?;

    let arch = main_obj.architecture();
    let endian = main_obj.endianness();

    // 2. Parse archive members
    let archive_members = parse_archive_members(archive_bytes)?;

    // 3. Create a new ELF writer
    let mut writer = WriteObject::new(
        BinaryFormat::Elf,
        arch,
        endian,
    );

    // 4. Copy all sections from the main ELF to the writer
    let mut section_map: BTreeMap<u32, SectionId> = BTreeMap::new();
    let mut symbol_map: BTreeMap<String, SymbolId> = BTreeMap::new();
    // Map section names to section IDs for merging sections with the same name
    let mut section_name_map: BTreeMap<String, SectionId> = BTreeMap::new();
    // Track cumulative offsets for each section (for symbol address adjustment)
    let mut section_offsets: BTreeMap<String, u64> = BTreeMap::new();

    // Copy sections from main ELF
    for section in main_obj.sections() {
        if let (Ok(name), Ok(data)) = (section.name(), section.data()) {
            let section_kind = section.kind();
            
            // Skip metadata sections that will be regenerated
            if section_kind == object::SectionKind::Metadata {
                continue;
            }

            let segment = match section_kind {
                SectionKind::Text => writer.segment_name(StandardSegment::Text).to_vec(),
                _ => writer.segment_name(StandardSegment::Data).to_vec(),
            };
            
            // Check if we already have a section with this name (for merging)
            let section_id = if let Some(&existing_id) = section_name_map.get(name) {
                existing_id
            } else {
                let new_id = writer.add_section(
                    segment,
                    name.as_bytes().to_vec(),
                    section_kind,
                );
                section_name_map.insert(String::from(name), new_id);
                section_offsets.insert(String::from(name), 0);
                new_id
            };

            // Set section flags and append data
            let section_mut = writer.section_mut(section_id);
            let flags = section.flags();
            section_mut.flags = flags;
            let current_offset = *section_offsets.get(name).unwrap_or(&0);
            writer.append_section_data(section_id, data, 1);
            // Update cumulative offset for this section
            section_offsets.insert(String::from(name), current_offset + data.len() as u64);

            // Map old section index to new section ID
            let index = section.index();
            section_map.insert(index.0 as u32, section_id);
        }
    }

    // Map old symbol indices to new symbol IDs for relocation copying
    let mut symbol_index_map: BTreeMap<u32, SymbolId> = BTreeMap::new();
    
    // Copy symbols from main ELF
    for (idx, symbol) in main_obj.symbols().enumerate() {
        if let Ok(name) = symbol.name() {
            let symbol_kind = symbol.kind();
            let symbol_scope = symbol.scope();
            
            // Copy all symbols (including undefined ones for relocations)
            let section = match symbol.section() {
                object::SymbolSection::Section(section_index) => {
                    let section_index_u32 = section_index.0 as u32;
                    if let Some(&new_section_id) = section_map.get(&section_index_u32) {
                        SymbolSection::Section(new_section_id)
                    } else {
                        // For undefined symbols, use Undefined section
                        if symbol.section() == object::SymbolSection::Undefined {
                            SymbolSection::Undefined
                        } else {
                            SymbolSection::None
                        }
                    }
                }
                object::SymbolSection::Undefined => SymbolSection::Undefined,
                _ => SymbolSection::None,
            };

            // For undefined symbols, ensure scope is Unknown (required by object crate)
            let final_scope = if symbol.section() == object::SymbolSection::Undefined {
                object::SymbolScope::Unknown
            } else {
                symbol_scope
            };

            let symbol_id = writer.add_symbol(Symbol {
                name: name.as_bytes().to_vec(),
                value: symbol.address(),
                size: symbol.size(),
                kind: symbol_kind,
                scope: final_scope,
                weak: symbol.is_weak(),
                section,
                flags: SymbolFlags::None,
            });

            symbol_index_map.insert(idx as u32, symbol_id);
            
            // Also add to name-based map for defined symbols
            if symbol_kind == SymbolKind::Text || symbol_kind == SymbolKind::Data {
                symbol_map.insert(String::from(name), symbol_id);
            }
        }
    }

    // 5. Process archive members and merge their sections/symbols
    for member_data in archive_members {

        // Parse the member as an ELF object file
        if let Ok(member_obj) = object::File::parse(&member_data[..]) {
            // Only process if architecture matches
            if member_obj.architecture() != arch {
                continue;
            }

            // Build section map for this member first (old index -> new section ID)
            let mut member_section_map: BTreeMap<u32, SectionId> = BTreeMap::new();
            
            // Build section map for this member first (old index -> new section ID)
            let mut member_section_map: BTreeMap<u32, SectionId> = BTreeMap::new();
            
            // Copy sections from archive member
            for section in member_obj.sections() {
                if let (Ok(name), Ok(data)) = (section.name(), section.data()) {
                    if data.is_empty() {
                        continue;
                    }

                    let section_kind = section.kind();
                    
                    // Skip metadata sections
                    if section_kind == object::SectionKind::Metadata {
                        continue;
                    }

                    // Find or create section in writer (merge sections with same name)
                    let segment = match section_kind {
                        SectionKind::Text => writer.segment_name(StandardSegment::Text).to_vec(),
                        _ => writer.segment_name(StandardSegment::Data).to_vec(),
                    };
                    
                    // Check if we already have a section with this name (for merging)
                    let section_id = if let Some(&existing_id) = section_name_map.get(name) {
                        existing_id
                    } else {
                        let new_id = writer.add_section(
                            segment,
                            name.as_bytes().to_vec(),
                            section_kind,
                        );
                        section_name_map.insert(String::from(name), new_id);
                        section_offsets.insert(String::from(name), 0);
                        new_id
                    };

                    // Map old section index to new section ID
                    let old_index = section.index().0 as u32;
                    member_section_map.insert(old_index, section_id);

                    // Get current offset for this section before appending
                    let section_offset = *section_offsets.get(name).unwrap_or(&0);
                    // Append data to section
                    writer.append_section_data(section_id, data, 1);
                    // Update cumulative offset for this section
                    section_offsets.insert(String::from(name), section_offset + data.len() as u64);
                    
                    // Update section flags
                    let flags = section.flags();
                    let section_mut = writer.section_mut(section_id);
                    section_mut.flags = flags;
                }
            }
            
            // Copy symbols from this archive member ONCE (not per section!)
            for symbol in member_obj.symbols() {
                if let Ok(sym_name) = symbol.name() {
                    // Add defined symbols, replacing undefined ones from main ELF
                    if symbol.kind() == SymbolKind::Text || symbol.kind() == SymbolKind::Data {
                                let sym_section = match symbol.section() {
                                    object::SymbolSection::Section(sym_section_index) => {
                                        let section_index_u32 = sym_section_index.0 as u32;
                                        if let Some(&new_section_id) = member_section_map.get(&section_index_u32) {
                                            SymbolSection::Section(new_section_id)
                                } else {
                                    // Fallback: try to find section by name
                                    if let Ok(section_name) = member_obj.section_by_index(sym_section_index).and_then(|s| s.name()) {
                                        if let Some(&new_section_id) = section_name_map.get(section_name) {
                                            SymbolSection::Section(new_section_id)
                                        } else {
                                            SymbolSection::None
                                        }
                                    } else {
                                        SymbolSection::None
                                    }
                                }
                            }
                            _ => SymbolSection::None,
                        };

                        // Find which section this symbol belongs to and get its offset
                        let symbol_value = match symbol.section() {
                            object::SymbolSection::Section(sym_section_index) => {
                                if let Ok(section_name) = member_obj.section_by_index(sym_section_index).and_then(|s| s.name()) {
                                    let section_offset = *section_offsets.get(section_name).unwrap_or(&0);
                                    symbol.address().wrapping_add(section_offset)
                                } else {
                                    symbol.address()
                                }
                            }
                            _ => symbol.address(),
                        };

                        let symbol_id = writer.add_symbol(Symbol {
                            name: sym_name.as_bytes().to_vec(),
                            value: symbol_value,
                            size: symbol.size(),
                            kind: symbol.kind(),
                            scope: symbol.scope(),
                            weak: symbol.is_weak(),
                            section: sym_section,
                            flags: SymbolFlags::None,
                        });

                        // Update symbol map (overwrites undefined symbols from main ELF)
                        symbol_map.insert(String::from(sym_name), symbol_id);
                    }
                }
            }
        }
    }

    // 6. Copy relocations from main ELF
    use crate::debug;
    use object::write::Relocation;
    use object::read::{RelocationFlags, RelocationTarget};
    
    debug!("=== Copying relocations from main ELF ===");
    for section in main_obj.sections() {
        if let Ok(name) = section.name() {
            // Find the corresponding section in the writer
            if let Some(&section_id) = section_name_map.get(name) {
                let mut reloc_count = 0;
                
                for (reloc_offset, reloc) in section.relocations() {
                    reloc_count += 1;
                    debug!("  Relocation at offset 0x{:x} in section '{}'", reloc_offset, name);
                    
                    // Get target symbol ID
                    // First try to find the symbol by name in symbol_map (which has the defined version from archive)
                    // If not found, fall back to symbol_index_map (which has the original from main ELF)
                    let target_symbol_id = match reloc.target() {
                        RelocationTarget::Symbol(sym_idx) => {
                            // Try to get symbol name first
                            if let Ok(sym) = main_obj.symbol_by_index(sym_idx) {
                                if let Ok(sym_name) = sym.name() {
                                    // Check if we have a defined version from archive
                                    if let Some(&defined_symbol_id) = symbol_map.get(sym_name) {
                                        debug!("    Target symbol '{}' -> defined symbol ID {:?} (from archive)", sym_name, defined_symbol_id);
                                        Some(defined_symbol_id)
                                    } else if let Some(&original_symbol_id) = symbol_index_map.get(&(sym_idx.0 as u32)) {
                                        debug!("    Target symbol '{}' -> original symbol ID {:?} (from main ELF)", sym_name, original_symbol_id);
                                        Some(original_symbol_id)
                                    } else {
                                        debug!("    Target symbol '{}' not found in any map", sym_name);
                                        None
                                    }
                                } else {
                                    // Fall back to index-based lookup
                                    if let Some(&new_symbol_id) = symbol_index_map.get(&(sym_idx.0 as u32)) {
                                        debug!("    Target symbol index {} -> symbol ID {:?}", sym_idx.0, new_symbol_id);
                                        Some(new_symbol_id)
                                    } else {
                                        debug!("    Target symbol index {} not found in symbol_index_map", sym_idx.0);
                                        None
                                    }
                                }
                            } else {
                                debug!("    Target symbol index {} invalid", sym_idx.0);
                                None
                            }
                        }
                        _ => {
                            debug!("    Non-symbol relocation target");
                            None
                        }
                    };
                    
                    if let Some(target_symbol_id) = target_symbol_id {
                        // Convert relocation flags
                        let flags = match reloc.flags() {
                            RelocationFlags::Elf { r_type } => {
                                match r_type {
                                    object::elf::R_RISCV_CALL_PLT => {
                                        object::write::RelocationFlags::Elf {
                                            r_type: object::elf::R_RISCV_CALL_PLT,
                                        }
                                    }
                                    object::elf::R_RISCV_PCREL_HI20 => {
                                        object::write::RelocationFlags::Elf {
                                            r_type: object::elf::R_RISCV_PCREL_HI20,
                                        }
                                    }
                                    object::elf::R_RISCV_PCREL_LO12_I => {
                                        object::write::RelocationFlags::Elf {
                                            r_type: object::elf::R_RISCV_PCREL_LO12_I,
                                        }
                                    }
                                    object::elf::R_RISCV_32 => {
                                        object::write::RelocationFlags::Elf {
                                            r_type: object::elf::R_RISCV_32,
                                        }
                                    }
                                    _ => {
                                        debug!("    Unsupported relocation type: {}", r_type);
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                debug!("    Unsupported relocation flags format");
                                continue;
                            }
                        };
                        
                        // Add relocation to writer
                        writer.add_relocation(
                            section_id,
                            Relocation {
                                offset: reloc_offset,
                                symbol: target_symbol_id,
                                addend: reloc.addend(),
                                flags,
                            },
                        ).map_err(|e| {
                            LinkerError::WriteError(format!("Failed to add relocation: {}", e))
                        })?;
                        
                        debug!("    Added relocation successfully");
                    }
                }
                
                if reloc_count > 0 {
                    debug!("  Copied {} relocations from section '{}'", reloc_count, name);
                }
            }
        }
    }

    // 7. Write the final ELF
    writer.write().map_err(|e| {
        LinkerError::WriteError(format!("Failed to write merged ELF: {}", e))
    })
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
    use crate::emu::Riscv32Emulator;
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
        let mut external_addr = 0u32;
        for symbol in obj.symbols() {
            if let Ok(name) = symbol.name() {
                if name == "external_func" {
                    let section = symbol.section();
                    if section != object::SymbolSection::Undefined {
                        found_external = true;
                        external_addr = symbol.address() as u32;
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
}
