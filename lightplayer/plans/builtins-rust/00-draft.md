# Builtin System Architecture - Rust-Based Implementation

## Overview

Create a three-crate system for managing builtin functions written in Rust:

- **`lp-glsl-builtins-src`**: Source Rust implementations of builtins
- **`lp-glsl-builtins-tool`**: Single binary tool for extracting CLIF, validating, transforming, and generating registry + CLIF files
- **`lp-glsl-builtins-loader`**: Crate providing proc_macro for loading binary CLIF at compile time

The tool (`lp-glsl-builtins-tool`) discovers builtins from the source crate, extracts CLIF using `rustc +nightly -Zcodegen-backend=cranelift`, validates/transforms them, and generates registry code + CLIF files. The loader crate provides a macro to load binary CLIF files at compile time.

## Architecture

```
lp-glsl-builtins-src/          Source implementations
  └── src/builtins/
      └── fixed32/
          └── sqrt_recip.rs

lp-glsl-builtins-tool/         Single binary tool
  └── src/
      ├── main.rs              CLI entry point
      ├── generator/           Extract, validate, transform CLIF
      └── generated/           Generated (committed to git)
          ├── registry.rs      Enum + registry
          └── clif/            Binary + text CLIF files
              ├── *.clif       Textual CLIF
              └── *.bclif      Binary CLIF (postcard)

lp-glsl-builtins-loader/       Binary CLIF loading crate
  └── src/
      ├── lib.rs               Macro definitions
      └── macro/               Proc macro implementation
          └── load_bclif.rs    Macro that reads .bclif and generates deserializer

lp-glsl/                       Compiler (consumes generated artifacts)
  └── src/backend/builtins/    Generated integration code (uses loader macro)
```

## Extraction Strategy

**Use Cranelift codegen backend**: Compile Rust source using `rustc +nightly -Zcodegen-backend=cranelift` (as demonstrated in `lp-glsl-builtins-src/build.sh`)

Process:

1. Compile `lp-glsl-builtins-src` crate with `rustc +nightly -Zcodegen-backend=cranelift`
2. CLIF files are generated in a `.clif` subdirectory
3. Parse CLIF files using `cranelift-reader::parse_functions()`
4. Extract functions matching `#[no_mangle]` symbols
5. Match discovered builtins to CLIF functions by name

## File Naming Convention

- Builtin function: `builtins/fixed32/sqrt_recip.rs`
- Function name: `__lp_fixed32_sqrt_recip`
- Enum variant: `Fixed32SqrtRecip` (derived from path)
- CLIF files: `__lp_fixed32_sqrt_recip.clif` and `__lp_fixed32_sqrt_recip.bclif`

## Implementation Phases

### Phase 1: Set up `lp-glsl-builtins-src` structure

- Update `Cargo.toml` for `#![no_std]` support
- Create `src/builtins/builtin.rs` with common utilities
- Implement `src/builtins/fixed32/sqrt_recip.rs` (port from reference)
- Update `src/builtins/fixed32/mod.rs` with `#[no_mangle]` wrapper
- Update `src/builtins/mod.rs` to export modules

### Phase 2: Create `lp-glsl-builtins-tool` crate foundation

- Create `Cargo.toml` with dependencies (cranelift-codegen, cranelift-reader, postcard, clap, walkdir)
- Create `src/main.rs` as single binary entry point
- Create `src/generator/mod.rs` module structure
- Create `src/generator/clif_format.rs` for binary serialization (postcard)
- Create `src/generator/validate.rs` for validation rules
- Create `src/generator/transform.rs` for transformations

### Phase 3: Implement CLIF extraction

- Create `src/generator/extract.rs` with:
  - `compile_to_clif()` - Run rustc with Cranelift backend
  - `parse_clif_file()` - Parse CLIF using cranelift-reader
  - `discover_builtins()` - Walk directory tree to find builtins
- Test extraction with `sqrt_recip` builtin

### Phase 4: Implement CLI tool

