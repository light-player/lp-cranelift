# ISLE Phase 2: 64-bit Operations on RV32

## Context

Phase 1 focused on getting proper 32-bit operations working on RV32. Phase 2 adds support for 64-bit operations using multi-instruction sequences. Since RV32 has 32-bit registers (XLEN=32), 64-bit values must be represented as register pairs and operations must be decomposed into multiple instructions.

**Key Insight**: The patterns used for I128 operations on RV64 can be adapted for I64 operations on RV32, since both involve register pairs and multi-instruction sequences.

## Strategy

We'll adapt the I128 lowering patterns from `cranelift/codegen/src/isa/riscv64/lower.isle` (41 references) to work for I64 on RV32. This involves:

1. Using `value_regs` to represent 64-bit values as (low, high) register pairs
2. Implementing multi-instruction sequences for arithmetic operations
3. Handling loads/stores of 64-bit values
4. Managing carries and borrows correctly

## Files to Modify

- `cranelift/codegen/src/isa/riscv32/lower.isle` - Add I64 lowering rules
- `cranelift/codegen/src/isa/riscv32/inst.isle` - May need helpers for register pairs
- `cranelift/codegen/src/isa/riscv32/abi.rs` - ABI for 64-bit arguments/returns
- `cranelift/filetests/` - Add tests for 64-bit operations

## Step 1: Understand I128 Patterns on RV64

The RV64 backend implements I128 operations using register pairs. Key example from `lower.isle` lines 106-114:

```isle
;; I128 iadd
(rule 7 (lower (has_type $I128 (iadd x y)))
  (let ((low XReg (rv_add (value_regs_get x 0) (value_regs_get y 0)))
        ;; compute carry
        (carry XReg (rv_sltu low (value_regs_get y 0)))
        ;; add high parts
        (high_tmp XReg (rv_add (value_regs_get x 1) (value_regs_get y 1)))
        ;; add carry to high
        (high XReg (rv_add high_tmp carry)))
    (value_regs low high)))
```

This pattern:
- Extracts low/high parts with `value_regs_get`
- Performs low-part addition
- Computes carry using `sltu` (set if less than unsigned)
- Adds high parts with carry
- Returns result as `value_regs`

## Step 2: Add I64 Constants

Add lowering for 64-bit constants on RV32:

```isle
;; 64-bit constant on RV32 - split into low/high 32-bit values
(rule (lower (has_type $I64 (iconst (u64_from_imm64 n))))
  (let ((low_bits u32 (u64_low_bits n))
        (high_bits u32 (u64_high_bits n))
        (low_reg XReg (imm $I32 low_bits))
        (high_reg XReg (imm $I32 high_bits)))
    (value_regs low_reg high_reg)))
```

We'll need helper functions:
- `u64_low_bits` - Extract lower 32 bits
- `u64_high_bits` - Extract upper 32 bits

## Step 3: Add I64 Arithmetic Operations

### 3a. Addition (iadd i64)

Adapt the I128 pattern:

```isle
(rule (lower (has_type $I64 (iadd x y)))
  (let ((x_lo XReg (value_regs_get x 0))
        (x_hi XReg (value_regs_get x 1))
        (y_lo XReg (value_regs_get y 0))
        (y_hi XReg (value_regs_get y 1))
        ;; Add low parts
        (sum_lo XReg (rv_add x_lo y_lo))
        ;; Compute carry: carry = (sum_lo < y_lo) ? 1 : 0
        (carry XReg (rv_sltu sum_lo y_lo))
        ;; Add high parts
        (sum_hi_tmp XReg (rv_add x_hi y_hi))
        ;; Add carry to high
        (sum_hi XReg (rv_add sum_hi_tmp carry)))
    (value_regs sum_lo sum_hi)))
```

### 3b. Subtraction (isub i64)

Similar pattern with borrow:

