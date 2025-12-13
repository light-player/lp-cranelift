# Fix RISC-V32 Emulator Test Failures - Summary

## Current Status (2025-12-13)

**Total Tests**: 51  
**Passing**: 22 (43%)  
**Failing**: 29 (57%)

See `PROGRESS-REPORT.md` for detailed progress analysis.

## Overview

This plan addresses test failures when running Cranelift filetests with the riscv32 target using the emulator. The failures are categorized into 8 phases, each addressing a specific category of issues.

## Failure Breakdown

- **Phase 1-2**: Instruction decoding (14 tests) - Missing instruction encodings ✅ **COMPLETED**
- **Phase 3**: i64 handling (10 tests) - Register pair bugs ✅ **MOSTLY COMPLETED** (5 tests remaining for i64 division/remainder)
- **Phase 4**: ISLE panics (6 tests) - Missing patterns ✅ **COMPLETED**
- **Phase 5**: Unsupported features (7 tests) - StructReturn, small type overflow, **i64 division/remainder** (5 tests remaining)
- **Phase 6/9**: Register allocator (15 tests) - Invalid register indices (12 tests), reg.is_virtual() assertion (3 tests) 🔄 **IN PROGRESS**
- **Phase 6a**: Fix `fits_in_64` architecture-awareness - Root cause fix for Phase 6 regalloc issues 🔄 **IN PROGRESS**
- **Phase 7**: Memory access (8 tests) - Runtime errors ("run" failures) ⏳ **NOT STARTED**
- **Phase 8**: Compilation/runtime errors (2 tests) - Global value type mismatches ⏳ **NOT STARTED**

**Current**: 29 failures across 8 categories (out of 51 total tests)

**Note**: After Phase 4 completion, some tests that previously panicked now fail with different errors:

- 3 call tests fail with f64 compilation errors (moved to Phase 5)
- 2 call tests fail with regalloc panics (moved to Phase 6)
- 1 call test fails with runtime relocation (moved to Phase 8)

## Phase Order

1. **Phase 1**: Investigate instruction encodings (prerequisite for Phase 2) ✅ **COMPLETED**
2. **Phase 2**: Fix instruction decoding (fixes 14 tests) ✅ **COMPLETED**
3. **Phase 3**: Fix i64 handling (fixes 10 tests) ✅ **MOSTLY COMPLETED** (5 tests remaining)
4. **Phase 4**: Fix ISLE panics (fixes 6 tests) ✅ **COMPLETED**
5. **Phase 5**: Handle unsupported features (5 tests remaining for i64 division/remainder) 🔄 **PARTIALLY COMPLETED**
6. **Phase 6/9**: Fix register allocator (fixes 15 tests) 🔄 **IN PROGRESS** - Validation added, root cause fixes ongoing
   6a. **Phase 6a**: Fix `fits_in_64` architecture-awareness (completes Phase 6 root cause fix) 🔄 **IN PROGRESS**
7. **Phase 7**: Fix memory access (fixes 8 tests) ⏳ **NOT STARTED**
8. **Phase 8**: Fix compilation errors (fixes 2 tests) ⏳ **NOT STARTED**

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
- Phase 6: 2-3 hours (regalloc fixes - partial)
- Phase 6a: 4-6 days (architecture-aware `fits_in_64` - root cause fix)
- Phase 7: 1 hour (memory fixes)
- Phase 8: 1-2 hours (compilation fixes)

**Total**: ~15-22 hours + 4-6 days for Phase 6a

## Success Criteria

After completing all phases:

- Most tests pass (target: 45+ out of 51, ~88%+)
- Remaining failures are documented and have clear paths forward
- No panics or crashes
- Clear error messages for unsupported features

## Current Progress

- ✅ **Completed**: Phases 1-4 (instruction decoding, i64 handling basics, ISLE panics)
- 🔄 **In Progress**: Phases 5-6/9 (unsupported features, register allocation)
- ⏳ **Not Started**: Phases 7-8 (memory access, compilation errors)

## Next Steps

1. **Priority 1**: Complete Phase 11 (register allocation invalid indices) - should fix ~15 tests
   - See `11-fix-register-allocation-invalid-indices.md`
2. **Priority 2**: Complete Phase 10 (i64 division/remainder) - should fix ~5 tests
   - See `10-fix-i64-division-remainder.md`
3. **Priority 3**: Investigate Phase 12 (runtime errors) - should fix ~8 tests
   - See `12-fix-runtime-errors.md`
4. **Priority 4**: Complete Phase 13 (compilation errors) - should fix remaining tests
   - See `13-fix-compilation-errors.md`

## Detailed Plans

- **Phase 10**: `10-fix-i64-division-remainder.md` - Implement i64 division/remainder operations
- **Phase 11**: `11-fix-register-allocation-invalid-indices.md` - Fix invalid register indices in register pairs
- **Phase 12**: `12-fix-runtime-errors.md` - Fix runtime errors ("run" failures)
- **Phase 13**: `13-fix-compilation-errors.md` - Fix remaining compilation errors

See `PROGRESS-REPORT.md` for detailed analysis.