- Create `src/main.rs` with argument parsing (single binary)
- Implement main workflow:
  1. Discover builtins
  2. Compile to CLIF
  3. Extract and parse CLIF files
  4. Validate and transform functions
  5. Serialize to binary/text CLIF (generate both `.clif` and `.bclif`)
  6. Generate registry code
- Add `[[bin]]` section to `Cargo.toml`

### Phase 5: Generate registry code and binary CLIF

- Generate `src/generated/registry.rs` with:
  - `BuiltinId` enum (variants from file paths)
  - `BuiltinRegistry` struct (references to `.bclif` file paths)
- Generate both `.clif` (textual) and `.bclif` (binary postcard) files
- Generate `src/generated/mod.rs` to organize modules

### Phase 6: Create `lp-glsl-builtins-loader` crate

- Create `Cargo.toml` with proc_macro dependencies (syn, quote, proc_macro2)
- Create `src/lib.rs` that exports the macro
- Create `src/macro/load_bclif.rs` proc_macro that:
  - Takes a path to `.bclif` file
  - Reads file at compile time using `include_bytes!()`
  - Generates a function that deserializes `FunctionStencil` using `postcard::from_bytes()`
  - Handles errors gracefully

### Phase 7: Integration with `lp-glsl`

- Generate `lp-glsl/src/backend/builtins/mod.rs` and `registry.rs`
- Update generated code to use `load_bclif!()` macro from `lp-glsl-builtins-loader`
- Update `lp-glsl/src/backend/transform/fixed32/instructions.rs` to use `BuiltinId`
- Update `lp-glsl/src/backend/transform/fixed32/builtins.rs` to use registry
- Update `lp-glsl/src/backend/transform/fixed32/transform.rs` for new loading
- Update `lp-glsl/Cargo.toml` to depend on `lp-glsl-builtins-tool` (for generated code) and `lp-glsl-builtins-loader` (for macro)

### Phase 7: Testing and validation

- Test codegen tool end-to-end
- Test builtin loading in transform
- Run filetests with `sqrt` to verify correctness
- Remove old manual CLIF generation code

## Dependencies

- `lp-glsl-builtins` depends on `lp-glsl-builtins-src` (for source access)
- `lp-glsl` depends on `lp-glsl-builtins` (for generated artifacts)
- `lp-glsl-builtins` needs `cranelift-codegen` with `enable-serde` feature
- `lp-glsl-builtins` needs `bincode` or `postcard` for binary serialization
- `lp-glsl-builtins` needs `clap` for CLI
- `lp-glsl-builtins` needs `walkdir` for directory traversal

## Files to Create/Modify

### New Files

- `lp-glsl-builtins-src/src/builtins/builtin.rs`
- `lp-glsl-builtins-src/src/builtins/fixed32/sqrt_recip.rs` (implement)
- `lp-glsl-builtins/Cargo.toml`
- `lp-glsl-builtins/src/lib.rs`
- `lp-glsl-builtins/src/generator/mod.rs`
- `lp-glsl-builtins/src/generator/extract.rs`
- `lp-glsl-builtins/src/generator/validate.rs`
- `lp-glsl-builtins/src/generator/transform.rs`
- `lp-glsl-builtins/src/generator/clif_format.rs`
- `lp-glsl-builtins/src/cli/main.rs`
- `lp-glsl/src/backend/builtins/mod.rs` (generated)
- `lp-glsl/src/backend/builtins/registry.rs` (generated)

### Modified Files

- `lp-glsl-builtins-src/src/builtins/fixed32/mod.rs`
- `lp-glsl-builtins-src/src/builtins/mod.rs`
- `lp-glsl-builtins-src/Cargo.toml`
- `lp-glsl/src/backend/transform/fixed32/instructions.rs`
- `lp-glsl/src/backend/transform/fixed32/builtins.rs` (replace manual generation)
- `lp-glsl/src/backend/transform/fixed32/transform.rs`
- `lp-glsl/Cargo.toml`
