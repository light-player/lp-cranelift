# 04: Add riscv32 Targets to Memory and Control Flow Tests

## Goal

Add `riscv32` targets to test files covering memory operations (load/store), control flow (branches, calls), and comparisons.

## Test Files to Update

### 4.1 i128-load-store.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-load-store.clif`

**Operations tested**:

- `stack_store.i128` - Stack store
- `stack_load.i128` - Stack load
- `store.i128` - Memory store
- `load.i128` - Memory load

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Memory operations are handled by the ABI and don't require special extensions. The ABI correctly handles 16-byte (128-bit) values.

**Note**: This test uses `enable_probestack=false`, which should work fine for riscv32.

### 4.2 i128-icmp.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-icmp.clif`

**Operations tested**:

- `icmp.i128 eq` - Equality comparison
- `icmp.i128 ne` - Inequality comparison
- `icmp.i128 slt` - Signed less than
- `icmp.i128 sgt` - Signed greater than
- `icmp.i128 ult` - Unsigned less than
- `icmp.i128 ugt` - Unsigned greater than
- And other comparison variants

**Changes needed**:

```clif
target riscv32
```

**Rationale**: All comparison operations are implemented via `lower_icmp_i128` helper (line 2582).

### 4.3 i128-br.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-br.clif`

**Operations tested**:

- Branches based on i128 comparisons

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Branch operations use the comparison lowering, which is implemented.

### 4.4 i128-call.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-call.clif`

**Operations tested**:

- Function calls with i128 arguments and return values

**Changes needed**:

```clif
target riscv32
```

**Rationale**: ABI handles i128 as 4 x I32 registers for arguments and returns. The test uses `enable_llvm_abi_extensions=true` which should be compatible.

**Important**: Verify that the ABI correctly handles:

- i128 arguments passed in registers (4 x I32)
- i128 return values in registers
- Stack spillage when registers are exhausted

### 4.5 i128-select.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-select.clif`

**Operations tested**:

- `select` - Conditional selection of i128 values

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Select operations use comparison results and should work with i128 comparisons.

### 4.6 i128-select-float.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-select-float.clif`

**Operations tested**:

- `select` with float conditions selecting i128 values

**Changes needed**:

```clif
target riscv32
```

**Rationale**: Similar to i128-select but with float conditions.

### 4.7 i128-min-max.clif

**Location**: `cranelift/filetests/filetests/runtests/i128-min-max.clif`

**Operations tested**:

- `smax`, `smin` - Signed min/max
- `umax`, `umin` - Unsigned min/max

**Changes needed**:

```clif
target riscv32
```

**Rationale**: All min/max operations are implemented (rules at lines 2206, 2225, 2244, 2263).

## Implementation Steps

1. For memory operation tests:

   - Verify ABI handles 16-byte stack slots correctly
   - Check that load/store operations use proper addressing modes
   - Ensure alignment is handled correctly (16-byte alignment for i128)

2. For control flow tests:

   - Verify comparison operations work correctly with 4-register i128 values
   - Check that branches use the correct comparison results
   - Ensure function call ABI handles i128 correctly

3. For each file:

   - Add `target riscv32` after riscv64 targets
   - Maintain existing flags (`enable_llvm_abi_extensions`, `enable_probestack`, etc.)

4. Test memory and control flow:
   ```bash
   cargo test --package cranelift-filetests --test filetests i128_load_store
   cargo test --package cranelift-filetests --test filetests i128_icmp
   cargo test --package cranelift-filetests --test filetests i128_call
   ```

## Potential Issues to Watch For

1. **ABI compatibility**: Ensure i128 arguments/returns use the correct register allocation (4 x I32)
2. **Stack alignment**: Verify 16-byte alignment for i128 stack operations
3. **Comparison ordering**: Ensure multi-register comparisons handle high/low parts correctly
4. **Call convention**: Verify that `enable_llvm_abi_extensions=true` works correctly with riscv32

## Expected Outcome

All memory and control flow test files will include riscv32 as a target, enabling testing of:

- 128-bit memory load/store operations
- 128-bit stack operations
- 128-bit comparisons and branches
- 128-bit function call ABI
- 128-bit conditional selection
- 128-bit min/max operations

## Files to Modify

- `cranelift/filetests/filetests/runtests/i128-load-store.clif`
- `cranelift/filetests/filetests/runtests/i128-icmp.clif`
- `cranelift/filetests/filetests/runtests/i128-br.clif`
- `cranelift/filetests/filetests/runtests/i128-call.clif`
- `cranelift/filetests/filetests/runtests/i128-select.clif`
- `cranelift/filetests/filetests/runtests/i128-select-float.clif`
- `cranelift/filetests/filetests/runtests/i128-min-max.clif`
