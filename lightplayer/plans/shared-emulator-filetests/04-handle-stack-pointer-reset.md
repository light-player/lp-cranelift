# Phase 4: Handle Stack Pointer Reset

## Goal

Ensure stack pointer is properly reset to a safe location when creating emulator instances from shared context.

## Implementation Steps

1. **Verify stack pointer setup in `create_emulator()`**
   - Location: `lightplayer/crates/lp-glsl/src/backend/codegen/shared_emulator.rs`
   - In `create_emulator()`, after creating emulator instance:
     - Calculate safe stack pointer location (high RAM, aligned)
     - Set stack pointer using `emulator.set_register(Gpr::Sp, sp_value)`
     - Ensure alignment (16-byte aligned)

2. **Stack pointer calculation**
   - Use same logic as bootstrap init:
     - `sp_value = RAM_START + ram_size - 16` (aligned, with some space)
     - Or: `sp_value = RAM_START + ram_size - (ram_size % 16) - 16`
   - Ensure it's within valid RAM range

3. **Verify in function call path**
   - Function calls (`call_function`) set up their own stack frames
   - Stack pointer reset in `create_emulator()` ensures clean starting point
   - Function call setup will adjust stack pointer as needed for arguments/return area

4. **Test stack pointer reset**
   - Verify emulator instances start with correct stack pointer
   - Verify function calls work correctly with reset stack pointer
   - Verify multiple tests don't interfere with each other's stack

## Success Criteria

- Stack pointer is set to safe location in `create_emulator()`
- Stack pointer is properly aligned (16-byte aligned)
- Function calls work correctly with reset stack pointer
- Multiple tests don't interfere with each other's stack usage
- All code compiles without warnings
- Tests pass with stack pointer reset

## Notes

- Stack pointer should be set before any function calls
- Safe location: high RAM, aligned, with some space for stack growth
- Function calls will adjust stack pointer for their own frames, but need clean starting point
- Each test gets fresh emulator instance, so stack pointer reset happens per test

