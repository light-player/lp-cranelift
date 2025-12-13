# Phase 9: Fix Register Allocation Invalid Indices

## Goal
Fix invalid register indices (specifically `<invalid>` registers with index 2097151) that occur when i64 values created from uextend operations are used in function calls on riscv32.

## Prerequisites
- Phase 8 completed: Global value type conversion implemented (uextend added for vmctx)
- Phase 6 completed: Basic register allocation understanding

## Problem Analysis

### Root Cause
When `vmctx_addr()` legalizer function converts i32 vmctx to i64 using `uextend`, it creates an i64 value that must be represented as a register pair (two i32 registers). However, when this i64 value is used in function calls:

1. The `uextend` operation may not be properly lowered to produce both registers in the pair
2. Function call argument preparation may not properly handle i64 register pairs
3. One register in the pair ends up as `<invalid>` (index 2097151 = 0x1FFFFF)

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

### Fix 1: Add General uextend Lowering Rule for i32->i64

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Issue**: There are specific rules for `uextend` in contexts like `iadd`, but no general rule for standalone `uextend` from i32 to i64.

**Solution**: Add a lowering rule that zero-extends i32 to i64 by:
- Using the i32 value as the low 32 bits
- Setting the high 32 bits to zero

**Implementation**:
```isle
;; General uextend from i32 to i64 on riscv32
;; Zero-extend: low 32 bits = input, high 32 bits = 0
(rule (lower (has_type $I64 (uextend x @ (value_type $I32))))
  (value_regs x (imm $I32 0)))
```

### Fix 2: Ensure Function Call Argument Preparation Handles Register Pairs

**File**: `cranelift/codegen/src/isa/riscv32/abi.rs`

**Issue**: When preparing function call arguments, i64 values split into register pairs may not have both registers properly initialized.

**Investigation Points**:
1. Check `gen_call()` function around line 580-650
2. Verify that when an i64 value is used as an argument, both registers in the pair are extracted
3. Ensure `CallArgPair` entries are created for both registers

**Potential Fix**: Ensure that when processing i64 arguments:
- Extract both registers from the `ValueRegs`
- Create `CallArgPair` entries for both registers
- Validate that both registers are valid before creating call info

### Fix 3: Add Validation for Register Pairs

**File**: `cranelift/codegen/src/machinst/compile.rs`

**Enhancement**: Improve the existing validation to catch invalid registers in register pairs earlier.

**Current code** (lines 65-72) checks for large indices but doesn't check for invalid sentinels.

**Enhancement**:
```rust
inst.get_operands(&mut |reg: &mut Reg, _, _, _| {
    if reg.is_invalid_sentinel() {
        invalid_regs.push((iix, reg.clone(), "invalid_sentinel"));
    } else if let Some(vreg) = reg.to_virtual_reg() {
        let index = vreg.index();
        if index >= 1000000 {
            invalid_regs.push((iix, reg.clone(), index));
        }
    }
});
```

### Fix 4: Alternative Approach - Avoid uextend in Legalizer

**File**: `cranelift/codegen/src/legalizer/globalvalue.rs`

**Alternative**: Instead of using `uextend` to convert i32 vmctx to i64, directly create a register pair:
- Low register: the i32 vmctx value
- High register: zero constant

**Implementation**:
```rust
if vmctx_ty == ir::types::I32 && result_ty == ir::types::I64 {
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    
    // Create i64 value as register pair: (vmctx, 0)
    let zero = pos.ins().iconst(ir::types::I32, 0);
    // Use a helper to create value_regs from two i32 values
    // This avoids uextend which may not be properly lowered
    let pair = pos.ins().iconcat(vmctx, zero); // If iconcat exists
    // Or manually construct the pair
    pos.func.dfg.replace(inst).copy(pair);
}
```

**Note**: This approach requires checking if there's a way to create register pairs directly, or if we need to add support for it.

## Investigation Steps

### Step 1: Check if uextend Lowering Rule Exists
```bash
grep -r "lower.*uextend\|uextend.*lower" cranelift/codegen/src/isa/riscv32/
```

### Step 2: Test uextend Lowering
Create a minimal test case:
```clif
test run
target riscv32

function %test_uextend(i32) -> i64 {
block0(v0: i32):
    v1 = uextend.i64 v0
    return v1
}
```

### Step 3: Trace Function Call Argument Preparation
Add debug logging in `gen_call()` to see how i64 arguments are processed.

### Step 4: Check ValueRegs Handling
Verify that when an i64 value is used, both registers in the `ValueRegs` are properly extracted.

## Testing

After implementing fixes:

```bash
# Test the specific cases that were failing
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/global_value.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/call_indirect.clif

# Test with a minimal uextend case
# (create test file first)
cargo run --package cranelift-tools --bin clif-util -- test /tmp/test_uextend.clif
```

## Success Criteria

- `global_value.clif` compiles and runs without register allocation errors
- `call_indirect.clif` compiles and runs without register allocation errors  
- No more `<invalid>` registers in `CallArgPair` entries
- No more register indices >= 1000000
- uextend from i32 to i64 properly produces register pairs

## Implementation Priority

1. **Fix 1** (Add uextend lowering rule) - Highest priority, likely root cause
2. **Fix 3** (Enhanced validation) - Helps catch issues earlier
3. **Fix 2** (Function call argument handling) - May be needed if Fix 1 doesn't fully resolve
4. **Fix 4** (Alternative legalizer approach) - Fallback if Fix 1 doesn't work

## Related Issues

- Phase 6: Register allocator issues (similar symptoms but different root cause)
- Phase 8: Global value type conversion (introduced the uextend that triggers this)

## Notes

- The index `2097151` (0x1FFFFF) is the invalid sentinel value for virtual registers
- This suggests that one register in a pair is not being properly initialized
- The issue manifests specifically when i64 values created from uextend are used in function calls
- May also affect other operations that use i64 register pairs

