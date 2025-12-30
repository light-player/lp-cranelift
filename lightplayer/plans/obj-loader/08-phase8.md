# Phase 8: Implement load_object_file Entry Point

## Goal
Implement `load_object_file()` entry point that orchestrates layout, section loading, symbol building, merging, and relocation.

## Changes Required

### 1. Implement `load_object_file()` in `mod.rs`
- Input: `obj_file_bytes: &[u8]`, `code: &mut Vec<u8>`, `ram: &mut Vec<u8>`, `symbol_map: &mut HashMap<String, u32>`
- Output: `Result<ObjectLoadInfo, String>`

### 2. Define `ObjectLoadInfo` struct
```rust
pub struct ObjectLoadInfo {
    pub main_address: Option<u32>,
    pub symbol_map: HashMap<String, u32>,
    pub text_start: u32,
    pub data_start: u32,
}
```

### 3. Orchestrate loading steps
1. Parse object file: `parse::parse_elf(obj_file_bytes)?`
2. Calculate layout: `calculate_object_layout(obj, base_code_end, base_ram_end)?`
3. Load sections: `load_object_sections(obj, code, ram, &layout)?`
4. Build object symbol map: `build_object_symbol_map(obj, layout.text_placement, layout.data_placement)`
5. Merge symbol maps: `merge_symbol_maps(base_symbol_map, obj_symbol_map)`
6. Apply relocations: `apply_object_relocations(obj, code, ram, &merged_map, &section_placement)?`
7. Update base symbol map: `*symbol_map = merged_map`
8. Return `ObjectLoadInfo`

### 4. Handle errors
- Return detailed error messages
- Clean up on error (if needed)

## Implementation Details

- Get `base_code_end` and `base_ram_end` from current buffer sizes
- Track section placement for relocation adjustment
- Update caller's symbol map with merged result

## Testing
- Test full object file loading workflow
- Test error handling (invalid object file, missing symbols, etc.)
- Test with multiple object files loaded sequentially
- Verify symbol map is updated correctly

## Success Criteria
- `load_object_file()` successfully loads an object file
- All steps execute in correct order
- Symbol map is updated correctly
- Return value contains correct info

