# Phase 4: Implement Test Functions

## Goal

Add test implementations in `lp-builtins` that use `std::println!` and check `DEBUG=1` env var, gated behind a feature flag.

## Tasks

1. Add feature flag to `lp-builtins/Cargo.toml`:
   - `test` feature flag (or use existing `std` feature if present)

2. Create `lp-builtins/src/host/test.rs` with:
   - `__host_debug`: Check `DEBUG=1` env var, call `std::println!` if set
   - `__host_println`: Call `std::println!` directly
   - Both use `std::fmt::Arguments` (same as `core::fmt::Arguments`)

3. Gate implementations with `#[cfg(feature = "test")]`:
   - Only compile when test feature is enabled
   - Keep `lp-builtins` no_std by default

4. Export test implementations:
   - Make them available when feature is enabled
   - Can override default declarations

## Success Criteria

- Test implementations compile with `test` feature enabled
- `lp-builtins` remains no_std by default
- `__host_debug` checks `DEBUG=1` env var correctly
- `__host_println` works with `std::println!`
- Unit tests can use host functions

