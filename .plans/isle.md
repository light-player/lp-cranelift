# ISLE Code Migration: RV64 → RV32

## Context

The RISC-V 64-bit ISLE files (~8,344 lines total) need to be copied to riscv32 and systematically transformed to work correctly for 32-bit architecture where XLEN=32. The key challenge is that RV64 has special "word" operations (like `addw`, `mulw`) that operate on 32-bit values and sign-extend to 64 bits. On RV32, we use the base operations (like `add`, `mul`) which operate on the full XLEN (32 bits).

**Phase 1 Goal**: Get proper 32-bit operations working (this plan)
**Phase 2 Goal**: Add multi-instruction 64-bit operations (deferred to future)

## Files Involved

- `cranelift/codegen/src/isa/riscv64/inst.isle` (3,281 lines) - instruction definitions
- `cranelift/codegen/src/isa/riscv64/lower.isle` (3,155 lines) - lowering rules
- `cranelift/codegen/src/isa/riscv64/inst_vector.isle` (1,908 lines) - vector extensions
- Target: `cranelift/codegen/src/isa/riscv32/` directory

## Strategy

Given the massive file sizes, we'll break the transformation into bite-sized, testable chunks. Each step will maintain compilability so we can verify progress incrementally.

## Step 1: Backup and Fresh Copy

1. Create backup of current riscv32 ISLE files (for reference)
2. Copy fresh versions from riscv64 to riscv32 (inst.isle, lower.isle, inst_vector.isle)
3. Verify baseline compilation works

## Step 2: Transform inst.isle - Remove RV64 Enum Variants

Work through `cranelift/codegen/src/isa/riscv32/inst.isle` removing RV64-specific instruction variants:

### 2a. AluOPRRR enum (~lines 483-564)

Remove: `Addw`, `Subw`, `Sllw`, `Srlw`, `Sraw`, `Mulw`, `Divw`, `Divuw`, `Remw`, `Remuw`, `Rolw`, `Rorw`, `Packw`

Keep: `Add`, `Sub`, `Mul`, `Div`, etc. (used by both RV32 and RV64)

### 2b. AluOPRRI enum (~lines 588-628)

Remove: `Addiw`, `Slliw`, `SrliW`, `Sraiw`, `Clzw`, `Ctzw`, `Cpopw`, `Roriw`

Keep: `Addi`, `Slli`, `Srli`, `Srai`, `Clz`, `Ctz`, `Cpop`, `Rori`

### 2c. Compressed enums (~lines 648-677)

Remove from `CaOp`: `CAddw`, `CSubw`  
Remove from `CiOp`: `CAddiw`

### 2d. AtomicOP enum (~lines 372-395)

**KEEP ALL** - Atomic `*W` variants are correct for RV32A (word = 32 bits on RV32)

Test compilation after each subsection.

## Step 3: Transform inst.isle - Remove RV64 Helper Functions

Remove RV64-specific helper function declarations and rules starting around line 1221:

- `rv_addw`, `rv_addiw`, `rv_sextw`
- `rv_subw`
- `rv_sllw`, `rv_slliw`
- `rv_srlw`, `rv_srliw`
- `rv_sraw`, `rv_sraiw`
- `rv_mulw`, `rv_divw`, `rv_divuw`, `rv_remw`, `rv_remuw`
- Zbb word helpers (~lines 1650-1740): `rv_clzw`, `rv_ctzw`, `rv_cpopw`, `rv_rolw`, `rv_rorw`, `rv_roriw`, `rv_packw`

Add comment: `;;  NOTE: RV64-specific *w instruction helpers removed for RV32`

Test compilation.

## Step 4: Transform lower.isle - Fix Type Selection Functions

Work through `cranelift/codegen/src/isa/riscv32/lower.isle`:

### 4a. Fix select_addi (~line 2182)

Change from:

```isle
(decl select_addi (Type) AluOPRRI)
(rule 1 (select_addi (fits_in_32 ty)) (AluOPRRI.Addiw))
(rule (select_addi (fits_in_64 ty)) (AluOPRRI.Addi))
```

