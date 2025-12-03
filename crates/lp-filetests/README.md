# lp-filetests

File-based tests for the Cranelift RISC-V32 backend.

## Overview

This crate provides compilation tests for the RISC-V32 backend of Cranelift. The tests verify that:

1. CLIF (Cranelift IR) functions compile successfully to RISC-V32 machine code
2. The generated instructions are correct for RV32 (not RV64)
3. Instruction selection follows expected patterns

## Test Structure

Tests are located in `filetests/riscv32/` and written in CLIF format:

```clif
test compile
target riscv32

function %add(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}

; Verify we use ADD instruction (RV32 opcode)
; check: add
; check-NOT: addw
```

## Running Tests

Run all tests:
```bash
cargo test --package lp-filetests
```

Run a specific test:
```bash
cargo test --package lp-filetests test_iadd
```

Run tests with output:
```bash
cargo test --package lp-filetests -- --nocapture
```

## Test Categories

### Arithmetic Operations
- `iadd.clif` - Integer addition ✓ Execution test
- `isub.clif` - Integer subtraction ✓ Execution test
- `imul.clif` - Integer multiplication ✓ Execution test

### Division and Remainder
- `udiv.clif` - Unsigned division
- `sdiv.clif` - Signed division
- `urem.clif` - Unsigned remainder
- `srem.clif` - Signed remainder

### Shift Operations
- `ishl.clif` - Shift left logical ✓ Execution test
- `ushr.clif` - Shift right logical (unsigned) ✓ Execution test
- `sshr.clif` - Shift right arithmetic (signed)

### Bitwise Operations
- `band.clif` - Bitwise AND ✓ Execution test
- `bor.clif` - Bitwise OR ✓ Execution test
- `bxor.clif` - Bitwise XOR ✓ Execution test

### Constants and Memory
- `iconst.clif` - Integer constants ✓ Execution test
- `load.clif` - Load from memory
- `store.clif` - Store to memory

## Adding New Tests

1. Create a new `.clif` file in `filetests/riscv32/`:

```clif
test compile
target riscv32

function %your_test(i32) -> i32 {
block0(v0: i32):
    ; Your CLIF code here
    return v0
}

; Filecheck patterns (optional)
; check: expected_instruction
; check-NOT: wrong_instruction
```

2. Add a test function in `src/compile.rs`:

```rust
#[test]
fn test_your_test() {
    let content = include_str!("../filetests/riscv32/your_test.clif");
    run_compile_test(content).unwrap();
}
```

3. Run the test:

```bash
cargo test --package lp-filetests test_your_test
```

## Filecheck Syntax

Tests use filecheck directives for pattern matching:

- `check: pattern` - Verify pattern appears in output
- `check-NOT: pattern` - Verify pattern does NOT appear
- `nextln: pattern` - Pattern must appear on next line
- `sameln: pattern` - Pattern must appear on same line

Example:
```clif
; check: add a0, a0, a1     ; Exact match
; check: add {{[ast][0-9]+}}, {{[ast][0-9]+}}, {{[ast][0-9]+}}  ; Regex
; check-NOT: addw            ; Must not appear
```

## Implementation Details

The test framework provides two types of tests:

### Compilation Tests
1. **Parse CLIF** using `cranelift-reader`
2. **Compile** to RISC-V32 using `cranelift-codegen`
3. **Disassemble** using `lp-riscv-tools`
4. **Verify** output using `filecheck` patterns

### Execution Tests  
1. **Compile** CLIF to RISC-V32 machine code
2. **Load** code into `lp-riscv-tools` emulator
3. **Execute** the code with test inputs
4. **Verify** results match expected outputs

Execution tests provide end-to-end verification that the generated code actually works correctly, catching bugs that static assembly checks might miss.

Key modules:
- `src/compile.rs` - Compilation and test infrastructure
- `src/filecheck.rs` - Pattern matching utilities
- `filetests/riscv32/` - Test files

Current test count: **26 tests** (16 compilation + 10 execution)

## Dependencies

- `cranelift-codegen` - RISC-V32 backend
- `cranelift-control` - Compilation control
- `cranelift-reader` - CLIF parser
- `lp-riscv-tools` - Disassembler
- `filecheck` - Pattern matching

## Key Verification: RV32 vs RV64

A critical aspect of these tests is ensuring the backend generates RV32 instructions, not RV64:

- ✅ `add` (RV32 opcode 0x33) - correct
- ❌ `addw` (RV64 opcode 0x3b) - wrong for RV32

Most tests include `check-NOT: <insn>w` to catch this mistake.

## Known Limitations

Some instructions may show as `unknown_r_type` in disassembly if the disassembler doesn't recognize them yet. Tests verify:
1. Code compiles without errors
2. RV64-specific instructions (with 'w' suffix) are NOT generated

## Future Enhancements

Potential improvements:
- [x] Add emulator execution verification (9 execution tests)
- [ ] Test branch and call instructions
- [ ] Test floating-point operations (if supported)
- [ ] Test RISC-V extensions (Zba, Zbb, Zbs, Zbc)
- [ ] Performance benchmarks
- [ ] Add more execution tests for division, memory ops, etc.

## Related Documentation

- [Cranelift IR Reference](../../cranelift/docs/ir.md)
- [RISC-V32 Backend](../../cranelift/codegen/src/isa/riscv32/)
- [RISC-V32 ISLE Cleanup Guide](../../.plans/riscv32-isle-cleanup-guide.md)
- [Migration Plan](../../.plans/01-lp-filetests-migration.md)

