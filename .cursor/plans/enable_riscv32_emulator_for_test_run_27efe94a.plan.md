---
name: Enable riscv32 emulator for test run
overview: Modify `test run` to automatically use the RISC-V emulator when targeting riscv32, enabling runtests to work with `target riscv32` without needing a separate `test emu` directive.
todos:
  - id: move-emulator-logic
    content: Move emulator execution logic from test_emu.rs into test_run.rs, adapting it to work within the test_run context
    status: completed
  - id: modify-compile-testfile
    content: Modify compile_testfile in test_run.rs to use ObjectTestFileCompiler for riscv32 (ELF) and TestFileCompiler for native targets (JIT)
    status: completed
    dependencies:
      - move-emulator-logic
  - id: modify-run-test
    content: Modify run_test in test_run.rs to detect riscv32 architecture and branch to emulator execution instead of native trampoline execution
    status: completed
    dependencies:
      - move-emulator-logic
      - modify-compile-testfile
  - id: remove-test-emu
    content: Delete test_emu.rs and remove its registration from lib.rs (remove mod declaration and match case)
    status: completed
    dependencies:
      - modify-run-test
  - id: check-test-files
    content: Search for any .clif files using test emu directive and update them to use test run
    status: completed
    dependencies:
      - remove-test-emu
  - id: test-riscv32-runtests
    content: Test arithmetic.clif and other runtests with target riscv32 to verify test run works correctly
    status: completed
    dependencies:
      - modify-run-test
  - id: verify-other-targets
    content: Verify that native targets (x86_64, aarch64) still work correctly with test run
    status: completed
    dependencies:
      - modify-run-test
---

# Enable riscv32 Emulator Execution for `test run`

## Problem Analysis

Currently:

- `test run` compiles riscv32 code but fails at execution because it tries native trampoline execution
- `test emu` exists as a separate test type that uses the emulator
- `test run` already has special handling for riscv32 compilation (lines 172-179 in `test_run.rs`), but execution still uses native path

The goal is to make `test run` work seamlessly for riscv32 targets by automatically using the emulator when needed, and completely remove `test emu`.

## Solution Approach

1. Integrate emulator execution logic directly into `test_run.rs` for riscv32 targets
2. Remove `test emu` entirely (delete `test_emu.rs` and its registration)
3. `test run` automatically detects riscv32 and uses emulator, other architectures use native execution

## Implementation Plan

### 1. Move emulator execution logic into `test_run.rs`

- Copy the emulator execution code from `test_emu.rs` into `test_run.rs`
- Adapt it to work within the `test_run` context
- File: `cranelift/filetests/src/test_run.rs`

### 2. Modify `test_run.rs` compilation

- Update `compile_testfile` to compile to object format (ELF) when targeting riscv32
- Keep JIT compilation for native targets
- Use `ObjectTestFileCompiler` for riscv32, `TestFileCompiler` for others
- File: `cranelift/filetests/src/test_run.rs`

### 3. Modify `test_run.rs` execution

- In `run_test`, detect riscv32 architecture
- Branch to emulator execution path for riscv32, native trampoline execution for others
- File: `cranelift/filetests/src/test_run.rs` (lines 193-217)

### 4. Remove `test emu`

- Delete `cranelift/filetests/src/test_emu.rs`
- Remove `test_emu` module declaration from `lib.rs`
- Remove `"emu"` case from `new_subtest` function in `lib.rs`
- File: `cranelift/filetests/src/lib.rs`

### 5. Update any test files using `test emu`

- Find any `.clif` files that use `test emu` directive
- Change them to use `test run` instead (if any exist)
- Search for: `grep -r "test emu" cranelift/filetests/filetests/`

### 6. Test the changes

- Verify `arithmetic.clif` with `target riscv32` works with `test run`
- Ensure other targets (x86_64, aarch64, etc.) still work correctly
- Verify no references to `test emu` remain

## Key Files to Modify

1. `cranelift/filetests/src/test_run.rs`

- Add emulator execution logic (from `test_emu.rs`)
- Modify `compile_testfile` to support both JIT and object compilation
- Modify `run_test` to branch on architecture

2. `cranelift/filetests/src/lib.rs`

- Remove `mod test_emu;` declaration
- Remove `"emu" => test_emu::subtest(parsed),` case

3. `cranelift/filetests/src/test_emu.rs`

- **DELETE** this file entirely

4. `cranelift/filetests/src/object_runner.rs` (already exists)

- Ensure types/functions are accessible for use in `test_run`

## Design Decisions

- **Remove `test emu` completely**: Cleaner architecture with single `test run` that handles both native and emulated execution
- **Automatic detection**: `test run` automatically uses emulator for riscv32, no test file changes needed
- **Code integration**: Move emulator logic directly into `test_run.rs` rather than sharing (simpler since we're removing the other user)

## Testing Strategy

1. Test `arithmetic.clif` with `target riscv32` and `test run`
2. Test other runtests with riscv32 target
3. Verify native targets (x86_64, aarch64) still work
4. Verify `test interpret` still works (should be unaffected)
5. Search for any remaining references to `test emu` and verify they're removed