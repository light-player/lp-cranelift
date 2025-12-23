# Fix RISC-V32 Emulator Test Failures

## Failure Categories

### Category 1: Missing/Incorrect Instruction Decoding (15+ failures)

**Symptoms**: `Invalid instruction 0x... Unknown I-type instruction: funct3=0x5, funct7=0x1, funct6=0x0, funct12=0x20/0x30/0x38`

**Root Cause**: Emulator's decode logic doesn't recognize instruction encodings that Cranelift generates. The funct12 values (0x20, 0x30, 0x38) don't match expected Zbb encodings (CLZ=0x600, CTZ=0x601).

**Affected Tests**: `arithmetic-extends.clif`, `udiv.clif`, `shifts.clif`, `smulhi.clif`, `extend.clif`, `umulhi.clif`, `rotl.clif`, `fibonacci.clif`, `popcnt.clif`, `br_table.clif`, `uadd_overflow_trap.clif`, `icmp.clif`, `rotr.clif`, `bitrev.clif`

**Fix**: Add missing instruction decodings to emulator (`lightplayer/crates/lp-riscv-tools/src/decode.rs`)

### Category 2: i64 Value Handling Bugs (10+ failures)

**Symptoms**: Wrong results for i64 operations, e.g.:

- `%add_i64(-1, 1) == 0, actual: -4294967296`
- `%ineg_i64(1) == -1, actual: 4294967295`
- `%clz_i64(0) == 64, actual: 128`

**Root Cause**: i64 values are split into two 32-bit registers (low, high), but emulator or ABI handling is incorrect. The emulator correctly splits i64 into register pairs (see `emulator.rs:405-422`), but return value reconstruction or intermediate operations may be wrong.

**Affected Tests**: `div-checks.clif`, `urem.clif`, `ineg.clif`, `clz.clif`, `arithmetic.clif`, `bmask.clif`, `iabs.clif`, `bitselect.clif`, `cls.clif`, `i64-riscv32.clif`

**Fix**: Review and fix i64 register pair handling in:

- Return value reconstruction (`emulator.rs:529-548`)
- Intermediate operations that produce i64 results
- Sign extension for i64 values

### Category 3: Missing ISLE Lowering Patterns (5+ panics)

**Symptoms**: `panicked at cranelift/codegen/src/isa/riscv32/lower/isle.rs:68:5: called `Option::unwrap()`on a`None` value`

**Root Cause**: Missing ISLE patterns for certain instruction combinations. The panic occurs in `isle_lower_prelude_methods!()` macro expansion.

**Affected Tests**: `call.clif`, `return-call.clif`, `return-call-loop.clif`, `spill-reload.clif`, `call_indirect.clif`, `return-call-indirect.clif`

**Fix**: Add missing ISLE rules or handle None cases gracefully in `cranelift/codegen/src/isa/riscv32/lower/isle.rs`

### Category 4: Unsupported Features (4 failures)

**Symptoms**:

- `Unsupported feature: Too many return values to fit in registers. Use a StructReturn argument instead. (#9510)`
- `Unsupported feature: should be implemented in ISLE: inst = v2, v3 = uadd_overflow.i8 v0, v1`

**Root Cause**: RISC-V32 ABI limitations or missing ISLE implementations for certain instructions.

**Affected Tests**: `stack.clif`, `smul_overflow.clif`, `umul_overflow.clif`, `uadd_overflow_narrow.clif`

**Fix**: Skip these tests for riscv32 (mark as expected failures)

### Category 5: Register Allocator Bugs (2 panics)

**Symptoms**: `index out of bounds: the len is 203 but the index is 2097151` in `regalloc2-0.13.3/src/ssa.rs:64:51`

**Root Cause**: Invalid register index being passed to register allocator, possibly from incorrect i64 register pair handling.

**Affected Tests**: `bitops.clif`, `integer-minmax.clif`

**Fix**: Investigate register index generation for i64 values

### Category 6: Memory Access Issues (1 failure)

