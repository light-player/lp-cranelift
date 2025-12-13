# Phase 5: Implement Unsupported Features

## Goal

Implement features required for riscv32 that are currently unsupported:

1. **StructReturn automatic conversion** (CRITICAL - Light Player depends on this)
2. **Small type overflow operations** (i8/i16 overflow operations)
3. **Multiply overflow operations** (smul_overflow, umul_overflow)

**Note**: f64 operations are a separate issue and will be handled in a future phase.

## Prerequisites

- Phase 4 completed: ISLE panics fixed (tests now fail with clearer errors)

## Analysis

See `05-unsupported-features-analysis.md` for detailed analysis of why these features are needed.

### Why StructReturn is Critical

Light Player **actively uses StructReturn** for vector/matrix returns from GLSL shaders:

- `lp-glsl/src/backend/emu.rs` - Uses StructReturn for vector returns
- `lp-glsl/src/codegen/expr/function.rs` - Sets up StructReturn buffers
- `lp-jit-util/src/call.rs` - Has `call_structreturn_riscv32()` implementation

**Current Problem**: When functions return >2 register values (e.g., `i64, i8` = 3 registers), the compiler errors instead of automatically converting to StructReturn.

## Affected Test Files

These tests currently fail and will pass after implementation:

```bash
# StructReturn tests:
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/smul_overflow.clif --target riscv32
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/umul_overflow.clif --target riscv32

# Small type overflow tests:
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/uadd_overflow_narrow.clif --target riscv32
```

**Note**: `stack.clif` may be a false positive - it tests stack operations, not StructReturn.

## Error Patterns

1. **StructReturn**:

   ```
   Unsupported feature: Too many return values to fit in registers. Use a StructReturn argument instead. (#9510)
   ```

2. **Small type overflow operations**:

   ```
   Unsupported feature: should be implemented in ISLE: inst = `v2, v3 = uadd_overflow.i8 v0, v1`, type = `Some(types::I8)`
   ```

3. **Multiply overflow operations**:
   ```
   Unsupported feature: should be implemented in ISLE: inst = `v2, v3 = smul_overflow.i64 v0, v1`, type = `Some(types::I64)`
   ```

## Implementation Plan

### Part 1: StructReturn Automatic Conversion (Priority 1 - CRITICAL)

#### Overview

When a function returns more than 2 register values on riscv32, automatically convert it to StructReturn:

- Caller allocates a buffer and passes a pointer as the first argument
- Callee writes return values to the buffer
- Caller reads return values from the buffer

#### Step 1.1: Update ABI to Support Automatic StructReturn Conversion

**File**: `cranelift/codegen/src/isa/riscv32/abi.rs`

**Location**: `compute_arg_locs` method

**Current behavior**: When `enable_multi_ret_implicit_sret()` is false and there are too many return values, it errors out.

**Required change**: Instead of erroring, automatically convert to StructReturn when:

- Function returns more than 2 register values (riscv32 can return 2 values: a0, a1)
- `enable_multi_ret_implicit_sret()` is false (or we can enable it, but proper StructReturn is better)

**Implementation approach**:

1. **Detect when StructReturn is needed**:

   ```rust
   // In compute_arg_locs, when processing returns:
   let mut register_count = 0;
   for param in params {
       let (rcs, reg_tys) = Inst::rc_for_type(param.value_type)?;
       register_count += rcs.len();
   }

   // If more than 2 registers needed for returns, use StructReturn
   let needs_struct_return = args_or_rets == ArgsOrRets::Rets
       && register_count > 2
       && !flags.enable_multi_ret_implicit_sret();
   ```

2. **Add StructReturn parameter**:
   ```rust
   let ret_area_ptr = if add_ret_area_ptr || needs_struct_return {
       assert!(ArgsOrRets::Args == args_or_rets || needs_struct_return);
       if needs_struct_return {
           // For returns, we need to add this to the args side
           // This will be handled by the caller
       }
       next_x_reg += 1;
       Some(ABIArg::reg(
           x_reg(x_start).to_real_reg().unwrap(),
           I32,
           ir::ArgumentExtension::None,
           ir::ArgumentPurpose::StructReturn, // Changed from Normal
       ))
   } else {
       None
   };
   ```

**Reference**: Look at how `add_ret_area_ptr` is currently handled - it's similar but uses `ArgumentPurpose::Normal`. For StructReturn, we need `ArgumentPurpose::StructReturn`.

