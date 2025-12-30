//! Lazy linking utilities for determining which archive members to include.
//!
//! This module implements lazy/eager linking: only including archive members
//! that define symbols needed by the main ELF or by already-included members.

#![cfg(feature = "std")]

extern crate std;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::debug;
use crate::elf_linker::LinkerError;
use object::{
    Architecture, SymbolKind,
    read::{Object, ObjectSection, ObjectSymbol, RelocationTarget},
};

/// Build a symbol index mapping symbol names to archive member indices.
///
/// Scans all archive members and builds a map of defined symbols to the
/// member index that defines them. Handles weak symbols by preferring
/// non-weak definitions.
///
/// # Arguments
/// * `archive_members` - Vector of archive member object file bytes
/// * `arch` - Target architecture (members with different arch are skipped)
///
/// # Returns
/// * `Ok(BTreeMap<String, usize>)` - Map of symbol name -> member index
/// * `Err(LinkerError)` - If parsing fails
pub fn build_symbol_index(
    archive_members: &[Vec<u8>],
    arch: Architecture,
) -> Result<BTreeMap<String, usize>, LinkerError> {
    let mut symbol_index: BTreeMap<String, usize> = BTreeMap::new();
    let mut weak_symbols: BTreeMap<String, usize> = BTreeMap::new();

    debug!("Building symbol index from {} archive members...", archive_members.len());

    for (member_idx, member_data) in archive_members.iter().enumerate() {
        let member_obj = match object::File::parse(&member_data[..]) {
            Ok(obj) => obj,
            Err(_) => continue, // Skip invalid object files
        };

        // Skip members with wrong architecture
        if member_obj.architecture() != arch {
            continue;
        }

        let mut member_symbol_count = 0;
        for symbol in member_obj.symbols() {
            // Only index defined symbols (Text or Data)
            if symbol.kind() != SymbolKind::Text && symbol.kind() != SymbolKind::Data {
                continue;
            }

            // Skip if symbol is undefined
            if symbol.is_undefined() {
                continue;
            }

            let name = match symbol.name() {
                Ok(n) => n,
                Err(_) => continue,
            };

            // Filter out local labels (starting with .L) and debug symbols
            if name.starts_with(".L") || name.starts_with(".debug") {
                continue;
            }

            // Handle weak symbols: prefer non-weak definitions
            if symbol.is_weak() {
                // Only add weak symbol if we don't already have a strong definition
                if !symbol_index.contains_key(name) {
                    weak_symbols.entry(String::from(name)).or_insert(member_idx);
                }
            } else {
                // Strong symbol: always add (overwrites weak if present)
                symbol_index.insert(String::from(name), member_idx);
                weak_symbols.remove(name); // Remove weak version if it existed
                member_symbol_count += 1;
            }
        }

        if member_symbol_count > 0 {
            debug!("  Member {}: indexed {} symbols", member_idx, member_symbol_count);
        }
    }

    // Add weak symbols that don't have strong definitions
    for (name, member_idx) in weak_symbols {
        if !symbol_index.contains_key(&name) {
            symbol_index.insert(name, member_idx);
        }
    }

    debug!("Indexed {} unique symbols across archive members", symbol_index.len());
    Ok(symbol_index)
}