To:

```isle
(decl select_addi (Type) AluOPRRI)
(rule (select_addi ty) (AluOPRRI.Addi))  ;; Always use Addi on RV32
```

### 4b. Find and fix similar type-selection functions

Search for other `select_*` functions that choose between base and `*w` variants based on `fits_in_32`/`fits_in_64`.

Test compilation.

## Step 5: Transform lower.isle - Update Lowering Rules

### 5a. Fix iadd rules (starting ~line 44)

Remove rule using `rv_addw`:

```isle
(rule -1 (lower (has_type (fits_in_32 (ty_int ty)) (iadd x y)))
  (rv_addw x y))
```

Replace with proper 32-bit rule:

```isle
(rule 0 (lower (has_type (ty_int ty) (iadd x y)))
  (rv_add x y))
```

### 5b. Systematically update other ALU operations

Search for and update rules for: `isub`, `imul`, `udiv`, `sdiv`, `urem`, `srem`, `ishl`, `ushr`, `sshr`, `rotl`, `rotr`

Pattern: Remove `fits_in_32` checks that call RV64 `*w` helpers, use base instructions instead.

### 5c. Keep atomic operations unchanged

Verify atomic rules use `*W` operations correctly (e.g., `AmoaddW` for I32 atomics).

Test compilation after each ALU operation group.

## Step 6: Transform inst_vector.isle

The `cranelift/codegen/src/isa/riscv32/inst_vector.isle` is already minimal (82 lines vs 1908 in riscv64). Keep the current stub version - full vector support is deferred.

## Step 7: Verification

1. **Build test**: `cargo build --package cranelift-codegen --features riscv32,std`
2. **Instruction verification**: Run a simple test and check generated code:
   - For `iadd i32`: should generate `add` (opcode 0x33), not `addw` (opcode 0x3b)
   - Encoding for `add a0, a0, a1` should be `0x00b50533`
3. **Run filetests**: Execute relevant cranelift filetests for riscv32
4. **Update documentation**: Add notes to `.plans/isle.md` about what was changed

## Key Principles

- **Incremental changes**: Test compilation after each major section
- **Keep atomics**: `*W` atomic operations are correct for RV32
- **Remove RV64 word ops**: All `*w` variants from RV64I/M/Zbb/Zbc (but not atomics)
- **Type predicates**: Replace `fits_in_32`/`fits_in_64` checks with appropriate RV32 type predicates
- **XLEN-aware**: On RV32, XLEN=32, so base ops (`add`, `mul`, etc.) work on 32-bit values natively

## Success Criteria

✓ All three ISLE files compile successfully  
✓ No RV64-specific `*w` instruction variants in enums (except atomics)  
✓ No `rv_*w` helper functions (except atomic-related)  
✓ Type selection functions don't use `fits_in_32` to choose RV64 instructions  
✓ Generated code uses opcode 0x33 (OP) for integer operations, not 0x3b (OP-32)  
✓ Simple iadd/isub/imul operations generate correct RV32 instructions

## Implementation Checklist

- [ ] Backup current riscv32 ISLE files and copy fresh from riscv64
- [ ] Remove RV64-specific enum variants from inst.isle
- [ ] Remove RV64-specific helper functions from inst.isle
- [ ] Fix type selection functions in lower.isle
- [ ] Update iadd lowering rules in lower.isle
- [ ] Update remaining ALU operation lowering rules
- [ ] Verify compilation and run basic instruction tests
- [ ] Document changes in .plans/isle.md

## Notes on Future Work (Phase 2)

Phase 2 will involve adding support for 64-bit operations on RV32, which will require:

- Multi-instruction sequences for 64-bit arithmetic
- Proper handling of I64 types with register pairs
- Wide operations that were used for 128-bit on RV64 will be adapted for 64-bit on RV32
- This is deferred until Phase 1 is complete and tested
