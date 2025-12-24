---
name: Add Float Support to RISC-V32 Emulator
overview: Add full floating-point support (F extension) to the riscv32 emulator, including float registers, instruction decoding/execution, and filetest specification for float/nofloat configurations.
todos:
  - id: add-fpr-type
    content: Add Fpr enum type to regs.rs with f0-f31 registers and helper methods
    status: pending
  - id: add-float-regs
    content: "Add float register file (fregs: [f32; 32]) to Riscv32Emulator struct"
    status: pending
    dependencies:
      - add-fpr-type
  - id: add-float-inst-defs
    content: Add F extension instruction variants to Inst enum (FAddS, FSubS, FMulS, FDivS, Flw, Fsw, etc.)
    status: pending
    dependencies:
      - add-fpr-type
  - id: implement-float-decode
    content: Implement float instruction decoding in decode.rs (opcodes 0x07, 0x27, 0x53)
    status: pending
    dependencies:
      - add-float-inst-defs
  - id: implement-float-exec
    content: Implement float instruction execution in executor.rs (arithmetic, comparisons, conversions)
    status: pending
    dependencies:
      - add-float-regs
      - implement-float-decode
  - id: update-function-calls
    content: Update call_function() to handle f32/f64 arguments and return values using fa0-fa7 registers
    status: pending
    dependencies:
      - add-float-regs
  - id: add-float-tests
    content: Create filetests for float operations and verify float/nofloat configurations work correctly
    status: pending
    dependencies:
      - implement-float-exec
      - update-function-calls
---

# Add Full Float Support to RISC-V32 Emulator

## Overview

Add complete floating-point (F extension) support to the riscv32 emulator to enable testing of riscv32.imafc configurations. This includes float registers, instruction decoding/execution, and proper handling of float arguments/returns in function calls.

## Current State

- Emulator supports: I, M, A, C extensions (integer-only)
- ISA flags: `has_f` and `has_d` exist in riscv32 ISA settings (default: false)
- Filetests specify ISA flags: `target riscv32 has_m` syntax
- Function calls: Currently handle integer types (i8, i16, i32, i64, i128) via `call_function`

## Implementation Plan

### 1. Add Float Register File

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`

- Add `fregs: [f32; 32] `field to `Riscv32Emulator` struct
- Add getter/setter methods: `get_fregister()`, `set_fregister()` for f0-f31
- Initialize float registers to 0.0 in `new()`
- Update `call_function()` to reset float registers

### 2. Add Float Register Type

**File**: `lightplayer/crates/lp-riscv-tools/src/regs.rs`

- Add `Fpr` enum similar to `Gpr` for float registers (f0-f31)
- Add helper methods: `num()`, `new()`, `Display` implementation
- Add named constants: `Fpr::Fa0`, `Fpr::Fa1`, etc. for calling convention

### 3. Add Float Instruction Definitions

**File**: `lightplayer/crates/lp-riscv-tools/src/inst.rs`

Add F extension instructions to `Inst` enum:

- Arithmetic: `FAddS`, `FSubS`, `FMulS`, `FDivS`, `FSqrtS`
- Sign manipulation: `FSgnjS`, `FSgnjnS`, `FSgnjxS`
- Min/Max: `FMinS`, `FMaxS`
- Comparisons: `FEqS`, `FLtS`, `FLeS` (write to integer register)
- Classify: `FClassS`
- Conversions: `FCvtWS`, `FCvtWuS`, `FCvtSW`, `FCvtSWu` (int<->float)
- Move: `FMvXW`, `FMvWX` (bitwise move between int/float regs)
- Load/Store: `Flw`, `Fsw`

### 4. Implement Float Instruction Decoding

**File**: `lightplayer/crates/lp-riscv-tools/src/decode.rs`

Add decoding for F extension opcodes:

- Opcode 0x07: `FLW` (load word to float register)
- Opcode 0x27: `FSW` (store word from float register)
- Opcode 0x53: FP arithmetic/comparison instructions
  - funct7 + funct3 + rs2 determine operation
  - Examples: 0x00000053 (fadd.s), 0x08000053 (fsub.s), 0x10000053 (fmul.s), 0x18000053 (fdiv.s)
- Opcode 0x43: FP fused multiply-add (not needed initially, can add later)

### 5. Implement Float Instruction Execution

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/executor.rs`

