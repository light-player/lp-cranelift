# Phase 4: Integrate object file loading into GLSL compilation

## Goal

Modify GLSL compilation to load object file into emulator alongside builtins executable, and populate function address map.

## Changes

1. **Update `glsl_emu_riscv32` function**:
   - After compiling GLSL to object file, load builtins executable
   - Load object file into emulator using `load_object_file`
   - Populate `function_addresses` map from merged symbol map
   - Store `init_address` if present

2. **Handle builtins executable loading**:
   - Find builtins executable path (similar to test code)
   - Load it using `load_elf`
   - Pass code/ram/symbol_map to `load_object_file`

3. **Populate function address map**:
   - After object file loading, iterate merged symbol map
   - Filter for function symbols (exclude data symbols, special symbols like `__USER_MAIN_PTR`)
   - Store function addresses in `function_addresses` map

4. **Update emulator creation**:
   - Create emulator with loaded code and RAM buffers
   - Ensure emulator is properly initialized

## Files to Modify

- `lightplayer/crates/lp-glsl/src/exec/emu.rs` (glsl_emu_riscv32 function)
- May need to add helper function to find builtins executable

## Success Criteria

- GLSL compilation loads object file into emulator
- Function addresses are populated in `function_addresses` map
- Builtins executable is loaded correctly
- Code compiles and basic tests pass
- Function addresses are correct (can verify with debug output)

