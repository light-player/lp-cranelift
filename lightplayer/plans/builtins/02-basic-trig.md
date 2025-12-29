# Phase 2: Basic Trig Functions (sin, cos, tan)

**Goal**: Implement and test `sin`, `cos`, `tan` with dependency tracking to only include needed functions.

## Progress Report

**Status**: Implementation complete, minor precision issues

**Completed**:
- ✅ Dependency tracking implemented and working
- ✅ Intrinsics compiler refactored to compile directly into GlModule
- ✅ Intrinsic functions properly added to GlModule.fns for transformation
- ✅ CodegenContext refactored to store GlModule instead of dyn Module
- ✅ Trig intrinsics refactored to use loops with arrays (replacing 16 unrolled iterations)
- ✅ Array support verified - trig intrinsics compile successfully
- ✅ Array initialization syntax updated to use initializer list `{...}`

**Current Status**:
- ✅ Trig intrinsics compile and run
- ⚠️ Minor precision issues in tests (e.g., sin(0.0) returns -0.00015258789 instead of 0.0)
- ⚠️ Tests failing due to precision tolerance - may need to adjust test tolerances or improve CORDIC precision

**Next Steps**:
- Investigate precision issues - may be due to fixed-point conversion or CORDIC algorithm precision
- Consider increasing CORDIC iterations or adjusting algorithm
- Update test tolerances if precision is acceptable for use case

## Tasks

1. **Implement dependency tracking**
   - Parse intrinsic GLSL file to build function dependency graph
   - When a function is requested, compute transitive closure of dependencies
   - Only compile functions that are directly called or transitively needed
   - Store dependency information in `IntrinsicCache`

2. **Verify existing `sin` implementation**
   - Review `trig.glsl` to ensure `__lp_sin` is correct
   - Verify `__lp_cos` calls `__lp_sin` correctly
   - Test that `sin()` and `cos()` work from user code

3. **Verify `tan` implementation**
   - Review `__lp_tan` in `trig.glsl`
   - Ensure it uses `__lp_reduce_angle` and `__lp_cordic_rotation` correctly
   - Test that `tan()` works from user code

4. **Test dependency pruning**
   - Call `sin()` and verify only `sin` and its dependencies are compiled
   - Call `cos()` and verify it includes `sin` but not `tan`
   - Call `tan()` and verify it includes its dependencies but not `cos` if not needed

5. **Run filetests**
   - Ensure `trig-sin.glsl`, `trig-cos.glsl`, `trig-tan.glsl` tests pass
   - Fix any issues found

## Files to Modify

- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/loader.rs` (dependency tracking)
- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/compiler.rs` (selective compilation)
- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/trig.glsl` (verify implementations)

## Implementation Notes

- Dependency tracking can be done by parsing the GLSL AST to find function calls
- When compiling, only compile functions in the dependency closure
- Cache the dependency graph per intrinsic file to avoid re-parsing

## Acceptance Criteria

**Test File**: `lightplayer/crates/lp-glsl-filetests/filetests/builtins/phase/02-basic-trig.glsl`

**Run Test**:
```bash
scripts/glsl-filetests.sh builtins/phase/02-basic-trig.glsl
```

The phase is complete when:
- [ ] `sin()`, `cos()`, `tan()` work correctly from user GLSL code
- [ ] Only needed functions are compiled (dependency pruning works)
- [ ] Acceptance test `02-basic-trig.glsl` passes
- [ ] All filetests for `sin`, `cos`, `tan` pass
- [ ] Error messages show correct source locations

