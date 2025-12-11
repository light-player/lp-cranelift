# 06: Handle Missing Operations (If Needed)

## Goal

Implement any missing i128 operations that are not yet supported in riscv32 lowering, or verify that libcall infrastructure handles them correctly.

## Operations That May Need Special Handling

### 6.1 Division and Remainder Operations

**Operations**:

- `udiv.i128` - Unsigned division
- `sdiv.i128` - Signed division
- `urem.i128` - Unsigned remainder
- `srem.i128` - Signed remainder

**Expected behavior**: These operations typically require libcalls since RISC-V doesn't have native 128-bit division instructions.

**Verification steps**:

1. Check if cranelift automatically generates libcalls for these operations
2. Verify libcall signatures handle 4 x I32 register arguments/returns
3. Test that libcalls are properly linked

**Files to check**:

- `cranelift/codegen/src/ir/libcalls.rs` - Libcall definitions
- `cranelift/codegen/src/isa/riscv32/lower.rs` - Libcall lowering
- Test files: `i128-urem.clif`, `i128-srem.clif`

**If libcalls don't work automatically**:

- May need to add explicit libcall lowering rules in `riscv32/lower.isle`
- Ensure ABI correctly passes 4 x I32 registers to libcall functions
- Verify return value handling

### 6.2 Float-to-i128 Conversions

**Operations**:

- `fcvt_to_uint.i128` - Float to unsigned i128
- `fcvt_to_sint.i128` - Float to signed i128
- `fcvt_to_uint_sat.i128` - Float to unsigned i128 (saturating)
- `fcvt_to_sint_sat.i128` - Float to signed i128 (saturating)

**Expected behavior**: These may require libcalls or special handling.

**Verification steps**:

1. Check if there are lowering rules for these operations
2. Verify they use libcalls or are implemented directly
3. Test conversion accuracy and edge cases

**Files to check**:

- `cranelift/codegen/src/isa/riscv32/lower.isle` - Look for `fcvt_to_uint`/`fcvt_to_sint` with i128
- Test file: `i128-conversion.clif`

**If not implemented**:

- May need to add libcall rules
- Or implement using existing float-to-i64 conversion + extension
- Handle overflow/underflow correctly

### 6.3 Other Potential Missing Operations

**Check for**:

- `iadd_imm.i128` - Addition with immediate (may need special handling)
- `imul_imm.i128` - Multiplication with immediate
- `select.i128` - May need verification with 4-register values
- `select_spectre_guard.i128` - Already implemented (line 2875)

**Verification**:

- Run grep to find all i128 operations in test files
- Compare with implemented operations in `lower.isle`
- Identify any gaps

## Implementation Steps

### Step 1: Identify Missing Operations

```bash
# Find all i128 operations in test files
grep -h "i128" cranelift/filetests/filetests/runtests/i128-*.clif | \
  grep -oE "[a-z_]+\.i128|[a-z]+ i128" | sort -u

# Compare with implemented operations
grep "has_type \$I128" cranelift/codegen/src/isa/riscv32/lower.isle | \
  grep -oE "\([a-z_]+" | sort -u
```

### Step 2: Check Libcall Support

1. Review `cranelift/codegen/src/ir/libcalls.rs` for i128 libcall definitions
2. Check if riscv32 backend handles libcalls correctly
3. Verify ABI compatibility for libcall arguments/returns

### Step 3: Implement Missing Operations

If operations are missing:

1. **For libcall operations**:

   - Add libcall lowering rules in `riscv32/lower.isle`
   - Ensure proper argument/return value handling
   - Test with existing libcall infrastructure

2. **For direct operations**:
   - Implement lowering rules similar to riscv64
   - Adapt for 4-register representation (vs 2-register on riscv64)
   - Add tests

### Step 4: Verify Implementation

- Run all i128 tests to identify failures
- Fix any issues found
- Ensure all operations work correctly

## Expected Outcome

Either:

1. **All operations work via libcalls**: No changes needed, libcall infrastructure handles everything
2. **Some operations need implementation**: Add missing lowering rules or libcall handling
3. **Operations work but need fixes**: Fix bugs in existing implementation

## Files That May Need Changes

- `cranelift/codegen/src/isa/riscv32/lower.isle` - Add missing lowering rules
- `cranelift/codegen/src/isa/riscv32/lower.rs` - Add libcall handling if needed
- `cranelift/codegen/src/ir/libcalls.rs` - Verify/update libcall definitions

## Testing

After implementing missing operations:

```bash
# Run all i128 tests
cargo test --package cranelift-filetests --test filetests i128

# Run specific operation tests
cargo test --package cranelift-filetests --test filetests i128_urem
cargo test --package cranelift-filetests --test filetests i128_conversion
```

## Notes

- Most operations should already be implemented based on the grep analysis
- Division/remainder and float conversions are the most likely to need libcall support
- The riscv32 implementation should be similar to riscv64 but adapted for 4-register representation
- Test failures will guide what needs to be fixed
