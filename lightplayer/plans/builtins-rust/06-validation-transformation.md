# Phase 6: Implement validation and transformation

## Goal

Create validation rules (one file per rule), implement panic-to-trap transformation.

## Steps

### 6.1 Create validation module structure

- Create `src/generator/validate/mod.rs` - Validation module
- Create individual validation files (one per rule):
  - `src/generator/validate/no_external_calls.rs` - Check for external function calls
  - `src/generator/validate/no_panics.rs` - Check for panic instructions
  - `src/generator/validate/signature_match.rs` - Validate function signatures
  - `src/generator/validate/builtin_calls_only.rs` - Ensure calls are only to `__lp_*` functions

### 6.2 Implement validation rules

- **No external calls**: Check for `Call` and `CallIndirect` instructions to non-`__lp_*` functions
- **No panics**: Check for panic-related instructions (may need to check what rustc generates)
- **Signature match**: Validate parameter and return types match expected signature
- **Builtin calls only**: Ensure all function calls are to `__lp_*` functions (or allow `trap`)

### 6.3 Create validation runner

- Create `validate_function()` function that runs all validation rules
- Collect all validation errors (don't stop on first error)
- Return structured error information

### 6.4 Implement panic-to-trap transformation

- Create `src/generator/transform/panic_to_trap.rs`
- Find panic instructions in CLIF
- Replace with `trap` instructions (matching existing system)
- Ensure transformation preserves function semantics

### 6.5 Integrate validation and transformation

- Add validation step to `generate-clif` workflow
- Add transformation step after validation
- Report validation errors in filetest-style format
- Only proceed if validation passes (or with `--force`)

## Files to Create/Modify

### New Files
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/validate/mod.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/validate/no_external_calls.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/validate/no_panics.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/validate/signature_match.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/validate/builtin_calls_only.rs`
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/transform/panic_to_trap.rs`

### Modified Files
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/mod.rs` - Add validation/transform modules
- `lightplayer/crates/lp-glsl-builtins-tool/src/main.rs` - Integrate validation/transformation

## Success Criteria

- All validation rules are implemented (one file per rule)
- Validation errors are reported in filetest-style format
- Panic-to-trap transformation is implemented
- Validation and transformation are integrated into `generate-clif` workflow
- `sqrt_recip` passes all validations

## Notes

- Keep validation rules clean and focused (one file per rule)
- Allow calls to other `__lp_*` builtins (dependency handling comes later)
- Allow `trap` instructions for error handling
- Debug assertions shouldn't exist (compile with `-C debuginfo=0`)

