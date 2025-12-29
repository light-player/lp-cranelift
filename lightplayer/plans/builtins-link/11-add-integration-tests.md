# Phase 11: Add integration tests

## Goal

Add sanity tests for linking (both emulator and JIT), verify functions are callable.

## Steps

### 11.1 Create emulator integration test

- Create test that compiles a simple program calling builtins
- Link `lp-builtins` static library
- Load ELF in emulator
- Call builtin functions and verify results

### 11.2 Create JIT integration test

- Create test that compiles GLSL code using builtins
- Generate JIT code
- Execute JIT code
- Verify builtin functions are called and return correct results

### 11.3 Test all three functions

- Test `__lp_fixed32_div` in both contexts
- Test `__lp_fixed32_mul` in both contexts
- Test `__lp_fixed32_sqrt` in both contexts

### 11.4 Test edge cases

- Test division by zero (emulator and JIT)
- Test overflow cases (mul)
- Test edge values (sqrt)

## Files to Create

- Emulator integration test (in `lp-riscv-tools` or test crate)
- JIT integration test (in `lp-glsl` tests)

## Success Criteria

- Emulator can load and call builtin functions
- JIT can call builtin functions
- Results match expected values in both contexts
- Edge cases are handled correctly

## Notes

- These are sanity tests - full correctness testing happens via GLSL filetests
- Focus on verifying linking works, not exhaustive correctness
- Should catch linking/symbol resolution issues early

