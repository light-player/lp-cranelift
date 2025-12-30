# Phase 9: Add __USER_MAIN_PTR Update Logic

## Goal
Add `__USER_MAIN_PTR` update logic in `load_object_file()` when a `main` symbol is found.

## Changes Required

### 1. Check for `main` symbol in `load_object_file()`
- After building object symbol map, check if `main` symbol exists
- If found, get its address from merged symbol map

### 2. Update `__USER_MAIN_PTR` in RAM
- Find `__USER_MAIN_PTR` symbol address in merged symbol map
- Write `main` address to that location in RAM buffer
- Handle case where `__USER_MAIN_PTR` doesn't exist (skip update)

### 3. Return `main_address` in `ObjectLoadInfo`
- Set `main_address: Some(addr)` if `main` found
- Set `main_address: None` if not found

## Implementation Details

- Check merged symbol map: `merged_map.get("main")`
- Get `__USER_MAIN_PTR` address: `merged_map.get("__USER_MAIN_PTR")`
- Write to RAM: `ram[offset..offset+4].copy_from_slice(&main_addr.to_le_bytes())`
- Handle alignment: ensure `__USER_MAIN_PTR` address is 4-byte aligned

## Testing
- Test with object file containing `main` symbol
- Test with object file without `main` symbol
- Test with multiple object files (verify last one wins)
- Verify `__USER_MAIN_PTR` is updated correctly in RAM
- Verify `main_address` is returned correctly

## Success Criteria
- `__USER_MAIN_PTR` is updated when `main` is found
- `main_address` is returned in `ObjectLoadInfo`
- Handles missing `main` or `__USER_MAIN_PTR` gracefully

