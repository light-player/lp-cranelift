# RISC-V32 Filetests Fix Plans

This directory contains plans for fixing RISC-V32 filetest failures.

## Current Status

**Total Tests**: 51  
**Passing**: 22 (43%)  
**Failing**: 29 (57%)

See `00-summary.md` for overview and `PROGRESS-REPORT.md` for detailed analysis.

## Plan Files

### Summary and Progress

- `00-summary.md` - Overview and current status
- `PROGRESS-REPORT.md` - Detailed progress analysis with test breakdown

### Implementation Plans

#### Completed Phases ✅

- **Phase 1-2**: Instruction decoding - COMPLETED
- **Phase 3**: i64 handling - MOSTLY COMPLETED (5 tests remaining)
- **Phase 4**: ISLE panics - COMPLETED

#### Active Plans 🔄

**Phase 10: i64 Division/Remainder** (`10-fix-i64-division-remainder.md`)

- **Goal**: Implement i64 division (`udiv.i64`, `sdiv.i64`) and remainder (`urem.i64`, `srem.i64`) operations
- **Impact**: Should fix 5 tests
- **Status**: ⏳ Not Started
- **Estimated Time**: 10-15 hours

**Phase 11: Register Allocation Invalid Indices** (`11-fix-register-allocation-invalid-indices.md`)

- **Goal**: Fix invalid register indices in register pairs (15 tests failing)
- **Impact**: Should fix 15 tests (biggest blocker)
- **Status**: 🔄 In Progress
- **Estimated Time**: 10-15 hours

**Phase 12: Runtime Errors** (`12-fix-runtime-errors.md`)

- **Goal**: Fix runtime errors ("run" failures) - 8 tests failing
- **Impact**: Should fix 8 tests
- **Status**: ⏳ Not Started
- **Estimated Time**: 9-17 hours

**Phase 13: Compilation Errors** (`13-fix-compilation-errors.md`)

- **Goal**: Fix remaining compilation errors
- **Impact**: Should fix remaining 1-2 tests
- **Status**: ⏳ Not Started
- **Estimated Time**: 3-8 hours

## Quick Reference

### Running Tests

```bash
# Run all riscv32 tests
cargo run --package cranelift-tools --bin clif-util -- test $(find cranelift/filetests/filetests/runtests -name "*.clif" -exec grep -l "^target riscv32" {} \;)

# Run specific test
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/stack.clif

# Run with verbose output
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/stack.clif --verbose
```

### Key Files

- `cranelift/codegen/src/isa/riscv32/lower.isle` - ISLE lowering rules
- `cranelift/codegen/src/isa/riscv32/inst/emit.rs` - Instruction emission
- `cranelift/codegen/src/isa/riscv32/abi.rs` - ABI and calling conventions
- `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs` - Emulator core
- `lightplayer/crates/lp-riscv-tools/src/emu/executor.rs` - Instruction execution

## Priority Order

1. **Phase 11** (Register Allocation) - Biggest blocker, 15 tests
2. **Phase 10** (i64 Division) - Clear gap, 5 tests
3. **Phase 12** (Runtime Errors) - 8 tests, needs investigation
4. **Phase 13** (Compilation Errors) - Remaining issues

## Success Criteria

After completing all phases:

- **Target**: 45+ tests passing (88%+)
- **Remaining failures**: Minimal and well-documented
- **No panics**: All tests compile and run without crashing

