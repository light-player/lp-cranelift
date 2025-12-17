# ELF Relocation Refactor Plan

## Overview

Refactor the GLSL-to-emulator compilation pipeline to properly use ELF as the intermediate format. Currently, we generate ELF via `ObjectModule`, immediately parse it to extract raw binary, and manually apply relocations. This plan moves relocation handling to the ELF loader, following standard toolchain architecture: compiler → ELF object file → loader (applies relocations) → emulator.

## Current Architecture Problems

1. **Compiler does too much**: `compile_clif_to_binary()` generates ELF, parses it, extracts code, and manually applies relocations
2. **Heuristic relocation detection**: Currently detects RISC-V CALL_PLT by checking instruction opcodes instead of using relocation type
3. **Code duplication**: Relocation logic is duplicated from `CompiledBlob::perform_relocations` but implemented differently
4. **Unused infrastructure**: `elf_loader.rs` exists but doesn't apply relocations

## Target Architecture

```
ClifModule → ObjectModule → ELF bytes → ELF Loader → Raw binary (relocations applied) → Emulator
```

**Separation of concerns:**

- **Compiler**: Generate ELF object file (what `ObjectModule` is designed for)
- **Loader**: Parse ELF, extract sections, apply relocations using proper APIs
- **Emulator**: Execute raw binary

## Changes Required

### 1. Refactor `compile_clif_to_binary()` in `link.rs`

**File**: `lightplayer/crates/lp-glsl/src/compiler/link.rs`

**Current behavior**: Generates ELF, parses it, extracts code, applies relocations, returns `(Vec<u8>, u32)`

**New behavior**: Generate ELF and return it directly

**Changes**:

- Rename function to `compile_clif_to_elf()`
- Return `Result<Vec<u8>, GlslError>` (ELF bytes)
- Remove all ELF parsing and relocation application code (lines 798-939)
- Keep only ELF generation (lines 750-796)
- Optionally return symbol information if needed (e.g., main function name)

**Code to remove**:

- Lines 798-939: All ELF parsing, symbol mapping, and relocation application

**Code to keep**:

- Lines 750-796: ObjectModule creation, linking, and ELF emission

### 2. Enhance `elf_loader.rs` to Apply Relocations

**File**: `lightplayer/crates/lp-riscv-tools/src/elf_loader.rs`

**Current behavior**: Extracts code/data segments, does not apply relocations

**New behavior**: Extract segments AND apply relocations using proper relocation type detection

**Changes**:

#### 2.1 Add relocation application function

Create a helper function that applies relocations to code bytes:

```rust
/// Apply relocations to code bytes using the object crate's relocation API
fn apply_relocations(
    obj: &object::File,
    code_bytes: &mut [u8],
    text_section_id: object::SectionIndex,
    text_section_base: u64,
) -> Result<(), String> {
    // Build symbol map (name -> address relative to text section base)
    let mut symbol_map: HashMap<String, u32> = HashMap::new();
    for symbol in obj.symbols() {
        if symbol.kind() == object::SymbolKind::Text {
            if let Ok(name) = symbol.name() {
                let addr = symbol.address();
                if addr >= text_section_base {
                    let offset = (addr - text_section_base) as u32;
                    symbol_map.insert(name.to_string(), offset);
                }
            }
        }
    }

    // Find text section and apply relocations
    for section in obj.sections() {
        if section.index() == text_section_id {
            for (reloc_offset, reloc) in section.relocations() {
                apply_single_relocation(reloc, reloc_offset, code_bytes, &symbol_map, obj)?;
            }
            break;
        }
    }
    Ok(())
}
```

#### 2.2 Implement relocation type detection and application

Create `apply_single_relocation()` that:

- Uses `reloc.flags()` to detect relocation type (e.g., `R_RISCV_CALL_PLT`)
- Maps ELF relocation types to Cranelift `Reloc` enum equivalents
- Applies relocations using logic from `CompiledBlob::perform_relocations`

**Key relocation types to handle**:

- `R_RISCV_CALL_PLT` (object::elf::R_RISCV_CALL_PLT) → Use `CompiledBlob` logic for `Reloc::RiscvCallPlt`
- `R_RISCV_PCREL_HI20` → Use `Reloc::RiscvPCRelHi20` logic
- `R_RISCV_PCREL_LO12_I` → Use `Reloc::RiscvPCRelLo12I` logic
- Other types as needed

**Reference implementation**: `cranelift/jit/src/compiled_blob.rs` lines 126-161 for RISC-V CALL_PLT

#### 2.3 Update `load_elf()` function

**Current signature**: `pub fn load_elf(elf_data: &[u8]) -> Result<ElfLoadInfo, String>`

**Changes**:

- After extracting code segments (line 102), apply relocations before returning
- Call `apply_relocations()` with the parsed object file, mutable code bytes, section ID, and base address
- Ensure relocations are applied to the code bytes in-place

**New flow**:

1. Parse ELF (existing)
2. Extract code/data segments (existing)
3. **NEW**: Apply relocations to code bytes
4. Return `ElfLoadInfo` with relocated code

#### 2.4 Add symbol lookup helper

Add a function to find symbol addresses by name:

```rust
/// Find the address of a symbol by name (relative to text section base)
pub fn find_symbol_address(
    obj: &object::File,
    symbol_name: &str,
    text_section_base: u64,
) -> Result<u32, String> {
    // Implementation to find symbol and return offset
}
```

This can be used to find the `main` function address.

### 3. Update `link_glsl_for_emulator()` in `link.rs`

**File**: `lightplayer/crates/lp-glsl/src/compiler/link.rs`

**Current behavior**: Calls `compile_clif_to_binary()` which returns raw binary

**New behavior**: Call `compile_clif_to_elf()`, then use `load_elf()` from `lp-riscv-tools`

