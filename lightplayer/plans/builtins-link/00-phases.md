# Builtin System Implementation Phases (Linking Approach)

## Phase List

1. **Create `lp-builtins` crate structure** - Create `no_std` crate with `core` only, set up module structure (`fixed32/` submodule), add to workspace

2. **Implement `fixed32/div.rs`** - Implement `__lp_fixed32_div` using reciprocal multiplication (avoid 64-bit types in compiler)

3. **Implement `fixed32/mul.rs`** - Implement `__lp_fixed32_mul` handling overflow/saturation (avoid 64-bit types in compiler)

4. **Implement `fixed32/sqrt.rs`** - Implement `__lp_fixed32_sqrt` using Newton-Raphson (avoid 64-bit types in compiler)

5. **Add unit tests for fixed32 functions** - Add tests for div, mul, sqrt with various edge cases

6. **Set up ELF generation** - Create `build.rs` script to compile `lp-builtins` to static library (`.a`) for `riscv32imac-unknown-none-elf` target

7. **Set up static linking for emulator** - Configure emulator tests to statically link the `.a` file, verify linking works

8. **Update compiler to use builtins** - Replace 64-bit arithmetic generation in `convert_fmul` and `convert_fdiv` with calls to `__lp_fixed32_mul` and `__lp_fixed32_div`

9. **Set up JIT integration** - Add `lp-builtins` as dependency to `lp-glsl`, set up direct function calls in JIT code generation

10. **Create registry system** - Generate `BuiltinId` enum and registry for type-safe references and iteration for JIT linking

11. **Add integration tests** - Add sanity tests for linking (both emulator and JIT), verify functions are callable

12. **Migrate remaining builtins** - Move other builtins from GLSL-based system to `lp-builtins` crate

13. **Cleanup and documentation** - Remove old GLSL-based builtin system, remove 64-bit code generation from compiler, update documentation

