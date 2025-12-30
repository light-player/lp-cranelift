# Phase 5: Register Host Functions in JIT

## Goal

Add host function registration in `GlJitModule` so JIT mode can call host functions, delegating to `lp-glsl` macros.

## Tasks

1. Create implementations in `lp-glsl/src/exec/jit.rs` or new module:
   - `__host_debug`: Call `lp_glsl::debug!` macro
   - `__host_println`: Call `std::println!` (or `lp_glsl` equivalent)
   - Both take `fmt::Arguments` and format them

2. Add to `symbol_lookup_fn` in `lp-glsl/src/backend/module/gl_module.rs`:
   - Register host functions alongside builtins
   - Return function pointers to JIT implementations

3. Update `HostId` registry in `lp-glsl`:
   - Add host function lookup to symbol lookup function
   - Match on `HostId` enum values

4. Ensure function signatures match:
   - JIT implementations must match declared signatures
   - Handle `fmt::Arguments` correctly

## Success Criteria

- Host functions registered in JIT mode
- `host::debug!` and `host::println!` work in JIT execution
- Output appears via `lp-glsl` debug/println macros
- No linker errors when using host functions in JIT

