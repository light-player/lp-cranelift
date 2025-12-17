# Phase 9: Proper Implementation Plan - Register Allocation Invalid Indices

## Problem Analysis

### Root Cause

When `uextend` creates an i64 value from i32 on riscv32, it produces `value_regs` containing:

- Low register: the i32 value (actual register)
- High register: `(imm $I32 0)` (immediate, not a register)

When this `ValueRegs` is used in function calls/returns, the ABI code calls `from_regs.regs().iter()` which expects actual `Reg` values. The immediate `0` is not a valid `Reg`, causing invalid register sentinels (index 2097151).

### Why Simple Cases Work

- Simple returns work because the return path may handle immediates differently
- Function calls/returns require actual registers for the ABI to process

### Why `iconst` Works

- `iconst` creates `value_regs` with immediates, but those immediates are materialized to registers when the value is actually used
- The materialization happens during lowering when `put_in_regs` is called
- However, for `uextend`, the immediate in the pair isn't being materialized

## Solution Strategy

### Option 1: Materialize Zero Register in uextend Lowering (Recommended)

**Approach**: When lowering `uextend`, emit an instruction to load zero into a register, then use that register in the pair.

**Pros**:

- Ensures both parts of the pair are actual registers
- Works correctly in all contexts (calls, returns, etc.)
- Follows the pattern used elsewhere in the codebase

**Cons**:

- Requires emitting an extra instruction (but zero is cheap - can use `zero_reg` or `addi x0, x0, 0`)

### Option 2: Materialize Immediates in ValueRegs During Function Call Prep

**Approach**: In the ABI code, detect immediates in `ValueRegs` and materialize them before use.

**Pros**:

- Handles all cases where immediates appear in pairs
- More general solution

**Cons**:

- More complex - need to detect immediates vs registers
- May have performance implications
- Less clear separation of concerns

### Option 3: Use zero_reg Directly

**Approach**: Use the architectural zero register (`x0`) directly instead of an immediate.

**Pros**:

- No extra instruction needed
- Zero register is always available

**Cons**:

- Need to verify `zero_reg` can be used in `ValueRegs` pairs
- May not work if zero_reg is special-cased elsewhere

## Recommended Implementation: Option 1

### Step 1: Understand Current uextend Lowering

**Current code** (`cranelift/codegen/src/isa/riscv32/lower.isle`):

```isle
;; On RV32, extending to I64 requires 2 registers
(rule 1 (lower (has_type $I64 (uextend val)))
  (value_regs val (imm $I32 0)))
```

**Problem**: `(imm $I32 0)` creates an immediate, not a register.

### Step 2: Materialize Zero Register

We need to emit an instruction to put zero in a register. Options:

**Option A: Use zero_reg helper**

```isle
(rule 1 (lower (has_type $I64 (uextend val)))
  (value_regs val (zero_reg)))
```

**Option B: Emit instruction to load zero**

```isle
(rule 1 (lower (has_type $I64 (uextend val)))
  (let ((zero_reg XReg (zero_reg)))
    (value_regs val zero_reg)))
```

**Option C: Use iconst to create zero value, then put in reg**

```isle
(rule 1 (lower (has_type $I64 (uextend val)))
  (let ((zero_val Value (iconst $I32 0))
        (zero_reg Reg (put_in_reg zero_val)))
    (value_regs val zero_reg)))
```

### Step 3: Verify zero_reg Usage

Check if `zero_reg` can be used directly in `value_regs`:

- Look at how `zero_reg` is defined in `inst.isle`
- Verify it returns a `Reg` (not a special type)
- Check if it's used elsewhere in `value_regs`

### Step 4: Test Implementation

1. **Simple test**: Verify uextend still works for simple returns
2. **Function call test**: Test with `global_value.clif`
3. **Function return test**: Test with `call_indirect.clif`

## Implementation Steps

### Step 1: Investigate zero_reg Availability

**File**: `cranelift/codegen/src/isa/riscv32/inst.isle`

Check:

- Is `zero_reg` declared? What does it return?
- Can it be used in `value_regs`?
- Are there any restrictions?

**Command**:

```bash
grep -n "zero_reg\|decl.*zero" cranelift/codegen/src/isa/riscv32/inst.isle
```

### Step 2: Check Existing Patterns

Look for other places where zero registers are created:

- How does `iconst` handle zero?
- Are there patterns for creating zero registers?

**Command**:

```bash
grep -n "iconst.*0\|imm.*0" cranelift/codegen/src/isa/riscv32/lower.isle | head -20
```

### Step 3: Implement Materialization

