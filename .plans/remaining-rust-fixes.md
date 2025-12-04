# Remaining Rust Compilation Fixes

## Status

✅ ISLE files compile successfully  
✅ lower/isle.rs vector methods commented out  
✅ inst/vector.rs replaced with minimal stubs  
✅ Vector-related errors in emit.rs and encode.rs fixed  
✅ **cranelift-codegen builds successfully!**

⚠️  Note: Tests have RV64-specific instruction failures (separate issue from vector support)

## Remaining Errors (from cargo build)

### 1. VecAluOpRRImm5::VmvrV references
- `inst/emit.rs:209` - pattern match
- `inst/emit.rs:1357` - register copy
- **Fix**: Comment out or gate behind vector feature

### 2. VecAMode methods (lumop, sumop, mop, nf)
- `inst/emit.rs:2706` - lumop()
- `inst/emit.rs:2753` - sumop()
- `inst/encode.rs` - various  
- **Fix**: Gate vector load/store emission behind feature or comment out

### 3. VecAluOp methods (funct6, opcode, funct3, encode)
- Multiple files referencing vector operation encoding
- **Fix**: These are in the disabled vector.rs.disabled file - may need minimal stubs

## Recommended Approach

### Option 1: Comment Out Vector Instruction Emission (Fastest)

```bash
# In emit.rs, comment out vector instruction emission blocks
sed -i '' '/Inst::VecAlu/,/^            }/s/^/            \/\/ /' emit.rs

# In encode.rs, comment out vector encoding
sed -i '' '/VecAluOp/,/}/s/^/\/\/ /' encode.rs
```

### Option 2: Add Minimal Vector Method Stubs (Cleaner)

Add to `inst/vector.rs`:

```rust
use crate::isa::riscv32::lower::isle::generated_code::{
    VecAMode, VecAluOpRR, VecAluOpRRImm5, VecAluOpRRR,  
    VecAluOpRRRImm5, VecAluOpRImm5, VecAluOpRRRR,
    VecOpMasking,
};

impl VecAMode {
    pub fn lumop(&self) -> u32 { unreachable!("Vector support deferred") }
    pub fn sumop(&self) -> u32 { unreachable!("Vector support deferred") }
    pub fn mop(&self) -> u32 { unreachable!("Vector support deferred") }
    pub fn nf(&self) -> u32 { unreachable!("Vector support deferred") }
}

impl VecAluOpRRR {
    pub fn funct6(&self) -> u32 { unreachable!("Vector support deferred") }
    pub fn opcode(&self) -> u32 { unreachable!("Vector support deferred") }
    pub fn funct3(&self) -> u32 { unreachable!("Vector support deferred") }
}

// Similar for other VecAluOp* enums
impl VecOpMasking {
    pub fn encode(&self) -> u32 { unreachable!("Vector support deferred") }
}
```

### Option 3: Feature-Gate Vector Code (Most Proper)

1. Add `vector` feature to `Cargo.toml`
2. Gate all vector code behind `#[cfg(feature = "vector")]`
3. Provide stub implementations when feature is disabled

## Files Requiring Updates

1. `cranelift/codegen/src/isa/riscv32/inst/emit.rs` (~10 changes)
2. `cranelift/codegen/src/isa/riscv32/inst/encode.rs` (~5-10 changes)
3. `cranelift/codegen/src/isa/riscv32/inst/vector.rs` (add method stubs)
4. `cranelift/codegen/src/isa/riscv32/lower/isle.rs` (already done ✓)

## Estimated Effort

- Option 1 (comment out): 15-30 minutes
- Option 2 (minimal stubs): 30-60 minutes  
- Option 3 (feature gates): 1-2 hours

## Testing After Fixes

```bash
# ✅ Build succeeds!
cargo build --package cranelift-codegen --features riscv32,std

# ⚠️ Tests fail due to RV64 instruction references (separate issue)
# cargo test --package cranelift-codegen --features riscv32,std -- riscv32

# Check generated code
# TODO: Write small test to verify iadd i32 generates correct opcode
```

## Completed Work

### Phase 1: Vector Support Stubs (✅ Done)

1. ✅ Added `VecOpMasking` as primitive type with Copy/Clone
2. ✅ Added method stubs for all `VecAluOp*` encoding methods
3. ✅ Added method stubs for `VecAMode` addressing modes
4. ✅ Added `Display` implementations for vector types
5. ✅ Commented out vector pseudo-instruction formatting
6. ✅ Disabled `ty_vec_fits_in_register` (returns None)
7. ✅ Replaced `VmvrV` register moves with `unreachable!()`
8. ✅ Fixed `forbids_overlaps` signature to accept `VecOpMasking`
9. ✅ Exported `VecOpMasking` from `lower/isle.rs`

### Commits

- `lpc: Comment out vector methods in lower/isle.rs`
- `lpc: Add vector instruction stubs to fix remaining build errors`

## Current File States

- ✓ `inst.isle` - Transformed, compiles
- ✓ `lower.isle` - Transformed, compiles
- ✓ `inst_vector.isle` - Minimal stubs  
- ✓ `lower/isle.rs` - Vector methods commented out
- ✓ `inst/vector.rs` - Minimal type stubs
- ❌ `inst/emit.rs` - Needs vector emission removed/stubbed
- ❌ `inst/encode.rs` - Needs vector encoding removed/stubbed

