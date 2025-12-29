# Phase 12: Testing and cleanup

## Goal

Test end-to-end, verify `sqrt` works, remove old manual CLIF generation code.

## Steps

### 12.1 Run end-to-end tests

- Run `cargo build` on `lp-glsl` (triggers build script)
- Verify all generated files are created
- Verify compilation succeeds
- Run existing GLSL filetests that use `sqrt`

### 12.2 Test `sqrt` functionality

- Run filetests that exercise `sqrt` function
- Verify results are correct
- Check for any regressions
- Test edge cases (0, 1, max values, etc.)

### 12.3 Run generated CLIF filetests

- Run filetests in `cranelift/filetests/filetests/32bit/builtins/`
- Verify interpreter and emulator tests pass
- Check that test expectations are correct

### 12.4 Remove old code

- Remove `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/builtins.rs` (old manual CLIF generation)
- Remove `Fixed32Builtin` enum references (if any remain)
- Remove old reference implementations (if no longer needed)
- Clean up any unused imports or code

### 12.5 Update documentation

- Update any documentation that references old system
- Document new builtin system architecture
- Document how to add new builtins
- Document build process

### 12.6 Final verification

- Run full test suite
- Verify no regressions
- Check that all phases are complete
- Ensure code is clean and well-organized

## Files to Create/Modify

### Deleted Files
- `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/builtins.rs` - Old manual CLIF generation

### Modified Files
- Documentation files (if any)
- Remove any remaining references to old system

## Success Criteria

- End-to-end build and test process works
- `sqrt` function works correctly with new builtin system
- Generated CLIF filetests pass
- Old manual CLIF generation code is removed
- No regressions introduced
- Code is clean and well-documented

## Notes

- This is the final phase - ensure everything works together
- Test thoroughly before considering complete
- Keep old reference implementations for now (may be useful for comparison)
- Document the new system for future developers

