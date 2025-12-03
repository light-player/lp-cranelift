# RISC-V32 ISLE Cleanup Guide

## Overview

This guide documents the specific changes needed to convert riscv64 ISLE files to clean riscv32 versions.

**Problem**: The riscv64 ISLE backend generates RV64-specific instructions (like `addw`) that don't exist in RV32.

**Current Issue**: Generated code uses opcode `0x3b` (OP-32, for RV64 word operations) instead of `0x33` (OP, for RV32/64 base operations).

**Example**: For `iadd i32`, we're getting `addw` (RV64) instead of `add` (RV32).

## Files to Modify

1. `cranelift/codegen/src/isa/riscv32/inst.isle` - Instruction definitions
2. `cranelift/codegen/src/isa/riscv32/lower.isle` - Lowering rules
3. `cranelift/codegen/src/isa/riscv32/inst_vector.isle` - Vector instructions (minimal changes)

## Part 1: inst.isle Changes

### A. Remove RV64-specific enum variants

#### 1. In `AluOPRRR` enum (lines ~483-564):

**Remove these variants:**

```isle
;; RV64I Base Instruction Set (in addition to RV32I)
(Addw)
(Subw)
(Sllw)
(Srlw)
(Sraw)

;; RV64M Standard Extension (in addition to RV32M)
(Mulw)
(Divw)
(Divuw)
(Remw)
(Remuw)

;; Zbb word variants
(Rolw)
(Rorw)

;; Zbkb word variant
(Packw)
```

**Keep everything else** in AluOPRRR (Add, Sub, Mul, Div, etc. are used by both RV32 and RV64)

#### 2. In `AluOPRRI` enum (lines ~588-628):

**Remove these variants:**

```isle
;; RV64 immediate operations
(Addiw)
(Slliw)
(SrliW)
(Sraiw)

;; Zbb word variants
(Clzw)
(Ctzw)
(Cpopw)
(Roriw)
```

**Keep**: Addi, Slti, SltiU, Xori, Ori, Andi, Slli, Srli, Srai, Clz, Ctz, Cpop, etc.

#### 3. In `CaOp` enum (compressed) (lines ~648-655):

**Remove:**

```isle
(CAddw)
(CSubw)
```

#### 4. In `CiOp` enum (compressed immediate) (lines ~664-677):

**Remove:**

```isle
(CAddiw)
```

#### 5. In `AtomicOP` enum (lines ~372-395):

**KEEP ALL `*W` VARIANTS** - These are available in both RV32A and RV64A. On RV32, "word" = 32 bits, so these are correct for 32-bit atomics:

```isle
(LrW)
(ScW)
(AmoswapW)
(AmoaddW)
(AmoxorW)
(AmoandW)
(AmoorW)
(AmominW)
(AmomaxW)
(AmominuW)
(AmomaxuW)
```

**Also keep**: LrD, ScD, AmoswapD, etc. (64-bit atomics for doubleword operations, available on both RV32 and RV64)

### B. Remove RV64-specific helper function declarations and rules

Search for and remove all functions starting around line ~1221:

```isle
;; RV64I Base Integer Instruction Set
;; Unlike RV32I instructions these are only present in the 64bit ISA

(decl rv_addw (XReg XReg) XReg)
(rule (rv_addw rs1 rs2)
  (alu_rrr (AluOPRRR.Addw) rs1 rs2))

(decl rv_addiw (XReg Imm12) XReg)
(rule (rv_addiw rs1 imm)
  (alu_rr_imm12 (AluOPRRI.Addiw) rs1 imm))

(decl rv_sextw (XReg) XReg)
(rule (rv_sextw rs1)
  (rv_addiw rs1 (imm12_const 0)))

(decl rv_subw (XReg XReg) XReg)
(rule (rv_subw rs1 rs2)
  (alu_rrr (AluOPRRR.Subw) rs1 rs2))

(decl rv_sllw (XReg XReg) XReg)
(rule (rv_sllw rs1 rs2)
  (alu_rrr (AluOPRRR.Sllw) rs1 rs2))

(decl rv_slliw (XReg Imm12) XReg)
(rule (rv_slliw rs1 imm)
  (alu_rr_imm12 (AluOPRRI.Slliw) rs1 imm))

(decl rv_srlw (XReg XReg) XReg)
(rule (rv_srlw rs1 rs2)
  (alu_rrr (AluOPRRR.Srlw) rs1 rs2))

(decl rv_srliw (XReg Imm12) XReg)
(rule (rv_srliw rs1 imm)
  (alu_rr_imm12 (AluOPRRI.SrliW) rs1 imm))

(decl rv_sraw (XReg XReg) XReg)
(rule (rv_sraw rs1 rs2)
  (alu_rrr (AluOPRRR.Sraw) rs1 rs2))

(decl rv_sraiw (XReg Imm12) XReg)
(rule (rv_sraiw rs1 imm)
  (alu_rr_imm12 (AluOPRRI.Sraiw) rs1 imm))

;; And similar for mulw, divw, remw, etc.
```

