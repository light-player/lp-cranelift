# Phase 8: Fix Compilation Errors

## Goal
Fix compilation errors and runtime issues related to global values, relocations, and other riscv32-specific issues.

## Prerequisites
- Phase 4 completed: ISLE panics fixed (tests now compile successfully)

## Affected Test Files

These tests fail with compilation or runtime errors:

```bash
# Test the fixes:
cargo run --bin clif-util -- test filetests/filetests/runtests/global_value.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/call_indirect.clif
# (Other compilation errors may appear in various tests)
```

## Error Patterns

1. **Global value type mismatch**:
   ```
   error: inst0 (v1 = global_value.i64 gv0): global_value instruction with type i64 references global value with type i32
   ```

2. **Runtime relocation errors** (NEW - discovered in Phase 4):
   ```
   Emulator setup failed: ELF load failed: Could not resolve relocation target at offset 80
   ```
   
   **Note**: After Phase 4 ISLE panic fixes, `call_indirect.clif` compiles successfully but fails at runtime with relocation errors. This indicates the code generation works, but the emulator can't resolve function pointer relocations.

3. **Other compilation errors**: Various verifier errors during compilation

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

### Step 4: Fix Runtime Relocation Errors

**Issue**: `call_indirect.clif` compiles successfully but fails at runtime with:
```
ELF load failed: Could not resolve relocation target at offset 80
```

**Root Cause**: The emulator can't resolve function pointer relocations for indirect calls.

**Files to check**:
- `cranelift/filetests/src/object_runner.rs` - How relocations are handled
- `lightplayer/crates/lp-riscv-tools/src/emu/` - Emulator relocation resolution

**Investigation**:
1. What type of relocation is at offset 80?
2. Is the relocation target (function address) available?
3. Does the emulator support this relocation type?

**Potential fixes**:
1. **Add relocation resolution** in emulator:
   ```rust
   // In emulator.rs or object_runner.rs
   fn resolve_relocation(&mut self, reloc: &Relocation, symbol: &str) -> Result<u64, Error> {
       // Look up function address by name
       if let Some(addr) = self.functions.get(symbol) {
           Ok(*addr)
       } else {
           Err(format!("Unknown function: {}", symbol))
       }
   }
   ```

2. **Ensure function addresses are available**: When loading ELF, collect all function addresses and make them available for relocation resolution.

3. **Handle RISC-V relocation types**: RISC-V uses specific relocation types (R_RISCV_CALL, R_RISCV_PCREL_HI20, etc.) that need proper handling.

### Step 5: Check Other Compilation Errors

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

# Test runtime relocation fixes:
cargo run --bin clif-util -- test filetests/filetests/runtests/call_indirect.clif
```

## Common Fixes

1. **Type mismatch**: Ensure global value types match their usage
2. **Missing patterns**: Add ISLE patterns for global value access
3. **ABI issues**: Fix calling convention for global values
4. **Relocation resolution**: Ensure emulator can resolve function pointer relocations for indirect calls

## Success Criteria

- `global_value.clif` compiles without verifier errors
- `call_indirect.clif` compiles and runs without relocation errors
- Tests may still fail for other reasons (execution errors, etc.)
- No more "global_value instruction with type i64 references global value with type i32" errors
- No more "Could not resolve relocation target" errors

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

