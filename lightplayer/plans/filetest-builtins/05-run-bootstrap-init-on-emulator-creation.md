# Phase 5: Run bootstrap init on emulator creation

## Goal

Execute bootstrap init code once when emulator is created, handling user _init if present.

## Changes

1. **Add bootstrap init execution to emulator creation**:
   - After creating emulator with loaded code/RAM, set PC to entry point
   - Execute until bootstrap init completes (reaches halt or user _init returns)
   - Handle user _init execution if `init_address` is present

2. **Execution logic**:
   - Set PC to entry point (typically 0x0)
   - Execute emulator steps until:
     - User _init returns (if present)
     - Bootstrap code halts (if no user _init)
     - Error occurs (propagate error)

3. **Error handling**:
   - Bootstrap init failures should be fatal (fail fast)
   - Return clear error messages for init failures

4. **Update emulator state**:
   - After init, emulator should be ready for function calls
   - Stack pointer should be properly initialized
   - Memory should be initialized (.bss zeroed, .data copied)

## Files to Modify

- `lightplayer/crates/lp-glsl/src/exec/emu.rs` (glsl_emu_riscv32 function, after emulator creation)

## Success Criteria

- Bootstrap init runs once when emulator is created
- User _init is called if present, gracefully skipped if missing
- Emulator is ready for function calls after init
- Clear error messages for init failures
- Tests pass with bootstrap init execution

