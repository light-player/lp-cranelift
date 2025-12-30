# Phase 9: Update filetest execution to call functions directly

## Goal

Modify filetest execution to parse function calls from `// run:` directives and call them directly.

## Changes

1. **Update `run_test_file_with_line_filter` in `test_run/mod.rs`**:
   - Parse function call from `RunDirective.expression_str` (e.g., "add_float(1.5, 2.5)")
   - Extract function name and arguments
   - Call function directly using appropriate `call_*` method based on return type
   - Compare result with expected value

2. **Add function call parsing**:
   - Parse expression string to extract:
     - Function name
     - Arguments (as strings, need to convert to GlslValue)
   - Handle different argument types (int, float, bool, vectors, etc.)

3. **Add argument conversion**:
   - Convert parsed argument strings to `GlslValue` enum
   - Handle type inference or explicit type hints
   - Support all GLSL types used in filetests

4. **Update result comparison**:
   - Use existing comparison logic (exact for int/bool, approximate for float)
   - Handle vector and matrix comparisons

5. **Update error handling**:
   - Handle function not found errors
   - Handle signature mismatch errors
   - Provide clear error messages with function name and arguments

## Files to Modify

- `lightplayer/crates/lp-glsl-filetests/src/test_run/mod.rs`
- May need helper functions for parsing and argument conversion

## Success Criteria

- Filetests can call functions directly from `// run:` directives
- Function arguments are parsed and converted correctly
- Results are compared correctly (exact/approximate)
- Clear error messages for parsing or execution failures
- All filetests pass (except main() tests which are removed)