**✅ Solution: Use zero_reg with conversion**

Based on investigation:

- `zero_reg` returns `XReg` (declared in `inst.isle` line 3110)
- `xreg_to_reg` converts `XReg` to `Reg` (declared in `inst.isle` line 853)
- `value_regs` requires `Reg` values (declared in `prelude_lower.isle` line 47)
- `zero_reg` is already used in `value_regs` elsewhere (e.g., line 1715 in `lower.isle`)

**Implementation**:

```isle
;; On RV32, extending to I64 requires 2 registers
(rule 1 (lower (has_type $I64 (uextend val)))
  (value_regs val (xreg_to_reg (zero_reg))))
```

**Alternative if automatic conversion works**:

```isle
;; On RV32, extending to I64 requires 2 registers
(rule 1 (lower (has_type $I64 (uextend val)))
  (value_regs val (zero_reg)))
```

**Note**: `val` is a `Value`, so we need to ensure it's converted to a register. The current code uses `val` directly, which suggests ISLE may handle the conversion automatically, or `val` is already a register in this context. If not, we may need:

```isle
(rule 1 (lower (has_type $I64 (uextend val)))
  (value_regs (put_in_reg val) (xreg_to_reg (zero_reg))))
```

### Step 4: Verify Type Compatibility

Ensure the types match:

- `val` should be `Reg` (from `put_in_reg` or similar)
- Second part should be `Reg` (not `Imm`)
- `value_regs` should accept two `Reg` values

### Step 5: Test and Validate

1. **Compile test**:

```bash
cargo build --package cranelift-tools
```

2. **Simple uextend test**:

```bash
cargo run --package cranelift-tools --bin clif-util -- test /tmp/test_uextend.clif
```

3. **Complex tests**:

```bash
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/global_value.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/call_indirect.clif
```

## Alternative: Fix in ABI Layer

If materialization in ISLE doesn't work, we can fix it in the ABI layer:

### Location

`cranelift/codegen/src/machinst/abi.rs` around line 1915

### Implementation

```rust
for (slot, from_reg) in slots.iter().zip(from_regs.regs().iter()) {
    // Materialize immediate if needed
    let actual_reg = if from_reg.is_invalid_sentinel() {
        // This shouldn't happen, but handle gracefully
        panic!("Invalid register in function call argument");
    } else if /* check if from_reg is immediate */ {
        // Materialize immediate to register
        let tmp = vregs.alloc_with_deferred_error(word_ty).only_reg().unwrap();
        insts.push(M::gen_move(
            Writable::from_reg(tmp),
            /* materialize immediate */,
            word_ty,
        ));
        tmp
    } else {
        *from_reg
    };
    // ... rest of processing
}
```

**Problem**: This is harder because we need to detect if a `Reg` is actually an immediate, which may not be possible at this layer.

## Decision Matrix

| Approach                      | Complexity | Performance | Correctness | Recommendation             |
| ----------------------------- | ---------- | ----------- | ----------- | -------------------------- |
| Option 1: Materialize in ISLE | Low        | Good        | High        | ✅ **Recommended**         |
| Option 2: Fix in ABI          | Medium     | Good        | Medium      | Consider if Option 1 fails |
| Option 3: Use zero_reg        | Very Low   | Best        | High        | ✅ **Try first**           |

## Next Steps

1. ✅ **Investigated** `zero_reg` availability - confirmed it exists and returns `XReg`
2. ✅ **Found conversion** - `xreg_to_reg` converts `XReg` to `Reg`
3. **Implement** using `zero_reg` with `xreg_to_reg` conversion
4. **Test thoroughly** with both simple and complex cases
5. **Document** the solution and why it works

## Implementation Code

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Change** (line 1573):

```isle
;; On RV32, extending to I64 requires 2 registers
(rule 1 (lower (has_type $I64 (uextend val)))
  (value_regs val (xreg_to_reg (zero_reg))))
```

**Rationale**:

- `zero_reg` provides the architectural zero register (`x0`)
- `xreg_to_reg` converts it to `Reg` type required by `value_regs`
- This ensures both parts of the register pair are actual registers, not immediates
- Works correctly in all contexts (function calls, returns, etc.)

## Success Criteria

- ✅ `global_value.clif` compiles and runs without register errors
- ✅ `call_indirect.clif` compiles and runs without register errors
- ✅ No more `<invalid>` registers in `CallArgPair` or `RetPair` entries
- ✅ No more register indices >= 1000000 or invalid sentinels
- ✅ uextend from i32 to i64 properly produces register pairs with actual registers
