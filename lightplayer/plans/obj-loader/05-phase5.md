# Phase 5: Implement Object Symbol Map Building

## Goal
Implement `build_object_symbol_map()` to build a symbol map with addresses adjusted for section placement.

## Changes Required

### 1. Implement `build_object_symbol_map()` in `symbols.rs`
- Input: `obj: &object::File`, `text_placement: u32`, `data_placement: u32`
- Output: `HashMap<String, u32>`

### 2. Iterate object file symbols
- Use `obj.symbols()` to iterate all symbols
- Filter out unnamed symbols
- Determine which section each symbol belongs to

### 3. Adjust symbol addresses
- For `.text` section symbols: `final_addr = text_placement + symbol_offset`
- For `.data` section symbols: `final_addr = data_placement + symbol_offset`
- Handle undefined symbols (keep as-is, will be resolved via merge)

### 4. Build map
- Similar structure to `build_symbol_map()` in `elf_loader/symbols.rs`
- Prefer defined symbols over undefined
- Handle duplicates (keep first or higher address)

## Implementation Details

- Use `symbol.section()` to determine section
- Use `symbol.address()` to get section-relative offset
- Use `section.name()` to match against `.text`/`.data`
- For RAM symbols: `data_placement` is relative to RAM start, so final address is `DEFAULT_RAM_START + data_placement + offset`

## Testing
- Test symbol map building with `.text` symbols
- Test symbol map building with `.data` symbols
- Test undefined symbols (should be included)
- Verify addresses are adjusted correctly
- Compare with expected addresses

## Success Criteria
- Symbol map contains all object file symbols
- Addresses are correctly adjusted for section placement
- Undefined symbols are included

