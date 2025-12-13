# Option 2: Fix ISLE Patterns for Register Pairs (Proper Solution)

## Goal
Update riscv32 ISLE patterns to properly handle i64 values stored in register pairs, ensuring semantic correctness throughout the lowering process.

## Quick Reference

**Priority Order**:
1. **Phase 2** (Immediate Extraction) - Fixes the panic, start here
2. **Phase 3** (Arithmetic) - Most common operations
3. **Phase 4** (Calls) - May already work via ABI, verify first
4. **Phase 5** (Memory) - If needed
5. **Phase 6** (Control Flow) - If needed
6. **Phase 7** (Cleanup) - Remove Option 1 workaround

**Key Files**:
- `cranelift/codegen/src/isa/riscv32/lower.isle` - Main lowering patterns
- `cranelift/codegen/src/isa/riscv32/inst.isle` - Instruction definitions and extractors

**Common Patterns**:
- Extract from pair: `(value_regs_get regs 0)` for low, `(value_regs_get regs 1)` for high
- Create pair: `(value_regs low_reg high_reg)`
- Check if pair: `(if-let (value_regs_get regs 1) high_reg ...)`

## Overview

riscv32 stores i64 values in register pairs (two 32-bit registers: low/high), but current ISLE patterns assume single-register handling. This plan systematically updates all affected patterns to work correctly with register pairs.

## Architecture Understanding

### Register Pair Representation
- i64 values: `ValueRegs` with 2 registers `[low_32bits, high_32bits]`
- i32 values: `ValueRegs` with 1 register `[value]`
- Other types: Single registers (F32, F64, etc.)

### Key ISLE Functions
- `value_regs_get regs index`: Get register at index (0=low, 1=high)
- `value_regs reg1 reg2`: Create ValueRegs from two registers
- `put_in_regs val`: Get ValueRegs for a value (may be pair for i64)
- `put_in_reg val`: Get single register (fails for pairs - this is the problem)

## Phase 1: Analysis and Categorization

### Step 1.1: Identify All Patterns Using `put_in_reg`

**Command**:
```bash
grep -n "put_in_reg" cranelift/codegen/src/isa/riscv32/lower.isle
grep -n "put_in_reg" cranelift/codegen/src/isa/riscv32/inst.isle
```

**Expected findings**:
- Patterns extracting immediates from values
- Patterns handling function pointers
- Patterns doing register-to-register operations

### Step 1.2: Identify Immediate Extraction Patterns

**Command**:
```bash
grep -n "imm12_from_value\|imm5_from_value\|imm20_from_value" cranelift/codegen/src/isa/riscv32/
```

**Patterns to check**:
- `imm12_from_value`: Used in arithmetic with immediates
- `imm5_from_value`: Used in shift operations
- `imm20_from_value`: Used in LUI/AUIPC

### Step 1.3: Categorize Patterns by Type

**Category A: Immediate Extraction**
- Patterns that extract immediate values from i64 constants
- Must handle: i64 constants split into register pairs
- Solution: Check if value is register pair, extract from low register if it's a constant

**Category B: Arithmetic Operations**
- iadd, isub, imul with immediates
- Must handle: Operands in register pairs
- Solution: Operate on low register, propagate carry to high register

**Category C: Call Arguments/Returns**
- Function call argument passing
- Return value handling
- Must handle: i64 arguments/returns as register pairs
- Solution: Ensure ABI correctly handles pairs (should already work, but verify)

**Category D: Memory Operations**
- Load/store of i64 values
- Must handle: Register pairs for addresses and values
- Solution: Use low register for address, handle value pairs correctly

**Category E: Control Flow**
- Branches, jumps with i64 comparisons
- Must handle: Comparing register pairs
- Solution: Compare low registers, then high if low are equal

## Phase 2: Fix Immediate Extraction

### Problem
`imm12_from_value` extractor tries to extract immediate from i64 values that are in register pairs. It calls `put_in_reg()` which fails.

### Solution: Update `imm12_from_value` Extractor

**File**: `cranelift/codegen/src/isa/riscv32/inst.isle`

**Current code** (line ~1867):
```isle
(extractor (imm12_from_value n) (i64_from_iconst (imm12_from_i64 n)))
```

**New approach**: Handle register pairs by checking if value is a constant in a register pair:

```isle
;; Helper: Extract immediate from value, handling register pairs
(decl pure partial imm12_from_value_pair (ValueRegs) Imm12)
(rule (imm12_from_value_pair (value_regs (imm $I32 low) (imm $I32 high)))
  ;; If both parts are immediates, reconstruct i64 and extract imm12
  (if-let (imm12_from_i64) imm (i64_from_parts low high)
    imm))

;; Updated extractor: try single register first, then pair
(extractor (imm12_from_value n)
  (or (i64_from_iconst (imm12_from_i64 n))
      (if-let (put_in_regs n) regs
        (imm12_from_value_pair regs))))
```

