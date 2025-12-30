# Host Module Implementation Plan

## Overview

This plan implements a `host::` module that provides functions (like `debug!` and `println!`) that work differently depending on execution context:

- **Emulator**: Functions defined in `lp-builtins-app` (no_std, syscall-based)
- **Tests**: Functions defined in `lp-builtins` using `std` (for unit tests)
- **JIT**: Functions registered by `GlJitModule` (delegate to `lp-glsl` macros)

## Design Decisions

- **Link-time resolution**: Functions declared in `lp-builtins`, linked differently per context (similar to builtins pattern)
- **Macro-based API**: Macros (`host::debug!`, `host::println!`) are universal, underlying functions (`__host_debug`, `__host_println`) linked differently
- **Format strings**: Use `core::format_args!` pattern (same as `lp-builtins-app` `println!`)
- **Registry pattern**: `HostId` enum similar to `BuiltinId` for consistency
- **Function naming**: Use `__host_` prefix (e.g., `__host_debug`, `__host_println`)

## Implementation Strategy

1. Create `host` module in `lp-builtins` with function declarations
2. Implement macros that expand to function calls
3. Implement emulator versions (syscall-based) in `lp-builtins-app`
4. Implement test versions (std-based) in `lp-builtins` with feature flag
5. Register host functions in JIT mode (delegate to `lp-glsl` macros)
6. Register host functions in emulator mode (linker symbols)
7. Test all three contexts

## Success Criteria

- `host::debug!` and `host::println!` macros work in all three contexts
- Emulator: Output via syscalls to host
- JIT: Output via `lp-glsl` macros
- Tests: Output via `std::println!` (debug checks `DEBUG=1` env var)
- All code compiles without warnings
- All tests pass

