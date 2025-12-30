# Phase 2: Implement Macros

## Goal

Create `host::debug!` and `host::println!` macros that expand to calls to the underlying functions, following the pattern from `lp-builtins-app`.

## Tasks

1. Create `lp-builtins/src/host/macros.rs` with:
   - `debug!` macro that expands to `__host_debug(core::format_args!(...))`
   - `println!` macro that expands to `__host_println(core::format_args!(...))`

2. Export macros from `host` module:
   - Re-export macros in `mod.rs`
   - Make them available at crate root or via `host::debug!` syntax

3. Follow existing pattern:
   - Use `core::format_args!` for format string handling
   - Macros should work in no_std environments
   - Support all standard format specifiers

## Success Criteria

- Macros compile and expand correctly
- `host::debug!("test")` and `host::println!("test")` compile
- Macros work in no_std context
- Format strings work: `host::debug!("value: {:x}", 123)`

