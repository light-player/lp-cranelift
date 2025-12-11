# 07: Testing and Verification

## Goal

Run all i128 tests with riscv32 target enabled, verify they pass, and fix any issues that arise.

## Testing Strategy

### 7.1 Run All i128 Tests

```bash
# Run all i128 filetests
cargo test --package cranelift-filetests --test filetests i128

# Or run specific test files
cargo test --package cranelift-filetests --test filetests i128_arithmetic
cargo test --package cranelift-filetests --test filetests i128_bitops
cargo test --package cranelift-filetests --test filetests i128_load_store
# ... etc
```

### 7.2 Verify Test Execution

For each test file, verify:

1. **Compilation**: Tests compile without errors
2. **Lowering**: CLIF lowers to riscv32 instructions correctly
3. **Execution**: Tests run and produce correct results
4. **Interpretation**: Interpreter tests pass (if applicable)

### 7.3 Check for Common Issues

#### Issue 1: Register Allocation Problems

**Symptoms**: Compiler errors about register allocation, "too many registers"
**Cause**: i128 uses 4 registers, may exhaust available registers
**Fix**: Verify register allocator handles multi-register values correctly

#### Issue 2: ABI Mismatches

**Symptoms**: Function calls fail, wrong argument/return values
**Cause**: ABI doesn't handle 4-register i128 correctly
**Fix**: Check `rc_for_type` and argument passing in `abi.rs`

#### Issue 3: Comparison Failures

**Symptoms**: Comparisons produce wrong results
**Cause**: Multi-register comparison logic incorrect
**Fix**: Verify `lower_icmp_i128` handles high/low parts correctly

#### Issue 4: Shift Amount Handling

**Symptoms**: Shifts produce wrong results
**Cause**: Shift amount from i128 not extracted correctly
**Fix**: Verify shift amount extraction (use low 7 bits or extract from i128)

#### Issue 5: Libcall Failures

**Symptoms**: Division/remainder/conversion operations fail
**Cause**: Libcalls not properly linked or ABI incompatible
**Fix**: Verify libcall infrastructure and ABI compatibility

### 7.4 Test Categories

#### Arithmetic Operations

```bash
cargo test --package cranelift-filetests --test filetests i128_arithmetic
cargo test --package cranelift-filetests --test filetests i128_arithmetic_extends
cargo test --package cranelift-filetests --test filetests i128_ineg
cargo test --package cranelift-filetests --test filetests i128_iabs
```

**Expected**: All pass, verify carry/borrow propagation works correctly

#### Bitwise Operations

```bash
cargo test --package cranelift-filetests --test filetests i128_bitops
cargo test --package cranelift-filetests --test filetests i128_bnot
cargo test --package cranelift-filetests --test filetests i128_bitops_count
```

**Expected**: All pass, verify bitwise operations work on 4-register values

#### Memory Operations

```bash
cargo test --package cranelift-filetests --test filetests i128_load_store
```

**Expected**: All pass, verify 16-byte alignment and addressing

#### Control Flow

```bash
cargo test --package cranelift-filetests --test filetests i128_icmp
cargo test --package cranelift-filetests --test filetests i128_br
cargo test --package cranelift-filetests --test filetests i128_call
cargo test --package cranelift-filetests --test filetests i128_select
```

**Expected**: All pass, verify comparisons and branches work correctly

#### Conversions and Special Operations

```bash
cargo test --package cranelift-filetests --test filetests i128_extend
cargo test --package cranelift-filetests --test filetests i128_ireduce
cargo test --package cranelift-filetests --test filetests i128_shifts
cargo test --package cranelift-filetests --test filetests i128_rotate
cargo test --package cranelift-filetests --test filetests i128_concat_split
```

**Expected**: Most pass, some may need libcall support

## Debugging Failed Tests

### 7.5 Inspect Generated Code

For failing tests, inspect the generated riscv32 code:

```bash
# Run with verbose output
cargo test --package cranelift-filetests --test filetests i128_arithmetic -- --nocapture

# Or use cranelift-print to see CLIF
cargo run --bin cranelift-print -- file.clif
```

### 7.6 Check Lowering Rules

Verify that lowering rules match the operation:

```bash
# Search for specific operation in lower.isle
grep -A 10 "has_type \$I128.*operation_name" cranelift/codegen/src/isa/riscv32/lower.isle
```

### 7.7 Verify Register Representation

Check that i128 values are represented correctly:

- Should use 4 x I32 registers
- Low part in register 0, high parts in registers 1, 2, 3
- Verify `value_regs_get` indices are correct

### 7.8 Test Individual Operations

Create minimal test cases for failing operations:

```clif
test run
target riscv32
target riscv32 has_m

function %test_op(i128) -> i128 {
block0(v0: i128):
    v1 = operation v0
    return v1
}
; run: %test_op(0) == expected_result
```

## Success Criteria

### 7.9 All Tests Pass

- ✅ All 31 i128 test files compile
- ✅ All tests execute successfully
- ✅ Results match expected values
- ✅ No regressions in other riscv32 tests

### 7.10 Code Quality

- ✅ No compiler warnings
- ✅ Code follows cranelift style guidelines
- ✅ Comments explain non-obvious logic
- ✅ Register handling is correct

### 7.11 Documentation

- ✅ Test files have correct target declarations
- ✅ Any special handling is documented
- ✅ Known limitations are noted (if any)

## Rollback Plan

If tests reveal major issues:

1. Revert target additions to test files
2. Document issues found
3. Create follow-up tasks for fixing issues
4. Re-enable tests incrementally as issues are fixed

## Final Verification

### 7.12 Complete Test Run

```bash
# Run all filetests (not just i128)
cargo test --package cranelift-filetests --test filetests

# Run riscv32-specific tests
cargo test --package cranelift-filetests --test filetests -- riscv32

# Run with verbose output to catch warnings
RUST_LOG=debug cargo test --package cranelift-filetests --test filetests i128
```

### 7.13 Performance Check

Verify that i128 operations don't cause significant performance regressions:

- Check code size (should be reasonable)
- Verify instruction count (should be efficient)
- Compare with riscv64 implementation if possible

## Expected Outcome

After completing all testing:

- All i128 tests pass on riscv32
- No regressions in existing functionality
- Code is ready for commit
- Documentation is complete

## Files Modified Summary

All 31 test files in `cranelift/filetests/filetests/runtests/i128-*.clif`:

1. i128-arithmetic.clif
2. i128-arithmetic-extends.clif
3. i128-bitcast.clif
4. i128-bitops.clif
5. i128-bitops-count.clif
6. i128-bitrev.clif
7. i128-bmask.clif
8. i128-bornot.clif
9. i128-bswap.clif
10. i128-br.clif
11. i128-call.clif
12. i128-cls.clif
13. i128-concat-split.clif
14. i128-conversion.clif
15. i128-extend.clif
16. i128-iabs.clif
17. i128-icmp.clif
18. i128-ineg.clif
19. i128-ireduce.clif
20. i128-load-store.clif
21. i128-min-max.clif
22. i128-rotate.clif
23. i128-select.clif
24. i128-select-float.clif
25. i128-shifts.clif
26. i128-srem.clif
27. i128-urem.clif
28. i128-bandnot.clif
29. i128-bnot.clif
30. i128-bxornot.clif
31. i128-bitselect.clif
