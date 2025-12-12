# Fix RISC-V32 Emulator Test Failures - Summary

## Overview

This plan addresses 39 test failures when running Cranelift filetests with the riscv32 target using the emulator. The failures are categorized into 8 phases, each addressing a specific category of issues.

## Failure Breakdown

- **Phase 1-2**: Instruction decoding (14 tests) - Missing instruction encodings
- **Phase 3**: i64 handling (10 tests) - Register pair bugs
- **Phase 4**: ISLE panics (6 tests) - Missing patterns
- **Phase 5**: Unsupported features (4 tests) - StructReturn, small type overflow
- **Phase 6**: Register allocator (2 tests) - Invalid register indices
- **Phase 7**: Memory access (1 test) - Stack setup issues
- **Phase 8**: Compilation errors (2 tests) - Global value type mismatches

**Total**: 39 failures across 8 categories

## Phase Order

1. **Phase 1**: Investigate instruction encodings (prerequisite for Phase 2)
2. **Phase 2**: Fix instruction decoding (fixes 14 tests)
3. **Phase 3**: Fix i64 handling (fixes 10 tests)
4. **Phase 4**: Fix ISLE panics (fixes 6 tests)
5. **Phase 5**: Handle unsupported features (skips 4 tests)
6. **Phase 6**: Fix register allocator (fixes 2 tests)
7. **Phase 7**: Fix memory access (fixes 1 test)
8. **Phase 8**: Fix compilation errors (fixes 2 tests)

## Quick Start

To run all failing tests:

```bash
# Run all runtests with riscv32 target
cd cranelift
cargo run --bin clif-util -- test filetests/filetests/runtests/*.clif
```

To run a specific test:

```bash
cargo run --bin clif-util -- test filetests/filetests/runtests/arithmetic.clif
```

## Key Files

- `lightplayer/crates/lp-riscv-tools/src/decode.rs` - Instruction decoding
- `lightplayer/crates/lp-riscv-tools/src/emu/executor.rs` - Instruction execution
- `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs` - Emulator core, i64 handling
- `cranelift/codegen/src/isa/riscv32/lower/isle.rs` - ISLE lowering, panics
- `cranelift/filetests/src/test_run.rs` - Test execution, skip logic

## Estimated Time

- Phase 1: 1-2 hours (investigation)
- Phase 2: 4-6 hours (decoding fixes)
- Phase 3: 3-4 hours (i64 fixes)
- Phase 4: 2-3 hours (ISLE fixes)
- Phase 5: 1 hour (skip logic)
- Phase 6: 2-3 hours (regalloc fixes)
- Phase 7: 1 hour (memory fixes)
- Phase 8: 1-2 hours (compilation fixes)

**Total**: ~15-22 hours

## Success Criteria

After completing all phases:
- Most tests pass (expected: ~35+ out of 39)
- Remaining failures are documented and have clear paths forward
- No panics or crashes
- Clear error messages for unsupported features

## Next Steps

Start with Phase 1 to understand the instruction encoding issues, then proceed sequentially through the phases.

