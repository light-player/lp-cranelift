# Phase 4: Fix ISLE Panics

## Goal
Fix panics in ISLE lowering code that occur when missing patterns cause `Option::unwrap()` to be called on `None`.

## Prerequisites
- Phase 2 completed: Instruction decoding works
- Phase 3 completed: i64 handling works (may help with some panics)

## Affected Test Files

These tests panic with "called `Option::unwrap()` on a `None` value":

```bash
# Test the fixes:
cargo run --bin clif-util -- test filetests/filetests/runtests/call.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/return-call.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/return-call-loop.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/spill-reload.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/call_indirect.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/return-call-indirect.clif
```

## Error Pattern

```
thread 'worker #X' panicked at cranelift/codegen/src/isa/riscv32/lower/isle.rs:68:5:
called `Option::unwrap()` on a `None` value
```

## Root Cause

The panic occurs in `isle.rs:68` which is inside the `isle_lower_prelude_methods!()` macro expansion. This suggests that some ISLE pattern matching is returning `None` when it's expected to return `Some(value)`.

Common causes:
1. Missing ISLE rules for certain instruction combinations
2. Missing ABI/calling convention patterns for riscv32
3. Missing register allocation patterns

## Implementation Steps

### Step 1: Identify the Exact Panic Location

File: `cranelift/codegen/src/isa/riscv32/lower/isle.rs`

Location: Line 68

**Current code** (from earlier reading):
```rust
impl generated_code::Context for RV64IsleContext<'_, '_, MInst, Riscv32Backend> {
    isle_lower_prelude_methods!();
    // ...
}
```

The `isle_lower_prelude_methods!()` macro expands to include various helper methods. The panic at line 68 suggests one of these methods is calling `.unwrap()` on an `Option`.

**To debug**:
1. Run a failing test with `RUST_BACKTRACE=1`:
   ```bash
   RUST_BACKTRACE=1 cargo run --bin clif-util -- test filetests/filetests/runtests/call.clif
   ```

2. Look at the backtrace to see which method is panicking
3. Check if it's related to:
   - Call info generation (`gen_call_info`, `gen_call_ind_info`)
   - Register allocation
   - ABI handling

### Step 2: Check Call-Related Methods

File: `cranelift/codegen/src/isa/riscv32/lower/isle.rs`

Location: Lines 70-138

**Methods to check**:
- `gen_call_info` (lines 70-118)
- `gen_call_ind_info` (lines 120-138)

These methods handle function calls and may need riscv32-specific patterns.

**Potential issues**:
- Missing patterns for riscv32 calling convention
- Incorrect handling of register pairs for i64 arguments/returns
- Missing patterns for indirect calls

### Step 3: Add Missing ISLE Patterns

File: `cranelift/codegen/src/isa/riscv32/inst.isle` or `lower.isle`

**Check for missing rules**:
- Call/return patterns for riscv32
- Register pair handling in calls
- Stack argument handling

**Common patterns to add**:
- Rules for handling i64 arguments in calls
- Rules for handling i64 return values
- Rules for indirect calls with register pairs

### Step 4: Handle None Cases Gracefully

Instead of panicking, return an error or use a fallback pattern:

**Before** (panics):
```rust
let value = some_option.unwrap();
```

**After** (handles None):
```rust
let value = some_option.ok_or_else(|| {
    CodegenError::Unsupported("Missing ISLE pattern for riscv32")
})?;
```

Or add a fallback ISLE rule that handles the missing case.

## Testing Strategy

1. **Start with simple call**:
   ```bash
   cargo run --bin clif-util -- test filetests/filetests/runtests/call.clif
   ```

2. **Then indirect call**:
   ```bash
   cargo run --bin clif-util -- test filetests/filetests/runtests/call_indirect.clif
   ```

3. **Test return calls**:
   ```bash
   cargo run --bin clif-util -- test filetests/filetests/runtests/return-call.clif
   ```

4. **Test with register pressure** (spill-reload):
   ```bash
   cargo run --bin clif-util -- test filetests/filetests/runtests/spill-reload.clif
   ```

## Debugging Tips

1. **Enable verbose logging**:
   ```bash
   RUST_LOG=debug cargo run --bin clif-util -- test filetests/filetests/runtests/call.clif
   ```

2. **Check ISLE compilation**:
   - ISLE files are compiled to Rust code
   - Check if there are ISLE compilation errors
   - Look for missing pattern warnings

3. **Compare with riscv64**:
   - Check `cranelift/codegen/src/isa/riscv64/lower/isle.rs`
   - See if riscv64 has patterns that riscv32 is missing

4. **Check ABI implementation**:
   - File: `cranelift/codegen/src/isa/riscv32/abi.rs`
   - Verify riscv32 ABI handles register pairs correctly

## Common Fixes

1. **Add missing call patterns**: If calls fail, add ISLE rules for riscv32 calling convention
2. **Fix register pair handling**: Ensure i64 arguments/returns use register pairs
3. **Add fallback patterns**: Instead of panicking, add default rules that handle edge cases

## Success Criteria

- All 6 call-related tests compile without panicking
- Tests may still fail for other reasons (wrong results, etc.) - that's expected
- No more "called `Option::unwrap()` on a `None` value" panics

## Next Phase

Once ISLE panics are fixed, proceed to Phase 5 to handle unsupported features.

