# Phase 11 Remaining: Fix Register Allocation Invalid Indices (Part 2)

## Status Update

### ✅ Completed (Part 1)

1. **Fixed uextend lowering for i32->i64**: Changed from `(xreg_to_reg (zero_reg))` to `(imm $I32 0)` to properly create virtual registers for i64 register pairs
2. **Added typed stack_addr rules**: Added specific rules for i32 and i64 types
3. **extend.clif now compiles**: No more "Invalid register indices" errors for uextend operations

### 🔄 Remaining Issues

**Primary Issue**: Operations that return i64 values are creating invalid register pairs where the second register is `<invalid>` (index 2097151).

**Affected Tests**:
- `bitrev.clif` - Returns i64, second register in RetPair is invalid
- `brif.clif` - May have similar issues with i64 values
- `i64-riscv32.clif` - Various i64 operations
- `global_value.clif` - stack_addr.i64 type mismatch issue

## Problem Analysis

### Issue 1: Return Value Register Pairs for i64 Operations

**Error Pattern**:
```
Instruction 37: Rets { rets: [
    RetPair { vreg: v234, preg: p10i }, 
    RetPair { vreg: <invalid>, preg: p11i }  // <-- Invalid register!
]}
```

**Root Cause**: Operations like `bitrev`, `popcnt`, and other i64 operations that return i64 values are not properly creating register pairs. The operation creates a single register result, but when it's used as a return value, the ABI expects two registers (low and high 32-bit parts).

**Location**: `cranelift/codegen/src/machinst/abi.rs::gen_copy_regs_to_retval()`

**Current Behavior**:
- When an i64 operation returns a `ValueRegs` with only one register
- The return value handling code expects `from_regs.len() == slots.len()`
- For i64 returns, there should be 2 slots but only 1 register in `from_regs`
- This causes the second `RetPair` to be created with an invalid register

### Issue 2: Operations That Return i64 Need Register Pair Support

**Affected Operations**:
- `bitrev.i64` - Currently returns single register, needs pair
- `popcnt.i64` - May need pair support
- `iabs.i64` - May need pair support
- Other i64 operations that produce single-register results

**Solution**: These operations need to be updated to return `ValueRegs` with two registers (low and high parts) when targeting riscv32.

### Issue 3: stack_addr.i64 Type Mismatch

**Error Pattern**:
```
assertion `left == right` failed: Aliasing v4 to v0 would change its type i32 to i64
  left: types::I32
 right: types::I64
```

**Root Cause**: On riscv32, addresses are 32-bit (i32), but the frontend is creating `stack_addr.i64` instructions. When the lowering tries to alias the result, it fails because the types don't match.

**Solution Options**:
1. Fix frontend to not create `stack_addr.i64` on riscv32 (preferred)
2. Add legalization pass to convert `stack_addr.i64` to `stack_addr.i32` + `uextend.i64`
3. Handle the type conversion in the lowering rule

## Implementation Plan

### Fix 1: Ensure i64 Operations Return Register Pairs

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Operations to Fix**:
1. `bitrev.i64` - Currently returns single register via `gen_bitrev`
2. `popcnt.i64` - Check if it returns pairs
3. `iabs.i64` - Check if it returns pairs
4. Other i64 operations that may return single registers

**Implementation**:
```isle
;; For bitrev.i64 on riscv32, we need to return a register pair
;; The gen_bitrev function returns a single XReg, but we need two
(rule 0 (lower (has_type $I64 (bitrev x)))
  (let ((result XReg (gen_bitrev $I64 x)))
    ;; For riscv32, i64 values need two registers
    ;; The result is already 64-bit reversed, so we need to split it
    ;; Low 32 bits: result
    ;; High 32 bits: result >> 32 (but we need to handle this properly)
    (value_regs result (imm $I32 0))))  ;; Placeholder - needs proper implementation
```

**Note**: This is complex because `bitrev` operates on the full 64-bit value, so we can't just split it. We may need to:
- Implement a proper 64-bit bitrev that works on register pairs
- Or ensure the operation properly handles the register pair throughout

### Fix 2: Fix gen_copy_regs_to_retval to Handle Incomplete Register Pairs

**File**: `cranelift/codegen/src/machinst/abi.rs`

**Issue**: When `from_regs.len() < slots.len()`, we need to create the missing registers.

**Current Code** (around line 1699):
```rust
assert_eq!(from_regs.len(), slots.len());
```

