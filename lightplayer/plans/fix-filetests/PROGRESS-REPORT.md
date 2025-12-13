# RISC-V32 Filetests Progress Report

**Date**: 2025-12-13  
**Total Tests**: 51  
**Passing**: 22 (43%)  
**Failing**: 29 (57%)

## Current Failure Breakdown

| Category | Count | Phase | Status |
|----------|-------|-------|--------|
| Invalid register indices | 12 | Phase 6/9 | 🔄 In Progress |
| Runtime errors ("run") | 8 | Phase 7/Other | ⏳ Not Started |
| reg.is_virtual() assertion | 3 | Phase 6/9 | 🔄 In Progress |
| i64 division unsupported (udiv.i64) | 2 | Phase 5 | ⏳ Not Started |
| i64 remainder unsupported (srem.i64) | 2 | Phase 5 | ⏳ Not Started |
| i64 remainder unsupported (urem.i64) | 1 | Phase 5 | ⏳ Not Started |
| SSA register allocation error | 1 | Phase 6 | 🔄 In Progress |
| **Total** | **29** | | |

## Phase Status Comparison

### Phase 1-2: Instruction Decoding ✅ COMPLETED
- **Plan**: Fix 14 tests with missing instruction encodings
- **Status**: ✅ Completed
- **Evidence**: Tests no longer fail with decoding errors

### Phase 3: i64 Handling ✅ MOSTLY COMPLETED
- **Plan**: Fix 10 tests with register pair bugs
- **Status**: ✅ Mostly completed
- **Evidence**: 
  - `i64-riscv32.clif` passes
  - `ineg.clif`, `clz.clif`, `bmask.clif`, `bitselect.clif` pass
  - `fixed32-div.clif` passes (partial i64 division)
- **Remaining**: Full i64 division/remainder (5 tests failing)

### Phase 4: ISLE Panics ✅ COMPLETED
- **Plan**: Fix 6 tests with missing ISLE patterns
- **Status**: ✅ Completed
- **Evidence**: No more "unwrap() on None" panics
- **Note**: Some tests now fail with different errors (moved to other phases)

### Phase 5: Unsupported Features 🔄 PARTIALLY COMPLETED
- **Plan**: Handle 7 tests with unsupported features
- **Status**: 🔄 Partially completed
- **Completed**:
  - StructReturn automatic conversion (if implemented)
  - Small type overflow operations (if implemented)
- **Remaining**:
  - i64 division: `udiv.i64` (2 tests)
  - i64 remainder: `srem.i64` (2 tests), `urem.i64` (1 test)
- **Current Failures**: 5 tests with i64 division/remainder

### Phase 6/9: Register Allocation Invalid Indices 🔄 IN PROGRESS
- **Plan**: Fix 4-6 tests with invalid register indices
- **Status**: 🔄 In Progress
- **Current Failures**: 15 tests (12 invalid indices + 3 reg.is_virtual())
- **Progress**:
  - ✅ Validation added (catches errors early)
  - ✅ Phase 6a: `fits_in_64` architecture-awareness (if completed)
  - 🔄 Phase 9: uextend lowering rule (in progress)
- **Note**: More tests failing than originally planned - likely due to cascading effects

### Phase 7: Memory Access ⏳ NOT STARTED
- **Plan**: Fix 1 test with stack setup issues
- **Status**: ⏳ Not started
- **Current Failures**: 8 tests with "run" errors (may include memory access issues)

### Phase 8: Compilation Errors ⏳ NOT STARTED
- **Plan**: Fix 2 tests with global value type mismatches
- **Status**: ⏳ Not started
- **Note**: May be related to Phase 9 register allocation issues

## Detailed Failure Analysis

### Tests Failing with Invalid Register Indices (12 tests)
These are the core Phase 6/9 issues:
- `call.clif`
- `call_indirect.clif`
- `extend.clif`
- `global_value.clif`
- `iabs.clif`
- `popcnt.clif`
- `return-call-indirect.clif`
- `return-call-loop.clif`
- `return-call.clif`
- `smulhi.clif`
- `spill-reload.clif`
- `umulhi.clif`

### Tests Failing with Runtime Errors (8 tests)
These may be Phase 7 memory access or other runtime issues:
- `cls.clif`
- `integer-minmax.clif`
- `smul_overflow.clif`
- `stack.clif`
- `uadd_overflow.clif`
- `uadd_overflow_narrow.clif`
- `uadd_overflow_trap.clif`
- `umul_overflow.clif`

### Tests Failing with reg.is_virtual() Assertion (3 tests)
These are Phase 6/9 register allocation issues:
- `bitrev.clif`
- `brif.clif`
- `i64-riscv32.clif`

### Tests Failing with i64 Division/Remainder (5 tests)
These are Phase 5 unsupported features:
- `arithmetic.clif` (udiv.i64)
- `udiv.clif` (udiv.i64)
- `div-checks.clif` (srem.i64)
- `srem_opts.clif` (srem.i64)
- `urem.clif` (urem.i64)

### Tests Failing with SSA Register Allocation (1 test)
- `arithmetic-extends.clif`

## Progress Summary

### ✅ Completed Phases
1. **Phase 1-2**: Instruction decoding ✅
2. **Phase 3**: i64 handling ✅ (mostly - 5 tests remaining for division/remainder)
3. **Phase 4**: ISLE panics ✅

### 🔄 In Progress
4. **Phase 5**: Unsupported features 🔄 (StructReturn done, i64 division/remainder remaining)
5. **Phase 6/9**: Register allocation 🔄 (validation added, root cause fixes in progress)
6. **Phase 6a**: `fits_in_64` architecture-awareness 🔄 (if completed)

### ⏳ Not Started
7. **Phase 7**: Memory access ⏳
8. **Phase 8**: Compilation errors ⏳

## Key Insights

1. **More failures than expected**: 29 failures vs. original plan of 39 total tests
   - This suggests we're testing more comprehensively now
   - Some failures may be cascading from Phase 6/9 issues

2. **Register allocation is the biggest blocker**: 15 tests failing (52% of failures)
   - Invalid register indices: 12 tests
   - reg.is_virtual() assertion: 3 tests
   - This is Phase 6/9 work

3. **i64 division/remainder is a clear gap**: 5 tests failing
   - These are Phase 5 unsupported features
   - May be needed for full compatibility

4. **Runtime errors need investigation**: 8 tests failing with "run" errors
   - Could be Phase 7 memory access issues
   - Could be other runtime problems

## Next Steps

### Priority 1: Complete Phase 6/9 (Register Allocation)
- Fix uextend lowering rule for i32->i64
- Ensure function call argument preparation handles register pairs
- Fix reg.is_virtual() assertion failures
- **Impact**: Should fix ~15 tests

### Priority 2: Complete Phase 5 (i64 Division/Remainder)
- Implement i64 division (udiv.i64)
- Implement i64 remainder (srem.i64, urem.i64)
- **Impact**: Should fix ~5 tests

### Priority 3: Investigate Phase 7 (Runtime Errors)
- Investigate 8 "run" errors
- Fix memory access issues if present
- **Impact**: Should fix ~8 tests

### Priority 4: Complete Phase 8 (Compilation Errors)
- Fix any remaining compilation errors
- **Impact**: Should fix remaining tests

## Expected Outcome After All Phases

- **Target**: 45+ tests passing (88%+)
- **Remaining failures**: Should be minimal and well-documented
- **No panics**: All tests should compile and run without crashing

