# Problem: Trap Instructions Not Triggering at Runtime

## Summary

The `trapnz` instruction is being generated correctly in CLIF IR for bounds checking on vector/matrix indexing, but traps are not being triggered at runtime in the RISC-V 32 emulator.

## Current State

- **Code Location**: `lightplayer/crates/lp-glsl/src/frontend/codegen/expr/component.rs::emit_bounds_check()`
- **Instruction Used**: `trapnz` with `TrapCode::user(1)` for "vector/matrix index out of bounds"
- **CLIF IR Generation**: ✅ Trap instructions are correctly emitted in the CLIF IR
- **Runtime Execution**: ❌ Traps are not being triggered when out-of-bounds indices are accessed

## Evidence

1. **Test File**: `lightplayer/crates/lp-glsl-filetests/filetests/bvec2/index-variable-bounds.glsl`

   - Contains tests expecting traps for negative indices and indices >= vector size
   - All tests currently fail because traps don't trigger

2. **CLIF IR Inspection**:

   - When inspecting generated CLIF IR with `DEBUG=1`, trap instructions are present
   - Example: `trapnz v8, user1` where `v8` is the out-of-bounds condition

3. **Execution Behavior**:
   - Code executes successfully instead of trapping
   - Returns default values (e.g., `false` for bool, `0` for int) instead of raising a trap

## Possible Root Causes

1. **Lowering Not Implemented**: The `trapnz` instruction may not have proper lowering rules for RISC-V 32 ISA

   - Check: `cranelift/codegen/src/isa/riscv32/lower.isle` and `riscv32/inst.isle`
   - May need to implement `gen_trapnz` helper or lowering rules

2. **Emulator Not Handling Traps**: The RISC-V 32 emulator may not be correctly executing trap instructions

   - Check: `lightplayer/crates/lp-riscv-tools/src/emu/emulator/execution.rs`
   - Verify that trap instructions are being recognized and handled

3. **Instruction Encoding**: The trap instruction may not be encoded correctly for the target ISA

   - Verify that trap instructions are being emitted in the machine code
   - Check disassembly output to see if trap instructions appear

4. **Condition Evaluation**: The out-of-bounds condition may not be evaluating correctly
   - Verify that the `bor` of `index_lt_zero` and `index_ge_bound` produces the expected I1 value
   - Check if `trapnz` expects a different type (I8/I32) instead of I1

## Investigation Steps

1. Check if `trapnz` lowering exists for riscv32 in `cranelift/codegen/src/isa/riscv32/`
2. Verify emulator trap handling in `lp-riscv-tools/src/emu/`
3. Inspect generated machine code/disassembly for trap instructions
4. Test with `trapz` (trap when zero) to see if the issue is specific to `trapnz`
5. Check if trap instructions need to be converted to a different form for the emulator

## Related Files

- `lightplayer/crates/lp-glsl/src/frontend/codegen/expr/component.rs` - Bounds check implementation
- `lightplayer/crates/lp-glsl-filetests/filetests/bvec2/index-variable-bounds.glsl` - Failing tests
- `lightplayer/crates/lp-riscv-tools/src/emu/error.rs` - Trap code to string conversion (already handles `user(1)`)
- `cranelift/codegen/src/isa/riscv32/lower.isle` - ISA lowering rules
- `cranelift/codegen/src/isa/riscv32/inst.isle` - ISA instruction generation

## Expected Behavior

When an out-of-bounds index is used (e.g., `vec[-1]` or `vec[4]` for a `vec2`), the code should:

1. Evaluate the bounds check condition
2. Execute `trapnz` when condition is true
3. Emulator should catch the trap and return `EmulatorError::Trap { code: TrapCode::user(1), ... }`
4. Test runner should match the trap against `EXPECT_TRAP` or `EXPECT_TRAP_CODE` directives
