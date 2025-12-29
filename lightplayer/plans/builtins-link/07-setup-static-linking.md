# Phase 7: Set up static linking for emulator

## Goal

Configure emulator tests to statically link the `.a` file, verify linking works.

## Steps

### 7.1 Understand emulator test structure

- Check how emulator tests are currently built
- Look at `lp-riscv-tools` test structure
- Understand how ELF files are loaded in emulator

### 7.2 Set up linking in test build

- Create `build.rs` in test crate (or modify existing)
- Link the `.a` file from `lp-builtins` target directory
- Use `cc` crate or linker flags to link static library

### 7.3 Create test program

- Create a simple test that calls `__lp_fixed32_div`, `__lp_fixed32_mul`, `__lp_fixed32_sqrt`
- Compile test program with linked builtins
- Generate ELF file

### 7.4 Load and test in emulator

- Use `elf_loader.rs` to load the test ELF
- Call builtin functions via emulator
- Verify results match expected values

## Implementation Approach

Option 1: Use `cc` crate in test `build.rs`:
```rust
cc::Build::new()
    .file("test.c")
    .static_flag(true)
    .object("path/to/liblp_builtins.a")
    .compile("test");
```

Option 2: Use linker flags in Cargo:
- Set `RUSTFLAGS` to include library path
- Link using `-L` and `-l` flags

## Files to Create/Modify

- Test crate `build.rs` (or modify existing)
- Simple test program that calls builtins

## Success Criteria

- Test program compiles and links successfully
- ELF file contains calls to builtin functions
- Emulator can load ELF and execute builtin calls
- Results match expected values

## Notes

- May need to handle symbol resolution (ensure `__lp_*` functions are found)
- Emulator uses `find_symbol_address` to locate functions
- Test should verify both function availability and correctness