#### Step 1.2: Update Signature Legalization

**File**: `cranelift/codegen/src/machinst/abi.rs`

**Location**: `from_func_sig` method

**Required change**: When a function signature has too many return values, automatically convert it to StructReturn:

```rust
pub fn from_func_sig<M: ABIMachineSpec>(
    &mut self,
    sig: &ir::Signature,
    flags: &settings::Flags,
) -> CodegenResult<SigData> {
    // ... existing code ...

    // Check if we need automatic StructReturn conversion
    let mut needs_sret = false;
    if sig.special_param_index(ArgumentPurpose::StructReturn).is_none() {
        // Count how many registers the returns need
        let mut ret_register_count = 0;
        for ret in &sig.returns {
            let (rcs, _) = M::I::rc_for_type(ret.value_type)?;
            ret_register_count += rcs.len();
        }

        // riscv32 can return 2 values in registers (a0, a1)
        if ret_register_count > 2 && !flags.enable_multi_ret_implicit_sret() {
            needs_sret = true;
        }
    }

    // If needed, convert signature to use StructReturn
    let mut legalized_sig = if needs_sret {
        let mut new_sig = sig.clone();
        // Add StructReturn parameter
        let pointer_type = M::word_type();
        new_sig.params.insert(0, AbiParam::special(
            pointer_type,
            ArgumentPurpose::StructReturn,
        ));
        // Clear returns - they'll be written to the StructReturn buffer
        new_sig.returns.clear();
        new_sig
    } else {
        sig.clone()
    };

    // ... rest of function using legalized_sig ...
}
```

**Reference**: See `ensure_struct_return_ptr_is_returned` function for similar logic.

#### Step 1.3: Update Call Site Code Generation

**File**: `cranelift/codegen/src/machinst/lower.rs`

**Location**: `gen_call` method and related call generation code

**Required change**: When calling a function that uses StructReturn:

1. **Allocate buffer** for return values
2. **Pass buffer pointer** as first argument
3. **Load return values** from buffer after call

**Implementation**:

```rust
// In gen_call or similar:
if sig.uses_special_param(ArgumentPurpose::StructReturn) {
    // Calculate return buffer size
    let mut buffer_size = 0;
    for ret in &sig.returns {
        buffer_size += ret.value_type.bytes();
        buffer_size = align_to(buffer_size, ret.value_type.bytes());
    }

    // Allocate stack space for return buffer
    let buffer_slot = self.f.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        buffer_size,
        // alignment
    ));

    // Get buffer address
    let buffer_ptr = self.ins().stack_addr(pointer_type, buffer_slot, 0);

    // Add buffer pointer as first argument
    args.insert(0, buffer_ptr);

    // After call, load return values from buffer
    // ... generate loads from buffer ...
}
```

#### Step 1.4: Update Return Code Generation

**File**: `cranelift/codegen/src/machinst/lower.rs`

**Location**: `gen_return` method

**Required change**: When a function uses StructReturn, write return values to the StructReturn buffer instead of returning them in registers.

**Implementation**:

```rust
pub fn gen_return(&mut self, rets: &[ValueRegs<Reg>]) {
    let sig = self.abi().signature();

    if sig.uses_special_param(ArgumentPurpose::StructReturn) {
        // Get StructReturn pointer
        let sret_ptr = self.f.special_param(ArgumentPurpose::StructReturn)
            .expect("StructReturn parameter must exist");
        let sret_ptr_reg = self.value_regs[sret_ptr].only_reg().unwrap();

        // Write each return value to buffer
        let mut offset = 0;
        for (i, ret_val_regs) in rets.iter().enumerate() {
            let ret_ty = sig.returns[i].value_type;
            // Generate store to buffer
            self.emit(M::gen_store_base_offset(
                sret_ptr_reg,
                offset,
                ret_val_regs.only_reg().unwrap(),
                ret_ty,
            ));
            offset += ret_ty.bytes();
            offset = align_to(offset, ret_ty.bytes());
        }

        // Return void (or just return)
        self.emit(self.abi().gen_rets(vec![]));
    } else {
        // ... existing return code ...
    }
}
```

**Reference**: See `gen_copy_regs_to_retval` in `cranelift/codegen/src/machinst/abi.rs` for how return values are written to StructReturn buffers.

#### Step 1.5: Test StructReturn Implementation

**Test files**:

