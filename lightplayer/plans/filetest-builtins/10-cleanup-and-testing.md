# Phase 10: Cleanup and testing

## Goal

Remove temporary code, fix warnings, ensure all tests pass, clean up code, remove plan directory, and commit.

## Changes

1. **Remove temporary code**:
   - Remove any temporary code or TODOs
   - Remove debug print statements
   - Clean up any experimental or commented-out code

2. **Fix all compiler warnings**:
   - Remove unused code
   - Fix unused imports
   - Fix unused variables (or prefix with `_`)

3. **Run all tests**:
   - Run filetests: `cargo test -p lp-glsl-filetests --test filetests`
   - Run unit tests: `cargo test -p lp-glsl --lib`
   - Run object loader tests: `cargo test -p lp-riscv-tools --lib`
   - Fix any failing tests

4. **Code cleanup**:
   - Update comments and documentation
   - Ensure code follows project style guidelines
   - Ensure all code is clean and readable

5. **Verify functionality**:
   - Test direct function calls work correctly
   - Test user _init runs if present
   - Test graceful handling of missing _init
   - Test error messages are clear and helpful

6. **Remove plan directory**:
   - Delete `lightplayer/plans/filetest-builtins/` directory

7. **Commit changes**:
   - Commit with message: "lpc: complete plan filetest-builtins"
   - Include details of the effect of the plan in the commit message (but not implementation details)
   - Example: "Integrate object file loading into GLSL filetests, enabling direct function calls without main() wrappers. Rename user initialization from main to _init. Remove main() test files and bootstrap generation."

## Files to Review

- All modified files from previous phases
- Check for consistency and code quality
- Remove plan directory before committing

## Success Criteria

- All temporary code and TODOs removed
- All code compiles without warnings
- All tests pass
- Code is clean, well-documented, and maintainable
- Functionality works as expected
- Plan directory removed
- Changes committed with proper message

