# Phase 1: Set up `lp-glsl-builtins-src` structure

## Goal

Create the module structure for `lp-glsl-builtins-src`, implement the `sqrt_recip` builtin, add `#[no_mangle]` wrappers, and figure out the testing approach.

## Steps

### 1.1 Update `Cargo.toml` for `#![no_std]` support

- Ensure `lp-glsl-builtins-src/Cargo.toml` supports `no_std`
- Add necessary dependencies (if any)
- Verify `#![no_std]` can be used

### 1.2 Create module structure

- Create `src/builtins/mod.rs` - Main builtins module
- Create `src/builtins/fixed32/mod.rs` - Fixed32 builtins module
- Create `src/builtins/fixed32/sqrt_recip.rs` - Square root reciprocal implementation

### 1.3 Port `sqrt_recip` from reference implementation

- Copy implementation from `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/reference/sqrt_recip.rs`
- Adapt to `no_std` environment
- Ensure it compiles with `#![no_std]`

### 1.4 Add `#[no_mangle]` wrapper

- Create wrapper function `__lp_fixed32_sqrt_recip` in `src/builtins/fixed32/mod.rs`
- Ensure function signature matches expected CLIF signature
- Add `pub extern "C"` attributes

### 1.5 Figure out testing approach

- Decide on unit test structure
- Consider how to test `no_std` code
- Plan for formal expectations format (to be defined in Phase 2)

## Files to Create/Modify

### New Files
- `lightplayer/crates/lp-glsl-builtins-src/src/builtins/mod.rs`
- `lightplayer/crates/lp-glsl-builtins-src/src/builtins/fixed32/mod.rs`
- `lightplayer/crates/lp-glsl-builtins-src/src/builtins/fixed32/sqrt_recip.rs`

### Modified Files
- `lightplayer/crates/lp-glsl-builtins-src/Cargo.toml` (if needed)
- `lightplayer/crates/lp-glsl-builtins-src/src/lib.rs` (if needed)

## Success Criteria

- `lp-glsl-builtins-src` compiles with `#![no_std]`
- `sqrt_recip` implementation exists and matches reference
- `__lp_fixed32_sqrt_recip` function is exported with `#[no_mangle]`
- Module structure is in place for future builtins

## Notes

- Reference implementation is in `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/reference/sqrt_recip.rs`
- Function name convention: `__lp_<category>_<function_name>`
- Testing format will be defined in Phase 2

