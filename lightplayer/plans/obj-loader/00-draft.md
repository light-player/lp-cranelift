# Object File Loader Plan

## Overview

Create an object file loader that loads PIC object files into the emulator after the base executable has been loaded. The object file can reference symbols from the base executable, and relocations will be resolved accordingly.

## Design Decisions

### 1. Symbol Resolution Strategy

**Decision**: Build a comprehensive symbol map from the base executable once, reuse it for all object file loads.

**Rationale**:
- More efficient than rebuilding the map each time
- Object files may reference any symbol from the base executable
- Supports loading multiple object files that all reference the base executable

**Implementation**:
- Build symbol map during base executable loading (already done in `elf_loader`)
- Pass this map to the object file loader
- Object file loader resolves symbols from this map when applying relocations

### 2. Memory Layout

**Decision**: Place object file sections sequentially after the base executable, grouped by section type.

**Rationale**:
- Simpler than trying to merge sections
- Keeps code and data separate (good for memory protection if we add it later)
- Easy to calculate addresses

**Layout**:
```
Base Executable:
  .text at 0x1000
  .rodata at 0x2000
  .data at 0x80000000

Object File (loaded after base):
  .text at (base_end + alignment)
  .rodata at (object_text_end + alignment)
  .data at (object_rodata_end + alignment, or in RAM after base .data)
```

**Implementation**:
- Calculate highest address used by base executable
- Align to 16 bytes
- Place object file sections sequentially, respecting alignment requirements

### 3. Data Section Handling

**Decision**: Handle object file's `.data` section like the base executable's `.data` section.

**Rationale**:
- Consistent behavior
- Object file's data needs to be in RAM for runtime access
- Can be placed after base executable's `.data` section in RAM

**Implementation**:
- Copy object file's `.data` section data into RAM at calculated VMA
- Apply relocations to the `.data` section
- Handle initialization similar to base executable

### 4. Relocation Application

**Decision**: Use existing relocation system, but extend symbol resolution to include base executable symbols.

**Rationale**:
- We already have a robust relocation system
- Just need to extend symbol lookup to check base executable's symbol map
- Handles all relocation types we already support

**Implementation**:
- Pass base executable's symbol map to relocation handlers
- When resolving a symbol:
  1. First check object file's own symbols
  2. Then check base executable's symbol map
  3. If not found, error (undefined symbol)

## Architecture

### New Module: `obj_loader`

```
obj_loader/
  mod.rs          - Main entry point, orchestrates loading
  layout.rs       - Calculate where to place object file sections
  sections.rs     - Load object file sections into memory
  symbols.rs      - Build symbol map for object file
  relocations.rs  - Apply relocations (extends existing system)
```

### Integration Points

1. **Base Executable Loading** (existing `elf_loader`):
   - Returns `ElfLoadInfo` with code, ram, entry_point
   - Also returns symbol map for object file loading

2. **Object File Loading** (new `obj_loader`):
   - Takes base executable's symbol map
   - Takes object file bytes
   - Returns load information (where sections were placed, entry point, etc.)

3. **Emulator Integration**:
   - Load base executable
   - Load object file(s)
   - Update `__USER_MAIN_PTR` to point to object file's `main` function

## Implementation Steps

### Step 1: Extract Symbol Map from Base Executable

**File**: `elf_loader/mod.rs`

**Changes**:
- Modify `load_elf` to return symbol map along with `ElfLoadInfo`
- Or create a separate function to get symbol map from loaded executable

**API**:
```rust
pub struct ElfLoadInfo {
    pub code: Vec<u8>,
    pub ram: Vec<u8>,
    pub entry_point: u32,
    pub symbol_map: HashMap<String, u32>, // NEW
}
```

### Step 2: Create Object File Loader Module

**File**: `obj_loader/mod.rs`

**Function**:
```rust
pub fn load_object_file(
    obj_file_bytes: &[u8],
    base_symbol_map: &HashMap<String, u32>,
    base_code_end: u32,  // Where base executable's code ends
    base_ram_end: u32,   // Where base executable's RAM ends
    code: &mut [u8],     // ROM buffer (will be extended if needed)
    ram: &mut [u8],      // RAM buffer (will be extended if needed)
) -> Result<ObjectLoadInfo, String>
```

**Returns**:
```rust
pub struct ObjectLoadInfo {
    pub text_start: u32,
    pub data_start: u32,
    pub main_address: Option<u32>, // Address of 'main' function if found
    pub symbol_map: HashMap<String, u32>, // Object file's symbols
}
```

### Step 3: Calculate Object File Layout

**File**: `obj_loader/layout.rs`

**Function**:
```rust
pub fn calculate_object_layout(
    obj: &object::File,
    base_code_end: u32,
    base_ram_end: u32,
) -> Result<ObjectLayout, String>
```

**Returns**:
```rust
pub struct ObjectLayout {
    pub text_start: u32,
    pub rodata_start: u32,
    pub data_start: u32,
    pub section_addresses: HashMap<String, u32>,
}
```

**Logic**:
- Start `.text` at `(base_code_end + 15) & !15` (16-byte aligned)
- Place `.rodata` after `.text` (aligned)
- Place `.data` in RAM after base `.data` (aligned)

