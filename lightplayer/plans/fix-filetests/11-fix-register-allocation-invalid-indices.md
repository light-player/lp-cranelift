# Phase 11: Fix Register Allocation Invalid Indices

## Goal

Fix invalid register indices (specifically `<invalid>` registers with index 2097151) that occur when i64 values are used in function calls and other operations on riscv32. This should fix 15 failing tests (12 invalid indices + 3 reg.is_virtual() assertions).

## Prerequisites

- Phase 3 completed: Basic i64 handling works
- Phase 6a completed: `fits_in_64` architecture-awareness (if applicable)
- Phase 8 completed: Global value type conversion (if applicable)

## Problem Analysis

### Current Failures

**12 tests failing with "Invalid register indices detected before register allocation"**:

- `call.clif`
- `call_indirect.clif`
- `extend.clif`
- `global_value.clif`
- `iabs.clif`
- `popcnt.clif`
- `return-call-indirect.clif`
- `return-call-loop.clif`
- `return-call.clif`
- `smulhi.clif`
- `spill-reload.clif`
- `umulhi.clif`

**3 tests failing with "assertion failed: reg.is_virtual()"**:

- `bitrev.clif`
- `brif.clif`
- `i64-riscv32.clif`

### Root Cause

When i64 values are created (e.g., from `uextend`, `sextend`, or operations), they must be represented as register pairs (two i32 registers). However:

1. **uextend/sextend lowering**: May not properly produce both registers in the pair
2. **Function call argument preparation**: May not properly handle i64 register pairs
3. **Return value handling**: May not properly reconstruct i64 from register pairs
4. **Register pair extraction**: One register in the pair ends up as `<invalid>` (index 2097151 = 0x1FFFFF)

### Error Pattern

```
Instruction 6: CallInd { info: CallInfo {
    uses: [
        CallArgPair { vreg: v202, preg: p10i },
        CallArgPair { vreg: <invalid>, preg: p11i }  // <-- Invalid register!
    ],
    ...
}}
Inst 6: Reg <invalid> has VReg index 2097151
```

## Implementation Plan

### Fix 1: Add General uextend/sextend Lowering Rules for i32->i64

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Issue**: There may be specific rules for `uextend`/`sextend` in contexts like `iadd`, but no general rule for standalone `uextend`/`sextend` from i32 to i64.

**Solution**: Add lowering rules that extend i32 to i64 by:

- Using the i32 value as the low 32 bits
- Setting the high 32 bits to zero (uextend) or sign-extending (sextend)

**Implementation**:

```isle
;; General uextend from i32 to i64 on riscv32
;; Zero-extend: low 32 bits = input, high 32 bits = 0
(rule (lower (has_type $I64 (uextend x @ (value_type $I32))))
  (value_regs x (imm $I32 0)))

;; General sextend from i32 to i64 on riscv32
;; Sign-extend: low 32 bits = input, high 32 bits = sign bit
(rule (lower (has_type $I64 (sextend x @ (value_type $I32))))
  (let (
      (x_reg XReg x)
      ;; Sign-extend: shift right arithmetic by 31 bits to get sign bit
      (sign_bit XReg (rv_srai x_reg (imm12 31)))
      (high_reg XReg sign_bit)
    )
    (value_regs x_reg high_reg)))
```

### Fix 2: Ensure Function Call Argument Preparation Handles Register Pairs

**File**: `cranelift/codegen/src/isa/riscv32/abi.rs`

**Issue**: When preparing function call arguments, i64 values split into register pairs may not have both registers properly initialized.

**Investigation Points**:

1. Check `gen_call()` function around line 580-650
2. Verify that when an i64 value is used as an argument, both registers in the pair are extracted
3. Ensure `CallArgPair` entries are created for both registers
4. Validate that both registers are valid before creating call info

**Potential Fix**: Ensure that when processing i64 arguments:

- Extract both registers from the `ValueRegs`
- Create `CallArgPair` entries for both registers
- Validate that both registers are valid before creating call info
- Handle cases where one register might be missing

### Fix 3: Fix Return Value Handling for i64

**File**: `cranelift/codegen/src/isa/riscv32/abi.rs`

**Issue**: When functions return i64 values, the return value reconstruction may not properly handle register pairs.

