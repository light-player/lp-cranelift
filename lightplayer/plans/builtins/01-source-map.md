# Phase 1: Integrate GlSourceMap

**Goal**: Pass main `GlSourceMap` to intrinsic compilation and add intrinsic files to it for unified error reporting.

## Tasks

1. **Update `get_or_create_intrinsic()` signature**
   - Add `source_map: &mut GlSourceMap` parameter
   - Add `current_file_id: GlFileId` parameter (for context)
   - Update all call sites in `helpers.rs`

2. **Add intrinsic files to GlSourceMap**
   - When loading an intrinsic file (e.g., `trig.glsl`), add it to the main `GlSourceMap`
   - Use `GlFileSource::Intrinsic("trig")` to mark it as an intrinsic file
   - Store the file ID for use in compilation

3. **Update `compile_intrinsic_functions()` signature**
   - Change from creating its own `GlSourceMap` to accepting one
   - Accept `source_map: &GlSourceMap` and `file_id: GlFileId` parameters
   - Use the provided file ID instead of creating a synthetic one

4. **Update error reporting**
   - Ensure errors from intrinsic compilation use the correct file ID from `GlSourceMap`
   - Test that error messages show "error in trig.glsl: line X" format

5. **Update call sites**
   - Update `get_or_create_intrinsic()` to pass `source_map` and file ID to `compile_intrinsic_functions()`
   - Ensure `CodegenContext` has access to `source_map` (it already does)

## Files to Modify

- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/loader.rs`
- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/compiler.rs`
- `lightplayer/crates/lp-glsl/src/frontend/codegen/builtins/helpers.rs`

## Acceptance Criteria

**Test File**: `lightplayer/crates/lp-glsl-filetests/filetests/builtins/phase/01-source-map.glsl`

**Run Test**:
```bash
scripts/glsl-filetests.sh builtins/phase/01-source-map.glsl
```

The phase is complete when:
- [ ] Intrinsic files are added to main `GlSourceMap` when loaded
- [ ] Error messages show correct file context for intrinsic files
- [ ] No separate `GlSourceMap` created in `compile_intrinsic_functions()`
- [ ] Acceptance test `01-source-map.glsl` passes
- [ ] All existing tests still pass