```isle
(rule (lower (has_type $I64 (isub x y)))
  (let ((x_lo XReg (value_regs_get x 0))
        (x_hi XReg (value_regs_get x 1))
        (y_lo XReg (value_regs_get y 0))
        (y_hi XReg (value_regs_get y 1))
        ;; Subtract low parts
        (diff_lo XReg (rv_sub x_lo y_lo))
        ;; Compute borrow: borrow = (x_lo < y_lo) ? 1 : 0
        (borrow XReg (rv_sltu x_lo y_lo))
        ;; Subtract high parts
        (diff_hi_tmp XReg (rv_sub x_hi y_hi))
        ;; Subtract borrow from high
        (diff_hi XReg (rv_sub diff_hi_tmp borrow)))
    (value_regs diff_lo diff_hi)))
```

### 3c. Multiplication (imul i64)

64-bit multiplication is complex. The RV64 I128 implementation (lines 497-521) provides the formula:

```
result_lo = x_lo * y_lo
result_hi = mulhu(x_lo, y_lo) + (x_lo * y_hi) + (x_hi * y_lo)
```

Implementation:

```isle
(rule (lower (has_type $I64 (imul x y)))
  (let ((x_lo XReg (value_regs_get x 0))
        (x_hi XReg (value_regs_get x 1))
        (y_lo XReg (value_regs_get y 0))
        (y_hi XReg (value_regs_get y 1))
        ;; High part: mulhu + cross products
        (hi1 XReg (rv_mulhu x_lo y_lo))
        (hi2 XReg (madd x_lo y_hi hi1))
        (result_hi XReg (madd x_hi y_lo hi2))
        ;; Low part
        (result_lo XReg (rv_mul x_lo y_lo)))
    (value_regs result_lo result_hi)))
```

### 3d. Division and Remainder

Division requires a software implementation or runtime call, as there's no native 64-bit division on RV32. Options:

1. **Library call**: Call `__udivdi3`, `__divdi3`, etc. from compiler-rt
2. **Inline sequence**: Multi-instruction division algorithm (expensive)
3. **Trap/unimplemented**: Mark as unsupported initially

Recommended: Start with library calls.

```isle
;; 64-bit unsigned division - call runtime helper
(rule (lower (has_type $I64 (udiv x y)))
  (call_runtime_helper "__udivdi3" (value_regs x y)))

;; 64-bit signed division - call runtime helper  
(rule (lower (has_type $I64 (sdiv x y)))
  (call_runtime_helper "__divdi3" (value_regs x y)))
```

Similar for `urem` (`__umoddi3`) and `srem` (`__moddi3`).

## Step 4: Add I64 Shifts and Rotates

Shifts require special handling when shift amount ≥ 32.

### 4a. Left Shift (ishl i64)

```isle
(rule (lower (has_type $I64 (ishl x (iconst amt))))
  (if-let shift_amt (u64_from_imm64 amt))
  (lower_i64_shl_const x shift_amt))

(decl lower_i64_shl_const (ValueRegs u64) ValueRegs)
(rule (lower_i64_shl_const x amt)
  ;; If amt >= 32: result_lo = 0, result_hi = x_lo << (amt - 32)
  ;; If amt < 32: result_lo = x_lo << amt
  ;;              result_hi = (x_hi << amt) | (x_lo >> (32 - amt))
  ...)
```

### 4b. Right Shifts (ushr, sshr i64)

Similar decomposition based on shift amount.

## Step 5: Add I64 Comparisons

Comparisons must handle multi-word values:

```isle
;; I64 equal: (x_hi == y_hi) && (x_lo == y_lo)
(rule (lower (has_type $I64 (icmp (IntCC.Equal) x y)))
  (let ((x_lo XReg (value_regs_get x 0))
        (x_hi XReg (value_regs_get x 1))
        (y_lo XReg (value_regs_get y 0))
        (y_hi XReg (value_regs_get y 1))
        (eq_lo XReg (rv_eq x_lo y_lo))   ;; x_lo == y_lo
        (eq_hi XReg (rv_eq x_hi y_hi))   ;; x_hi == y_hi
        (result XReg (rv_and eq_lo eq_hi)))  ;; both equal
    result))

;; I64 less than (signed): more complex, need to check high parts first
(rule (lower (has_type $I64 (icmp (IntCC.SignedLessThan) x y)))
  (let ((x_lo XReg (value_regs_get x 0))
        (x_hi XReg (value_regs_get x 1))
        (y_lo XReg (value_regs_get y 0))
        (y_hi XReg (value_regs_get y 1)))
    ;; If x_hi < y_hi: true
    ;; If x_hi > y_hi: false
    ;; If x_hi == y_hi: compare x_lo < y_lo (unsigned)
    (lower_i64_slt x_hi x_lo y_hi y_lo)))
```

