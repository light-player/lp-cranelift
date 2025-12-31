# Questions: Shared Emulator for Filetests

## Current State

Filetests have become significantly slower since builtin support was added. The bottleneck is that every test:
1. Loads the builtins executable (`lp-builtins-app`) from disk
2. Links the test's object file into the builtins executable
3. Creates a fresh emulator instance
4. Runs bootstrap init (initializes .bss/.data sections, ~10k instructions)

For a test file with multiple `// run:` directives, or when running many test files, this overhead is repeated for every single test.

## Proposed Solution

Share the emulator between tests by:
- Loading the builtins executable once (per file or globally)
- Reusing the same emulator instance
- For each test, just link the new object file on top of the existing emulator state
- Skip bootstrap init for subsequent tests (since we're assuming no `_init` functions)

## Questions

1. **Scope of sharing**: ✅ **ANSWERED**: Share globally across all tests (all files share one emulator)

2. **API Design**: ✅ **ANSWERED**: Create a `SharedEmulatorContext` that:
   - Holds base `code` and `ram` buffers (loaded from builtins once)
   - Maintains shared symbol map
   - Tracks bootstrap init status
   - Provides `link_object_file()` and `create_emulator()` methods
   - Modify `build_emu_executable` to optionally accept `&mut SharedEmulatorContext`

3. **State isolation**: ✅ **ANSWERED**: 
   - Each test creates a fresh emulator instance from shared buffers
   - Stack pointer should be reset to a safe location before each test
   - PC is set per function call, so no reset needed
   - If test hits instruction limit or traps, emulator is "dirty" - create fresh instance for next test
   - Function calls set up their own execution context, but stack pointer needs explicit reset

4. **Bootstrap init**: ✅ **ANSWERED**: 
   - Run bootstrap init once when creating shared context (after loading builtins)
   - Skip bootstrap init for subsequent tests
   - Each test gets fresh emulator instance, but function calls handle PC/stack setup

5. **Memory layout**: ✅ **ANSWERED**: 
   - `load_object_file` appends sections AFTER existing code/data (extends buffers)
   - Previous test's code/data remains in memory (not overwritten)
   - Symbol map tracks all symbols - base (builtins) takes precedence, then first object file, then subsequent ones
   - No need to track individual test locations - symbol map handles lookups

6. **Error handling**: ✅ **ANSWERED**: 
   - Each test creates a fresh emulator instance from shared buffers
   - If test hits instruction limit or traps → emulator is "dirty", create fresh instance for next test
   - Stack pointer reset needed before each test (set to safe location)
   - Shared buffers persist but don't interfere since each test uses its own emulator instance

7. **CLIF filetests**: ✅ **ANSWERED**: GLSL filetests only (`lp-glsl-filetests`) for now. Can extend to CLIF filetests later if needed.