**Replace with:** A comment like `;;  NOTE: RV64-specific *w instruction helpers removed for RV32`

### C. Remove Zbb word-operation helpers

Around lines ~1650-1740, remove:

- rv_clzw
- rv_ctzw
- rv_cpopw
- rv_rolw
- rv_rorw
- rv_roriw
- rv_packw

## Part 2: lower.isle Changes

### Key Issue: `select_addi` and Type Selection

Around line ~2182-2184 in riscv64/lower.isle:

```isle
(decl select_addi (Type) AluOPRRI)
(rule 1 (select_addi (fits_in_32 ty)) (AluOPRRI.Addiw))
(rule (select_addi (fits_in_64 ty)) (AluOPRRI.Addi))
```

**For RV32, change to:**

```isle
(decl select_addi (Type) AluOPRRI)
(rule (select_addi ty) (AluOPRRI.Addi))  ;; Always use Addi on RV32
```

### Rules Using `fits_in_32` and Type Predicates

Many lowering rules check `fits_in_32` and use `*w` instructions. For RV32:

- Remove or modify any rule that uses `fits_in_32` to select RV64-specific `*w` instructions (addw, subw, mulw, etc.)
- These rules are trying to optimize for 32-bit values on 64-bit arch - not needed on RV32
- Replace with appropriate type predicates like `ty_int` or `ty_int_ref_scalar_64`

**Search for and remove/modify:**

- Rules matching `(fits_in_32 ...)` that call RV64-specific `*w` instructions (addw, subw, mulw, divw, etc.)
- Rules for `I32` or `I64` types that incorrectly use RV64-specific `*w` variants
- Replace with base instructions (add, sub, mul, div, etc.) that work on full XLEN

**Note**: Atomic `*w` operations should remain - they're correct for 32-bit atomics on RV32.

### Atomic Operations

**IMPORTANT CORRECTION**: Based on RISC-V ISA specification, atomic `*w` (word) operations are available in BOTH RV32A and RV64A. On RV32, a "word" is 32 bits, so `*w` variants are the CORRECT operations for 32-bit atomics.

**DO NOT REMOVE** atomic `*w` operations. They should remain in the `AtomicOP` enum and be used for 32-bit atomic operations on RV32.

The current implementation is correct:

```isle
(rule
  (get_atomic_rmw_op $I32 (AtomicRmwOp.Add))
  (AtomicOP.AmoaddW))  ;; Correct for RV32 - word = 32 bits
```

**What to remove**: Only the RV64-specific word operations from RV64I/RV64M (addw, subw, mulw, etc.), NOT atomic word operations.

## Part 3: inst_vector.isle Changes

Check for vector word-width operations. These are less common but may exist:

```isle
(VwaddWV)  ;; Vector widening add word
(VwsubWV)  ;; Vector widening sub word
(VwaddWX)
(VwsubWX)
```

Remove if present (lines ~69-72 in riscv64 version).

## Part 4: Verification Strategy

After making changes:

1. **Syntax check**: `cargo build --package cranelift-codegen --features riscv32,std`
2. **Instruction check**: Run `simple_codegen` example and verify:
   - Opcode should be `0x33` (OP) not `0x3b` (OP-32)
   - Should generate `add a0, a0, a1` not `addw a0, a0, a1`
3. **Decode check**:
   ```python
   # Expected for RV32 add a0, a0, a1:
   # opcode=0x33, rd=a0 (x10), funct3=0x0, rs1=a0 (x10), rs2=a1 (x11), funct7=0x00
   # Encoding: 0x00b50533
   ```

## Part 5: Systematic Approach

### Option A: Surgical Edits (Recommended)

1. Start with `inst.isle`:
   - Remove enum variants one section at a time
   - Remove helper functions (rv\_\*w) one section at a time
   - Test build after each section
2. Move to `lower.isle`:

   - Fix `select_addi` and similar functions
   - Remove/modify `fits_in_32` rules
   - Fix atomic operation rules
   - Test build

