# 03: Add riscv32 Targets to Bitwise Operation Tests

## Goal

Add `riscv32` targets to test files covering bitwise operations, bit manipulation, and bit counting operations.

## Test Files to Update

### 3.1 i128-bitops.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bitops.clif`

**Operations tested**:

- `band` - Bitwise AND
- `bor` - Bitwise OR
- `bxor` - Bitwise XOR

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Basic bitwise operations are implemented and don't require special extensions.

### 3.2 i128-bnot.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bnot.clif`

**Operations tested**:

- `bnot` - Bitwise NOT

**Changes needed**:

```clif
target riscv32
```

### 3.3 i128-bandnot.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bandnot.clif`

**Operations tested**:

- `bandnot` - Bitwise AND-NOT

**Changes needed**:

```clif
target riscv32
```

### 3.4 i128-bornot.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bornot.clif`

**Operations tested**:

- `bornot` - Bitwise OR-NOT

**Changes needed**:

```clif
target riscv32
```

### 3.5 i128-bxornot.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bxornot.clif`

**Operations tested**:

- `bxornot` - Bitwise XOR-NOT

**Changes needed**:

```clif
target riscv32
```

### 3.6 i128-bitselect.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bitselect.clif`

**Operations tested**:

- `bitselect` - Bit select (ternary bitwise operation)

**Changes needed**:

```clif
target riscv32
```

### 3.7 i128-bswap.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bswap.clif`

**Operations tested**:

- `bswap` - Byte swap

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Implemented via `lower_bswap_i128` helper (rule 3, line 1143).

### 3.8 i128-bitrev.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bitrev.clif`

**Operations tested**:

- `bitrev` - Bit reverse

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Implemented (rule 1, line 1111).

### 3.9 i128-bitops-count.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bitops-count.clif`

**Operations tested**:

- `ctz` - Count trailing zeros
- `clz` - Count leading zeros
- `popcnt` - Population count

**Changes needed**:

```clif
target riscv32
```

**Rationale**: All implemented (rules at lines 1199, 1225, 1347, 1363).

### 3.10 i128-cls.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-cls.clif`

**Operations tested**:

- `cls` - Count leading sign bits

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Implemented (rule 2, line 1286).

### 3.11 i128-bmask.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bmask.clif`

**Operations tested**:

- `bmask` - Bit mask operations

**Changes needed**:

```clif
target riscv32
```

## Implementation Steps

1. For each file:

   - Add `target riscv32` after the riscv64 target declarations
   - No special extensions needed for bitwise operations
   - Maintain consistent formatting

2. Verify bitwise operation lowering:

   - Check that `band`, `bor`, `bxor` use standard RISC-V AND/OR/XOR instructions
   - Verify `bnot` uses XOR with -1 pattern
   - Confirm compound operations (bandnot, etc.) are properly lowered

3. Test bitwise operations:
   ```bash
   cargo test --package cranelift-filetests --test filetests i128_bitops
   cargo test --package cranelift-filetests --test filetests i128_bnot
   ```

## Expected Outcome

All bitwise operation test files will include riscv32 as a target, enabling testing of:

- Basic bitwise operations (AND, OR, XOR, NOT)
- Compound bitwise operations (AND-NOT, OR-NOT, XOR-NOT)
- Bit manipulation (byte swap, bit reverse)
- Bit counting operations (CTZ, CLZ, CLS, popcnt)
- Bit selection operations

## Files to Modify

- `cranelift/filetests/filetests/runtests/i128-bitops.clif`
- `cranelift/filetests/filetests/runtests/i128-bnot.clif`
- `cranelift/filetests/filetests/runtests/i128-bandnot.clif`
- `cranelift/filetests/filetests/runtests/i128-bornot.clif`
- `cranelift/filetests/filetests/runtests/i128-bxornot.clif`
- `cranelift/filetests/filetests/runtests/i128-bitselect.clif`
- `cranelift/filetests/filetests/runtests/i128-bswap.clif`
- `cranelift/filetests/filetests/runtests/i128-bitrev.clif`
- `cranelift/filetests/filetests/runtests/i128-bitops-count.clif`
- `cranelift/filetests/filetests/runtests/i128-cls.clif`
- `cranelift/filetests/filetests/runtests/i128-bmask.clif`
