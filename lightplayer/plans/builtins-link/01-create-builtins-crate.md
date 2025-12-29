# Phase 1: Create `lp-builtins` crate structure

## Goal

Create the `lp-builtins` crate as a `no_std` library with `core` only, set up module structure with `fixed32/` submodule, and add it to the workspace.

## Steps

### 1.1 Create crate directory and Cargo.toml

- Create `lightplayer/crates/lp-builtins/` directory
- Create `Cargo.toml` with:
  - `no_std` configuration
  - `edition = "2021"`
  - `crate-type = ["lib"]`
  - No dependencies initially (core only)

### 1.2 Create module structure

- Create `src/lib.rs` - Main library entry point
- Create `src/fixed32/mod.rs` - Fixed32 module declaration
- Set up module exports

### 1.3 Add to workspace

- Add `"crates/lp-builtins"` to `lightplayer/Cargo.toml` workspace members

### 1.4 Verify crate compiles

- Run `cargo check` to ensure crate structure is correct

## Files to Create

- `lightplayer/crates/lp-builtins/Cargo.toml`
- `lightplayer/crates/lp-builtins/src/lib.rs`
- `lightplayer/crates/lp-builtins/src/fixed32/mod.rs`

## Files to Modify

- `lightplayer/Cargo.toml` (add workspace member)

## Success Criteria

- `lp-builtins` crate exists and compiles
- Module structure is set up (`fixed32/` submodule)
- Crate is added to workspace
- `cargo check` passes

## Notes

- Keep it minimal - no implementations yet, just structure
- `no_std` with `core` only (no `alloc`)
- Module structure should be ready for one-file-per-function approach

