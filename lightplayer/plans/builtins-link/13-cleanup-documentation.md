# Phase 13: Cleanup and documentation

## Goal

Remove old GLSL-based builtin system, remove 64-bit code generation from compiler, update documentation.

## Steps

### 13.1 Remove old builtin code

- Remove GLSL-based builtin implementations
- Remove old `Fixed32Builtin` enum if no longer needed
- Remove old builtin generation code
- Clean up unused imports and code

### 13.2 Remove 64-bit code generation

- Remove all i64/u64 operations from fixed32 arithmetic converters
- Remove `sextend` to i64 for multiplication
- Remove i64 division code
- Verify no 64-bit operations remain in fixed32 transform

### 13.3 Update documentation

- Document `lp-builtins` crate structure
- Document how to add new builtins
- Document linking process (both JIT and emulator)
- Update README files

### 13.4 Final verification

- Run all tests (unit, integration, filetests)
- Verify no regressions
- Check code coverage

## Files to Delete

- Old GLSL-based builtin files
- Old 64-bit arithmetic generation code (after migration)

## Files to Modify

- Documentation files
- README files
- Code comments

## Success Criteria

- Old code removed
- No 64-bit operations in fixed32 compiler code
- Documentation is up to date
- All tests pass

## Notes

- Keep git history for reference
- Document the migration for future reference
- Ensure cleanup doesn't break anything


