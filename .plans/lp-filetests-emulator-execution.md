---
name: lp-filetests-emulator-execution
overview: Extend lp-glsl-filetests to support running tests in riscv32 emulator by compiling standalone binaries with bootstrap code and extracting results from emulator memory.
todos:
  - id: design_execution_abstraction
    content: Design clean abstraction for execution backends (native JIT vs emulator)
    status: pending
  - id: implement_bootstrap_codegen
    content: Implement bootstrap code generation that calls test function and stores result in memory
    status: pending
  - id: implement_binary_compilation
    content: Implement compilation to standalone binary (not JIT) for riscv32 target
    status: pending
  - id: implement_emulator_runner
    content: Implement emulator execution runner that loads binary and extracts result from memory
    status: pending
  - id: integrate_emulator_backend
    content: Integrate emulator backend into test_run.rs execution abstraction
    status: pending
  - id: test_emulator_execution
    content: Test emulator execution with sample test cases
    status: pending
---

# Extend Filetests to Run in Emulator

## Overview

Extend lp-glsl-filetests to support running tests in riscv32 emulator by compiling standalone binaries with bootstrap code and extracting results from emulator memory. Abstract the execution infrastructure to cleanly separate native JIT execution from emulator execution, with extensibility for future emulator targets.

## Current State

- Target directive parsing implemented (`target host`, `target riscv32.fixed32`, etc.)
- Tolerance defaults based on target
- Test execution infrastructure exists but riscv32 emulator execution is stubbed
- Need to extract machine code bytes from JIT module (current blocker)

## Proposed Solution

Instead of extracting machine code from JIT module, compile standalone binaries with bootstrap code that:
1. Calls the test function
2. Stores the result in a known memory location
3. Halts execution (EBREAK or similar)
4. Emulator extracts result from memory

This approach:
- Works with existing compilation infrastructure
- Doesn't require JIT module code extraction
- Provides clean separation between compilation and execution
- Allows for future emulator targets

## Architecture

### Execution Backend Abstraction

```rust
trait ExecutionBackend {
    fn execute_float(&self, code: CompiledCode, fixed_point_format: Option<FixedPointFormat>) -> Result<f32>;
    fn execute_int(&self, code: CompiledCode, fixed_point_format: Option<FixedPointFormat>) -> Result<i32>;
    fn execute_i64(&self, code: CompiledCode, fixed_point_format: Option<FixedPointFormat>) -> Result<i64>;
    // ... other return types
}

struct NativeJitBackend {
    // Uses existing JIT compilation
}

struct EmulatorBackend {
    emulator_type: EmulatorType,  // Riscv32, future: Riscv64, etc.
    memory_layout: MemoryLayout,
}
```

### Bootstrap Code Generation

For emulator execution, generate bootstrap code that:
1. Sets up stack pointer
2. Calls the test function
3. Stores result in predefined memory location (e.g., address 0x80001000 in RAM)
4. Executes EBREAK to halt

Bootstrap signature:
- Entry point: `_start` or `main`
- Result storage: Fixed memory address (configurable per emulator)
- Halt instruction: EBREAK for riscv32

### Binary Compilation

Create new compilation path that:
1. Compiles GLSL to Cranelift IR (same as current)
2. Applies fixed-point transformation if needed
3. Compiles to object file for riscv32 target
4. Links with bootstrap code
5. Produces standalone binary (ELF format)

Use cranelift's object backend instead of JIT backend for emulator targets.

### Memory Layout

Define memory layout for emulator (standard RISC-V layout):
- Code region: 0x00000000 - 0x7FFFFFFF (lower half of address space)
- RAM region: 0x80000000 - 0xFFFFFFFF (upper half of address space, standard RISC-V)
- Result storage: 0x80001000 (in RAM, configurable offset from RAM base)
- Stack: 0x8000F000 - 0x80010000 (in RAM, grows downward)

### Emulator Execution Flow

1. Load binary into emulator (code region at 0x00000000)
2. Initialize stack pointer to 0x80010000 (top of stack in RAM)
3. Set PC to entry point
4. Run until EBREAK
5. Extract result from memory at 0x80001000 (in RAM)
6. Convert based on return type and fixed-point format

## Implementation Plan

### 1. Design Execution Backend Abstraction

**Files to create/modify:**
- `crates/lp-glsl-filetests/src/execution/` - New module for execution backends
- `crates/lp-glsl-filetests/src/execution/backend.rs` - Trait definition
- `crates/lp-glsl-filetests/src/execution/native.rs` - Native JIT backend
- `crates/lp-glsl-filetests/src/execution/emulator.rs` - Emulator backend

**Changes:**
- Define `ExecutionBackend` trait with methods for each return type
- Implement `NativeJitBackend` using existing JIT compilation
- Create `EmulatorBackend` struct (initially stubbed)
- Update `test_run.rs` to use execution backend abstraction

### 2. Implement Bootstrap Code Generation

**Files to create:**
- `crates/lp-glsl-filetests/src/execution/bootstrap.rs` - Bootstrap code generation

**Changes:**
- Generate riscv32 assembly/bootstrap code that:
  - Sets up stack pointer
  - Calls test function (function pointer or direct call)
  - Stores result at fixed memory address
  - Executes EBREAK
- Support different return types (i32, i64, f32, vec2, vec3, vec4, mat2, mat3, mat4)
- Handle fixed-point format conversion in bootstrap if needed

