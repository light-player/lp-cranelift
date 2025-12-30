# Phase 6: Update call_* methods to support any function

## Goal

Remove `validate_main_only` and `validate_no_args` restrictions, add function existence/signature validation.

## Changes

1. **Remove validation restrictions**:
   - Remove `validate_main_only` calls from all `call_*` methods
   - Remove `validate_no_args` calls (functions can now have arguments)
   - Remove the `validate_main_only` and `validate_no_args` helper functions

2. **Add function existence validation**:
   - Check if function exists in `function_addresses` map
   - Return clear error if function not found: "Function '{}' not found in object file"

3. **Add signature validation**:
   - Check if function signature exists in `cranelift_signatures` map
   - Return clear error if signature not found: "Function signature for '{}' not found"
   - Validate argument count matches signature (if needed)

4. **Update function address lookup**:
   - Look up function address from `function_addresses` map instead of using `main_address`
   - Use looked-up address for `call_function` calls

5. **Update all `call_*` methods**:
   - `call_void`, `call_i32`, `call_f32`, `call_bool`
   - `call_bvec`, `call_ivec`, `call_uvec`, `call_vec`, `call_mat`
   - All should support calling any function with proper arguments

## Files to Modify

- `lightplayer/crates/lp-glsl/src/exec/emu.rs` (all call_* methods, remove validation helpers)

## Success Criteria

- All `call_*` methods support calling any function (not just main)
- Functions can be called with arguments
- Clear error messages for missing functions or signature mismatches
- Existing tests updated to work with new behavior
- Code compiles without warnings

