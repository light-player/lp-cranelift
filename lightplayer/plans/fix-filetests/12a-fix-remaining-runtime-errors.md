# Phase 12a: Fix Remaining Runtime Errors

## Goal

Fix the remaining 7 tests that fail with runtime errors after the initial Phase 12 fixes. These tests involve overflow operations, complex arithmetic operations, and multi-return value handling.

## Prerequisites

- Phase 12 completed: I64 load/store operations fixed (using 32-bit ops)
- Phase 12 completed: Stack operations working
- Phase 11 completed: Register allocation fixes

## Problem Analysis

### Current Status

**Fixed in Phase 12:**
- ✅ `stack.clif` - Fixed by correcting I64 load/store to use 32-bit operations

**Still Failing (7 tests):**
- `cls.clif` - Count leading sign bits operation
- `integer-minmax.clif` - Integer min/max operations
- `smul_overflow.clif` - Signed multiplication overflow
- `uadd_overflow.clif` - Unsigned addition overflow
- `uadd_overflow_narrow.clif` - Unsigned addition overflow (narrow types)
- `uadd_overflow_trap.clif` - Unsigned addition overflow with trap
- `umul_overflow.clif` - Unsigned multiplication overflow

### Root Cause Analysis

From investigation, the main issues are:

1. **Overflow Operations Not Properly Implemented**:
   - I64 overflow operations return incorrect values
   - Overflow detection logic is incomplete or incorrect
   - Multi-return values (result + overflow flag) not handled correctly

2. **Multi-Return Value Handling**:
   - `output_pair` ISLE construct may not be correctly lowering to register pairs
   - Return value extraction from registers may be incorrect
   - Register pair reconstruction for multi-return functions

3. **Complex Arithmetic Operations**:
   - `cls` (count leading sign bits) may not be correctly lowered
   - `smin`/`smax` operations may have issues with sign extension
   - Register allocation for complex operations

4. **Carry Propagation Issues**:
   - I64 addition/subtraction overflow needs proper carry propagation
   - High and low parts of I64 values not correctly combined

## Investigation Plan

### Step 1: Analyze Overflow Operation Failures

**Test**: `uadd_overflow.clif`

**Expected behavior**:
- Function `%uaddof_i64(i64, i64) -> i64, i8` returns `[sum, overflow_flag]`
- For `(0, 0)`: should return `[0, 0]`
- For `(0, 1)`: should return `[1, 0]`
- For `(-1, 1)`: should return `[0, 1]` (overflow)

**Current behavior**:
- Returns incorrect values (e.g., `[0, 1]` instead of `[1, 0]`)

**Investigation**:
```bash
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/uadd_overflow.clif --verbose
```

**Check**:
- How `output_pair` is lowered in ISLE
- How multi-return values are handled in ABI
- How registers are extracted in emulator

### Step 2: Analyze Complex Operation Failures

**Test**: `integer-minmax.clif`

**Expected behavior**:
- Function `%isort3(i8, i8, i8) -> i8, i8, i8` sorts three values
- For `(1, 2, 3)`: should return `[1, 2, 3]`

**Current behavior**:
- Returns `[2, 0, 3]` instead of `[1, 2, 3]`

**Investigation**:
```bash
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/integer-minmax.clif --verbose
```

**Check**:
- Sign extension for i8 values
- `smin`/`smax` instruction lowering
- Register allocation for multiple return values

### Step 3: Check Multi-Return Value Handling

**Files to check**:
- `cranelift/codegen/src/isa/riscv32/abi.rs` - Return value handling
- `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs` - Return value extraction
- `cranelift/codegen/src/isa/riscv32/lower.isle` - `output_pair` lowering

**Verify**:
- Multi-return functions correctly allocate registers
- Return values correctly extracted from registers
- Register pairs correctly reconstructed

## Implementation Plan

### Fix 1: Fix I64 Overflow Operations

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Issue**: I64 overflow operations (`uadd_overflow`, `sadd_overflow`, etc.) are not correctly implemented. The current implementation:
- Doesn't properly propagate carry between low and high parts
- Doesn't correctly detect overflow
- Returns incorrect values

**Current code** (simplified placeholder):
```isle
(rule 1 (lower (has_type $I64 (uadd_overflow x y)))
  (let ((sum_lo XReg (rv_add (value_regs_get x 0) (value_regs_get y 0)))
        (sum_hi XReg (rv_add (value_regs_get x 1) (value_regs_get y 1))))
    (output_pair (value_regs sum_lo sum_hi) (imm $I8 0))))
```

**Fix**: Implement proper carry propagation and overflow detection:

