# Phase 6: Fix Register Allocator Issues

## Goal
Fix invalid register indices being passed to regalloc2, causing "index out of bounds" panics.

## Prerequisites
- Phase 3 completed: i64 handling should be fixed (may resolve some regalloc issues)

## Affected Test Files

These tests panic with register allocator errors:

```bash
# Test the fixes:
cargo run --bin clif-util -- test filetests/filetests/runtests/bitops.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/integer-minmax.clif

# Additional tests discovered in Phase 4:
cargo run --bin clif-util -- test filetests/filetests/runtests/return-call-loop.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/spill-reload.clif
```

**Note**: After Phase 4 ISLE panic fixes, `return-call-loop.clif` and `spill-reload.clif` now fail with regalloc2 panics instead of ISLE panics, indicating they've progressed past the ISLE lowering stage.

## Error Pattern

```
thread 'worker #X' panicked at /Users/yona/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regalloc2-0.13.3/src/ssa.rs:64:51:
index out of bounds: the len is 203 but the index is 2097151
```

The index `2097151` (0x1FFFFF) is suspiciously large - it looks like a corrupted value or incorrect bit manipulation.

## Root Cause Analysis

The register allocator receives invalid register indices. Possible causes:

1. **i64 register pair handling**: When i64 values use register pairs, the register indices might be incorrectly calculated
2. **Register class mapping**: RISC-V32 maps i64 to two i32 registers, which might confuse the allocator
3. **Value numbering**: SSA value numbers might be incorrectly mapped to physical registers

## Implementation Steps

### Step 1: Understand Register Index Generation

File: `cranelift/codegen/src/isa/riscv32/inst/mod.rs`

Location: `rc_for_type` method (around line 820-863)

**Current code**:
```rust
fn rc_for_type(ty: Type) -> CodegenResult<(&'static [RegClass], &'static [Type])> {
    match ty {
        // ...
        I64 => Ok((&[RegClass::Int, RegClass::Int], &[I32, I32])),
        // ...
    }
}
```

This correctly maps i64 to two Int registers. The issue might be in how these are numbered.

### Step 2: Check Register Allocation for i64

File: `cranelift/codegen/src/machinst/regalloc.rs` or related files

**Investigation**:
1. How are register pairs allocated?
2. Are both registers in the pair assigned valid indices?
3. Is the high register index calculated correctly?

### Step 3: Add Bounds Checking

If register indices are generated incorrectly, add validation:

**Location**: Where register indices are generated or passed to regalloc2

**Add validation**:
```rust
fn validate_register_index(idx: usize, max_regs: usize) -> Result<usize, CodegenError> {
    if idx >= max_regs {
        return Err(CodegenError::InvalidRegisterIndex(idx, max_regs));
    }
    Ok(idx)
}
```

### Step 4: Fix i64 Register Pair Indexing

If the issue is with i64 register pairs:

**Check**: When an i64 value is allocated to registers (rd, rd+1), ensure:
- Both registers are valid
- The high register index doesn't overflow
- Register indices are within the valid range (0-31 for RISC-V)

**Potential fix**:
```rust
// When allocating i64 to register pair
let low_reg = allocate_register();
if low_reg.num() >= 31 {
    // Can't use register pair if low register is x31
    return Err("Not enough registers for i64");
}
let high_reg = Gpr::new(low_reg.num() + 1);
```

## Debugging Strategy

1. **Enable regalloc logging**:
   ```bash
   RUST_LOG=debug cargo run --bin clif-util -- test filetests/filetests/runtests/bitops.clif 2>&1 | grep -i regalloc
   ```

2. **Check register allocation trace**:
   - Look for where register indices are generated
   - Find where 2097151 (0x1FFFFF) comes from
   - Check if it's a bit manipulation error (e.g., wrong shift/mask)

3. **Compare with working tests**:
   - Run a test that works (e.g., simple arithmetic)
   - Compare register allocation between working and failing tests

4. **Check i64 operations**:
   - Focus on tests with i64 bit operations
   - Verify register pairs are handled correctly

## Testing

After making changes:

```bash
# Test register allocator fixes:
cargo run --bin clif-util -- test filetests/filetests/runtests/bitops.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/integer-minmax.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/return-call-loop.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/spill-reload.clif
```

## Common Issues

1. **Register pair overflow**: If low register is x31, high register would be x32 (invalid)
   - **Fix**: Ensure register pairs don't span register boundaries
   - **Fix**: Spill to stack if no register pair available

2. **Incorrect index calculation**: Bit manipulation errors in index calculation
   - **Fix**: Verify bit shifts and masks are correct
   - **Fix**: Add assertions to catch invalid indices early
   - **Note**: The index `2097151` (0x1FFFFF) suggests a bit manipulation error - check for wrong shift/mask operations

3. **SSA value to register mapping**: Incorrect mapping between SSA values and physical registers
   - **Fix**: Verify value numbering is correct
   - **Fix**: Check that register classes match

4. **Call-related register pressure**: Tests like `return-call-loop.clif` and `spill-reload.clif` may have high register pressure
   - **Fix**: Ensure spill/reload logic handles register pairs correctly
   - **Fix**: Verify that spilled i64 values are handled as pairs

## Success Criteria

- All 4 tests compile without regalloc panics
- Tests may still fail for other reasons (wrong results, etc.)
- No more "index out of bounds" errors from regalloc2
- The suspicious index `2097151` (0x1FFFFF) is resolved

## Next Phase

Once regalloc issues are fixed, proceed to Phase 7 to fix memory access issues.