### Step 4: Load Object File Sections

**File**: `obj_loader/sections.rs`

**Function**:
```rust
pub fn load_object_sections(
    obj: &object::File,
    layout: &ObjectLayout,
    code: &mut Vec<u8>,  // Extendable ROM buffer
    ram: &mut Vec<u8>,   // Extendable RAM buffer
) -> Result<(), String>
```

**Logic**:
- Extend code/ram buffers if needed
- Copy section data to calculated addresses
- Handle alignment requirements

### Step 5: Build Object File Symbol Map

**File**: `obj_loader/symbols.rs`

**Function**:
```rust
pub fn build_object_symbol_map(
    obj: &object::File,
    layout: &ObjectLayout,
) -> HashMap<String, u32>
```

**Logic**:
- Parse object file's symbol table
- Adjust symbol addresses based on where sections were loaded
- Return map of symbol name -> address

### Step 6: Apply Object File Relocations

**File**: `obj_loader/relocations.rs`

**Function**:
```rust
pub fn apply_object_relocations(
    obj: &object::File,
    base_symbol_map: &HashMap<String, u32>,
    object_symbol_map: &HashMap<String, u32>,
    layout: &ObjectLayout,
    code: &mut [u8],
    ram: &mut [u8],
) -> Result<(), String>
```

**Logic**:
- Use existing relocation handlers from `elf_loader/relocations/handlers.rs`
- Extend symbol resolution to check both maps:
  1. Check object_symbol_map first (local symbols)
  2. Check base_symbol_map second (external symbols)
- Apply relocations using existing handlers

### Step 7: Integration

**File**: `elf_loader/mod.rs` or new integration module

**Function**:
```rust
pub fn load_base_and_object(
    base_elf_bytes: &[u8],
    object_elf_bytes: &[u8],
) -> Result<CombinedLoadInfo, String>
```

**Returns**:
```rust
pub struct CombinedLoadInfo {
    pub code: Vec<u8>,
    pub ram: Vec<u8>,
    pub base_entry_point: u32,
    pub object_main_address: Option<u32>,
}
```

## Key Considerations

### Memory Buffer Management

**Challenge**: Object file may need more ROM/RAM than initially allocated.

**Solution**: Use `Vec<u8>` instead of `&mut [u8]` so we can extend buffers.

**Alternative**: Pre-calculate total size needed, allocate once.

### Symbol Conflicts

**Scenario**: Object file defines a symbol with the same name as base executable.

**Decision**: Object file's symbol takes precedence for object file's own relocations, but base executable's symbol is still available.

**Implementation**: Check object_symbol_map first, then base_symbol_map.

### GOT Entries

**Consideration**: Object file may have GOT entries that reference base executable symbols.

**Solution**: Existing GOT handling should work, just need to resolve symbols from base_symbol_map.

### Entry Point

**Question**: What should the entry point be?

**Answer**: 
- Base executable's entry point is the main entry point
- Object file's `main` function address is stored in `__USER_MAIN_PTR`
- Base executable calls the function pointer when ready

## Testing Strategy

### Unit Tests

1. **Layout Calculation**:
   - Test address calculation for various base executable sizes
   - Test alignment requirements

2. **Symbol Resolution**:
   - Test resolving symbols from base executable
   - Test resolving local symbols from object file
   - Test undefined symbol errors

3. **Relocation Application**:
   - Test various relocation types (CALL_PLT, GOT_HI20, PCREL_HI20, etc.)
   - Test relocations that reference base executable symbols

### Integration Tests

1. **End-to-End Loading**:
   - Load base executable
   - Load object file
   - Verify object file's `main` function can be called
   - Verify object file can call base executable functions (e.g., `__lp_fixed32_sqrt`)

2. **Multiple Object Files**:
   - Load base executable
   - Load first object file
   - Load second object file
   - Verify both work correctly

## Migration Path

1. **Phase 1**: Implement basic object file loader
   - Load sections
   - Apply relocations
   - Resolve symbols from base executable

2. **Phase 2**: Integration
   - Update emulator integration to use object loader
   - Update `__USER_MAIN_PTR` handling

3. **Phase 3**: Remove old linker
   - Once object loader is working, remove `executable_linker` module
   - Simplify codebase

## Success Criteria

1. ✅ Object file loads successfully after base executable
2. ✅ Object file's relocations resolve correctly
3. ✅ Object file can call functions in base executable
4. ✅ Object file's `main` function address is correctly set in `__USER_MAIN_PTR`
5. ✅ Multiple object files can be loaded sequentially
6. ✅ Memory layout is correct (no overlaps, proper alignment)

## Open Questions

1. **Multiple Object Files**: Should we support loading multiple object files? If so, how do we handle symbol conflicts between object files?

2. **Unloading**: Do we need to support unloading object files, or is it load-once?

3. **Dynamic Linking**: Should object files be able to reference other object files, or only the base executable?

4. **Error Handling**: What happens if object file references an undefined symbol? Should we fail fast or allow lazy resolution?

## Notes

- This approach is much simpler than full ELF linking
- Reuses existing relocation handling code
- No need to manipulate ELF structures directly
- Works with PIC code, which is what Cranelift generates
- Can be extended to support multiple object files later

