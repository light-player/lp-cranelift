# Embive Toy Language JIT Demonstration

This document describes the embive demonstration that showcases running the toy language compiler in a `no_std` environment on `riscv32`.

## Overview

Successfully implemented a demonstration program that:
- Compiles toy language code to RISC-V machine code using Cranelift
- Runs in a `no_std` environment on `riscv32`  
- Uses the embive VM for execution
- Demonstrates JIT-like behavior (though compilation happens at build time due to `no_std` constraints)

## Components Created

### 1. Runtime Infrastructure (`crates/runtime-embive/`)
- Complete `no_std` runtime for embive VM
- Entry point and initialization code
- Global allocator using `linked_list_allocator`
- Syscall interface for communicating with host
- Panic handler
- Print macros for debugging output

### 2. RISC-V Shared Library (`crates/lp-riscv-shared/`)
- Compiles toy language programs to RISC-V machine code
- Uses Cranelift with riscv32 backend
- Custom simple ELF generator (since cranelift-object doesn't support riscv32)
- Provides utility function `compile_add_function()` for testing

### 3. Embive Program (`apps/embive-program/`)
- Main demonstration application
- Build script (`build.rs`) pre-compiles toy language at build time
- Runtime code loads pre-compiled ELF, transpiles to embive bytecode, and executes
- Demonstrates calling a JIT-compiled function with arguments

## Build Instructions

```bash
# Build the embive program
cargo build --package embive-program --target riscv32imac-unknown-none-elf --release

# The binary will be at:
# target/riscv32imac-unknown-none-elf/release/embive-program
```

## Demo Program

The demonstration compiles and executes this simple toy language program:

```
fn add(a, b) -> (result) {
    result = a + b
}
```

At runtime, it:
1. Loads the pre-compiled RISC-V ELF (compiled at build time)
2. Transpiles the ELF to embive bytecode
3. Executes `add(5, 3)` 
4. Verifies the result is `8`

## Technical Highlights

### Why Pre-compilation?
Cranelift requires `std` for compilation. Since the embive VM runs in `no_std`, we:
- Compile at **build time** (has `std` available)
- Embed the compiled ELF in the binary
- Only transpile and execute at **runtime** (no_std)

### Custom ELF Generation
cranelift-object doesn't support riscv32, so we implemented a minimal ELF generator that:
- Creates proper ELF header for 32-bit RISC-V
- Adds a loadable program header
- Embeds the machine code

### No_std Compatibility
All components properly handle `no_std`:
- `runtime-embive`: Provides allocator, panic handler, entry point
- `lp-riscv-shared`: Uses `alloc` only, no `std` dependencies in the library
- `lp-toy-lang`: Already had `no_std` support

## Files Modified/Created

### New Crates
- `crates/runtime-embive/` - Runtime support for embive
- `crates/lp-riscv-shared/` - Toy language to RISC-V compiler

### New Applications  
- `apps/embive-program/` - Main demonstration program

### Modified Files
- `Cargo.toml` - Added new workspace members

## Testing

The program can be tested by running it in an embive VM environment. The expected output shows:
1. Greeting message
2. Step-by-step progress through compilation and execution
3. Final result verification (should be 8)
4. Success message

## Future Enhancements

Potential improvements:
- Support for more complex toy language programs
- Function calls and recursion
- Better ELF generation with proper symbol tables
- Integration tests with actual embive VM


