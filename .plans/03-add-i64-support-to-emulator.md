# Add i64 Support to RiscV32 Emulator

## Problem

The riscv32 emulator's `call_function()` method currently only supports I8, I16, and I32 types. When running filetests with i64 operations, the emulator fails with:

```
Emulator execution failed: InvalidInstruction { 
  reason: "Unsupported return type: types::I64", ... 
}
```

However, riscv32 DOES support i64 operations - they use register pairs and multi-instruction sequences. The emulator needs to be updated to handle i64 according to the RISC-V calling convention.

## RISC-V Calling Convention for i64 on RV32

On riscv32, i64 values use **register pairs**:

### Arguments (in order):
- **First i64**: `(a0, a1)` - low 32 bits in a0, high 32 bits in a1
- **Second i64**: `(a2, a3)` - low 32 bits in a2, high 32 bits in a3
- **Third i64**: `(a4, a5)` - low 32 bits in a4, high 32 bits in a5
- **Fourth i64**: `(a6, a7)` - low 32 bits in a6, high 32 bits in a7
- Further arguments go on stack

### Return Values:
- **Single i64**: `(a0, a1)` - low 32 bits in a0, high 32 bits in a1
- **Two i32 or one i32 + one i64**: Uses a0, a1, (a2, a3)
- Further returns go on stack

### Register Pair Order:
- **Little-endian convention**: Lower 32 bits in lower-numbered register, upper 32 bits in higher-numbered register
- Example: i64 value `0x0123456789ABCDEF` → a0=`0x89ABCDEF`, a1=`0x01234567`

## Implementation Plan

### File: `crates/lp-riscv-tools/src/emu/emulator.rs`

Modify the `call_function()` method to handle i64 types.

### Changes Needed

#### 1. Argument Setup (lines 375-403)

**Current code** only handles I8, I16, I32:
```rust
for (i, arg) in args.iter().enumerate() {
    let reg_value = match arg {
        DataValue::I8(v) => *v as i32,
        DataValue::I16(v) => *v as i32,
        DataValue::I32(v) => *v,
        _ => return Err(...),
    };
    self.regs[arg_reg_idx] = reg_value;
    arg_reg_idx += 1;
}
```

**New code** should handle I64 as register pairs:
```rust
for (i, arg) in args.iter().enumerate() {
    if arg_reg_idx > 17 {
        return Err(...); // Out of argument registers
    }

    match arg {
        DataValue::I8(v) => {
            self.regs[arg_reg_idx] = *v as i32;
            arg_reg_idx += 1;
        }
        DataValue::I16(v) => {
            self.regs[arg_reg_idx] = *v as i32;
            arg_reg_idx += 1;
        }
        DataValue::I32(v) => {
            self.regs[arg_reg_idx] = *v;
            arg_reg_idx += 1;
        }
        DataValue::I64(v) => {
            // i64 uses register pair: (low, high)
            if arg_reg_idx > 16 {
                // Need 2 registers, but only 1 available
                return Err(...);
            }
            let low = (*v & 0xFFFFFFFF) as i32;
            let high = ((*v >> 32) & 0xFFFFFFFF) as i32;
            self.regs[arg_reg_idx] = low;      // Lower register gets low 32 bits
            self.regs[arg_reg_idx + 1] = high; // Higher register gets high 32 bits
            arg_reg_idx += 2; // Consume 2 registers
        }
        _ => return Err(...),
    }
}
```

#### 2. Return Value Extraction (lines 447-478)

**Current code** only handles I8, I16, I32:
```rust
for (i, param) in signature.returns.iter().enumerate() {
    let reg_idx = 10 + i; // a0 or a1
    let reg_value = self.regs[reg_idx];
    
    let result_value = match param.value_type {
        types::I8 => DataValue::I8(reg_value as i8),
        types::I16 => DataValue::I16(reg_value as i16),
        types::I32 => DataValue::I32(reg_value),
        _ => return Err(...),
    };
    
    results.push(result_value);
}
```

