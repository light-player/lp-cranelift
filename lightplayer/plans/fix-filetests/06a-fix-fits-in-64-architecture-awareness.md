# Phase 6a: Fix `fits_in_64` Architecture-Awareness Issue

## Goal

Fix the root cause of register allocator issues by making the `fits_in_64` extractor architecture-aware, preventing i64 operations on RV32 from being incorrectly lowered as single-register operations.

## Prerequisites

- Phase 6 completed: Register allocator validation added (catches the issue but doesn't fix root cause)
- Understanding of ISLE extractor system and context types

## Problem Statement

The `fits_in_64` extractor in `isle_prelude.rs` incorrectly matches i64 types on RV32, causing i64 operations to be lowered as single-register operations instead of register-pair operations. This leads to invalid register indices (2097151) being passed to regalloc2.

### Current Implementation

**File**: `cranelift/codegen/src/isle_prelude.rs`

```rust
fn fits_in_64(&mut self, ty: Type) -> Option<Type> {
    if ty.bits() <= 64 && !ty.is_dynamic_vector() {
        Some(ty)
    } else {
        None
    }
}
```

**Issue**: On RV32, i64 requires 2 registers (register pair) but `fits_in_64` returns `Some(I64)`, causing incorrect lowering.

### Root Cause Analysis

1. **Architecture mismatch**: `fits_in_64` assumes 64-bit registers but RV32 only has 32-bit registers
2. **Global scope**: Extractor is defined globally but register sizes vary by architecture
3. **ISLE limitations**: Cannot easily override global extractors in architecture-specific contexts
4. **Impact**: i64 operations (bxor, band, bor, etc.) incorrectly use single-register instructions instead of register-pair instructions

### Error Pattern

```
thread 'worker #X' panicked at regalloc2/src/ssa.rs:64:51:
index out of bounds: the len is 216 but the index is 2097151
```

The index `2097151` (0x1FFFFF) indicates an invalid register index being passed to regalloc2, caused by i64 operations being lowered incorrectly.

## Affected Test Files

These tests fail with regalloc2 panics due to invalid register indices:

```bash
# Test the fixes:
cargo run --bin clif-util --package cranelift-tools -- test cranelift/filetests/filetests/runtests/return-call-loop.clif
cargo run --bin clif-util --package cranelift-tools -- test cranelift/filetests/filetests/runtests/bitops.clif
cargo run --bin clif-util --package cranelift-tools -- test cranelift/filetests/filetests/runtests/spill-reload.clif
```

**Note**: The validation added in Phase 6 catches these errors early, but the root cause needs to be fixed.

## Solution Options

### Option 1: Architecture-Aware Global Extractor (Recommended) ✅

Modify `fits_in_64` to be context-aware by checking available register width.

**Pros:**

- Fixes root cause globally
- Maintains DRY principle
- Works for all architectures automatically

**Cons:**

- Requires adding register width context to ISLE contexts
- More complex implementation

### Option 2: Architecture-Specific Extractors (Partial Implementation)

Create RV32-specific extractors that override the global ones.

**Current partial implementation:**

- Added `fits_in_rv32_reg` extractor for RV32
- Changed bxor rules to use `fits_in_rv32_reg` instead of `fits_in_64`

**Pros:**

- Quick fix for specific operations
- No changes to global code

**Cons:**

- Only fixes operations we explicitly update (bxor, but not band, bor, etc.)
- Requires updating all rules that use `fits_in_64` on RV32
- Not maintainable long-term

### Option 3: Rule-Based Architecture Detection

Modify rules to explicitly check architecture and use different logic.

**Pros:**

- No changes to extractor system

**Cons:**

- Verbose and error-prone
- Requires architecture detection in ISLE rules
- Hard to maintain consistency

## Current Implementation Status

### ✅ **Phase 1-2: Partial Implementation (Completed)**

**What was implemented:**

1. **Added validation** in `compile.rs` to catch invalid register indices before regalloc
2. **LP-specific simplifications**:
   - `ty_reg_pair` now only matches I64 (not I128/F128) since LP doesn't need 128-bit support
   - Renamed `lower_b128_binary` to `lower_i64_binary` for clarity
   - Added RV32-specific `fits_in_rv32_reg` extractor for bxor operations
   - Modified bxor rules to use architecture-appropriate extractors

**What works:**

- Validation catches invalid register indices with clear error messages
- I64 operations on RV32 correctly use register-pair instructions
- Smaller types (I32, I16, I8) use single-register instructions

**What still needs work:**

- The `fits_in_64` extractor is not architecture-aware (affects all architectures)
- Need proper architecture-aware implementation

### 🔄 **Phase 2-3: Architecture-Aware Implementation (Pending)**

**Challenge:** The ISLE system makes it difficult to make global extractors architecture-aware without major refactoring.

**Proposed approaches:**

#### **Option A: ISLE Context Architecture Awareness (Complex)**

1. Add register width context to ISLE trait
2. Modify `fits_in_64` to use `self.register_width_bits()`
3. Requires changes to ISLE generated code and trait definitions

#### **Option B: Architecture-Specific Rule Overrides (Recommended for LP)**

1. Keep `fits_in_64` as-is (works for other architectures)
2. Override problematic rules in RV32 to use RV32-specific extractors
3. Apply this pattern to all operations affected by the i64 issue:
   - `bxor` ✅ (already done)
   - `band` (needs similar treatment)
   - `bor` (needs similar treatment)
   - `bnot` (needs similar treatment)
   - Other operations using `fits_in_64`

#### **Option C: Global Architecture Dispatch (Future)**

Modify Cranelift's architecture system to allow extractors to be architecture-aware, but this would be a major change affecting the entire codebase.

### **Recommended Next Steps for LP**

Since LP only targets RV32 and the current fixes work for the reported issues, recommend:

1. **Complete Option B** for remaining operations (band, bor, bnot)
2. **Add comprehensive tests** to ensure the fixes work
3. **Document the approach** for future maintainers
4. **Consider Option C** as a long-term improvement if LP expands to other architectures

### **Implementation Details**

#### **Current RV32-Specific Fixes**

- `ty_reg_pair`: Only matches I64
- `fits_in_rv32_reg`: New extractor for types ≤ 32 bits
- `lower_i64_binary`: Renamed from `lower_b128_binary`

#### **Rules Modified**

```isle
;; I64 bxor uses register pairs
(rule 0 (lower (has_type (ty_reg_pair _) (bxor x y)))
  (lower_i64_binary (AluOPRRR.Xor) x y))

;; Smaller types use single registers with RV32-specific extractor
(rule 1 (lower (has_type (fits_in_rv32_reg (ty_int ty)) (bxor x y)))
  (rv_xor x y))
```

#### **Testing Status**

- ✅ Validation catches invalid register indices
- ✅ I64 operations work correctly on RV32
- ⚠️ Need to verify other operations (band, bor, bnot) work similarly
- ⚠️ Need to ensure no regressions on other architectures

## Original Implementation Plan (For Reference)

**Goal**: Add infrastructure to query register width from ISLE contexts.

#### Step 1.1: Add `register_width_bits()` Method to Contexts

**File**: `cranelift/codegen/src/isle_prelude.rs`

Add method to the `Context` trait:

```rust
/// Get the register width in bits for the current architecture.
/// For optimization contexts, returns conservative 64 bits.
/// For lowering contexts, returns actual register width from ISA.
fn register_width_bits(&mut self) -> u32 {
    // Default implementation for optimization contexts
    64
}
```

#### Step 1.2: Implement for Lowering Contexts

**File**: `cranelift/codegen/src/machinst/isle.rs`

Override `register_width_bits()` in lowering contexts:

```rust
fn register_width_bits(&mut self) -> u32 {
    // Get pointer width from ISA flags
    self.lower_ctx.flags().pointer_width().bits() as u32
}
```

**Note**: May need to expose `flags()` method or pointer width directly.

#### Step 1.3: Test Context Method

Verify the method works correctly:

- Optimization contexts return 64
- RV32 lowering contexts return 32
- RV64 lowering contexts return 64
- Other architectures return appropriate values

### Phase 2: Make `fits_in_64` Architecture-Aware

**Goal**: Update `fits_in_64` to use actual register width instead of hardcoded 64.

#### Step 2.1: Modify `fits_in_64` Implementation

**File**: `cranelift/codegen/src/isle_prelude.rs`

```rust
fn fits_in_64(&mut self, ty: Type) -> Option<Type> {
    let register_width = self.register_width_bits();
    if ty.bits() <= register_width && !ty.is_dynamic_vector() {
        Some(ty)
    } else {
        None
    }
}
```

#### Step 2.2: Update Documentation

Add comments explaining architecture-aware behavior:

```rust
/// Matches types that fit in a single register for the current architecture.
/// On RV32, this means types <= 32 bits (I8, I16, I32).
/// On RV64, this means types <= 64 bits (I8, I16, I32, I64).
/// For optimization contexts, conservatively assumes 64-bit registers.
fn fits_in_64(&mut self, ty: Type) -> Option<Type> {
    // ...
}
```

#### Step 2.3: Test All Architectures

Run comprehensive tests to ensure no regressions:

```bash
# Test RV32
cargo run --bin clif-util --package cranelift-tools -- test cranelift/filetests/filetests/runtests/return-call-loop.clif --target riscv32

# Test RV64
cargo run --bin clif-util --package cranelift-tools -- test cranelift/filetests/filetests/runtests/return-call-loop.clif --target riscv64

# Test other architectures
cargo run --bin clif-util --package cranelift-tools -- test cranelift/filetests/filetests/runtests/bitops.clif --target x86_64
cargo run --bin clif-util --package cranelift-tools -- test cranelift/filetests/filetests/runtests/bitops.clif --target aarch64
```

### Phase 3: Clean Up RV32-Specific Workarounds

**Goal**: Remove temporary workarounds now that root cause is fixed.

#### Step 3.1: Remove `fits_in_rv32_reg` Extractor

**Files to update:**

- `cranelift/codegen/src/isa/riscv32/inst.isle` - Remove extractor declaration
- `cranelift/codegen/src/isa/riscv32/lower/isle.rs` - Remove implementation

#### Step 3.2: Revert bxor Rules to Use Standard `fits_in_64`

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

Change rules back to use `fits_in_64`:

```isle
(rule 0 (lower (has_type (ty_reg_pair _) (bxor x y)))
  (lower_i64_binary (AluOPRRR.Xor) x y))

(rule 1 (lower (has_type (fits_in_64 (ty_int ty)) (bxor x y)))
  (rv_xor x y))

;; Special cases for when one operand is an immediate that fits in 12 bits.
(rule 2 (lower (has_type (fits_in_64 (ty_int ty)) (bxor x (imm12_from_value y))))
  (rv_xori x y))

(rule 3 (lower (has_type (fits_in_64 (ty_int ty)) (bxor (imm12_from_value x) y)))
  (rv_xori y x))
```

#### Step 3.3: Verify All Operations Work

Test that all bitwise operations work correctly:

- `bxor` (XOR)
- `band` (AND)
- `bor` (OR)
- `bnot` (NOT)

## Implementation Details

### Accessing Pointer Width in Lowering Contexts

The lowering context needs access to ISA flags to determine pointer width. Current structure:

```rust
pub struct Lower<'func, I: VCodeInst> {
    pub(crate) f: &'func Function,
    vcode: VCodeBuilder<I>,
    // ... other fields
}
```

**Challenge**: `Lower` doesn't directly expose flags. Need to either:

1. Add `flags()` method to `Lower`
2. Pass pointer width through context
3. Store pointer width in context during initialization

**Recommended**: Add `flags()` method or expose pointer width directly.

### Testing Strategy

1. **Unit tests**: Test `register_width_bits()` for different context types
2. **Integration tests**: Test `fits_in_64` behavior on different architectures
3. **Regression tests**: Run full filetest suite to catch any issues
4. **Specific tests**: Focus on i64 operations on RV32

## Success Criteria

- ✅ i64 operations on RV32 use register-pair instructions (`lower_i64_binary`)
- ✅ i64 operations on RV64 use single-register instructions (`rv_xor`, etc.)
- ✅ No invalid register indices passed to regalloc2
- ✅ All existing tests pass on all architectures
- ✅ Code is maintainable and follows DRY principles
- ✅ No RV32-specific workarounds remain

## Risks & Mitigations

| Risk                                    | Impact | Mitigation                                                |
| --------------------------------------- | ------ | --------------------------------------------------------- |
| Breaking changes on other architectures | High   | Comprehensive testing on all architectures before merging |
| Performance impact                      | Low    | Method call overhead is minimal, can be inlined           |
| Complexity increase                     | Medium | Well-documented, follows existing patterns                |
| Context method access issues            | Medium | Careful design of context API, may need refactoring       |

## Timeline

- **Phase 1**: 2-3 days (context method implementation & testing)
- **Phase 2**: 1-2 days (`fits_in_64` modification & comprehensive testing)
- **Phase 3**: 1 day (cleanup & final verification)

**Total**: ~4-6 days

## Alternative: Quick Fix (Not Recommended)

If time is critical, we can continue with Option 2 (architecture-specific extractors) but update all affected operations:

1. Add `fits_in_rv32_reg` extractor
2. Update all rules using `fits_in_64` for bitwise operations:
   - `bxor` (XOR)
   - `band` (AND)
   - `bor` (OR)
   - `bnot` (NOT)
   - Any other operations that use `fits_in_64`

**Downside**: This is a band-aid solution that doesn't fix the root cause and requires maintenance for every new operation.

## References

- Phase 6 plan: `06-fix-regalloc-issues.md`
- ISLE extractor documentation: `cranelift/codegen/src/prelude.isle`
- Context implementation: `cranelift/codegen/src/isle_prelude.rs`
- RV32 lowering: `cranelift/codegen/src/isa/riscv32/lower.isle`

## Next Steps

1. Review this plan and get approval
2. Start Phase 1: Add `register_width_bits()` method
3. Test thoroughly before proceeding to Phase 2
4. Complete Phase 2 and Phase 3
5. Update Phase 6 plan to mark as complete
