# Phase 3: Move Run Test Code to test_run/ Module

## Tasks

1. Move run test code to `test_run/` module:
   - Move `test_run/bootstrap.rs` → `test_run/test_glsl.rs` (rename "bootstrap" → "test glsl")
   - Move `test_run/execution.rs` → keep as is
   - Move `test_run/function_filter.rs` → merge into `test_run/test_glsl.rs`
   - Move `test_run/target.rs` → keep as is
   - Move `test_run/value_ops.rs` → `test_run/parse_assert.rs`
   - Split `test_run/mod.rs` into:
     - `test_run/run.rs` - Main delegator
     - `test_run/run_summary.rs` - Summary mode implementation
     - `test_run/run_detail.rs` - Detail mode implementation

2. Rename terminology:
   - `BootstrapResult` → `TestGlslResult`
   - `generate_bootstrap()` → `generate_test_glsl()` (or similar)
   - All variable names: `bootstrap_result` → `test_glsl_result`
   - Comments and string literals: "bootstrap" → "test glsl"

3. Update `test_run/mod.rs` to only contain re-exports

4. Update all imports throughout codebase

5. Replace `show_full_output: bool` with `output_mode: OutputMode` in all run test functions

## Files to Create

- `src/test_run/run.rs`
- `src/test_run/run_summary.rs`
- `src/test_run/run_detail.rs`
- `src/test_run/test_glsl.rs` (renamed from bootstrap.rs)
- `src/test_run/parse_assert.rs` (renamed from value_ops.rs)

## Files to Modify

- `src/test_run/mod.rs` - Convert to re-exports only
- `src/test_run/execution.rs` - Update imports
- `src/test_run/target.rs` - Update imports
- All files that import from `test_run::` - Update imports

## Files to Remove

- `src/test_run/bootstrap.rs` (after migration)
- `src/test_run/function_filter.rs` (merged into test_glsl.rs)
- `src/test_run/value_ops.rs` (renamed to parse_assert.rs)

## Success Criteria

- All run test code moved to `test_run/` module
- Summary and detail modes separated into different files
- Terminology updated ("test glsl" instead of "bootstrap")
- `OutputMode` enum used instead of boolean
- Code compiles
- No warnings
- Tests still pass

