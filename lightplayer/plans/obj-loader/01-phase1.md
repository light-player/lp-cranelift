# Phase 1: Extend ElfLoadInfo

## Goal
Extend `ElfLoadInfo` with `symbol_map`, `code_end`, and `ram_end` fields, and update `load_elf()` to populate them.

## Changes Required

### 1. Update `ElfLoadInfo` struct
- Add `pub symbol_map: HashMap<String, u32>` field
- Add `pub code_end: u32` field (where base executable's code sections end)
- Add `pub ram_end: u32` field (where base executable's RAM sections end)

### 2. Update `load_elf()` function
- Build symbol map (already done, but need to return it)
- Calculate `code_end`: maximum address of any code/ROM section
- Calculate `ram_end`: maximum offset in RAM buffer (relative to RAM start)
- Return these values in `ElfLoadInfo`

### 3. Update existing code
- Check if any code uses `ElfLoadInfo` and needs updates
- Ensure backward compatibility (old fields still work)

## Implementation Details

- `code_end`: Track maximum `section.address() + section.size()` for ROM sections
- `ram_end`: Track maximum RAM offset (relative to `DEFAULT_RAM_START`)
- `symbol_map`: Already built in `load_elf()`, just need to return it

## Testing
- Verify existing tests still pass
- Add test to verify `code_end` and `ram_end` are calculated correctly
- Add test to verify `symbol_map` contains expected symbols

## Success Criteria
- `ElfLoadInfo` has new fields populated correctly
- Existing code using `ElfLoadInfo` still works
- Tests pass

