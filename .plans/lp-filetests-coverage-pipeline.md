---
name: lp-filetests-coverage-pipeline
overview: Enhance lp-glsl-filetests with pipeline/target specification (like cranelift), ensure comprehensive builtin function test coverage, and add fixed-point conversion tests (16.16 and 32.32) for math functions running in riscv32 emulator.
todos:
  - id: parse_target_directives
    content: Add target directive parsing in filetest.rs (parse `target riscv32.fixed32` etc.)
    status: pending
  - id: add_compile_i64
    content: Add compile_i64() method to Compiler for 32.32 fixed point support
    status: pending
  - id: riscv32_emulator_integration
    content: Integrate riscv32 emulator execution in test_run.rs for target riscv32 tests
    status: pending
    dependencies:
      - parse_target_directives
  - id: audit_builtin_coverage
    content: Audit existing builtin function tests and identify missing coverage
    status: pending
  - id: create_missing_tests
    content: Create missing builtin function test files organized by function groups
    status: pending
    dependencies:
      - audit_builtin_coverage
  - id: add_fixed_point_math_tests
    content: Add fixed-point conversion tests (16.16 and 32.32) for math functions with riscv32 target
    status: pending
    dependencies:
      - add_compile_i64
      - riscv32_emulator_integration
  - id: update_readme
    content: Update README.md with target/pipeline specification documentation
    status: pending
    dependencies:
      - parse_target_directives
---

# Enhance lp-glsl-filetests with Pipeline Support and Comprehensive Coverage

## Overview

Add pipeline/target specification support with combined architecture and fixed-point format (e.g., `target riscv32.fixed32`), ensure full builtin function test coverage organized by function groups, and add fixed-point conversion tests (16.16 and 32.32) for math functions specifically running in riscv32 emulator.

## Target Specification Format

**Proposed format:** `target <arch>[.<format>]`

Examples:
- `target host` - Native/host execution with regular float (no fixed-point transformation)
- `target host.fixed32` - Native/host execution with 16.16 fixed-point (Fixed16x16)
- `target host.fixed64` - Native/host execution with 32.32 fixed-point (Fixed32x32)
- `target riscv32` - riscv32 emulator with regular float (no fixed-point transformation)
- `target riscv32.fixed32` - riscv32 emulator with 16.16 fixed-point (Fixed16x16)
- `target riscv32.fixed64` - riscv32 emulator with 32.32 fixed-point (Fixed32x32)

