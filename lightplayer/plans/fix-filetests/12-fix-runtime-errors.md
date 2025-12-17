# Phase 12: Fix Runtime Errors ("run" failures)

## Goal

Fix 8 tests that fail with runtime errors ("run" failures) when executing compiled code. These may be memory access issues, stack setup problems, or other runtime problems.

## Prerequisites

- Phase 3 completed: i64 handling works
- Phase 6/9 completed: Register allocation works (may fix some runtime issues)
- Phase 11 completed: Invalid register indices fixed (may fix some runtime issues)

## Problem Analysis

### Current Failures

**8 tests failing with "run" errors**:

- `cls.clif`
- `integer-minmax.clif`
- `smul_overflow.clif`
- `stack.clif`
- `uadd_overflow.clif`
- `uadd_overflow_narrow.clif`
- `uadd_overflow_trap.clif`
- `umul_overflow.clif`

### Error Pattern

Tests compile successfully but fail when executing:

- "run" test failures (execution doesn't match expected results)
- May be memory access violations
- May be stack setup issues
- May be incorrect register handling
- May be overflow detection issues

### Potential Root Causes

1. **Memory Access Issues**:

   - Stack pointer not properly initialized
   - Stack overflow/underflow
   - Invalid memory addresses
   - Memory alignment issues

2. **Register Handling Issues**:

   - Return values not properly reconstructed
   - Function arguments not properly passed
   - Register pairs not properly handled

3. **Overflow Detection Issues**:

   - Overflow operations (`uadd_overflow`, `smul_overflow`, etc.) may not work correctly
   - Overflow flags not properly set or checked

4. **Emulator Issues**:
   - Emulator may not correctly execute certain instructions
   - Emulator may have bugs in register pair handling
   - Emulator may have bugs in memory access

## Investigation Plan

### Step 1: Run Tests with Verbose Output

```bash
# Run with verbose output to see what's happening
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/stack.clif --verbose

# Check emulator output
RUST_LOG=debug cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/stack.clif
```

### Step 2: Check Stack Setup

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`

Verify:

- Stack pointer is properly initialized
- Stack has enough space
- Stack alignment is correct
- Stack grows in the correct direction

### Step 3: Check Memory Access

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/memory.rs`

Verify:

- Memory addresses are valid
- Memory access is properly bounds-checked
- Memory alignment is correct
- Stack memory is properly allocated

### Step 4: Check Overflow Operations

**Files**:

- `cranelift/codegen/src/isa/riscv32/lower.isle` - Overflow operation lowering
- `lightplayer/crates/lp-riscv-tools/src/emu/executor.rs` - Overflow execution

Verify:

- Overflow operations are correctly lowered
- Overflow flags are correctly set
- Overflow results are correctly returned

### Step 5: Check Return Value Handling

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`

Verify:

- Return values are correctly extracted from registers
- Register pairs are correctly reconstructed
- Return value types match expected types

## Implementation Plan

### Fix 1: Fix Stack Setup (if needed)

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`

**Issue**: Stack pointer may not be properly initialized or stack may not have enough space.

**Fix**:

- Ensure stack pointer is initialized to a valid address
- Ensure stack has enough space for function calls
- Ensure stack alignment is correct (16-byte aligned)
- Ensure stack grows downward (decreasing addresses)

### Fix 2: Fix Memory Access (if needed)

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/memory.rs`

**Issue**: Memory access may fail due to invalid addresses or bounds.

**Fix**:

- Ensure memory addresses are valid
- Ensure memory access is properly bounds-checked
- Ensure memory alignment is correct
- Ensure stack memory is properly allocated

### Fix 3: Fix Overflow Operations (if needed)

**Files**:

- `cranelift/codegen/src/isa/riscv32/lower.isle` - Overflow operation lowering
- `lightplayer/crates/lp-riscv-tools/src/emu/executor.rs` - Overflow execution

**Issue**: Overflow operations may not be correctly implemented.

**Fix**:

- Ensure overflow operations are correctly lowered
- Ensure overflow flags are correctly set
- Ensure overflow results are correctly returned
- Ensure overflow detection works for all types (i8, i16, i32, i64)

### Fix 4: Fix Return Value Handling (if needed)

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`

**Issue**: Return values may not be correctly extracted or reconstructed.

**Fix**:

- Ensure return values are correctly extracted from registers
- Ensure register pairs are correctly reconstructed for i64
- Ensure return value types match expected types
- Ensure return value handling works for all types

### Fix 5: Fix Specific Test Cases

Investigate each failing test individually:

1. **`stack.clif`**: Stack operations test

   - Check stack pointer initialization
   - Check stack memory access
   - Check stack alignment

2. **`uadd_overflow.clif`**: Unsigned addition overflow

   - Check overflow detection
   - Check overflow result handling

3. **`smul_overflow.clif`**: Signed multiplication overflow

   - Check overflow detection
   - Check overflow result handling

4. **`umul_overflow.clif`**: Unsigned multiplication overflow

   - Check overflow detection
   - Check overflow result handling

5. **`cls.clif`**: Count leading sign bits

   - Check instruction execution
   - Check result handling

6. **`integer-minmax.clif`**: Integer min/max operations
   - Check instruction execution
   - Check result handling

## Testing

After implementing fixes:

```bash
# Test individual failing tests
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/stack.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/uadd_overflow.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/smul_overflow.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/umul_overflow.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/cls.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/integer-minmax.clif

# Test all riscv32 tests
cargo run --package cranelift-tools --bin clif-util -- test $(find cranelift/filetests/filetests/runtests -name "*.clif" -exec grep -l "^target riscv32" {} \;)
```

## Success Criteria

- ✅ All 8 affected tests pass
- ✅ No runtime errors ("run" failures)
- ✅ Stack operations work correctly
- ✅ Overflow operations work correctly
- ✅ Memory access is correct
- ✅ Return values are correctly handled

## Estimated Time

- Investigation: 2-4 hours (identify root causes)
- Fix 1-2: 2-3 hours (stack/memory fixes)
- Fix 3: 2-4 hours (overflow operations)
- Fix 4: 1-2 hours (return value handling)
- Fix 5: 2-4 hours (specific test cases)

**Total**: 9-17 hours (depends on root causes)

## Related Issues

- Phase 7: Memory access (original plan)
- Phase 11: Register allocation (may fix some runtime issues)
- Phase 5: Unsupported features (overflow operations may be unsupported)

## Notes

- Runtime errors are harder to debug than compilation errors
- May need to add debug logging to emulator
- May need to check emulator implementation for bugs
- Some failures may be due to missing features rather than bugs
- Consider using interpreter mode to compare results