- `filetests/filetests/runtests/smul_overflow.clif` - Functions return `i64, i8` (3 registers)
- `filetests/filetests/runtests/umul_overflow.clif` - Same issue

**Verification**:

```bash
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/smul_overflow.clif --target riscv32
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/umul_overflow.clif --target riscv32
```

**Light Player integration test**:

- Test that Light Player's StructReturn usage still works
- Verify vector/matrix returns from GLSL shaders work correctly

### Part 2: Small Type Overflow Operations (Priority 2)

#### Overview

Implement overflow operations (`uadd_overflow`, `sadd_overflow`, `usub_overflow`, `ssub_overflow`) for i8 and i16 types.

#### Step 2.1: Implement i8/i16 uadd_overflow

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Current state**: Only i32, i64, i128 are implemented.

**Implementation pattern** (similar to aarch64):

```isle
;;;; Rules for uadd_overflow (i8/i16) ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

;; For i8: zero-extend to i32, add, check if result fits in i8
(rule (lower (has_type $I8 (uadd_overflow a b)))
  (let ((a_ext XReg (rv_zext_w a))  ; Zero-extend i8 to i32
        (b_ext XReg (rv_zext_w b))   ; Zero-extend i8 to i32
        (sum XReg (rv_add a_ext b_ext))  ; Add at i32 width
        (sum_trunc XReg (rv_and sum (i64_iconst 0xFF)))  ; Truncate to i8
        (overflow XReg (rv_sltu sum sum_trunc)))  ; Check overflow
    (output_pair sum_trunc overflow)))

;; For i16: zero-extend to i32, add, check if result fits in i16
(rule (lower (has_type $I16 (uadd_overflow a b)))
  (let ((a_ext XReg (rv_zext_w a))  ; Zero-extend i16 to i32
        (b_ext XReg (rv_zext_w b))   ; Zero-extend i16 to i32
        (sum XReg (rv_add a_ext b_ext))  ; Add at i32 width
        (sum_trunc XReg (rv_and sum (i64_iconst 0xFFFF)))  ; Truncate to i16
        (overflow XReg (rv_sltu sum sum_trunc)))  ; Check overflow
    (output_pair sum_trunc overflow)))
```

**Note**: May need to check available ISLE helpers for zero-extension. If `rv_zext_w` doesn't exist, use `rv_and` with mask.

**Reference**: See `cranelift/codegen/src/isa/aarch64/lower.isle` lines 3037-3038 for similar pattern.

#### Step 2.2: Implement i8/i16 sadd_overflow

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Implementation**:

```isle
;;;; Rules for sadd_overflow (i8/i16) ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

;; For i8: sign-extend to i32, add, check signed overflow
(rule (lower (has_type $I8 (sadd_overflow a b)))
  (let ((a_ext XReg (rv_sext_b a))  ; Sign-extend i8 to i32
        (b_ext XReg (rv_sext_b b))   ; Sign-extend i8 to i32
        (sum XReg (rv_add a_ext b_ext))  ; Add at i32 width
        (sum_trunc XReg (rv_slli (rv_srai sum 24) 24))  ; Truncate to i8 (preserve sign)
        (overflow XReg (rv_xor (rv_slt sum sum_trunc) (rv_slt sum_trunc sum))))  ; Check overflow
    (output_pair sum_trunc overflow)))

;; For i16: sign-extend to i32, add, check signed overflow
(rule (lower (has_type $I16 (sadd_overflow a b)))
  (let ((a_ext XReg (rv_sext_h a))  ; Sign-extend i16 to i32
        (b_ext XReg (rv_sext_h b))   ; Sign-extend i16 to i32
        (sum XReg (rv_add a_ext b_ext))  ; Add at i32 width
        (sum_trunc XReg (rv_slli (rv_srai sum 16) 16))  ; Truncate to i16 (preserve sign)
        (overflow XReg (rv_xor (rv_slt sum sum_trunc) (rv_slt sum_trunc sum))))  ; Check overflow
    (output_pair sum_trunc overflow)))
```

**Note**: May need to adjust overflow detection logic. Check if riscv32 has better instructions for this.

#### Step 2.3: Implement usub_overflow and ssub_overflow

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Implementation**: Similar pattern to add, but using subtraction and checking for underflow.

#### Step 2.4: Test Small Type Overflow Operations

**Test file**: `filetests/filetests/runtests/uadd_overflow_narrow.clif`

