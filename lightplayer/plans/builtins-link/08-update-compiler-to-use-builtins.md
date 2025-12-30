# Phase 8: Update compiler to use builtins

## Goal

Replace 64-bit arithmetic generation in `convert_fmul` and `convert_fdiv` with calls to `__lp_fixed32_mul` and `__lp_fixed32_div`.

## Steps

### 8.1 Understand current implementation

- Review `convert_fmul` in `arithmetic.rs`
- Review `convert_fdiv` in `arithmetic.rs`
- Understand how they currently generate 64-bit operations

### 8.2 Set up builtin function declarations

- Declare `__lp_fixed32_mul` and `__lp_fixed32_div` as external functions in module
- Create function signatures matching: `(i32, i32) -> i32`
- Set up function linking (for JIT, will link to host functions)

### 8.3 Replace `convert_fmul`

- Remove i64 intermediate generation (`sextend`, `imul` on i64, etc.)
- Generate call to `__lp_fixed32_mul` instead
- Pass operands as i32, get result as i32
- Remove saturation logic (builtin handles it)

### 8.4 Replace `convert_fdiv`

- Remove complex division logic with 64-bit operations
- Generate call to `__lp_fixed32_div` instead
- Pass operands as i32, get result as i32
- Remove division-by-zero handling (builtin handles it)

### 8.5 Update `convert_fsqrt` if exists

- Replace with call to `__lp_fixed32_sqrt`
- Remove 64-bit operations

### 8.6 Test compilation

- Verify compiler still generates valid code
- Verify no 64-bit operations remain in generated code
- Test with sample GLSL programs

## Files to Modify

- `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/converters/arithmetic.rs`
- `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/builtins.rs` (if exists)
- Module setup for declaring external functions

## Success Criteria

- Compiler generates calls to builtin functions instead of 64-bit operations
- No i64 types in generated code for fixed32 arithmetic
- Generated code compiles and runs correctly
- GLSL filetests still pass

## Notes

- This is the key phase - removes 64-bit operations from compiler
- Need to ensure function declarations are correct
- May need to update function linking logic for JIT
- Keep old code commented out initially for reference