/// Resolve which archive members are needed to satisfy undefined symbols.
///
/// Recursively resolves symbols: starts with initial undefined symbols,
/// finds members that define them, then resolves those members' dependencies,
/// repeating until all needed symbols are resolved.
///
/// # Arguments
/// * `archive_members` - Vector of archive member object file bytes
/// * `symbol_index` - Map of symbol name -> member index (from `build_symbol_index`)
/// * `initial_undefined` - Set of undefined symbols from the main ELF
/// * `arch` - Target architecture
///
/// # Returns
/// * `Ok(BTreeSet<usize>)` - Set of member indices to include
/// * `Err(LinkerError)` - If parsing fails
pub fn resolve_needed_members(
    archive_members: &[Vec<u8>],
    symbol_index: &BTreeMap<String, usize>,
    initial_undefined: &BTreeSet<String>,
    arch: Architecture,
) -> Result<BTreeSet<usize>, LinkerError> {
    let mut needed_symbols: BTreeSet<String> = initial_undefined.clone();
    let mut included_members: BTreeSet<usize> = BTreeSet::new();
    let mut resolved_symbols: BTreeSet<String> = BTreeSet::new();

    debug!("Starting recursive symbol resolution with {} initial undefined symbols", needed_symbols.len());
    if !needed_symbols.is_empty() {
        let symbol_list: Vec<String> = needed_symbols.iter().take(10).cloned().collect();
        debug!("  Initial symbols: {:?}{}", symbol_list, if needed_symbols.len() > 10 { "..." } else { "" });
    }

    loop {
        let mut progress = false;
        let mut new_symbols: BTreeSet<String> = BTreeSet::new();

        // Process each needed symbol
        for symbol_name in &needed_symbols {
            // Skip if already resolved
            if resolved_symbols.contains(symbol_name) {
                continue;
            }

            // Look up symbol in index
            if let Some(&member_idx) = symbol_index.get(symbol_name) {
                // Check if we've already included this member
                if !included_members.contains(&member_idx) {
                    debug!("Resolving symbol: {} -> including member {}", symbol_name, member_idx);
                    included_members.insert(member_idx);
                    progress = true;

                    // Parse this member to find what symbols it needs
                    let member_obj = match object::File::parse(&archive_members[member_idx][..]) {
                        Ok(obj) => obj,
                        Err(e) => {
                            return Err(LinkerError::ParseError(format!(
                                "Failed to parse archive member {}: {}",
                                member_idx, e
                            )));
                        }
                    };

                    // Skip if wrong architecture
                    if member_obj.architecture() != arch {
                        continue;
                    }

                    // Collect undefined symbols from this member's relocations
                    let mut member_needs: Vec<String> = Vec::new();
                    for section in member_obj.sections() {
                        for (_reloc_offset, reloc) in section.relocations() {
                            if let RelocationTarget::Symbol(sym_idx) = reloc.target() {
                                if let Ok(sym) = member_obj.symbol_by_index(sym_idx) {
                                    if sym.is_undefined() {
                                        if let Ok(sym_name) = sym.name() {
                                            // Filter out local labels and debug symbols
                                            if !sym_name.starts_with(".L")
                                                && !sym_name.starts_with(".debug")
                                            {
                                                member_needs.push(String::from(sym_name));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !member_needs.is_empty() {
                        debug!("  Member {} needs: {:?}", member_idx, member_needs);
                        for sym_name in member_needs {
                            new_symbols.insert(sym_name);
                        }
                    }
                }

                // Mark symbol as resolved
                resolved_symbols.insert(String::from(symbol_name));
            }
        }

        // Add new symbols to needed set
        needed_symbols.extend(new_symbols);

        // If no progress, we're done
        if !progress {
            break;
        }
    }

    debug!(
        "Lazy linking complete: included {}/{} members ({}% reduction)",
        included_members.len(),
        archive_members.len(),
        if archive_members.len() > 0 {
            100 - (included_members.len() * 100 / archive_members.len())
        } else {
            0
        }
    );

    Ok(included_members)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use cranelift_codegen::Context;
    use cranelift_codegen::ir::{AbiParam, Function, InstBuilder, Signature, types};
    use cranelift_codegen::isa::{CallConv, lookup};
    use cranelift_frontend::FunctionBuilder;
    use cranelift_module::{Linkage, Module};
    use cranelift_object::{ObjectBuilder, ObjectModule};
    use target_lexicon::Triple;

    /// Create a simple RISC-V object file with a function using Cranelift
    fn create_object_file(
        func_name: &str,
        external_calls: &[&str],
    ) -> Vec<u8> {
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
        let isa = isa_builder
            .finish(cranelift_codegen::settings::Flags::new(
                cranelift_codegen::settings::builder(),
            ))
            .unwrap();

        let mut module = ObjectModule::new(
            ObjectBuilder::new(isa, func_name, cranelift_module::default_libcall_names()).unwrap(),
        );

        // Create function signature
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::I32));
        sig.returns.push(AbiParam::new(types::I32));

        // Declare external functions
        for ext_name in external_calls {
            module
                .declare_function(ext_name, Linkage::Import, &sig)
                .unwrap();
        }

        // Create function
        let func_id = module
            .declare_function(func_name, Linkage::Export, &sig)
            .unwrap();

        let mut ctx = Context::new();
        ctx.func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32()),
            sig.clone(),
        );

        // Declare external function references before creating builder
        let mut ext_func_refs = Vec::new();
        for ext_name in external_calls {
            let ext_id = module.declare_function(ext_name, Linkage::Import, &sig).unwrap();
            ext_func_refs.push(module.declare_func_in_func(ext_id, &mut ctx.func));
        }

        {
            let mut func_ctx = cranelift_frontend::FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let arg = builder.block_params(entry_block)[0];

            // Call external functions if any
            if !ext_func_refs.is_empty() {
                let ext_func = ext_func_refs[0];
                let call_result = builder.ins().call(ext_func, &[arg]);
                let result_val = builder.inst_results(call_result)[0];
                builder.ins().return_(&[result_val]);
            } else {
                // If no external calls, just return the argument
                builder.ins().return_(&[arg]);
            }
            builder.finalize();
        }

        module.define_function(func_id, &mut ctx).unwrap();

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
    fn test_build_symbol_index() {
        // Create object files
        let obj1 = create_object_file("func1", &[]);
        let obj2 = create_object_file("func2", &[]);
        let obj3 = create_object_file("func3", &[]);

        // Create archive
        let archive = create_archive(vec![obj1, obj2, obj3]);
        let archive_members = crate::elf_linker::parse_archive_members(&archive).unwrap();

        // Build symbol index
        let index = build_symbol_index(&archive_members, Architecture::Riscv32).unwrap();

        // Verify all functions are indexed
        assert!(index.contains_key("func1"));
        assert!(index.contains_key("func2"));
        assert!(index.contains_key("func3"));

        // Verify member indices are correct
        assert_eq!(index.get("func1"), Some(&0));
        assert_eq!(index.get("func2"), Some(&1));
        assert_eq!(index.get("func3"), Some(&2));
    }

    #[test]
    fn test_resolve_needed_members() {
        // Create object files with dependencies:
        // - func1 calls func2
        // - func2 calls func3
        // - func3 has no dependencies
        let obj3 = create_object_file("func3", &[]);
        let obj2 = create_object_file("func2", &["func3"]);
        let obj1 = create_object_file("func1", &["func2"]);

        // Create archive
        let archive = create_archive(vec![obj1, obj2, obj3]);
        let archive_members = crate::elf_linker::parse_archive_members(&archive).unwrap();

        // Build symbol index
        let index = build_symbol_index(&archive_members, Architecture::Riscv32).unwrap();

        // Resolve starting from func1
        let mut initial_undefined = BTreeSet::new();
        initial_undefined.insert(String::from("func1"));

        let included = resolve_needed_members(&archive_members, &index, &initial_undefined, Architecture::Riscv32).unwrap();

        // Should include all three members (func1 -> func2 -> func3)
        assert_eq!(included.len(), 3);
        assert!(included.contains(&0)); // func1
        assert!(included.contains(&1)); // func2
        assert!(included.contains(&2)); // func3);
    }

    #[test]
    fn test_resolve_needed_members_partial() {
        // Create object files:
        // - func1 calls func2
        // - func2 has no dependencies
        // - func3 has no dependencies (unused)
        let obj3 = create_object_file("func3", &[]);
        let obj2 = create_object_file("func2", &[]);
        let obj1 = create_object_file("func1", &["func2"]);

        // Create archive
        let archive = create_archive(vec![obj1, obj2, obj3]);
        let archive_members = crate::elf_linker::parse_archive_members(&archive).unwrap();

        // Build symbol index
        let index = build_symbol_index(&archive_members, Architecture::Riscv32).unwrap();

        // Resolve starting from func1
        let mut initial_undefined = BTreeSet::new();
        initial_undefined.insert(String::from("func1"));

        let included = resolve_needed_members(&archive_members, &index, &initial_undefined, Architecture::Riscv32).unwrap();

        // Should only include func1 and func2, not func3
        assert_eq!(included.len(), 2);
        assert!(included.contains(&0)); // func1
        assert!(included.contains(&1)); // func2
        assert!(!included.contains(&2)); // func3 should NOT be included
    }

    #[test]
    fn test_circular_dependencies() {
        // Create object files with circular dependency:
        // - func1 calls func2
        // - func2 calls func1
        let obj2 = create_object_file("func2", &["func1"]);
        let obj1 = create_object_file("func1", &["func2"]);

        // Create archive
        let archive = create_archive(vec![obj1, obj2]);
        let archive_members = crate::elf_linker::parse_archive_members(&archive).unwrap();

        // Build symbol index
        let index = build_symbol_index(&archive_members, Architecture::Riscv32).unwrap();

        // Resolve starting from func1
        let mut initial_undefined = BTreeSet::new();
        initial_undefined.insert(String::from("func1"));

        let included = resolve_needed_members(&archive_members, &index, &initial_undefined, Architecture::Riscv32).unwrap();

        // Should include both members (circular dependency resolved)
        assert_eq!(included.len(), 2);
        assert!(included.contains(&0)); // func1
        assert!(included.contains(&1)); // func2
    }
}

