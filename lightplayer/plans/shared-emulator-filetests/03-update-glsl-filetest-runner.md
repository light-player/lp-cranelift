# Phase 3: Update GLSL Filetest Runner

## Goal

Update the GLSL filetest runner to create and use a shared `SharedEmulatorContext` across all tests.

## Implementation Steps

1. **Create shared context at test suite level**
   - Location: `lightplayer/crates/lp-glsl-filetests/src/test_run/mod.rs`
   - Create `SharedEmulatorContext` once when test suite starts
   - Store in a way that persists across all test files and `// run:` directives
   - Options:
     - Thread-local static (if tests run in parallel)
     - Global static with Mutex (if tests run sequentially)
     - Pass through test runner state

2. **Update `run_test_file_with_line_filter`**
   - Get or create shared context (lazy initialization)
   - Pass shared context to compilation/execution path
   - Need to thread it through: `glsl_emu_riscv32_with_metadata` → `build_emu_executable`

3. **Update `glsl_emu_riscv32_with_metadata`**
   - Location: `lightplayer/crates/lp-glsl/src/frontend/mod.rs`
   - Add optional `shared_context: Option<&mut SharedEmulatorContext>` parameter
   - Pass through to `module.build_executable()`

4. **Update `GlModule<ObjectModule>::build_executable`**
   - Add optional `shared_context: Option<&mut SharedEmulatorContext>` parameter
   - Pass through to `build_emu_executable()`

5. **Handle "dirty" emulator state**
   - After test execution, check if emulator hit instruction limit or traps
   - If dirty: mark context as needing fresh emulator for next test
   - Implementation: track dirty state in execution result, or check emulator state
   - When creating next emulator, if dirty, create fresh instance (already handled by `create_emulator()`)

6. **Update execution path**
   - `execute_function` in `lightplayer/crates/lp-glsl-filetests/src/test_run/execution.rs`
   - After execution, check for instruction limit/trap errors
   - Mark shared context as dirty if needed (or just create fresh emulator next time)

## Success Criteria

- Shared context is created once per test suite
- All tests use the shared context
- Tests still pass with shared emulator
- Performance improvement is measurable (fewer bootstrap init runs)
- Dirty state detection works correctly
- All code compiles without warnings (except unused code that will be used later)
- All filetests pass

## Notes

- Need to handle test execution order - shared context should persist across all tests
- Dirty state: if a test hits instruction limit or traps, next test should get fresh emulator instance
- Stack pointer reset is handled in `create_emulator()`, so each test gets clean stack
- Function calls set up their own execution context, so PC/registers are handled per call

