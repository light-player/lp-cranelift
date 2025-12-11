# 05: Add riscv32 Targets to Conversion and Special Operation Tests

## Goal

Add `riscv32` targets to test files covering type conversions, extensions, reductions, shifts, rotates, and concatenation/splitting operations.

## Test Files to Update

### 5.1 i128-extend.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-extend.clif`

**Operations tested**:

- `uextend.i128` - Zero extension to i128
- `sextend.i128` - Sign extension to i128

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Extension operations are implemented (rules at lines 1310, 1330).

### 5.2 i128-ireduce.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-ireduce.clif`

**Operations tested**:

- `ireduce.i64` - Reduce i128 to i64
- `ireduce.i32` - Reduce i128 to i32
- `ireduce.i16` - Reduce i128 to i16
- `ireduce.i8` - Reduce i128 to i8

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Reduction operations extract the low bits, which should work correctly.

### 5.3 i128-conversion.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-conversion.clif`

**Operations tested**:

- `fcvt_to_uint.i128` - Float to unsigned i128
- `fcvt_to_sint.i128` - Float to signed i128
- `fcvt_to_uint_sat.i128` - Float to unsigned i128 (saturating)
- `fcvt_to_sint_sat.i128` - Float to signed i128 (saturating)

**Changes needed**:

```clif
target riscv32
```

**Rationale**: These operations typically require libcalls or special handling. Verify that cranelift's libcall infrastructure handles them correctly.

**Note**: The test file comment says "fcvt*to*{u,s}int.i128 not currently supported by any backend", so these may need libcall support.

### 5.4 i128-shifts.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-shifts.clif`

**Operations tested**:

- `ishl` - Left shift
- `ushr` - Unsigned right shift
- `sshr` - Signed right shift

**Changes needed**:

```clif
target riscv32
```

**Rationale**: All shift operations are implemented (rules at lines 1453, 1522, 1593).

### 5.5 i128-rotate.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-rotate.clif`

**Operations tested**:

- `rotl` - Rotate left
- `rotr` - Rotate right

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Rotate operations are implemented (rules at lines 1668, 1731).

### 5.6 i128-concat-split.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-concat-split.clif`

**Operations tested**:

- `iconcat` - Concatenate two values into i128
- `isplit` - Split i128 into two values

**Changes needed**:

```clif
target riscv32
```

**Rationale**:

- `iconcat` is implemented (line 2180)
- `isplit` is implemented (line 2172)
- Special cases for `isplit` of extended multiplies (lines 2189, 2194)

### 5.7 i128-bitcast.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-bitcast.clif`

**Operations tested**:

- `bitcast` - Bitcast between i128 and other types

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Bitcast operations should work correctly with register representation.

### 5.8 i128-urem.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-urem.clif`

**Operations tested**:

- `urem.i128` - Unsigned remainder

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Remainder operations typically require libcalls. Verify libcall support.

### 5.9 i128-srem.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-srem.clif`

**Operations tested**:

- `srem.i128` - Signed remainder

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Remainder operations typically require libcalls. Verify libcall support.

## Implementation Steps

1. For conversion operations:

   - Verify that float-to-i128 conversions have libcall support
   - Check that i128-to-smaller reductions work correctly
   - Ensure extensions handle sign/zero extension properly

2. For shift/rotate operations:

   - Verify that shift amounts are handled correctly (may need to extract from i128)
   - Check that multi-register shifts propagate correctly
   - Ensure rotates wrap around properly

3. For concat/split operations:

   - Verify that `iconcat` creates proper 4-register i128 values
   - Check that `isplit` extracts high/low parts correctly
   - Ensure special cases for extended multiplies work

4. For remainder operations:

   - Verify libcall infrastructure handles i128 remainder
   - Check that arguments are passed correctly (4 x I32 registers)
   - Ensure return values are handled correctly

5. For each file:

   - Add `target riscv32` after riscv64 targets
   - Maintain existing test flags

6. Test conversions and special operations:
   ```bash
   cargo test --package cranelift-filetests --test filetests i128_extend
   cargo test --package cranelift-filetests --test filetests i128_shifts
   cargo test --package cranelift-filetests --test filetests i128_rotate
   ```

## Potential Issues to Watch For

1. **Libcall support**: Float-to-i128 and remainder operations may need libcall support
2. **Shift amount**: i128 shift operations need to handle shift amounts correctly (extract from i128 or use low 7 bits)
3. **Register ordering**: Ensure concat/split handle register ordering correctly (low/high parts)
4. **Extension sign handling**: Verify sign extension works correctly for negative values

## Expected Outcome

All conversion and special operation test files will include riscv32 as a target, enabling testing of:

- Type extensions and reductions
- Float-to-integer conversions (via libcalls if needed)
- Shift and rotate operations
- Concatenation and splitting
- Bitcast operations
- Remainder operations (via libcalls if needed)

## Files to Modify

- `cranelift/filetests/filetests/runtests/i128-extend.clif`
- `cranelift/filetests/filetests/runtests/i128-ireduce.clif`
- `cranelift/filetests/filetests/runtests/i128-conversion.clif`
- `cranelift/filetests/filetests/runtests/i128-shifts.clif`
- `cranelift/filetests/filetests/runtests/i128-rotate.clif`
- `cranelift/filetests/filetests/runtests/i128-concat-split.clif`
- `cranelift/filetests/filetests/runtests/i128-bitcast.clif`
- `cranelift/filetests/filetests/runtests/i128-urem.clif`
- `cranelift/filetests/filetests/runtests/i128-srem.clif`
