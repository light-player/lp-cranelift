# Phase 5: Testing and Verification

## Goal

Verify the shared emulator implementation works correctly, improves performance, and handles edge cases.

## Implementation Steps

1. **Run existing filetests**
   - Run all GLSL filetests: `cargo test -p lp-glsl-filetests --test filetests`
   - Verify all tests still pass
   - Check for any regressions

2. **Measure performance improvement**
   - Run filetests with timing: `time cargo test -p lp-glsl-filetests --test filetests`
   - Compare before/after (if possible) or note current performance
   - Focus on test files with many `// run:` directives

3. **Verify shared context behavior**
   - Add debug logging to verify:
     - Builtins executable loaded once
     - Bootstrap init runs once
     - Object files are linked incrementally
     - Fresh emulator instances are created per test

4. **Test dirty state handling**
   - Create test case that hits instruction limit
   - Verify next test gets fresh emulator instance
   - Create test case that traps
   - Verify next test gets fresh emulator instance

5. **Test edge cases**
   - Test with single `// run:` directive (should still work)
   - Test with many `// run:` directives (should see performance benefit)
   - Test with different test files (shared context persists)
   - Test error handling (compilation errors, execution errors)

6. **Verify memory usage**
   - Check that buffers grow as expected (object files appended)
   - Verify symbol map grows correctly
   - Check for memory leaks (if possible)

7. **Code cleanup**
   - Remove any debug logging added during development
   - Remove any temporary code or TODOs
   - Fix all warnings
   - Ensure code is clean and readable

## Success Criteria

- All existing filetests pass
- Performance improvement is measurable (fewer bootstrap init runs)
- Shared context works correctly (builtins loaded once, bootstrap init once)
- Dirty state handling works (fresh emulator after instruction limit/traps)
- Edge cases handled correctly
- No regressions in test behavior
- Code is clean (no debug prints, TODOs, warnings)

## Notes

- Performance improvement may be hard to measure precisely, but should be noticeable
- Focus on correctness first, then performance
- If performance improvement is minimal, investigate why (maybe bootstrap init isn't the bottleneck)
- Ensure backward compatibility - existing code path should still work

