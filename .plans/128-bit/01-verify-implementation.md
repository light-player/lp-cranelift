# 01: Verify i128 Implementation Completeness

## Goal

Verify that all i128 operations used in the test files have corresponding lowering rules in riscv32/lower.isle, and identify any gaps.

## Tasks

### 1.1 Check implemented operations in riscv32/lower.isle

Review `cranelift/codegen/src/isa/riscv32/lower.isle` for all `has_type $I128` patterns:

**Already implemented (from grep analysis):**

- `iadd` - Addition (rule 7, line 119)
- `isub` - Subtraction (rule 2, line 398, uses `sub_i128` helper)
- `imul` - Multiplication (rule 2, line 569)
- `imul` with extends (rules 6, lines 597, 602)
- `uadd_overflow` - Unsigned add with overflow (rule 2, line 373)
- `icmp` - All comparison types (via `lower_icmp_i128`, line 2582)
- `ishl` - Left shift (rule 4, line 1453)
- `ushr` - Unsigned right shift (rule 4, line 1522)
- `sshr` - Signed right shift (rule 4, line 1593)
- `rotl` - Rotate left (rule 2, line 1668)
- `rotr` - Rotate right (rule 2, line 1731)
- `bitrev` - Bit reverse (rule 1, line 1111)
- `bswap` - Byte swap (rule 3, line 1143)
- `ctz` - Count trailing zeros (rule 2, line 1199)
- `clz` - Count leading zeros (rule 2, line 1225)
- `cls` - Count leading sign bits (rule 2, line 1286)
- `popcnt` - Population count (rules 1, 4, lines 1347, 1363)
- `uextend` - Zero extension (rule 2, line 1310)
- `sextend` - Sign extension (rule 2, line 1330)
- `smax`, `smin`, `umax`, `umin` - Min/max (rules 1, lines 2206, 2225, 2244, 2263)
- `iconcat` - Concatenate (line 2180)
- `isplit` - Split (special cases for imul extends, lines 2189, 2194)
- `select_spectre_guard` - Select with spectre guard (rule 1, line 2875)

### 1.2 Check helper functions

Verify helper functions exist in `cranelift/codegen/src/isa/riscv32/inst.isle`:

- `sub_i128` - Declared at line 2718, implemented at line 2719
- `sub_i64` - Declared at line 2704 (for reference)

### 1.3 Identify operations that may need libcalls

Check if these operations require libcalls (they typically do):

- `udiv.i128` - Unsigned division
- `sdiv.i128` - Signed division
- `urem.i128` - Unsigned remainder
- `srem.i128` - Signed remainder
- `fcvt_to_uint.i128` - Float to unsigned i128
- `fcvt_to_sint.i128` - Float to signed i128

These should be handled by cranelift's libcall infrastructure automatically.

### 1.4 Verify ABI handling

Check `cranelift/codegen/src/isa/riscv32/abi.rs`:

- `rc_for_type` should return 4 x I32 registers for I128 (line 831)
- Stack operations should handle 16-byte i128 values correctly

### 1.5 Check for missing operations

Compare with riscv64 implementation to ensure no operations are missing:

- Run: `grep -o "has_type \$I128[^)]*" cranelift/codegen/src/isa/riscv64/lower.isle | sort -u`
- Run: `grep -o "has_type \$I128[^)]*" cranelift/codegen/src/isa/riscv32/lower.isle | sort -u`
- Diff the results to find any missing operations

### 1.6 Document findings

Create a checklist of:

- ✅ Operations fully implemented
- ⚠️ Operations requiring libcalls (expected)
- ❌ Operations missing (if any)

## Expected Outcome

A complete inventory of i128 operations showing that the implementation is ready for testing, with notes on any operations that may need special handling.

## Files to Review

- `cranelift/codegen/src/isa/riscv32/lower.isle`
- `cranelift/codegen/src/isa/riscv32/inst.isle`
- `cranelift/codegen/src/isa/riscv32/abi.rs`
- `cranelift/codegen/src/isa/riscv32/inst/mod.rs` (for `rc_for_type`)
