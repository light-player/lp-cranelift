# Phase 8: Remove main() test files

## Goal

Delete test files that specifically test main() requirements.

## Changes

1. **Delete main() test files**:
   - `lightplayer/crates/lp-glsl-filetests/filetests/function/main-entry.glsl`
   - `lightplayer/crates/lp-glsl-filetests/filetests/function/main-void.glsl`
   - `lightplayer/crates/lp-glsl-filetests/filetests/function/main-no-params.glsl`
   - `lightplayer/crates/lp-glsl-filetests/filetests/function2/main-entry.glsl`

2. **Verify no other references**:
   - Check for any other files that reference these tests
   - Update documentation if needed

## Files to Delete

- `lightplayer/crates/lp-glsl-filetests/filetests/function/main-entry.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/function/main-void.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/function/main-no-params.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/function2/main-entry.glsl`

## Success Criteria

- All main() test files deleted
- No broken references to deleted files
- Filetest discovery still works correctly