**Symptoms**: `Invalid memory write at address 0x00000000`

**Affected Tests**: `uadd_overflow.clif`

**Fix**: Review stack setup and memory layout in emulator

### Category 7: Compilation Errors (2 failures)

**Symptoms**:

- `global_value instruction with type i64 references global value with type i32`
- Various compilation errors

**Affected Tests**: `global_value.clif`, others

**Fix**: Review global value handling for riscv32

## Implementation Plan

### Phase 1: Investigate Instruction Encodings (1-2 hours)

1. Extract actual instruction bytes from failing tests to understand what Cranelift generates
2. Compare with Cranelift's encoding logic in `cranelift/codegen/src/isa/riscv32/inst/args.rs` and `encode.rs`
3. Map funct12 values (0x20, 0x30, 0x38) to actual instructions by checking:
   - Cranelift's `option_funct12()` method in `args.rs:994-1010`
   - Instruction encoding in `encode.rs`
   - Whether these are shift amounts vs. function codes

### Phase 2: Fix Instruction Decoding (Estimated: 4-6 hours)

1. Add missing instruction decodings to `lightplayer/crates/lp-riscv-tools/src/decode.rs`:

   - Handle funct12=0x20, 0x30, 0x38 cases
   - Verify these map to CLZ, CTZ, or other Zbb instructions
   - Check if Cranelift uses non-standard encodings that need special handling

2. Add instruction execution to `lightplayer/crates/lp-riscv-tools/src/emu/executor.rs`:
   - Implement any missing instruction handlers
   - Ensure i64 operations work correctly with register pairs

### Phase 3: Fix i64 Handling (Estimated: 3-4 hours)

1. Review register pair handling in `emulator.rs:405-422` (arguments) and `529-548` (returns)
2. Fix intermediate i64 operations that may not properly handle register pairs
3. Test with simple i64 operations to verify fixes

### Phase 4: Fix ISLE Panics (Estimated: 2-3 hours)

1. Add missing ISLE patterns or handle None cases gracefully
2. Review `isle.rs:68` to understand what's causing the unwrap panic
3. Add proper error handling instead of panicking

### Phase 5: Handle Unsupported Features (Estimated: 1 hour)

1. Skip tests that require StructReturn or other unsupported features
2. Add comments explaining why these are skipped

### Phase 6: Fix Register Allocator Issues (Estimated: 2-3 hours)

1. Investigate register index generation for i64 values
2. Fix invalid register indices being passed to regalloc2

### Phase 7: Fix Memory Access Issues (Estimated: 1 hour)

1. Review stack setup in emulator initialization
2. Fix memory layout to prevent writes to address 0

## Files to Modify

### Primary Fixes:

1. **`lightplayer/crates/lp-riscv-tools/src/decode.rs`** - Add missing instruction decodings for funct12=0x20/0x30/0x38
2. **`lightplayer/crates/lp-riscv-tools/src/emu/executor.rs`** - Add execution handlers for newly decoded instructions
3. **`lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`** - Fix i64 register pair handling in call_function and return value reconstruction
4. **`cranelift/codegen/src/isa/riscv32/lower/isle.rs`** - Fix ISLE panics by handling None cases or adding missing patterns
5. **`cranelift/filetests/src/test_run.rs`** - Add test skipping for unsupported features (StructReturn, etc.)

### Investigation Files:

- `cranelift/codegen/src/isa/riscv32/inst/args.rs` - Check `option_funct12()` to understand encodings
- `cranelift/codegen/src/isa/riscv32/inst/encode.rs` - Verify instruction encoding logic
- `lightplayer/crates/lp-riscv-tools/src/encode.rs` - Compare emulator's encoding with Cranelift's

## Implementation Order

1. **Start with instruction decoding** - This will fix the most failures (15+)
2. **Then fix i64 handling** - This will fix another 10+ failures
3. **Fix ISLE panics** - Critical for stability
4. **Handle remaining edge cases** - Unsupported features, register allocator, memory access




