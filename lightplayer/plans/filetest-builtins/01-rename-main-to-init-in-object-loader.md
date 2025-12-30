# Phase 1: Rename main to _init in object loader

## Goal

Update the object loader to look for "_init" symbol instead of "main", and update `__USER_MAIN_PTR` handling accordingly.

## Changes

1. **Update `load_object_file` in `object/mod.rs`**:
   - Change from looking for "main" symbol to "_init" symbol
   - Update `ObjectLoadInfo` to use `init_address` instead of `main_address`
   - Update comments to reflect "_init" naming

2. **Update `ObjectLoadInfo` struct**:
   - Rename `main_address` field to `init_address`
   - Update field documentation

3. **Update tests**:
   - Update test code that uses `main_address` to use `init_address`
   - Update test assertions and variable names

## Files to Modify

- `lightplayer/crates/lp-riscv-tools/src/elf_loader/object/mod.rs`
- `lightplayer/crates/lp-riscv-tools/src/elf_loader/object/mod.rs` (ObjectLoadInfo struct)
- `lightplayer/crates/lp-riscv-tools/src/elf_loader/object/tests.rs`

## Success Criteria

- Object loader looks for "_init" symbol instead of "main"
- `ObjectLoadInfo` uses `init_address` field
- All existing tests pass with updated naming
- Code compiles without warnings

