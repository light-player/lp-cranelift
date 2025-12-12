# Phase 8: Fix Compilation Errors

## Goal
Fix compilation errors related to global values and other riscv32-specific issues.

## Prerequisites
- Previous phases completed: Core functionality works

## Affected Test Files

These tests fail with compilation errors:

```bash
# Test the fixes:
cargo run --bin clif-util -- test filetests/filetests/runtests/global_value.clif
# (Other compilation errors may appear in various tests)
```

## Error Patterns

1. **Global value type mismatch**:
   ```
   error: inst0 (v1 = global_value.i64 gv0): global_value instruction with type i64 references global value with type i32
   ```

2. **Other compilation errors**: Various verifier errors during compilation

## Root Cause Analysis

### Global Value Type Mismatch

The error suggests that:
- A `global_value.i64` instruction references a global value declared as i32
- This is a type mismatch in the CLIF IR

Possible causes:
1. Global value declaration doesn't match usage
2. RISC-V32 specific handling of global values is incorrect
3. Type inference for global values is wrong

## Implementation Steps

### Step 1: Understand Global Value Handling

File: `cranelift/codegen/src/isa/riscv32/` (various files)

**Investigation**:
1. How are global values declared?
2. How are global values accessed?
3. What types can global values have on riscv32?

### Step 2: Check Global Value Declaration

File: `cranelift/filetests/filetests/runtests/global_value.clif`

**Read the test file** to understand what it's trying to do:
```bash
cat filetests/filetests/runtests/global_value.clif
```

**Expected pattern**:
```clif
test run
target riscv32

function %simple(i64 vmctx) -> i64 {
    gv0 = vmctx
    block0(v0: i64):
        v1 = global_value.i64 gv0
        return v1
}
```

**Issue**: The global value `gv0` is declared as `vmctx` (which might be i32 on riscv32), but accessed as i64.

### Step 3: Fix Global Value Type Handling

**Option A: Fix the test** (if test is wrong):
- Change `global_value.i64` to `global_value.i32`
- Or change vmctx type to i64

**Option B: Fix the compiler** (if compiler should handle this):
- Update riscv32 backend to handle i64 global values
- Convert i32 global values to i64 when needed

**Option C: Skip the test** (if feature not supported):
- Add skip logic for global_value.clif on riscv32
- Document that i64 global values aren't supported

### Step 4: Check Other Compilation Errors

Look for other compilation errors in test output:

```bash
# Run all tests and grep for compilation errors:
cargo run --bin clif-util -- test filetests/filetests/runtests/*.clif 2>&1 | grep -i "compilation error\|verifier error"
```

Common issues:
- Type mismatches
- Missing instruction patterns
- ABI violations
- Register allocation failures

## Testing

After making changes:

```bash
# Test compilation error fixes:
cargo run --bin clif-util -- test filetests/filetests/runtests/global_value.clif
```

## Common Fixes

1. **Type mismatch**: Ensure global value types match their usage
2. **Missing patterns**: Add ISLE patterns for global value access
3. **ABI issues**: Fix calling convention for global values

## Success Criteria

- Test compiles without verifier errors
- Test may still fail for other reasons (execution errors, etc.)
- No more "global_value instruction with type i64 references global value with type i32" errors

## Summary

After completing all phases:
- Instruction decoding works (Phase 2)
- i64 handling works (Phase 3)
- ISLE panics fixed (Phase 4)
- Unsupported features skipped (Phase 5)
- Register allocator works (Phase 6)
- Memory access works (Phase 7)
- Compilation errors fixed (Phase 8)

Most tests should now pass. Remaining failures can be addressed individually.

