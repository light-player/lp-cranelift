# Phase 9: Create `lp-glsl-builtins-loader` crate

## Goal

New crate providing a proc_macro that takes a `.clif` file path, reads it at compile time, parses it, and generates a function for deserializing the `FunctionStencil` at runtime.

## Steps

### 9.1 Create crate structure

- Create `lightplayer/crates/lp-glsl-builtins-loader/` directory
- Create `Cargo.toml` with dependencies:
  - `proc_macro` (built-in)
  - `syn`, `quote`, `proc_macro2` (for macro implementation)
  - `cranelift-reader` (for parsing CLIF at compile time)
  - `cranelift-codegen` with `enable-serde` feature (for `FunctionStencil` type)
  - `postcard` (for serialization at compile time)

### 9.2 Create macro implementation

- Create `src/lib.rs` - Main crate file, exports macro
- Create `src/macro/load_bclif.rs` - Proc macro implementation
- Macro name: `include_bclif!()`
- Macro input: String literal path to `.clif` file

### 9.3 Implement macro logic

- Read `.clif` file using `include_str!()`-like resolution (relative to file location)
- Parse CLIF using `cranelift-reader::parse_functions()`
- Extract first function (or handle multiple functions)
- Extract `FunctionStencil` from `Function`
- Serialize `FunctionStencil` to binary using `postcard::to_allocvec()`
- Generate code that:
  - Embeds binary bytes as `const BYTES: &[u8]`
  - Provides `fn load() -> FunctionStencil` that deserializes from bytes

### 9.4 Implement error handling

- File not found: Use `proc_macro::Error` to produce compile-time error
- CLIF parse error: Use `proc_macro::Error` to produce compile-time error
- Deserialization: Runtime panic (unlikely if compile-time parsing succeeded)

### 9.5 Test macro

- Create test crate that uses the macro
- Verify macro expands correctly
- Verify generated code compiles
- Verify runtime deserialization works

## Files to Create/Modify

### New Files
- `lightplayer/crates/lp-glsl-builtins-loader/Cargo.toml`
- `lightplayer/crates/lp-glsl-builtins-loader/src/lib.rs`
- `lightplayer/crates/lp-glsl-builtins-loader/src/macro/mod.rs`
- `lightplayer/crates/lp-glsl-builtins-loader/src/macro/load_bclif.rs`

### Modified Files
- `lightplayer/Cargo.toml` (add workspace member if needed)

## Success Criteria

- `lp-glsl-builtins-loader` crate exists and compiles
- `include_bclif!()` macro is implemented
- Macro reads `.clif` file, parses, and serializes at compile time
- Generated code deserializes at runtime
- Compile-time errors are produced for file/parsing issues
- Macro works with generated registry code

## Notes

- Macro uses same Cranelift version as runtime (ensures compatibility)
- Files stored as `.clif` (textual), macro converts to binary
- Generated function returns `FunctionStencil` directly (panics on error)
- Path resolution matches `include_str!()` behavior

