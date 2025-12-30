# Phase 7: Remove main() generation from bootstrap.rs

## Goal

Remove main() wrapper generation logic from filetest bootstrap code.

## Changes

1. **Update `generate_bootstrap` function**:
   - Remove main() function generation logic
   - Return only the filtered function code (no main wrapper)
   - Update `BootstrapResult` to remove `main_start_line` and `main_end_line` fields (or set to 0)

2. **Simplify bootstrap generation**:
   - Function filtering logic can remain (for call graph analysis)
   - Return type inference can remain (for validation)
   - But no main() wrapper generation

3. **Update `BootstrapResult` struct**:
   - Remove or deprecate `main_start_line` and `main_end_line` fields
   - Update any code that uses these fields

## Files to Modify

- `lightplayer/crates/lp-glsl-filetests/src/test_run/bootstrap.rs`

## Success Criteria

- Main() generation completely removed from bootstrap.rs
- Bootstrap code returns only function definitions (no main wrapper)
- Code compiles without warnings
- Filetest execution will need updates in next phase to work without main()

