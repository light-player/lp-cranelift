# 32-bit Target Test Suite

This directory contains tests specifically for 32-bit targets, particularly RISC-V 32-bit.

## Structure

- `runtests/` - Execution tests (similar to the main `runtests/` directory)
- `isa/riscv32/` - ISA-specific tests for RISC-V 32-bit

## Key Differences from 64-bit Tests

1. **Target Directive**: All tests include `target riscv32`
2. **Type Restrictions**: Tests avoid i128 operations (not supported)
3. **Extension Validation**: Tests verify that CPU extensions are properly validated
4. **32-bit Specific**: Focus on 32-bit operations and limitations

## Test Categories

### runtests/

Contains general CLIF instruction tests that are compatible with 32-bit targets:

- `arithmetic.clif` - Basic arithmetic operations (i32, i64 partial support)
- `control-flow.clif` - Jumps, branches, calls
- `memory.clif` - Load/store operations
- `floating-point.clif` - Floating point operations (requires F/D extensions)
- `conversions.clif` - Type conversions and casting

### isa/riscv32/

Contains RISC-V 32-bit specific tests:

- `basic.clif` - Basic RISC-V 32-bit operations
- `i64-support.clif` - Tests for i64 operations (partial support)
- Extension-specific tests for F, D, M, A, C extensions

## Validation Tests

Special test files exist to verify that validation works correctly:

- `validation-errors.clif` - Tests that unsupported features are properly rejected
- `validation-missing-extension.clif` - Tests extension requirement validation
- `validation-f64-requires-d.clif` - Tests type-to-extension mapping

## Running Tests

Tests are run using the standard filetest framework with `target riscv32`:

```bash
# Run all 32-bit tests
cargo test --features=cranelift/filetests -- filetests::32bit

# Run specific test category
cargo test --features=cranelift/filetests -- filetests::32bit::runtests
```

## i64 Arithmetic Support

i64 arithmetic is **not currently supported** on RISC-V32. While the validator allows i64 types for future GLSL fixed-point extensions, the actual code generation for i64 operations is not implemented.

### Current Status

- **Types**: i64 types are accepted by the validator (for future compatibility)
- **Operations**: No i64 operations are implemented in the ISLE lowering rules
- **Future**: i64 support may be added for Fixed32x32 format (32.32 fixed-point), but this is not currently implemented

### Not Supported

- **All i64 operations**: `iadd.i64`, `isub.i64`, `imul.i64`, `sdiv.i64`, `udiv.i64`, etc.
- **Overflow Detection**: `uadd_overflow`, `sadd_overflow`, `usub_overflow`, `ssub_overflow`, `umul_overflow`, `smul_overflow`
- **Carry Operations**: `iadd_cout`, `iadd_cin`, `isub_cout`, `isub_cin`, `iadd_carry`, `sadd_overflow_cin`, `uadd_overflow_cin`

## Test Selection Criteria

When copying tests from the main test suite, the following criteria were used:

1. **Compatible Types**: No i128 operations
2. **Supported Instructions**: Instructions that are supported on RISC-V 32-bit
3. **Extension Availability**: Instructions requiring only IMAC extensions (initial target)
4. **32-bit Relevance**: Tests that make sense for 32-bit targets

## Documentation Links

- [32-bit Validation Overview](../../plans/32-bit-validation/00-overview.md)
- [Infrastructure Setup](../../plans/32-bit-validation/01-infrastructure-setup.md)
- [Control Flow Instructions](../../plans/32-bit-validation/02-control-flow-instructions.md)