**Alternative simpler approach**: If the value is in a register pair, extract from the low register only (for small immediates that fit in 32 bits):

```isle
;; Updated extractor: handle register pairs
(extractor (imm12_from_value n)
  (or 
    ;; Try direct constant extraction
    (i64_from_iconst (imm12_from_i64 n))
    ;; If value is in register pair, extract from low register
    (if-let (put_in_regs n) regs
      (if-let (value_regs_get regs 0) low_reg
        (if-let (imm12_from_value low_reg) imm
          imm)))))
```

### Testing
```bash
# Test immediate extraction with i64 constants
cargo build --package cranelift-tools
./target/debug/clif-util test cranelift/filetests/filetests/runtests/call.clif
```

## Phase 3: Fix Arithmetic Patterns

### Problem
Arithmetic operations with i64 values assume single registers, but riscv32 uses pairs.

### Solution: Update Arithmetic Rules

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

### 3.1: iadd_imm with i64

**Current pattern** (if exists):
```isle
(rule (lower (has_type $I64 (iadd_imm x (imm12_from_value y))))
  (alu_rr_imm12 (select_addi $I64) x y))
```

**New pattern**:
```isle
;; iadd_imm for i64: add immediate to low register, propagate carry
(rule (lower (has_type $I64 (iadd_imm x (imm12_from_value y))))
  (let ((low_reg XReg (value_regs_get x 0))
        (high_reg XReg (value_regs_get x 1))
        ;; Add immediate to low register
        (low_result XReg (alu_rr_imm12 (select_addi $I32) low_reg y))
        ;; Check for carry: if low_result < low_reg (unsigned), carry = 1
        (carry XReg (rv_sltu low_result low_reg))
        ;; Add carry to high register
        (high_result XReg (rv_add high_reg carry)))
    (value_regs low_result high_result)))
```

### 3.2: isub_imm with i64

**New pattern**:
```isle
;; isub_imm for i64: subtract immediate from low register, handle borrow
(rule (lower (has_type $I64 (isub_imm x (imm12_from_value y))))
  (let ((low_reg XReg (value_regs_get x 0))
        (high_reg XReg (value_regs_get x 1))
        ;; Subtract immediate from low register
        (low_result XReg (alu_rr_imm12 (select_subi $I32) low_reg y))
        ;; Check for borrow: if low_result > low_reg (unsigned), borrow = 1
        (borrow XReg (rv_sltu low_reg low_result))
        ;; Subtract borrow from high register
        (high_result XReg (rv_sub high_reg borrow)))
    (value_regs low_result high_result)))
```

### 3.3: Other Arithmetic Operations

Similar patterns needed for:
- `imul_imm`: Multiply low register, handle overflow
- `ishl_imm`: Shift both registers
- `ishr_imm`: Shift both registers with sign extension
- `ushr_imm`: Shift both registers

### Testing
```bash
# Test arithmetic operations
./target/debug/clif-util test cranelift/filetests/filetests/runtests/i64-riscv32.clif
```

## Phase 4: Fix Call Patterns

### Problem
Call patterns may assume single registers for function pointers or arguments.

### Solution: Verify and Update Call Patterns

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

### 4.1: Function Pointer Extraction

**Current pattern** (line ~2989):
```isle
(rule (lower (call_indirect sig_ref ptr args))
  (let ((target Reg (put_in_reg ptr))
        ...))
```