**Changes**:

- Replace `compile_clif_to_binary()` call with `compile_clif_to_elf()`
- Import `lp_riscv_tools::elf_loader::load_elf`
- Call `load_elf()` with ELF bytes
- Use `ElfLoadInfo` to get code and entry point
- Use `find_symbol_address()` or `ElfLoadInfo` to get main function address
- Pass code to `Riscv32Emulator::new()` instead of raw binary

**Code changes** (around line 689):

```rust
// OLD:
let (binary, main_address) = compile_clif_to_binary(&module)?;

// NEW:
let elf_bytes = compile_clif_to_elf(&module)?;
let load_info = lp_riscv_tools::elf_loader::load_elf(&elf_bytes)
    .map_err(|e| GlslError::new(ErrorCode::E0400, format!("ELF load failed: {}", e)))?;
let binary = load_info.code;
let main_address = find_symbol_address(...)?; // or use entry_point if main is entry
```

### 4. Update `ElfLoadInfo` Structure (if needed)

**File**: `lightplayer/crates/lp-riscv-tools/src/elf_loader.rs`

**Consider adding**:

- Symbol map for looking up function addresses
- Or a helper method to find symbols

**Current structure is probably sufficient**:

```rust
pub struct ElfLoadInfo {
    pub code: Vec<u8>,      // Relocated code
    pub ram: Vec<u8>,       // RAM data
    pub entry_point: u32,   // Entry point address
}
```

### 5. Handle Relocation Type Mapping

**Challenge**: Map from `object::RelocationFlags` (ELF relocation types) to Cranelift `Reloc` enum

**Approach**: Check `reloc.flags()` and match on ELF relocation type constants:

```rust
use object::elf;

match reloc.flags() {
    RelocationFlags::Elf { r_type } => {
        match r_type {
            elf::R_RISCV_CALL_PLT => {
                // Apply RiscvCallPlt logic
            }
            elf::R_RISCV_PCREL_HI20 => {
                // Apply RiscvPCRelHi20 logic
            }
            elf::R_RISCV_PCREL_LO12_I => {
                // Apply RiscvPCRelLo12I logic
            }
            _ => {
                // Handle other types or error
            }
        }
    }
    _ => {
        // Handle non-ELF formats (shouldn't happen for RISC-V)
    }
}
```

**Reference**: `cranelift/object/src/backend.rs` lines 908-917 shows how `Reloc::RiscvCallPlt` maps to `R_RISCV_CALL_PLT`

### 6. Testing Strategy

1. **Unit tests for relocation application**:

   - Test `apply_relocations()` with known ELF files
   - Verify RISC-V CALL_PLT relocations are applied correctly
   - Test symbol address resolution

2. **Integration test**:

   - Run existing filetests (e.g., `add.glsl`)
   - Verify `add_float(1.5, 2.5)` returns 4.0
   - Check that function calls work correctly

3. **Regression test**:
   - Ensure all existing filetests still pass
   - Verify emulator execution logs show correct instruction flow

## Implementation Order

1. **Phase 1**: Enhance `elf_loader.rs`

   - Add relocation application logic
   - Test with manually created ELF files if possible
   - Ensure proper relocation type detection

2. **Phase 2**: Refactor `compile_clif_to_binary()`

   - Rename to `compile_clif_to_elf()`
   - Remove parsing/relocation code
   - Return ELF bytes only

3. **Phase 3**: Update `link_glsl_for_emulator()`

   - Use new `compile_clif_to_elf()`
   - Use `load_elf()` from loader
   - Update to use `ElfLoadInfo`

4. **Phase 4**: Testing and validation
   - Run all filetests
   - Verify relocation application works
   - Check debug logs show correct execution

## Benefits

1. **Proper separation of concerns**: Compiler generates ELF, loader handles loading
2. **Standard architecture**: Follows standard toolchain design
3. **Reusable loader**: `elf_loader.rs` can be used for other ELF files
4. **Proper relocation handling**: Uses relocation type from ELF instead of heuristics
5. **Maintainability**: Relocation logic in one place (loader), not duplicated
6. **Extensibility**: Easy to add support for more relocation types

## Potential Issues and Solutions

### Issue: Relocation type mapping complexity

**Solution**: Start with RISC-V CALL_PLT, add others as needed. Reference `CompiledBlob` implementation.

### Issue: Symbol address resolution

**Solution**: Build symbol map in loader, provide helper function to look up addresses.

### Issue: Entry point vs main function

**Solution**: Use symbol lookup to find `main` function address, or use entry point if it's set to main.

### Issue: Backward compatibility

**Solution**: This is an internal refactor. External API (`glsl_emu_riscv32`) remains the same.

## Files to Modify

1. `lightplayer/crates/lp-glsl/src/compiler/link.rs`

   - Refactor `compile_clif_to_binary()` → `compile_clif_to_elf()`
   - Update `link_glsl_for_emulator()` to use ELF loader

2. `lightplayer/crates/lp-riscv-tools/src/elf_loader.rs`

   - Add `apply_relocations()` function
   - Add `apply_single_relocation()` helper
   - Add `find_symbol_address()` helper
   - Update `load_elf()` to apply relocations

3. `lightplayer/crates/lp-riscv-tools/src/lib.rs` (if needed)
   - Export new functions from `elf_loader` module

## Dependencies

- `object` crate: Already used, provides relocation API
- `cranelift-jit`: Reference implementation for relocation logic (read-only)

## Notes

- The `CompiledBlob::perform_relocations` logic in `cranelift/jit/src/compiled_blob.rs` is the reference for how to apply relocations correctly
- ELF relocation types are defined in `object::elf` module
- The current heuristic detection (checking for auipc/jalr opcodes) should be replaced with proper relocation type checking via `reloc.flags()`