## Step 6: Add I64 Loads and Stores

### 6a. Load (load i64)

Load as two 32-bit loads:

```isle
(rule (lower (has_type $I64 (load flags addr offset)))
  (let ((addr_lo AMode (addr_offset addr offset))
        (addr_hi AMode (addr_offset addr (offset + 4)))
        (lo_reg XReg (gen_load addr_lo (LoadOP.Lw) flags))
        (hi_reg XReg (gen_load addr_hi (LoadOP.Lw) flags)))
    (value_regs lo_reg hi_reg)))
```

### 6b. Store (store i64)

Store as two 32-bit stores:

```isle
(rule (lower (store flags value addr offset))
  (if-let $I64 (value_type value))
  (let ((addr_lo AMode (addr_offset addr offset))
        (addr_hi AMode (addr_offset addr (offset + 4)))
        (lo_reg XReg (value_regs_get value 0))
        (hi_reg XReg (value_regs_get value 1)))
    (seq
      (gen_store addr_lo lo_reg (StoreOP.Sw) flags)
      (gen_store addr_hi hi_reg (StoreOP.Sw) flags))))
```

## Step 7: Update ABI for I64 Arguments and Returns

The ABI must handle 64-bit values in register pairs. According to the RISC-V calling convention:

- I64 arguments use **two consecutive argument registers** (e.g., a0+a1, a2+a3)
- I64 returns use **a0 (low) and a1 (high)**
- Register pairs must respect alignment (may skip odd registers)

Update `cranelift/codegen/src/isa/riscv32/abi.rs`:

```rust
// For I64 arguments
match arg_ty {
    types::I64 => {
        // Need 2 consecutive registers
        if next_gpr % 2 != 0 {
            next_gpr += 1; // Align to even register
        }
        if next_gpr + 1 < MAX_GPRS {
            let low_reg = gpr_regs[next_gpr];
            let high_reg = gpr_regs[next_gpr + 1];
            next_gpr += 2;
            // Return ValueRegs pair
        } else {
            // Spill to stack
        }
    }
    // ...
}
```

## Step 8: Add Conversion Operations

### 8a. Extension (uextend, sextend to I64)

```isle
;; Zero-extend I32 to I64
(rule (lower (has_type $I64 (uextend x @ (value_type $I32))))
  (let ((lo_reg XReg (zext x))
        (hi_reg XReg (zero_reg)))  ;; High part is zero
    (value_regs lo_reg hi_reg)))

;; Sign-extend I32 to I64
(rule (lower (has_type $I64 (sextend x @ (value_type $I32))))
  (let ((lo_reg XReg (sext x))
        ;; High part: arithmetic shift right by 31 to replicate sign bit
        (hi_reg XReg (rv_srai lo_reg (imm12_const 31))))
    (value_regs lo_reg hi_reg)))
```

### 8b. Truncation (ireduce from I64)

```isle
;; Truncate I64 to I32 - just take low part
(rule (lower (has_type $I32 (ireduce x @ (value_type $I64))))
  (value_regs_get x 0))  ;; Return only low register
```

## Step 9: Helper Functions and Utilities

Add helper functions to support I64 operations:

```isle
;; Extract low/high 32 bits from u64
(decl pure u64_low_bits (u64) u32)
(extern extractor u64_low_bits u64_low_bits)

(decl pure u64_high_bits (u64) u32)
(extern extractor u64_high_bits u64_high_bits)

;; Conditional move for I64 (used in comparisons)
(decl lower_i64_select (ValueRegs ValueRegs ValueRegs) ValueRegs)
(rule (lower_i64_select cond true_val false_val)
  (let ((true_lo XReg (value_regs_get true_val 0))
        (true_hi XReg (value_regs_get true_val 1))
        (false_lo XReg (value_regs_get false_val 0))
        (false_hi XReg (value_regs_get false_val 1))
        (result_lo XReg (rv_select cond true_lo false_lo))
        (result_hi XReg (rv_select cond true_hi false_hi)))
    (value_regs result_lo result_hi)))
```