**Rationale:**
- `host` represents native JIT execution (we can't emulate all platforms)
- Dot notation (`riscv32.fixed32`) clearly combines architecture and format
- Matches existing `test fixed32` / `test fixed64` naming convention
- Allows multiple target directives per file (like cranelift)
- Backward compatible: `target riscv32` defaults to regular float

**Alternative considered:** `target riscv32:fixed32` (colon separator) - dot notation is cleaner and more compact.

## Tolerance Defaults

Tests no longer need to specify tolerance explicitly. Instead, sensible defaults are automatically applied based on the target:

- **`host`** (regular float): **0.0001** - Standard f32 precision
- **`host.fixed32`** / **`riscv32.fixed32`** (16.16 fixed-point): **0.001** - Accounts for fixed-point precision (1/65536 ≈ 0.00001526) and operation accumulation
- **`host.fixed64`** / **`riscv32.fixed64`** (32.32 fixed-point): **0.0001** - Very high precision (1/4294967296 ≈ 0.00000000023), but accounts for operation accumulation

Run directives use simplified format: `// run: ~= <value>` without tolerance specifiers.

## Implementation Plan

### 1. Add Pipeline/Target Specification Support

**Files to modify:**
- `crates/lp-glsl-filetests/src/filetest.rs` - Parse `target` directives with format suffix
- `crates/lp-glsl-filetests/src/test_run.rs` - Support multiple targets and riscv32 emulator execution
- `crates/lp-glsl-filetests/src/test_compile.rs` - Support target-specific compilation

**Changes:**
- Parse `target <arch>[.<format>]` directives from test files
  - Parse format suffix: `.fixed32` → Fixed16x16, `.fixed64` → Fixed32x32
  - If no format specified, use regular float (no fixed-point transformation)
- Support multiple target directives per test file (like cranelift)
- For `target host*`, use native JIT execution (current host platform)
- For `target riscv32*`, use riscv32 emulator via `lp-riscv-tools::Riscv32Emulator`
- Default to host target if no target specified (backward compatible)
- Store target list with associated fixed-point format per test file

**Data structure:**
```rust
enum TestTarget {
    Host(Option<FixedPointFormat>),  // Native JIT execution
    Riscv32(Option<FixedPointFormat>),  // riscv32 emulator
}
```

### 2. Add Support for 32.32 Fixed Point Runtime Tests

**Files to modify:**
- `crates/lp-glsl/src/compiler.rs` - Add `compile_i64` method for 32.32 fixed point
- `crates/lp-glsl-filetests/src/test_run.rs` - Remove skip for 32.32 tests, add i64 result handling

**Changes:**
- Implement `compile_i64()` in `Compiler` to return `fn() -> i64` for Fixed32x32
- Update `test_run.rs` to handle i64 return values and convert from fixed32x32 to float
- Ensure riscv32 emulator can execute i64-returning functions
- When `target riscv32.fixed64` is specified, use `compile_i64()` and execute in emulator
- Remove tolerance parsing from run directives - use target-based defaults instead

### 3. Analyze and Complete Builtin Function Test Coverage

**Builtin function groups to verify:**
- **Geometric** (5 functions): dot, cross, length, normalize, distance
- **Common** (11 functions): min, max, clamp, abs, sqrt, floor, ceil, fract, mod, sign, pow
- **Trigonometric** (15 functions): radians, degrees, sin, cos, tan, asin, acos, atan, atan2, sinh, cosh, tanh, asinh, acosh, atanh
- **Interpolation** (3 functions): mix, step, smoothstep
- **Matrix** (5 functions): matrixCompMult, outerProduct, transpose, determinant, inverse

**Action:**
- Audit existing tests in `filetests/builtins/` directories
- Create missing test files (one per function group or comprehensive coverage)
- Ensure tests cover scalar and vector variants where applicable
- Group tests by function category in subdirectories
- Verify each builtin has at least one test file

### 4. Add Fixed-Point Conversion Tests for Math Functions

**Files to create/modify:**
- New test files in `filetests/builtins/` with combined target directives
- Tests should specify `target riscv32.fixed32` and/or `target riscv32.fixed64` for math functions

**Test structure:**
- Create fixed-point variants of math function tests
- Each math function test should:
  - Include `target riscv32.fixed32` and/or `target riscv32.fixed64` directives
  - Verify correctness using target-based tolerance defaults (no explicit tolerance needed)
  - Test in riscv32 emulator to ensure consistent results
  - Can be in same file with multiple target directives, or separate files

**Functions requiring fixed-point tests:**
- All trigonometric functions (sin, cos, tan, asin, acos, atan, atan2, sinh, cosh, tanh, asinh, acosh, atanh)
- Common math functions (sqrt, pow, floor, ceil, fract, mod)
- Interpolation functions (mix, step, smoothstep)

**Example test file:**
```glsl
// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return sin(0.0);
}

// ... CLIF IR expectations ...
// run: ~= 0.0
```

Tolerances are automatically applied based on the target (see Tolerance Defaults section above).

### 5. Update Test Infrastructure

**Files to modify:**
- `crates/lp-glsl-filetests/src/filetest.rs` - Parse target directives with format suffix
- `crates/lp-glsl-filetests/src/test_run.rs` - Execute tests for each specified target
- `crates/lp-glsl-filetests/README.md` - Document target/pipeline specification

**Changes:**
- Extend directive parsing to support `target <arch>[.<format>]` format
- Store target list with fixed-point format per test file
- Pass target information to test runners
- Execute test for each specified target (like cranelift)
- Update run directive parsing to remove tolerance specifiers (use target-based defaults)
- Update README with examples of target specification and tolerance defaults

### 6. Integration with Riscv32 Emulator

**Files to modify:**
- `crates/lp-glsl-filetests/src/test_run.rs` - Integrate riscv32 emulator execution

**Changes:**
- When `target host*` is specified, use native JIT compilation and execution
- When `target riscv32*` is specified, compile for riscv32 ISA and use emulator
- Use `lp-riscv-tools::Riscv32Emulator` to execute riscv32 compiled code
- Handle both i32 (16.16) and i64 (32.32) return types in emulator
- Convert fixed-point results back to floats for comparison
- Apply appropriate fixed-point format based on target suffix
- Apply target-specific tolerance defaults:
  - `host`: 0.0001
  - `host.fixed32` / `riscv32.fixed32`: 0.001
  - `host.fixed64` / `riscv32.fixed64`: 0.0001

## Test Organization

- Keep existing test structure in `filetests/builtins/` subdirectories
- Fixed-point tests can be:
  - In same file with multiple `target` directives (e.g., both `target riscv32.fixed32` and `target riscv32.fixed64`)
  - Or as separate files (e.g., `sin_scalar_fixed32.glsl`, `sin_scalar_fixed64.glsl`)
- Math functions should have dedicated fixed-point test files with riscv32 targets

## Example Test File Formats

**Single target with fixed-point:**
```glsl
// test compile
// test run
// target riscv32.fixed32

float main() {
    return sin(0.0);
}

// ... CLIF IR expectations ...
// run: ~= 0.0
```

**Multiple targets (host and riscv32 with fixed-point):**
```glsl
// test compile
// test run
// target host
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return sqrt(16.0);
}

// ... CLIF IR expectations ...
// run: ~= 4.0
```

**Math function with fixed-point only (riscv32 emulator):**
```glsl
// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return cos(3.14159);
}

// ... CLIF IR expectations ...
// run: ~= -1.0
```

**Host with fixed-point:**
```glsl
// test compile
// test run
// target host.fixed32

float main() {
    return sin(0.0);
}

// ... CLIF IR expectations ...
// run: ~= 0.0
```

Tolerances are automatically applied based on the target (see Tolerance Defaults section above).

## Migration Strategy

- Existing tests without `target` directives continue to work (default to `host`)
- Existing `test fixed32` / `test fixed64` directives can coexist with `target` directives
  - If both present, `target` format takes precedence
  - Or deprecate separate `test fixed32`/`test fixed64` in favor of `target` format
- Gradually migrate math function tests to use `target riscv32.fixed32` / `target riscv32.fixed64`
- Use `target host` for tests that should run on native JIT (most existing tests)
- Use `target riscv32*` specifically for fixed-point math function tests that need emulator verification

## Dependencies

- `lp-riscv-tools` crate (already in dependencies)
- Cranelift riscv32 ISA support
- Existing fixed-point transformation code in `crates/lp-glsl/src/transform/fixed_point.rs`