**Bootstrap code structure:**
```rust
const RESULT_ADDR: u32 = 0x80001000;  // In RAM region
const STACK_BASE: u32 = 0x80010000;   // Top of stack (grows downward)

fn generate_bootstrap(
    test_func_addr: u32,
    result_addr: u32,  // Default: RESULT_ADDR
    return_type: Type,
    fixed_point_format: Option<FixedPointFormat>,
) -> Vec<u8> {
    // Generate riscv32 instructions:
    // 1. lui sp, 0x80010  (load stack base: 0x80010000)
    // 2. addi sp, sp, 0    (stack pointer = 0x80010000)
    // 3. jal ra, test_func_addr (call test function)
    // 4. Store result at result_addr (in RAM: 0x80001000)
    // 5. ebreak
}
```

### 3. Implement Binary Compilation

**Files to create/modify:**
- `crates/lp-glsl/src/compiler.rs` - Add `compile_to_binary()` method
- `crates/lp-glsl/src/object.rs` - New module for object file generation

**Changes:**
- Use cranelift's `cranelift-object` backend instead of JIT
- Compile GLSL to Cranelift IR
- Apply fixed-point transformation
- Compile to object file for riscv32
- Link bootstrap code with test function
- Produce ELF binary

**Dependencies:**
- `cranelift-object` crate (for object file generation)
- `object` crate (for ELF manipulation/linking, or use simple linker)

**Alternative:** Use existing `compile_to_code()` from no_std compiler and manually create ELF structure.

### 4. Implement Emulator Runner

**Files to create:**
- `crates/lp-glsl-filetests/src/execution/emulator.rs` - Emulator execution
- `crates/lp-glsl-filetests/src/execution/riscv32.rs` - Riscv32-specific emulator logic

**Changes:**
- Load ELF binary into emulator memory
- Initialize emulator state (registers, memory)
- Execute until EBREAK
- Extract result from memory
- Convert result based on type and fixed-point format

**Memory extraction:**
```rust
const RESULT_ADDR: u32 = 0x80001000;  // In RAM region
const STACK_BASE: u32 = 0x80010000;   // Top of stack (grows downward)

fn extract_result(
    emu: &Riscv32Emulator,
    result_addr: u32,  // Default: RESULT_ADDR
    return_type: Type,
    fixed_point_format: Option<FixedPointFormat>,
) -> Result<f32> {
    match return_type {
        Type::I32 => {
            let value = emu.read_memory_i32(result_addr)?;
            match fixed_point_format {
                Some(Fixed16x16) => Ok(value as f32 / 65536.0),
                None => Ok(value as f32),  // For int tests
            }
        }
        Type::I64 => {
            let value = emu.read_memory_i64(result_addr)?;
            match fixed_point_format {
                Some(Fixed32x32) => Ok((value as f64 / 4294967296.0) as f32),
                None => Ok(value as f32),
            }
        }
        // ... other types
    }
}
```

### 5. Integrate Emulator Backend

**Files to modify:**
- `crates/lp-glsl-filetests/src/test_run.rs` - Use execution backend abstraction

**Changes:**
- Replace direct JIT calls with execution backend calls
- Select backend based on target:
  - `target host*` → `NativeJitBackend`
  - `target riscv32*` → `EmulatorBackend` with `Riscv32` emulator type
- Pass compiled code to backend for execution
- Backend handles compilation differences internally

### 6. Test Emulator Execution

**Files to create:**
- Test cases in `filetests/` with `target riscv32.fixed32` directives
- Integration tests in `tests/`

**Test cases:**
- Simple float operations
- Fixed-point operations (16.16 and 32.32)
- Vector operations
- Matrix operations
- Math functions (trigonometric, etc.)

## Design Decisions

### Why Bootstrap Instead of Direct Function Execution?

- **Simplicity**: Don't need to extract machine code from JIT module
- **Flexibility**: Can add setup/teardown code if needed
- **Debugging**: Easier to inspect memory state
- **Future-proof**: Works for any emulator target

### Why Separate Execution Backend?

- **Clean separation**: Compilation vs execution concerns
- **Testability**: Can test execution backends independently
- **Extensibility**: Easy to add new emulator targets
- **Maintainability**: Clear boundaries between components

### Memory Layout

- Fixed addresses for simplicity
- Configurable per emulator type
- Large enough for test functions
- Stack grows downward (standard convention)

### Binary Format

- ELF format for riscv32 (standard)
- Can be extended to other formats for other targets
- Simple linking: bootstrap + test function
- No external dependencies needed

## Dependencies

- `cranelift-object` - Object file generation
- `object` - ELF parsing/manipulation (optional, for linking)
- `lp-riscv-tools` - Already available, riscv32 emulator
- Existing compilation infrastructure

## Future Extensions

- **Other emulator targets**: Add `Riscv64`, `Aarch64`, etc. as new emulator types
- **Debugging support**: Add memory/register inspection
- **Performance metrics**: Track instruction count, cycles
- **Coverage**: Track which code paths executed
- **Fault injection**: Test error handling

## Migration Path

1. Implement execution backend abstraction
2. Move existing JIT execution to `NativeJitBackend`
3. Implement `EmulatorBackend` with bootstrap compilation
4. Update `test_run.rs` to use backends
5. Test with existing test cases
6. Add riscv32-specific test cases

## Example Usage

```glsl
// test compile
// test run
// target riscv32.fixed32

float main() {
    return sin(0.0);
}

// run: ~= 0.0
```

Execution flow:
1. Parse target → `riscv32.fixed32`
2. Select `EmulatorBackend` with `Riscv32` emulator
3. Compile GLSL to binary with bootstrap
4. Load binary into riscv32 emulator
5. Execute until EBREAK
6. Extract result from memory at 0x80001000 (in RAM)
7. Convert from fixed16x16 to float
8. Compare with expected value (tolerance: 0.001)

