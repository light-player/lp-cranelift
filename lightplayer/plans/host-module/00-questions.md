# Questions for Host Module Implementation

## Overview
We need to create a `host::` module that provides functions (like `debug!` and `println!`) that work differently depending on execution context:
- **Emulator**: Functions defined in `lp-builtins-app` (no_std, linked at build time)
- **Tests**: Functions defined in `lp-builtins` using `std` (for unit tests)
- **JIT**: Functions defined by `GlJitModule` (function pointers registered via `symbol_lookup_fn`)

## Questions - Answered

1. **Module Structure**: ✅ **Link-time resolution** - Functions declared in `lp-builtins`, linked differently per context (emulator/JIT/tests)

2. **Function Signatures**: ✅ **Follow `lp-builtins-app` pattern** - Macros use `core::format_args!`, functions take `fmt::Arguments`

3. **Macro vs Function**: ✅ **Both** - Macros are universal, underlying functions (`__host_debug`, `__host_println`) linked differently

4. **Registry Pattern**: ✅ **`HostId` enum** - Similar to `BuiltinId`, function names use `__host_` prefix

5. **Test Context**: ✅ **Use `std::println!` directly** - `__host_debug` checks `DEBUG=1` env var (like `lp-glsl`)

6. **JIT Integration**: ✅ **Delegate to `lp-glsl` macros** - `__host_println` calls `lp-glsl::println!`, `__host_debug` calls `lp-glsl::debug!`

7. **Initial Functions**: ✅ **Start with `debug` and `println`** - Can add `panic` later if needed

8. **No_std Compatibility**: ✅ **Feature flags** - Use `#[cfg(feature = "test")]` to gate `std`-dependent code in `lp-builtins`

