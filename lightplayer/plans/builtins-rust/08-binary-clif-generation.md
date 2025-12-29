# Phase 8: Binary CLIF generation

## Goal

Update `lp-glsl-builtins-tool` to serialize validated CLIF functions to binary using `postcard`, generating `.bclif` files alongside textual `.clif` files.

## Steps

### 8.1 Implement binary serialization

- Create serialization function in `src/generator/clif_format.rs`
- Extract `FunctionStencil` from `Function` (using `.stencil` field)
- Serialize `FunctionStencil` to binary using `postcard::to_allocvec()`
- Include `VersionMarker` in serialized data (for safety)

### 8.2 Generate `.bclif` files

- Write binary CLIF files alongside textual `.clif` files
- File naming: `__lp_fixed32_sqrt_recip.bclif` (same base name as `.clif`)
- Output locations:
  - `lp-glsl/src/backend/builtins/clif/` - For integration
  - (Filetests don't need binary - they use textual CLIF)

### 8.3 Integrate into `generate-binaries` command

- Update `generate-binaries` command to:
  - Read existing `.clif` files (from `generate-clif` step)
  - Parse CLIF functions
  - Serialize to binary
  - Write `.bclif` files
- Ensure command can run independently (doesn't require nightly)

### 8.4 Add validation for binary format

- Verify binary files can be deserialized
- Check version marker matches current Cranelift version
- Report errors if deserialization fails

### 8.5 Update `generate-clif` to also generate binaries

- Optionally generate `.bclif` files in `generate-clif` command
- Or keep them separate (user preference)
- Document the relationship between commands

## Files to Create/Modify

### New Files (Generated)
- `lightplayer/crates/lp-glsl/src/backend/builtins/clif/*.bclif` - Generated binary CLIF files

### Modified Files
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/clif_format.rs` - Add binary serialization
- `lightplayer/crates/lp-glsl-builtins-tool/src/main.rs` - Update `generate-binaries` command

## Success Criteria

- Binary CLIF files are generated using `postcard`
- `.bclif` files are written alongside `.clif` files
- Binary files can be deserialized successfully
- Version marker is included in serialized data
- `generate-binaries` command works without nightly Rust

## Notes

- Binary format uses `postcard` (as decided in Q1)
- Serialize `FunctionStencil`, not full `Function` (matches Cranelift's incremental cache pattern)
- Binary files are for runtime loading (via macro in Phase 9)
- Textual CLIF files remain for human inspection and filetests