**New code** should handle I64 from register pairs:
```rust
let mut reg_idx = 10; // Start at a0

for (i, param) in signature.returns.iter().enumerate() {
    if reg_idx > 17 {
        return Err(...); // Out of return registers
    }
    
    use cranelift_codegen::ir::types;
    let result_value = match param.value_type {
        types::I8 => {
            let value = DataValue::I8(self.regs[reg_idx] as i8);
            reg_idx += 1;
            value
        }
        types::I16 => {
            let value = DataValue::I16(self.regs[reg_idx] as i16);
            reg_idx += 1;
            value
        }
        types::I32 => {
            let value = DataValue::I32(self.regs[reg_idx]);
            reg_idx += 1;
            value
        }
        types::I64 => {
            if reg_idx > 16 {
                // Need 2 registers, but only 1 available
                return Err(...);
            }
            // i64 returned in register pair: (low, high)
            let low = self.regs[reg_idx] as u32 as u64;
            let high = self.regs[reg_idx + 1] as u32 as u64;
            let value = DataValue::I64((high << 32) | low);
            reg_idx += 2; // Consumed 2 registers
            value
        }
        _ => return Err(...),
    };
    
    results.push(result_value);
}
```

#### 3. Update Error Messages

Update the error message limits:
- Change "More than 8 arguments" check to account for register pairs
- Change "More than 2 return values" check to account for i64 using 2 registers

## Testing

After implementation, these tests should pass:

```bash
# Test basic i64 arithmetic
./target/debug/clif-util test cranelift/filetests/filetests/runtests/arithmetic.clif

# Test i64 constants
./target/debug/clif-util test cranelift/filetests/filetests/runtests/const.clif

# Test i64 with control flow
./target/debug/clif-util test cranelift/filetests/filetests/runtests/br.clif

# Test i64 comparisons
./target/debug/clif-util test cranelift/filetests/filetests/runtests/icmp.clif
```

## Edge Cases to Handle

1. **Mixed argument types**: e.g., `func(i32, i64, i32)` should use a0, (a2,a3), a4
   - Note: RISC-V calling convention aligns i64 arguments to even register pairs
   - So the second i64 might need to skip a register for alignment

2. **Multiple i64 returns**: e.g., `func() -> (i64, i64)` uses (a0,a1), (a2,a3)

3. **Alignment requirements**: Check if i64 arguments need even-register alignment on riscv32

4. **Stack arguments**: Not yet supported, but should fail gracefully

5. **Stack returns**: Not yet supported, but should fail gracefully

## References

- **RISC-V Calling Convention**: https://riscv.org/wp-content/uploads/2015/01/riscv-calling.pdf
- **Cranelift ABI implementation**: `cranelift/codegen/src/isa/riscv32/abi.rs`
- **Register pair handling**: See how riscv32 backend handles `value_regs` for I64
- **Current emulator code**: `crates/lp-riscv-tools/src/emu/emulator.rs:359-481`

## Alternative: Use Cranelift's ABI Implementation

Instead of manually implementing the calling convention, we could potentially:
1. Use Cranelift's `Riscv32MachineDeps` to determine argument/return locations
2. Query the ABI to get register assignments for each parameter
3. This would automatically handle alignment, stack args, etc.

However, this adds complexity and coupling. The manual approach is simpler for now.

## Implementation Steps

1. **Add i64 argument handling** in `call_function()` (lines 375-403)
2. **Add i64 return value handling** in `call_function()` (lines 447-478)
3. **Update error messages** to reflect register pair consumption
4. **Add tests** to verify i64 support works correctly
5. **Check alignment requirements** - research if i64 args need even-register alignment
6. **Document the calling convention** in code comments

## Expected Outcome

After implementation, all 41 riscv32 runtests added in the previous commit should be able to execute i64 functions successfully, including:
- i64 arithmetic (add, sub, mul, div, rem)
- i64 bit operations (shifts, rotates, etc.)
- i64 comparisons
- i64 constants
- i64 in control flow (function calls, returns, branches)

