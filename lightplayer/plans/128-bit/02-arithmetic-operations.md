# 02: Add riscv32 Targets to Arithmetic Operation Tests

## Goal

Add `riscv32` and `riscv32 has_m` targets to test files covering basic arithmetic operations (add, subtract, multiply, negate, absolute value).

## Test Files to Update

### 2.1 i128-arithmetic.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-arithmetic.clif`

**Operations tested**:

- `iadd` - Addition
- `isub` - Subtraction
- `imul` - Multiplication
- `iadd_imm` - Addition with immediate

**Changes needed**:

```clif
target riscv32
target riscv32 has_m
```

Add after line 8 (after `target riscv64 has_c has_zcb`).

**Rationale**:

- `has_m` is required for multiplication operations (`imul`)
- Basic arithmetic operations are fully implemented

### 2.2 i128-arithmetic-extends.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-arithmetic-extends.clif`

**Operations tested**:

- `imul` with `uextend` and `sextend`

**Changes needed**:

```clif
target riscv32
target riscv32 has_m
```

**Rationale**: Tests extended multiplication which requires M extension.

### 2.3 i128-ineg.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-ineg.clif`

**Operations tested**:

- `ineg` - Negation

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Negation doesn't require special extensions.

### 2.4 i128-iabs.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-iabs.clif`

**Operations tested**:

- `iabs` - Absolute value

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Absolute value uses comparison and select, no special extensions needed.

## Implementation Steps

1. For each file:

   - Read the file to understand its structure
   - Find the target declarations section (usually near the top)
   - Add `target riscv32` line
   - Add `target riscv32 has_m` if multiplication is tested
   - Ensure proper placement (typically after riscv64 targets, before other targets)

2. Verify the format matches existing targets:

   - Check indentation
   - Ensure no duplicate targets
   - Maintain alphabetical or logical ordering if present

3. Test each file individually:
   ```bash
   cargo test --package cranelift-filetests --test filetests i128_arithmetic
   ```

## Expected Outcome

All arithmetic operation test files will include riscv32 as a target, enabling testing of:

- 128-bit addition with carry propagation
- 128-bit subtraction with borrow propagation
- 128-bit multiplication (4 x 32-bit register handling)
- Negation and absolute value operations

## Files to Modify

- `cranelift/filetests/filetests/runtests/i128-arithmetic.clif`
- `cranelift/filetests/filetests/runtests/i128-arithmetic-extends.clif`
- `cranelift/filetests/filetests/runtests/i128-ineg.clif`
- `cranelift/filetests/filetests/runtests/i128-iabs.clif`
