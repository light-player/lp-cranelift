# Overview: Shared Emulator for Filetests

## Problem

Filetests have become significantly slower since builtin support was added. Every test currently:
1. Loads the builtins executable (`lp-builtins-app`) from disk
2. Links the test's object file into the builtins executable
3. Creates a fresh emulator instance
4. Runs bootstrap init (initializes .bss/.data sections, ~10k instructions)

This overhead is repeated for every single `// run:` directive, making test runs very slow.

## Solution

Share the emulator infrastructure between tests by:
- Loading the builtins executable **once** (globally across all tests)
- Maintaining shared `code` and `ram` buffers that accumulate object files
- Creating fresh emulator instances from shared buffers (no bootstrap init needed after first)
- Detecting "dirty" emulator state (instruction limit/traps) and creating fresh instances when needed

## Key Design Decisions

- **Scope**: Share globally across all tests (all files share one context)
- **API**: Create `SharedEmulatorContext` that manages shared state
- **State isolation**: Each test gets a fresh emulator instance, but shares underlying buffers
- **Bootstrap init**: Run once when creating shared context, skip for subsequent tests
- **Memory layout**: Object files are appended (not overwritten), symbol map handles lookups
- **Error handling**: Detect dirty state and create fresh emulator instances
- **Scope**: GLSL filetests only (`lp-glsl-filetests`) for now

## Expected Impact

- Eliminate repeated builtins executable loading (once per test suite instead of per test)
- Eliminate repeated bootstrap init (once per test suite instead of per test)
- Significant speedup for test suites with many `// run:` directives
- Maintain backward compatibility (existing code path still works)

