# Phase 3: Create `lp-glsl-builtins-tool` crate foundation

## Goal

Create the `lp-glsl-builtins-tool` crate with dependencies, generator modules (extract, validate, transform, clif_format), and CLI structure.

## Steps

### 3.1 Create crate structure

- Create `lightplayer/crates/lp-glsl-builtins-tool/` directory
- Create `Cargo.toml` with dependencies:
  - `cranelift-codegen` (with `enable-serde` feature)
  - `cranelift-reader`
  - `postcard`
  - `clap` (for CLI)
  - `walkdir` (for directory traversal)
  - `anyhow` (for error handling)
  - Other utilities as needed

### 3.2 Create module structure

- Create `src/main.rs` - Single binary entry point
- Create `src/generator/mod.rs` - Generator module
- Create `src/generator/extract.rs` - CLIF extraction logic (placeholder)
- Create `src/generator/validate.rs` - Validation rules (placeholder)
- Create `src/generator/transform.rs` - Transformations (placeholder)
- Create `src/generator/clif_format.rs` - CLIF formatting/serialization (placeholder)
- Create `src/generator/registry.rs` - Registry code generation (placeholder)

### 3.3 Set up CLI structure

- Add `[[bin]]` section to `Cargo.toml` pointing to `src/main.rs`
- Create basic CLI argument parsing structure (to be implemented in Phase 4)
- Plan for two commands: `generate-clif` and `generate-binaries`

### 3.4 Create placeholder implementations

- Add stub functions in each module
- Ensure crate compiles
- Add basic error types if needed

## Files to Create/Modify

### New Files
- `lightplayer/crates/lp-glsl-builtins-tool/Cargo.toml`
- `lightplayer/crates/lp-glsl-builtins-tool/src/main.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/mod.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/extract.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/validate.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/transform.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/clif_format.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/registry.rs`

### Modified Files
- `lightplayer/Cargo.toml` (add workspace member if needed)

## Success Criteria

- `lp-glsl-builtins-tool` crate exists and compiles
- All module files are created with placeholder code
- Dependencies are correctly specified
- CLI structure is in place (basic argument parsing)

## Notes

- This is a foundation phase - implementations come in later phases
- Keep modules focused on single responsibilities
- Error handling should be consistent across modules

