# Phase 12: Migrate remaining builtins

## Goal

Move other builtins from GLSL-based system to `lp-builtins` crate.

## Steps

### 12.1 Identify remaining builtins

- List all builtins currently implemented in GLSL
- Determine which ones need migration
- Prioritize based on usage and complexity

### 12.2 Migrate builtins one by one

- For each builtin:
  - Create function in `lp-builtins` (appropriate module)
  - Implement using Rust stdlib where possible
  - Add unit tests if custom implementation
  - Update compiler to use new builtin
  - Remove old GLSL implementation

### 12.3 Update registry

- Add new builtins to `BuiltinId` enum
- Update registry generation
- Ensure all builtins are linked

### 12.4 Verify migration

- Run GLSL filetests to ensure nothing broke
- Verify no GLSL-based builtins remain
- Check that all builtins work in both JIT and emulator

## Files to Create/Modify

- New builtin function files in `lp-builtins`
- Compiler code using new builtins
- Registry updates

## Success Criteria

- All builtins migrated to `lp-builtins`
- No GLSL-based builtin implementations remain
- All filetests pass
- Both JIT and emulator work correctly

## Notes

- This is a gradual migration - can be done incrementally
- Keep old code until migration is complete and verified
- Focus on commonly used builtins first


