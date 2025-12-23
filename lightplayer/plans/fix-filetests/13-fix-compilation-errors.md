# Phase 13: Fix Remaining Compilation Errors

## Goal

Fix any remaining compilation errors that prevent tests from compiling. This should fix the remaining 1-2 tests that fail with compilation errors.

## Prerequisites

- Phase 11 completed: Register allocation invalid indices fixed
- Phase 12 completed: Runtime errors fixed (may reveal compilation errors)

## Problem Analysis

### Current Failures

After fixing register allocation and runtime errors, there may be 1-2 tests still failing with compilation errors. These need to be investigated individually.

### Potential Issues

1. **Global Value Type Mismatches**:

   - Global values may have incorrect types
   - Type conversion may not be working correctly

2. **Missing ISLE Patterns**:

   - Some instruction combinations may not have lowering rules
   - May need additional ISLE patterns

3. **ABI Issues**:

   - Function signatures may not match ABI requirements
   - Return value handling may be incorrect

4. **Type System Issues**:
   - Type conversions may not be working
   - Type checking may be too strict

## Investigation Plan

### Step 1: Identify Remaining Failures

After completing Phases 11-12, run all tests and identify which ones still fail:

```bash
cargo run --package cranelift-tools --bin clif-util -- test $(find cranelift/filetests/filetests/runtests -name "*.clif" -exec grep -l "^target riscv32" {} \;) 2>&1 | grep -E "(FAIL|Error|panic)" > /tmp/remaining_failures.txt
```

### Step 2: Categorize Failures

Categorize remaining failures:

- Compilation errors
- Runtime errors
- Unsupported features
- Other issues

### Step 3: Investigate Each Failure

For each compilation error:

1. Read the error message
2. Check the test file
3. Identify the root cause
4. Determine the fix

## Implementation Plan

### Fix 1: Fix Global Value Type Mismatches (if needed)

**File**: `cranelift/codegen/src/legalizer/globalvalue.rs`

**Issue**: Global values may have type mismatches (e.g., i32 global value used as i64).

**Fix**: Ensure type conversion works correctly:

- i32 -> i64 conversion (uextend)
- Proper handling of global value types
- Correct type checking

### Fix 2: Add Missing ISLE Patterns (if needed)

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Issue**: Some instruction combinations may not have lowering rules.

**Fix**: Add missing ISLE patterns:

- Check error messages for missing patterns
- Add lowering rules for missing patterns
- Test that patterns work correctly

### Fix 3: Fix ABI Issues (if needed)

**File**: `cranelift/codegen/src/isa/riscv32/abi.rs`

**Issue**: Function signatures may not match ABI requirements.

**Fix**: Ensure ABI compliance:

- Function argument handling
- Return value handling
- Calling conventions

### Fix 4: Fix Type System Issues (if needed)

**Files**: Various legalizer and type system files

**Issue**: Type conversions or type checking may not be working correctly.

**Fix**: Ensure type system works correctly:

- Type conversions
- Type checking
- Type inference

## Testing

After implementing fixes:

```bash
# Test all riscv32 tests
cargo run --package cranelift-tools --bin clif-util -- test $(find cranelift/filetests/filetests/runtests -name "*.clif" -exec grep -l "^target riscv32" {} \;)

# Check final status
cargo run --package cranelift-tools --bin clif-util -- test $(find cranelift/filetests/filetests/runtests -name "*.clif" -exec grep -l "^target riscv32" {} \;) 2>&1 | tail -5
```

## Success Criteria

- ✅ All compilation errors fixed
- ✅ All tests compile successfully
- ✅ Remaining failures are runtime errors or unsupported features (documented)
- ✅ No panics or crashes

## Estimated Time

- Investigation: 1-2 hours (identify remaining failures)
- Fix 1-4: 2-6 hours (depending on issues found)

**Total**: 3-8 hours (depends on what issues remain)

## Related Issues

- Phase 8: Compilation errors (original plan)
- Phase 11: Register allocation (may fix some compilation errors)
- Phase 12: Runtime errors (may reveal compilation errors)

## Notes

- This phase depends on completing previous phases
- May find that some "compilation errors" are actually runtime errors or unsupported features
- Some failures may be acceptable (documented as expected failures)
- Goal is to have all tests compile successfully, even if some fail at runtime