**Fix**: Instead of asserting, handle the case where we need to create additional registers:
```rust
// If we have fewer registers than slots, we need to create the missing ones
if from_regs.len() < slots.len() {
    // This happens when an i64 operation returns a single register
    // but the ABI expects two registers for i64 return values
    // We need to create the high register (typically zero for most operations)
    // TODO: Determine the correct value for the high register based on the operation
}
```

**Challenge**: We need to know what value to put in the high register. For operations like `bitrev`, this is non-trivial.

### Fix 3: Fix stack_addr.i64 Type Issue

**Option A: Frontend Fix (Preferred)**
- Modify the frontend to never create `stack_addr.i64` on riscv32
- Always use `stack_addr.i32` and extend if needed

**Option B: Legalization Fix**
- Add a legalization pass that converts `stack_addr.i64` to `stack_addr.i32` + `uextend.i64`
- File: `cranelift/codegen/src/legalizer/`

**Option C: Lowering Fix**
- Handle the type conversion in the lowering rule
- Convert i64 result to i32 + uextend

**Recommendation**: Option A is cleanest, but Option B is more robust.

### Fix 4: Add Validation for Register Pairs

**File**: `cranelift/codegen/src/machinst/lower.rs` or `abi.rs`

**Enhancement**: Add validation in `gen_copy_regs_to_retval` to ensure:
- If return type is i64 on riscv32, `from_regs` must have 2 registers
- If it doesn't, create the missing register(s) with appropriate values

## Investigation Steps

### Step 1: Trace bitrev.i64 Return Value Flow

1. Find where `bitrev.i64` is lowered
2. Check what `ValueRegs` it returns
3. Trace how this flows into `gen_copy_regs_to_retval`
4. Identify where the second register should be created

### Step 2: Check All i64 Operations

```bash
# Find all i64 operations in riscv32 lowering
grep -n "has_type.*\$I64" cranelift/codegen/src/isa/riscv32/lower.isle | grep -v "value_type"
```

### Step 3: Understand Register Pair Requirements

- When does an i64 value need to be a register pair?
- When can it be a single register?
- How do we determine the high register value?

### Step 4: Test Each Affected Operation

Create minimal test cases for each operation:
```clif
test run
target riscv32

function %test_bitrev_i64(i64) -> i64 {
block0(v0: i64):
    v1 = bitrev.i64 v0
    return v1
}
```

## Implementation Priority

1. **High Priority**: Fix `gen_copy_regs_to_retval` to handle incomplete pairs
   - This will fix multiple tests at once
   - Estimated: 2-3 hours

2. **Medium Priority**: Fix `bitrev.i64` to return proper register pairs
   - Specific operation fix
   - Estimated: 2-3 hours

3. **Medium Priority**: Fix `stack_addr.i64` type issue
   - May require frontend changes
   - Estimated: 1-2 hours

4. **Low Priority**: Add validation and fix other i64 operations
   - Can be done incrementally
   - Estimated: 3-4 hours

## Testing Strategy

### Test Each Fix Individually

```bash
# Test bitrev fix
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/bitrev.clif

# Test global_value fix
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/global_value.clif

# Test all affected tests
for test in bitrev brif i64-riscv32 global_value; do
    cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/${test}.clif
done
```

### Create Minimal Test Cases

Create test cases that isolate each issue:
- `test_bitrev_i64_return.clif` - Tests bitrev returning i64
- `test_stack_addr_i64.clif` - Tests stack_addr.i64 issue

## Success Criteria

- ✅ `bitrev.clif` passes (no invalid registers in RetPair)
- ✅ `brif.clif` passes (if it has similar issues)
- ✅ `i64-riscv32.clif` passes (all i64 operations work)
- ✅ `global_value.clif` passes (stack_addr.i64 issue resolved)
- ✅ No more `<invalid>` registers in `RetPair` entries
- ✅ All i64 operations return proper register pairs on riscv32

## Estimated Time

- Fix 1 (gen_copy_regs_to_retval): 2-3 hours
- Fix 2 (bitrev.i64): 2-3 hours
- Fix 3 (stack_addr.i64): 1-2 hours
- Fix 4 (other operations): 3-4 hours
- Testing and validation: 2-3 hours

**Total**: 10-15 hours

## Related Work

- Phase 11 Part 1: Fixed uextend lowering (completed)
- Phase 6: Register allocator validation (already catches these errors)
- Phase 3: Basic i64 handling (foundation for this work)

## Notes

- The invalid register index `2097151` (0x1FFFFF) is the sentinel value for invalid virtual registers
- This occurs when `ValueRegs` doesn't have enough registers for the expected register pair
- The fix needs to ensure that all i64 values are represented as register pairs throughout the lowering process
- Some operations may need architecture-specific implementations for riscv32 to properly handle i64 register pairs

