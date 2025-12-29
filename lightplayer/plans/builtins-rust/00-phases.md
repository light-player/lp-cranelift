# Builtin System Implementation Phases

## Phase List

1. **Set up `lp-glsl-builtins-src` structure** - Create module structure, implement `sqrt_recip` builtin, add `#[no_mangle]` wrappers, figure out testing approach.

2. **Define test expectations format** - Design format for test expectations (call args, expected results) that works for both unit tests and CLIF runtest transformation.

3. **Create `lp-glsl-builtins-tool` crate foundation** - Create crate with dependencies, generator modules (extract, validate, transform, clif_format), and CLI structure.

4. **Implement CLI tool commands and wrapper script** - Create `generate-clif` and `generate-binaries` commands with filetest-style error handling (basic structure, can be run incrementally). Create `scripts/build-builtins.sh` wrapper with workspace defaults. Errors should reference the wrapper script.

5. **Implement CLIF extraction** - Compile `lp-glsl-builtins-src` with Cranelift backend, parse CLIF files, extract `__lp_*` functions.

6. **Implement validation and transformation** - Create validation rules (one file per rule), implement panic-to-trap transformation.

7. **Generate registry code and filetests** - Generate `BuiltinId` enum and `BuiltinRegistry` with dependency detection. Generate textual CLIF to both `lp-glsl/src/backend/builtins/clif/` and `cranelift/filetests/filetests/32bit/builtins/` with formal expectations.

8. **Binary CLIF generation** - Update `lp-glsl-builtins-tool` to serialize validated CLIF functions to binary using `postcard`, generating `.bclif` files alongside textual `.clif` files.

9. **Create `lp-glsl-builtins-loader` crate** - New crate providing a proc_macro that takes a `.bclif` file path, reads it at compile time, and generates a function/constant for deserializing the `FunctionStencil` at runtime.

10. **Integration with `lp-glsl`** - Generate integration code to `lp-glsl/src/backend/builtins/`, update fixed32 transform to use new registry with macro-based binary CLIF loading.

11. **Build integration** - Add `build.rs` to `lp-glsl` to invoke `lp-glsl-builtins-tool` during build, ensure generated files are fresh.

12. **Testing and cleanup** - Test end-to-end, verify `sqrt` works, remove old manual CLIF generation code.