**Issue**: If `ptr` is somehow in a register pair (shouldn't happen, but verify)

**Solution**: Ensure function pointers are always single registers (they should be, but add assertion):

```isle
(rule (lower (call_indirect sig_ref ptr args))
  (let ((ptr_regs ValueRegs (put_in_regs ptr))
        (target Reg (value_regs_get ptr_regs 0))  ;; Always use first register
        ...))
```

### 4.2: Call Arguments

**Current pattern** (line ~2968):
```isle
(rule (lower (call (func_ref_data sig_ref name (RelocDistance.Near)) args))
  (let ((output ValueRegsVec (gen_call_output sig_ref))
        (abi Sig (abi_sig sig_ref))
        (uses CallArgList (gen_call_args abi args))
        ...))
```

**Verification**: The `gen_call_args` function should already handle register pairs correctly via the ABI. Verify this works.

**If needed**: Update to ensure args are properly converted to ValueRegs:

```isle
;; Helper: Convert ValueSlice to ValueRegsVec, handling pairs
(decl gen_call_args_from_slice (Sig ValueSlice) CallArgList)
(extern constructor gen_call_args_from_slice gen_call_args_from_slice)

(rule (lower (call (func_ref_data sig_ref name (RelocDistance.Near)) args))
  (let ((output ValueRegsVec (gen_call_output sig_ref))
        (abi Sig (abi_sig sig_ref))
        ;; Convert args to ValueRegsVec (handles pairs automatically)
        (uses CallArgList (gen_call_args_from_slice abi args))
        ...))
```

### Testing
```bash
# Test all call variants
./target/debug/clif-util test cranelift/filetests/filetests/runtests/call.clif
./target/debug/clif-util test cranelift/filetests/filetests/runtests/call_indirect.clif
./target/debug/clif-util test cranelift/filetests/filetests/runtests/return-call.clif
```

## Phase 5: Fix Memory Operations

### Problem
Load/store operations may not handle i64 register pairs correctly.

### Solution: Update Load/Store Patterns

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

### 5.1: i64 Load

**Current pattern** (if exists):
```isle
(rule (lower (has_type $I64 (load ty amode)))
  (load_op ty amode))
```

**New pattern**: Load low and high parts separately:

```isle
(rule (lower (has_type $I64 (load ty amode)))
  (let ((low_addr AMode amode)
        (high_addr AMode (gen_reg_offset_amode (amode_base amode) (amode_offset amode + 4)))
        (low_reg XReg (load_op $I32 low_addr))
        (high_reg XReg (load_op $I32 high_addr)))
    (value_regs low_reg high_reg)))
```

### 5.2: i64 Store

**New pattern**: Store both parts:

```isle
(rule (lower (has_type $I64 (store val amode)))
  (let ((val_regs ValueRegs (put_in_regs val))
        (low_reg XReg (value_regs_get val_regs 0))
        (high_reg XReg (value_regs_get val_regs 1))
        (low_addr AMode amode)
        (high_addr AMode (gen_reg_offset_amode (amode_base amode) (amode_offset amode + 4))))
    (side_effect (store_op $I32 low_reg low_addr))
    (side_effect (store_op $I32 high_reg high_addr))))
```

### Testing
```bash
# Test memory operations
./target/debug/clif-util test cranelift/filetests/filetests/runtests/i64-riscv32.clif
```

## Phase 6: Fix Control Flow

### Problem
Branches and comparisons with i64 values need to handle register pairs.

### Solution: Update Comparison Patterns

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

### 6.1: i64 Comparisons

**New pattern**: Compare high registers first, then low if high are equal:

```isle
;; icmp for i64: compare register pairs
(rule (lower (has_type $I64 (icmp cond x y)))
  (let ((x_low XReg (value_regs_get x 0))
        (x_high XReg (value_regs_get x 1))
        (y_low XReg (value_regs_get y 0))
        (y_high XReg (value_regs_get y 1))
        ;; Compare high parts first
        (high_cmp XReg (int_compare cond x_high y_high))
        ;; Compare low parts
        (low_cmp XReg (int_compare cond x_low y_low))
        ;; If high parts equal, use low comparison; otherwise use high
        (high_eq XReg (rv_eq x_high y_high))
        (result XReg (select high_eq low_cmp high_cmp)))
    (value_regs result (zero_reg))))
```

### Testing
```bash
# Test control flow
./target/debug/clif-util test cranelift/filetests/filetests/runtests/br.clif
```

## Phase 7: Remove Option 1 Workaround

Once all patterns are fixed:

1. **Remove `put_in_reg` override** from `isle.rs`
2. **Verify** that `put_in_reg` now works correctly (should only be called on single-register values)
3. **Test** all affected files

## Implementation Strategy

### Incremental Approach

1. **Start with immediate extraction** (Phase 2) - fixes the panic
2. **Fix arithmetic** (Phase 3) - most common operations
3. **Verify calls** (Phase 4) - may already work via ABI
4. **Fix memory** (Phase 5) - if needed
5. **Fix control flow** (Phase 6) - if needed
6. **Remove workaround** (Phase 7) - final cleanup

### Testing After Each Phase

After each phase:
1. Build and test affected files
2. Verify no regressions
3. Check that Option 1 workaround still works as fallback
4. Document any patterns that still need fixing

## Verification Checklist

- [ ] Immediate extraction works for i64 constants
- [ ] Arithmetic operations handle register pairs correctly
- [ ] Call arguments/returns work with i64 pairs
- [ ] Memory operations handle pairs correctly
- [ ] Control flow comparisons work with pairs
- [ ] No panics occur
- [ ] Tests pass with correct results
- [ ] Option 1 workaround can be removed

## Common Patterns Reference

### Extracting from Register Pair
```isle
(let ((low_reg XReg (value_regs_get regs 0))
      (high_reg XReg (value_regs_get regs 1)))
  ...)
```

### Creating Register Pair
```isle
(value_regs low_reg high_reg)
```

### Checking if ValueRegs is Pair
```isle
(if-let (value_regs_get regs 1) high_reg
  ;; It's a pair
  ...
  ;; It's single register
  ...)
```

## Notes

- riscv32 ABI should already handle register pairs correctly for calls
- Most patterns only need to handle the low register for immediate operations
- High register operations are needed for full-width arithmetic
- Some patterns may be able to use existing riscv64 patterns as reference (but riscv64 uses single registers, so be careful)

## Success Criteria

- All 6 call-related tests pass with correct results
- No panics occur
- i64 operations work correctly throughout
- Code is maintainable and follows ISLE best practices

