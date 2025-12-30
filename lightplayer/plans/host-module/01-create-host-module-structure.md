# Phase 1: Create Host Module Structure

## Goal

Create the `host` module in `lp-builtins` with function declarations and `HostId` enum, following the same pattern as builtins.

## Tasks

1. Create `lp-builtins/src/host/mod.rs` with:
   - `HostId` enum (similar to `BuiltinId`)
   - Function declarations for `__host_debug` and `__host_println`
   - Module structure

2. Create `lp-builtins/src/host/registry.rs` with:
   - `HostId` enum variants: `Debug`, `Println`
   - `name()` method returning symbol names (`__host_debug`, `__host_println`)
   - `signature()` method returning Cranelift signatures
   - `all()` method returning all host IDs

3. Add function declarations:
   - `pub extern "C" fn __host_debug(args: core::fmt::Arguments)`
   - `pub extern "C" fn __host_println(args: core::fmt::Arguments)`

4. Export from `lp-builtins/src/lib.rs`:
   - Add `pub mod host;`
   - Re-export `HostId` and function declarations

## Success Criteria

- `lp-builtins` compiles without errors
- `HostId` enum exists with `Debug` and `Println` variants
- Function declarations exist for both functions
- Signatures are `(fmt::Arguments) -> ()` (no return value)