## Step 10: Testing Strategy

### 10a. Unit Tests

Create filetests in `cranelift/filetests/filetests/`:

```clif
test compile precise-output
target riscv32

function %iadd_i64(i64, i64) -> i64 {
block0(v0: i64, v1: i64):
    v2 = iadd v0, v1
    return v2
}

; check: add a0, a0, a2  ; low + low
; check: sltu a4, a0, a2 ; carry
; check: add a1, a1, a3  ; high + high
; check: add a1, a1, a4  ; high + carry
```

### 10b. Integration Tests

Test with actual compiled code:

```rust
#[test]
fn test_i64_add_rv32() {
    let mut builder = FunctionBuilder::new(...);
    let sig = builder.func.signature;
    sig.params.push(AbiParam::new(types::I64));
    sig.params.push(AbiParam::new(types::I64));
    sig.returns.push(AbiParam::new(types::I64));
    
    // v0 = param(0), v1 = param(1)
    // v2 = iadd v0, v1
    // return v2
    
    // Compile and verify instruction sequence
}
```

### 10c. Emulator Verification

The emulator in `crates/lp-riscv-tools/src/emu/emulator.rs` already supports I64 with register pairs (lines 399-415). Use it to verify:

```rust
let result = emulator.call_function(
    &compiled_code,
    &[DataValue::I64(0x123456789ABCDEF0)],
);
```

## Step 11: Performance Optimization

After basic I64 support works, optimize common patterns:

### 11a. Strength Reduction

- Multiply by power of 2 → shift
- Add constant → optimize immediate encoding

### 11b. Peephole Optimization

- Eliminate redundant register moves
- Combine operations where possible

### 11c. Special Cases

- Addition with immediate
- Shift by constant (avoid branches)
- Comparison with zero

## Implementation Checklist

- [ ] Add u64_low_bits and u64_high_bits helpers
- [ ] Implement I64 iconst lowering
- [ ] Implement I64 iadd lowering
- [ ] Implement I64 isub lowering
- [ ] Implement I64 imul lowering
- [ ] Implement I64 division (library calls)
- [ ] Implement I64 shift operations (const and variable)
- [ ] Implement I64 comparisons (all IntCC variants)
- [ ] Implement I64 load operations
- [ ] Implement I64 store operations
- [ ] Update ABI for I64 arguments and returns
- [ ] Implement I64 conversions (extend, reduce)
- [ ] Add comprehensive filetests
- [ ] Verify with emulator tests
- [ ] Performance optimization pass

## Success Criteria

✓ All I64 arithmetic operations work correctly  
✓ I64 loads and stores handle alignment properly  
✓ I64 function arguments and returns follow ABI  
✓ Comparisons produce correct boolean results  
✓ Shifts handle all shift amounts (0-63) correctly  
✓ Conversions between I32 and I64 work  
✓ Filetests pass for all I64 operations  
✓ Emulator tests verify correctness  
✓ Generated code is reasonably efficient

## Notes and Caveats

1. **Register Pressure**: I64 operations use twice as many registers. This may increase spilling.

2. **Code Size**: Multi-instruction sequences increase code size significantly.

3. **Performance**: I64 operations are much slower than I32 (4-6x for arithmetic, worse for mul/div).

4. **Alignment**: The ABI may require register pair alignment (even/odd) - verify against RISC-V ABI spec.

5. **Atomics**: I64 atomic operations may not be available on RV32 (or require RV32A `lr.d`/`sc.d` if present).

6. **Floating Point**: F64 operations are natively supported by RV32D extension (different from integer I64).

7. **Library Calls**: Division/remainder require linking with compiler-rt or equivalent runtime library.

## References

- RV64 I128 implementation in `cranelift/codegen/src/isa/riscv64/lower.isle`
- Emulator I64 support in `crates/lp-riscv-tools/src/emu/emulator.rs`
- ValueRegs infrastructure in `cranelift/codegen/src/machinst/valueregs.rs`
- RISC-V calling convention: https://github.com/riscv-non-isa/riscv-elf-psabi-doc

## Future Enhancements

- Optimize common I64 patterns
- Consider inline division for small divisors
- Add I128 support (4 registers) if needed
- Explore using RV32B bit manipulation for I64 operations