Add execution handlers for all float instructions:

- Use Rust's `f32` operations (IEEE 754 compliant)
- Handle special cases: NaN, infinity, denormals
- For comparisons (`feq.s`, `flt.s`, `fle.s`): write 0 or 1 to integer register
- For conversions: handle rounding modes (use default rounding)
- For `fclass.s`: return classification bits per RISC-V spec

**Note**: Use `f32::from_bits()` and `f32::to_bits()` for bitwise operations.

### 6. Update Function Call/Return for Floats

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`

Update `call_function()`:

- Handle `DataValue::F32` and `DataValue::F64` arguments
- RISC-V calling convention: fa0-fa7 (f10-f17) for float arguments
- Handle float return values in fa0-fa1
- Update signature handling to support `types::F32` and `types::F64`

### 7. Add Float Support to Filetest Specification

**File**: `cranelift/filetests/src/test_run.rs`

- No changes needed - ISA flags are already parsed from `target riscv32 has_f` syntax
- The `build_riscv32_isa()` function already copies ISA flags from test file
- Ensure emulator respects ISA flags (skip float instructions if `has_f=false`)

### 8. Add Float Instruction Encoding (Optional)

**File**: `lightplayer/crates/lp-riscv-tools/src/encode.rs`

Add encoding helpers for float instructions (useful for tests):

- `flw()`, `fsw()`, `fadd_s()`, `fsub_s()`, etc.

### 9. Update Error Handling

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/error.rs`

- Add error for unsupported float instructions when `has_f=false`
- Add error for invalid float operations (e.g., division by zero, invalid conversions)

### 10. Update Logging

**File**: `lightplayer/crates/lp-riscv-tools/src/emu/logging.rs`

- Add float register logging to `InstLog` variants
- Update `format_debug_info()` to show float registers
- Add float-specific log types if needed

### 11. Testing

Create test files:

- `cranelift/filetests/filetests/isa/riscv32/float-basic.clif` - Basic float operations
- `cranelift/filetests/filetests/isa/riscv32/float-nofloat.clif` - Test that float tests are skipped when `has_f=false`

Example filetest syntax:

```clif
test run
target riscv32 has_f

function %add_floats(f32, f32) -> f32 {
block0(v0: f32, v1: f32):
    v2 = fadd v0, v1
    return v2
}

; run: %add_floats(1.5, 2.5) == 4.0
```

For no-float tests:

```clif
test run
target riscv32  ; no has_f flag

function %no_float(i32) -> i32 {
block0(v0: i32):
    return v0
}
```

## Filetest Specification

Filetests specify float support using ISA flags:

- **With floats**: `target riscv32 has_f` or `target riscv32 has_f has_m`
- **Without floats**: `target riscv32 has_m` (default, `has_f=false`)

The emulator should:

1. Check ISA flags when decoding instructions
2. Reject float instructions if `has_f=false` (or skip test)
3. Support both configurations seamlessly

## Implementation Order

1. Add `Fpr` register type and float register file
2. Add float instruction definitions to `Inst` enum
3. Implement float instruction decoding
4. Implement float instruction execution (start with basic arithmetic)
5. Update function call/return handling
6. Add comprehensive float instruction support (comparisons, conversions, etc.)
7. Add filetests and verify both float/nofloat configurations work

## Notes

- Focus on F extension (single-precision) first; D extension (double-precision) can be added later
- Use Rust's native `f32` type for IEEE 754 compliance
- Handle edge cases: NaN comparisons, infinity, denormals
- RISC-V uses soft-float ABI by default (floats passed in integer registers), but we'll support hard-float ABI (floats in float registers) for testing