```isle
(rule 1 (lower (has_type $I64 (uadd_overflow x y)))
  (let ((x_lo XReg (value_regs_get x 0))
        (x_hi XReg (value_regs_get x 1))
        (y_lo XReg (value_regs_get y 0))
        (y_hi XReg (value_regs_get y 1))
        ;; Add low parts
        (sum_lo XReg (rv_add x_lo y_lo))
        ;; Check for carry: if sum_lo < x_lo, then carry occurred
        (carry XReg (rv_sltu sum_lo x_lo))
        ;; Add high parts
        (sum_hi_tmp XReg (rv_add x_hi y_hi))
        ;; Add carry to high part
        (sum_hi XReg (rv_add sum_hi_tmp carry))
        ;; Overflow occurs if:
        ;; 1. High part overflowed (sum_hi < x_hi), OR
        ;; 2. High part overflowed due to carry (sum_hi < sum_hi_tmp)
        (overflow_hi XReg (rv_sltu sum_hi x_hi))
        (overflow_carry XReg (rv_sltu sum_hi sum_hi_tmp))
        (overflow XReg (rv_or overflow_hi overflow_carry)))
    (output_pair (value_regs sum_lo sum_hi) overflow)))
```

**Similar fixes needed for**:
- `sadd_overflow` (signed addition overflow)
- `usub_overflow` (unsigned subtraction overflow)
- `ssub_overflow` (signed subtraction overflow)
- `umul_overflow` (unsigned multiplication overflow)
- `smul_overflow` (signed multiplication overflow)

### Fix 2: Fix Multi-Return Value Handling

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`

**Issue**: Multi-return values may not be correctly extracted from registers.

**Check**:
- How `output_pair` results are returned
- Register allocation for multi-return functions
- Return value extraction logic

**Fix**:
- Ensure return values are extracted in correct order
- Ensure register pairs are correctly reconstructed
- Ensure overflow flags are correctly extracted

### Fix 3: Fix Complex Arithmetic Operations

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Issue**: Complex operations like `cls`, `smin`, `smax` may have issues.

**For `cls` (count leading sign bits)**:
- Check if instruction is correctly lowered
- Verify sign extension handling

**For `smin`/`smax`**:
- Check sign extension for narrow types (i8, i16)
- Verify comparison logic
- Check register allocation

**Fix**:
- Review and fix lowering rules for these operations
- Ensure proper sign extension
- Test with various input values

### Fix 4: Fix Narrow Type Overflow Operations

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Issue**: Overflow operations for narrow types (i8, i16) may not be correctly implemented.

**Current implementation** (from investigation):
- i8/i16 overflow operations exist but may have bugs
- Need to verify overflow detection logic

**Fix**:
- Review i8/i16 overflow operation implementations
- Ensure overflow detection is correct
- Test with edge cases

### Fix 5: Fix Overflow Trap Operations

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Issue**: `uadd_overflow_trap` may not correctly trap on overflow.

**Fix**:
- Ensure trap is correctly generated when overflow occurs
- Verify trap handling in emulator

## Testing

After implementing fixes:

```bash
# Test overflow operations
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/uadd_overflow.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/uadd_overflow_narrow.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/uadd_overflow_trap.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/smul_overflow.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/umul_overflow.clif

# Test complex operations
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/cls.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/integer-minmax.clif

# Test all riscv32 tests
cargo run --package cranelift-tools --bin clif-util -- test $(find cranelift/filetests/filetests/runtests -name "*.clif" -exec grep -l "^target riscv32" {} \;)
```

## Success Criteria

- ✅ All 7 remaining runtime error tests pass
- ✅ Overflow operations correctly detect overflow
- ✅ Overflow operations return correct values
- ✅ Multi-return values correctly handled
- ✅ Complex arithmetic operations work correctly
- ✅ No regression in previously fixed tests

## Estimated Time

- Investigation: 2-3 hours (analyze each failing test)
- Fix 1: 4-6 hours (implement proper overflow operations)
- Fix 2: 1-2 hours (fix multi-return value handling)
- Fix 3: 2-3 hours (fix complex operations)
- Fix 4: 1-2 hours (fix narrow type overflow)
- Fix 5: 1-2 hours (fix overflow trap)
- Testing: 1-2 hours

**Total**: 12-20 hours

## Related Issues

- Phase 12: Initial runtime error fixes
- Phase 11: Register allocation (may affect multi-return values)
- Phase 3: I64 handling (foundation for overflow operations)

## Notes

- Overflow operations are complex and require careful implementation
- Multi-return value handling is critical for overflow operations
- May need to add debug logging to understand register values
- Consider comparing with interpreter results to verify correctness
- Some overflow operations may need library calls for complex cases

## Implementation Order

1. **First**: Fix multi-return value handling (Fix 2) - this affects all overflow operations
2. **Second**: Fix I64 overflow operations (Fix 1) - core functionality
3. **Third**: Fix narrow type overflow (Fix 4) - simpler cases
4. **Fourth**: Fix overflow trap (Fix 5) - depends on overflow detection
5. **Fifth**: Fix complex operations (Fix 3) - may depend on other fixes

## References

- RISC-V overflow detection algorithms
- Two's complement arithmetic overflow rules
- Multi-register return value ABI conventions





