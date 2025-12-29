# Phase 2: Define test expectations format

## Goal

Design a format for test expectations (call args, expected results) that works for both unit tests and CLIF runtest transformation.

## Decision: CLIF Runtest Format in Source

**Format**: Use CLIF runtest format directly in Rust source files:

- `// run: %function_name(args) == expected` or `// run: %function_name(args) ~= expected (tolerance: 0.001)`
- Parse with `include_str!(file!())` in unit tests
- Transform to CLIF filetests in later phases

## Module Structure

```
src/
  lib.rs                    - Re-exports: pub use test_util::run_runtests;
  test_util/                - Test expectations and runtest infrastructure (pub(crate))
    mod.rs                  - Module exports, public API
    expectations.rs         - Data structures (RunDirective, ComparisonOp, ParsedNumber)
    number.rs               - Number format parsing with format tracking
    parser.rs               - Parse // run: directives from source text
    macro.rs                - run_runtests! macro implementation
    clif.rs                 - Render expectations to CLIF filetest format
```

## Steps

### 2.1 Implement number format parsing (`number.rs`)

- Create `NumberFormat` enum: `Hex`, `Decimal`, `Fixed32`, `Float32`
- Create `ParsedNumber` enum: `I32 { value, format }`, `U32 { value, format }`, `F32 { value, format }`
- Implement `parse_number(s: &str) -> Result<ParsedNumber>`
- Support formats:
  - Hex: `0x00040000`
  - Decimal: `65536`
  - Fixed32 literal: `4.0fx32` (convert float to fixed16x16)
  - Float32 literal: `0.0f32`
- Preserve original format for CLIF rendering

### 2.2 Implement expectations data structures (`expectations.rs`)

- `ComparisonOp` enum: `Exact`, `Approx { tolerance: f32 }`
- `RunDirective` struct:
  - Function name
  - Arguments (as `ParsedNumber` vec)
  - Expected value (as `ParsedNumber`)
  - Comparison operator
  - Line number
- Default tolerance: `0.000001` (for float32)

### 2.3 Implement directive parser (`parser.rs`)

- Duplicate `parse_run_directive` logic from `lp-glsl-filetests/src/filetest/directives.rs`
- Parse format: `<expression> == <expected>` or `<expression> ~= <expected> [ (tolerance: <value>) ]`
- Extract function call: `%function_name(arg1, arg2)`
- Parse each argument with `parse_number()`
- Parse expected value with `parse_number()`
- Handle single-line only, flexible whitespace

### 2.4 Implement run_runtests! macro (`macro.rs`)

- Variadic macro: `run_runtests!(function_name, (i32), |x| function(x))`
- Or: `run_runtests!(function_name, (i32, i32), |a, b| function(a, b))`
- Parse `// run:` directives from `include_str!(file!())`
- Type checking: Verify all `ParsedNumber` variants match function signature
  - Arguments must match parameter types
  - Expected value must match return type
  - Error if mismatch
- Execute tests: Call closure with parsed arguments
- Compare results with tolerance handling
- Collect all failures, format nicely, then panic

### 2.5 Implement CLIF rendering (`clif.rs`)

- `render_to_clif(run_directives: &[RunDirective], function_name: &str) -> String`
- Preserve original format when rendering (e.g., `4.0fx32` → hex `0x00040000`)
- Generate `; run:` lines for CLIF filetests
- Handle format conversions appropriately

### 2.6 Wire up module (`mod.rs` and `lib.rs`)

- `test_util/mod.rs`: Export all modules, `pub(crate)` visibility
- `lib.rs`: Re-export `pub use test_util::run_runtests;` for `crate::run_runtests!` usage

### 2.7 Add example tests to `sqrt_recip`

- Add `// run:` directives to `sqrt_recip.rs`
- Add test using `crate::run_runtests!(fixed32_sqrt, (i32), |x| fixed32_sqrt(x))`
- Include edge cases: 0, 1, 4, 9, max fixed32 value, etc.

## Files to Create/Modify

### New Files

- `lightplayer/crates/lp-glsl-builtins-src/src/test_util/mod.rs`
- `lightplayer/crates/lp-glsl-builtins-src/src/test_util/expectations.rs`
- `lightplayer/crates/lp-glsl-builtins-src/src/test_util/number.rs`
- `lightplayer/crates/lp-glsl-builtins-src/src/test_util/parser.rs`
- `lightplayer/crates/lp-glsl-builtins-src/src/test_util/macro.rs`
- `lightplayer/crates/lp-glsl-builtins-src/src/test_util/clif.rs`

### Modified Files

- `lightplayer/crates/lp-glsl-builtins-src/src/lib.rs` - Add re-export
- `lightplayer/crates/lp-glsl-builtins-src/src/builtins/fixed32/sqrt_recip.rs` - Add `// run:` directives and test

## Success Criteria

- Number format parsing works for all supported formats
- `// run:` directives can be parsed from source
- `run_runtests!` macro works with type checking
- Example test cases exist for `sqrt_recip` and pass
- Format can be transformed to CLIF runtest format (design complete)

## Design Decisions

1. **Format**: CLIF runtest format (`// run:`) directly in source files
2. **Type Safety**: Macro verifies `ParsedNumber` variants match function signature
3. **Format Tracking**: Preserve original format for CLIF rendering
4. **Tolerance**: Default `0.000001` for float32, parseable `(tolerance: 0.001)`
5. **Module Location**: `test_util/` module, re-exported from `lib.rs`
6. **Error Handling**: Collect all failures, format nicely, then panic
