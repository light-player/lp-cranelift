# Phase 7: Integration and Testing

## Goal

Test host functions in all three contexts (emulator, JIT, tests) and verify functionality works correctly.

## Tasks

1. Create test in `lp-builtins`:
   - Unit test that uses `host::debug!` and `host::println!`
   - Verify output appears (with `DEBUG=1` for debug)
   - Test format strings work correctly

2. Test in emulator context:
   - Create GLSL test file that calls host functions
   - Verify output appears via syscalls
   - Test both `debug!` and `println!`

3. Test in JIT context:
   - Create GLSL test that uses host functions
   - Verify output appears via `lp-glsl` macros
   - Test format strings

4. Verify all contexts work:
   - Emulator: syscall-based output
   - JIT: delegate to `lp-glsl` macros
   - Tests: `std::println!` with env var check

5. Test edge cases:
   - Empty format strings
   - Various format specifiers (`{}`, `{:x}`, `{:?}`, etc.)
   - Multiple arguments

## Success Criteria

- All tests pass in all three contexts
- Output appears correctly in each context
- Format strings work as expected
- No runtime errors or panics
- Code compiles without warnings

