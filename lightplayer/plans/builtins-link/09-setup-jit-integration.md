# Phase 9: Set up JIT integration

## Goal

Add `lp-builtins` as dependency to `lp-glsl`, set up direct function calls in JIT code generation.

## Steps

### 9.1 Add dependency

- Add `lp-builtins` to `lp-glsl/Cargo.toml` dependencies
- Ensure it's available in JIT compilation context

### 9.2 Set up function linking

- In JIT module setup, declare `__lp_fixed32_*` functions as external
- Link them to host function pointers (direct references to `lp_builtins::*` functions)
- Use `module.declare_function()` and `module.define_function_linkage()`

### 9.3 Verify function availability

- Ensure functions are callable from JIT-generated code
- Test that JIT code can call builtin functions
- Verify results match expected values

### 9.4 Update function call generation

- Ensure `convert_fmul`, `convert_fdiv`, etc. generate correct function calls
- Verify function signatures match
- Test end-to-end: GLSL → JIT → builtin call → result

## Implementation Details

In JIT setup:
```rust
let sqrt_func = module.declare_function(
    "__lp_fixed32_sqrt",
    Linkage::Import,
    &signature
)?;
module.define_function_linkage(
    sqrt_func,
    Linkage::Import,
    lp_builtins::fixed32::sqrt::__lp_fixed32_sqrt as *const std::ffi::c_void
)?;
```

## Files to Modify

- `lightplayer/crates/lp-glsl/Cargo.toml` (add dependency)
- JIT module setup code (declare and link functions)
- Function call generation code

## Success Criteria

- `lp-builtins` is available as dependency in `lp-glsl`
- JIT code can call builtin functions
- Function calls work correctly in JIT execution
- Results match expected values

## Notes

- Functions are in same binary (JIT compiler), so direct function pointers work
- No need for dynamic loading or symbol resolution
- This is simpler than emulator case (no ELF loading needed)