**Verification**:

```bash
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/uadd_overflow_narrow.clif --target riscv32
```

### Part 3: Multiply Overflow Operations (Priority 2)

#### Overview

Implement `smul_overflow` and `umul_overflow` for all integer types (i8, i16, i32, i64).

#### Step 3.1: Implement umul_overflow

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Current state**: Not implemented at all.

**Implementation approach**:

For i32:

```isle
(rule (lower (has_type $I32 (umul_overflow a b)))
  (let ((result_lo XReg (rv_mul a b))  ; Low 32 bits
        (result_hi XReg (rv_mulhu a b)))  ; High 32 bits (unsigned)
        (overflow XReg (rv_snez result_hi)))  ; Overflow if high bits non-zero
    (output_pair result_lo overflow)))
```

For i64 (on riscv32, needs 2-register pattern):

```isle
(rule (lower (has_type $I64 (umul_overflow x y)))
  (let ((x_regs ValueRegs x)
        (y_regs ValueRegs y)
        (x_lo XReg (value_regs_get x_regs 0))
        (x_hi XReg (value_regs_get x_regs 1))
        (y_lo XReg (value_regs_get y_regs 0))
        (y_hi XReg (value_regs_get y_regs 1))
        ;; Compute: result = x * y (128-bit intermediate)
        ;; This is complex - need to implement full 64x64->128 multiply
        ;; For now, can use library call or multi-instruction sequence
        )
    ;; TODO: Implement full 64x64->128 multiply
    ;; Then check if high 64 bits are non-zero for overflow
    ))
```

**Note**: i64 multiply overflow on riscv32 is complex. May need to:

1. Use library call to `__muldi3` or similar
2. Or implement full 64x64->128 multiply using multiple instructions

For i8/i16: Similar to add overflow - extend to i32, multiply, check if result fits.

#### Step 3.2: Implement smul_overflow

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

**Implementation**: Similar to umul_overflow, but:

- Use signed multiply (`rv_mulh` instead of `rv_mulhu`)
- Check for signed overflow (result doesn't fit in signed range)

#### Step 3.3: Test Multiply Overflow Operations

**Test files**:

- `filetests/filetests/runtests/smul_overflow.clif`
- `filetests/filetests/runtests/umul_overflow.clif`

**Verification**:

```bash
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/smul_overflow.clif --target riscv32
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/umul_overflow.clif --target riscv32
```

## Implementation Order

1. **Part 1: StructReturn** (CRITICAL - do first)

   - Step 1.1: Update ABI
   - Step 1.2: Update signature legalization
   - Step 1.3: Update call site code generation
   - Step 1.4: Update return code generation
   - Step 1.5: Test

2. **Part 2: Small Type Overflow** (Can be done in parallel with Part 3)

   - Step 2.1: i8/i16 uadd_overflow
   - Step 2.2: i8/i16 sadd_overflow
   - Step 2.3: usub_overflow and ssub_overflow
   - Step 2.4: Test

3. **Part 3: Multiply Overflow** (Can be done in parallel with Part 2)
   - Step 3.1: umul_overflow
   - Step 3.2: smul_overflow
   - Step 3.3: Test

## Testing Strategy

### Unit Tests

Test each feature independently:

```bash
# StructReturn
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/smul_overflow.clif --target riscv32

# Small type overflow
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/uadd_overflow_narrow.clif --target riscv32

# Multiply overflow
cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/umul_overflow.clif --target riscv32
```

### Integration Tests

1. **Light Player StructReturn test**:

   - Verify that Light Player's existing StructReturn usage still works
   - Test vector/matrix returns from GLSL shaders

2. **Run full filetest suite**:
   ```bash
   cargo run --package cranelift-tools --bin clif-util -- test filetests/filetests/runtests/ --target riscv32
   ```

## Success Criteria

- ✅ All affected test files pass
- ✅ StructReturn automatic conversion works for functions returning >2 register values
- ✅ Small type overflow operations (i8/i16) work correctly
- ✅ Multiply overflow operations work correctly
- ✅ Light Player's StructReturn usage continues to work
- ✅ No regressions in existing functionality

## Known Limitations

1. **i64 multiply overflow on riscv32**: May require library calls or complex multi-instruction sequences
2. **f64 operations**: Separate issue, not addressed in this phase

## Next Phase

Once these features are implemented, proceed to Phase 6 to fix register allocator issues.
