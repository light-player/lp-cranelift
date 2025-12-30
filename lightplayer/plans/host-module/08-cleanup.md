# Phase 8: Cleanup

## Goal

Remove temporary code, fix warnings, ensure all tests pass, format code, and remove plan directory.

## Tasks

1. Remove temporary code:
   - Remove any TODOs or FIXMEs
   - Remove debug prints or temporary logging
   - Clean up commented-out code

2. Fix warnings:
   - Fix all compiler warnings
   - Only allow unused code warnings if code will be used later
   - Ensure no dead code warnings

3. Ensure all tests pass:
   - Run all tests in `lp-builtins`
   - Run GLSL filetests that use host functions
   - Verify emulator and JIT tests pass

4. Code quality:
   - Ensure code is clean and readable
   - Add documentation where needed
   - Follow existing code style

5. Format code:
   - Run `cargo +nightly fmt` in `lightplayer/` directory
   - Ensure consistent formatting

6. Remove plan directory:
   - Delete `lightplayer/plans/host-module/` directory
   - Plan is complete

## Success Criteria

- No temporary code or TODOs remain
- No compiler warnings
- All tests pass
- Code is formatted and readable
- Plan directory removed

