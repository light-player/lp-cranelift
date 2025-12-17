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
