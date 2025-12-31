# Phase 1: Create SharedEmulatorContext Structure

## Goal

Create a `SharedEmulatorContext` structure that manages shared emulator state across all tests.

## Implementation Steps

1. **Create new module/file for SharedEmulatorContext**
   - Location: `lightplayer/crates/lp-glsl/src/backend/codegen/shared_emulator.rs`
   - This will hold the shared context implementation

2. **Define SharedEmulatorContext struct**
   ```rust
   pub struct SharedEmulatorContext {
       code: Vec<u8>,
       ram: Vec<u8>,
       symbol_map: HashMap<String, u32>,
       bootstrap_done: bool,
       entry_point: u32,
   }
   ```

3. **Implement `new()` method**
   - Load builtins executable using `link_and_verify_builtins` (or `load_elf` + verification)
   - Store `code`, `ram`, `symbol_map`, `entry_point`
   - Set `bootstrap_done = false` initially
   - Run bootstrap init once (set up stack pointer, run init loop, mark `bootstrap_done = true`)

4. **Implement `link_object_file()` method**
   - Takes `&mut self` and `elf_bytes: &[u8]`
   - Calls `load_object_file()` to extend `code`/`ram` buffers
   - Updates `symbol_map` with merged symbols
   - Returns `ObjectLoadInfo` for caller to use

5. **Implement `create_emulator()` method**
   - Takes `&self`, `options: &EmulatorOptions`, `traps: &[(u32, TrapCode)]`
   - Creates new `Riscv32Emulator` with cloned `code`/`ram` buffers
   - Sets up stack pointer to safe location (high RAM)
   - Returns emulator instance (bootstrap init already done, so skip it)

6. **Add helper methods**
   - `get_symbol_map()` - return reference to symbol map
   - `is_bootstrap_done()` - check if bootstrap init completed
   - `code_size()` / `ram_size()` - get buffer sizes

## Success Criteria

- `SharedEmulatorContext` can be created from builtins executable
- Bootstrap init runs once and completes successfully
- `link_object_file()` can be called multiple times, extending buffers
- `create_emulator()` creates fresh emulator instances with proper stack setup
- All code compiles without warnings (except unused code that will be used later)
- Unit tests pass (if we add any)

## Notes

- Bootstrap init should only run once in `new()`, not in `create_emulator()`
- Stack pointer setup in `create_emulator()` should use safe location (high RAM, aligned)
- Symbol map merging: base (builtins) takes precedence, then object files in order

