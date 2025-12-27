# GLSL to Rust Converter for Blessing Filetests

## Problem

We need to validate expectations in GLSL test files by converting them to Rust, executing them, and updating expectations if they differ. This is needed for the rewritten function tests in `function2/` directory.

## Solution

Create a GLSL to Rust converter using the existing `glsl-parser` infrastructure, then build a bless tool that uses it to validate and update test expectations.

## Implementation

### Phase 1: Create GLSL to Rust Transpiler

Create a transpiler module in `lp-glsl-filetests` that:
- Uses `glsl-parser` AST (already a dependency via `lp-glsl`)
- Converts GLSL AST nodes to Rust code
- Handles basic GLSL features needed for test validation:
  - Scalar types (`float`, `int`, `uint`, `bool`)
  - Vector types (`vec2`, `vec3`, `vec4`, etc.)
  - Matrix types (`mat2`, `mat3`, `mat4`)
  - Function definitions and calls
  - Basic expressions and statements
  - Control flow (if/else, loops)
- **Fails fast** if it encounters unsupported features

### Phase 2: Create Bless Binary

Create `bless-filetest` binary in `lp-glsl-filetests` that:
- Parses GLSL test files
- Extracts function definitions and `// run:` directives
- Converts GLSL to Rust using the transpiler
- Compiles and executes the Rust code
- Compares results with expectations
- Updates expectations in-place if they differ
- Skips tests with `inout`/`out` parameters (manual validation needed)
- Skips error tests (compile-time errors, not runtime behavior)

## Key Files

- Transpiler: `lightplayer/crates/lp-glsl-filetests/src/transpiler/rust.rs`
- Bless binary: `lightplayer/crates/lp-glsl-filetests/src/bin/bless-filetest.rs`
- Test files: `lightplayer/crates/lp-glsl-filetests/filetests/function2/`

## Dependencies

- `glsl-parser` (via `lp-glsl` dependency)
- Rust standard library for execution

## Limitations

The converter should be conservative and only handle features needed for test validation:
- **Skip**: `inout`/`out` parameters (Rust doesn't have equivalent semantics)
- **Skip**: Error tests (compile-time errors)
- **Skip**: Complex GLSL features not needed for basic test validation

## Usage

```bash
# Bless a single test file
cargo run -p lp-glsl-filetests --bin bless-filetest -- function2/call-multiple.glsl

# Bless all function2 tests (excluding inout/out/error tests)
cargo run -p lp-glsl-filetests --bin bless-filetest -- function2/*.glsl
```

## Notes

- This is a minimal converter - only what's needed for test validation
- Focus on correctness over completeness
- Fail fast on unsupported features rather than trying to handle everything