**Fix**: Ensure that:

- Return value pairs are properly extracted from return registers
- Both registers in the pair are valid
- Return value reconstruction handles register pairs correctly

### Fix 4: Fix reg.is_virtual() Assertion Failures

**File**: `cranelift/codegen/src/machinst/reg.rs` or related files

**Issue**: Some operations expect virtual registers but receive physical registers or invalid registers.

**Investigation**: Check where `reg.is_virtual()` assertions fail:

- `bitrev.clif`
- `brif.clif`
- `i64-riscv32.clif`

**Fix**: Ensure that:

- Register pairs are properly converted to virtual registers when needed
- Physical registers are not used where virtual registers are expected
- Invalid registers are caught earlier (validation already added in Phase 6)

### Fix 5: Add Validation for Register Pairs in Lowering

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle` or `lower/isle.rs`

**Enhancement**: Add validation to ensure register pairs are complete before use.

**Implementation**: After creating register pairs, validate that both registers are valid.

### Fix 6: Fix Specific Operations That Create Register Pairs

**Investigation**: Check specific operations that may create incomplete register pairs:

- `uextend.i64` from i32
- `sextend.i64` from i32
- `iconcat` (if used)
- i64 arithmetic operations that produce pairs

## Investigation Steps

### Step 1: Check Current uextend/sextend Lowering Rules

```bash
grep -r "uextend\|sextend" cranelift/codegen/src/isa/riscv32/lower.isle
```

### Step 2: Test uextend/sextend Lowering

Create a minimal test case:

```clif
test run
target riscv32

function %test_uextend(i32) -> i64 {
block0(v0: i32):
    v1 = uextend.i64 v0
    return v1
}

function %test_sextend(i32) -> i64 {
block0(v0: i32):
    v1 = sextend.i64 v0
    return v1
}
```

### Step 3: Trace Function Call Argument Preparation

Add debug logging in `gen_call()` to see how i64 arguments are processed:

- What registers are extracted?
- Are both registers in the pair valid?
- How are `CallArgPair` entries created?

### Step 4: Check ValueRegs Handling

Verify that when an i64 value is used:

- Both registers in the `ValueRegs` are properly extracted
- Both registers are valid virtual registers
- Register pairs are complete before use

### Step 5: Check Return Value Handling

Verify that when functions return i64 values:

- Both return registers are properly extracted
- Return value pairs are correctly reconstructed
- Both registers in the pair are valid

## Testing

After implementing fixes:

```bash
# Test the specific cases that were failing
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/global_value.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/call_indirect.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/call.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/extend.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/i64-riscv32.clif

# Test all riscv32 tests
cargo run --package cranelift-tools --bin clif-util -- test $(find cranelift/filetests/filetests/runtests -name "*.clif" -exec grep -l "^target riscv32" {} \;)
```

## Success Criteria

- ✅ All 15 affected tests pass
- ✅ No more `<invalid>` registers in `CallArgPair` or `RetPair` entries
- ✅ No more register indices >= 1000000 or invalid sentinels
- ✅ No more `reg.is_virtual()` assertion failures
- ✅ uextend/sextend from i32 to i64 properly produces register pairs
- ✅ Function call arguments handle i64 register pairs correctly
- ✅ Return values handle i64 register pairs correctly

## Estimated Time

- Fix 1: 1-2 hours (uextend/sextend lowering rules)
- Fix 2: 3-4 hours (function call argument handling)
- Fix 3: 2-3 hours (return value handling)
- Fix 4: 2-3 hours (reg.is_virtual() assertions)
- Fix 5-6: 2-3 hours (validation and specific operations)

**Total**: 10-15 hours

## Related Issues

- Phase 6: Register allocator issues (similar symptoms but different root cause)
- Phase 8: Global value type conversion (may have introduced uextend that triggers this)
- Phase 6a: `fits_in_64` architecture-awareness (root cause fix)

## Notes

- The index `2097151` (0x1FFFFF) is the invalid sentinel value for virtual registers
- This suggests that one register in a pair is not being properly initialized
- The issue manifests specifically when i64 values created from uextend/sextend are used in function calls
- May also affect other operations that use i64 register pairs
- Validation added in Phase 6 catches these errors early, but root cause needs to be fixed

