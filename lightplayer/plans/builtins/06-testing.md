# Phase 6: Testing and Validation

**Goal**: Ensure all filetests pass, add edge case tests, validate accuracy.

## Tasks

1. **Run all builtin filetests**
   - Run all tests in `lightplayer/crates/lp-glsl-filetests/filetests/builtins/`
   - Document any failures
   - Fix any issues found

2. **Test edge cases**
   - Verify `edge-trig-domain.glsl` tests pass (domain errors)
   - Verify `edge-exp-domain.glsl` tests pass (domain errors)
   - Verify `edge-nan-inf-propagation.glsl` tests pass (NaN/infinity handling)
   - Verify `edge-precision.glsl` tests pass (precision requirements)
   - Verify `edge-component-wise.glsl` tests pass (vector operations)

3. **Validate accuracy**
   - Compare intrinsic results against reference implementations (libm)
   - Test key values for each function
   - Document accuracy achieved (±ULP measurements)
   - Identify any functions that need accuracy improvements

4. **Test dependency pruning**
   - Verify that only needed functions are compiled
   - Test various combinations of function calls
   - Ensure unused functions are not included in final code

5. **Test error reporting**
   - Verify error messages show correct file context
   - Test error messages for intrinsic files vs. user code
   - Ensure source locations are correct in error messages

6. **Performance testing (optional)**
   - Benchmark intrinsic functions vs. external calls
   - Identify any functions that are significantly slower
   - Document performance characteristics

7. **Documentation**
   - Document which functions are implemented as intrinsics
   - Document accuracy characteristics
   - Document any known limitations
   - Update overview document with completion status

## Files to Review/Modify

- All intrinsic GLSL files
- Test files in `lightplayer/crates/lp-glsl-filetests/filetests/builtins/`
- `lightplayer/plans/builtins/00-overview.md` (update success criteria)

## Acceptance Criteria

**Test File**: `lightplayer/crates/lp-glsl-filetests/filetests/builtins/phase/06-testing.glsl`

**Run Test**:
```bash
scripts/glsl-filetests.sh builtins/phase/06-testing.glsl
```

The phase is complete when:
- [ ] Acceptance test `06-testing.glsl` passes (verifies all phases work together)
- [ ] All builtin filetests pass
- [ ] Edge case tests pass
- [ ] Accuracy is acceptable (±2-3 ULP for most cases)
- [ ] Dependency pruning works correctly
- [ ] Error reporting is clear and accurate
- [ ] Documentation is updated