3. Fix `inst_vector.isle`:
   - Remove vector \*w operations if any
   - Test build

### Option B: Template-Based Rewrite

Create new files from scratch with only RV32-relevant content:

1. Copy structure from riscv64
2. Include only non-\*w variants
3. Adjust lowering rules for 32-bit arch
4. Test incrementally

## Quick Reference: RV64-Only Instructions

**Integer:**

- `addw`, `addiw`, `subw` - Word arithmetic
- `sllw`, `slliw` - Word shift left
- `srlw`, `srliw` - Word shift right logical
- `sraw`, `sraiw` - Word shift right arithmetic
- `mulw`, `divw`, `divuw`, `remw`, `remuw` - Word multiply/divide

**Bit manipulation:**

- `clzw`, `ctzw`, `cpopw` - Word count leading/trailing zeros, population count
- `rolw`, `rorw`, `roriw` - Word rotate left/right
- `packw` - Pack word

**Compressed:**

- `c.addiw`, `c.addw`, `c.subw` - Compressed word operations

**Atomic:**

- **KEEP** `amo*w` - These are available in both RV32A and RV64A and are correct for 32-bit atomics on RV32
- Note: `amo*d` (doubleword) variants are also available for 64-bit atomics on both RV32 and RV64

## Expected Outcome

After cleanup:

- No opcode `0x3b` (OP-32) generated
- Only opcode `0x33` (OP) for integer ALU operations
- `iadd i32` generates `add`, not `addw`
- All ISLE files parse correctly
- Backend compiles and generates correct RV32 machine code

## Notes

- The emulator already supports all base RV32I instructions
- Keep Zba, Zbb, Zbc, Zbs extensions (just remove RV64-specific \*w variants like clzw, ctzw, cpopw, rolw, rorw, roriw)
- Keep floating-point and vector extensions (no \*w variants there)
- **Atomic operations**: Keep `*w` variants - they're correct for 32-bit atomics on RV32 (word = 32 bits on RV32)
- **Key insight**: RV64-specific `*w` instructions operate on 32-bit values and sign-extend to 64 bits. On RV32, we use base instructions (add, sub, etc.) that operate on full XLEN (32 bits)

## Summary of Key Corrections (Based on RISC-V ISA Reference)

After reviewing the official RISC-V instruction set reference, the following corrections were made:

1. **Atomic operations are CORRECT**: The `*w` atomic variants (amoaddw, amoswapw, etc.) are part of RV32A and RV64A. They should NOT be removed. On RV32, "word" = 32 bits, so these are the correct operations for 32-bit atomics.

2. **What to remove**: Only RV64I and RV64M specific `*w` instructions:

   - From RV64I: addw, addiw, subw, sllw, slliw, srlw, srliw, sraw, sraiw
   - From RV64M: mulw, divw, divuw, remw, remuw
   - From Zbb (RV64): clzw, ctzw, cpopw, rolw, rorw, roriw
   - From Zbkb (RV64): packw
   - From RVC (RV64): c.addiw, c.addw, c.subw

3. **Opcode difference**: RV64-specific `*w` instructions use opcode `0x3b` (OP-32) or `0x1b` (OP-IMM-32). RV32 base instructions use opcode `0x33` (OP) or `0x13` (OP-IMM).

4. **Type predicates**: Replace uses of undefined predicates like `ty_int_scalar_64_or_less` with standard ones like `ty_int`, `ty_int_ref_scalar_64`, or specific type checks like `$I32`, `$I64`.

5. **Helper functions to remove from inst.isle**:

   - rv_packw (line ~2083) - uses removed Packw variant
   - rv_sextw (line ~2127) - uses removed Addiw variant
   - rv_ctzw usages in lower_ctz (line ~2052, 2056)

6. **Replacement strategy**:
   - For rv_packw: use rv_pack or rv_zexth
   - For rv_sextw: on RV32, I32 values don't need sign-extension to 64 bits (no 64-bit registers)
   - For rv_ctzw: use rv_ctz for all integer types on RV32

## Remaining Issues

The plan has been updated based on the RISC-V ISA reference. Key findings:

1. **Atomic operations were CORRECT** - They should NOT have been removed
2. **Fixed**: Removed RV64-specific \*w instructions from inst.isle and lower.isle
3. **Fixed**: Replaced undefined type predicates with standard ones
4. **Remaining**: emit.rs and args.rs still have references to removed RV64 instructions

Next steps: Remove remaining RV64 instruction references from emitter code.
