# Phase 10: Add Unit Tests

## Goal
Add unit tests for object file loading, including base + object file scenarios.

## Changes Required

### 1. Create test object file
- Compile a simple PIC object file with `main` function
- Include relocations (PC-relative, GOT, absolute)
- Include symbols (defined and undefined)

### 2. Add tests in `elf_loader/object/mod.rs` or separate test file
- Test `load_object_file()` with valid object file
- Test loading base + object file
- Test symbol resolution (object → base, object → object)
- Test relocation application
- Test `__USER_MAIN_PTR` update
- Test multiple object files loaded sequentially

### 3. Add integration tests
- Test full workflow: load base, load object, run emulator
- Test that object file code executes correctly
- Test that object file can call base executable functions
- Test that base executable can call object file functions (if applicable)

### 4. Test error cases
- Undefined symbol error
- Invalid object file error
- Memory overflow error (if implemented)

## Implementation Details

- Use `#[cfg(test)]` for test modules
- Create test object files using `rustc --emit=obj`
- Use existing test infrastructure (similar to `elf_loader::tests`)

## Testing
- Run all tests: `cargo test --package lp-riscv-tools --lib elf_loader::object`
- Verify tests pass
- Verify tests are fast (base loaded once, multiple object files tested)

## Success Criteria
- All tests pass
- Tests cover main use cases
- Tests demonstrate faster iteration (base loaded once)

