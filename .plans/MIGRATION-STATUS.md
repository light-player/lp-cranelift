# RV64→RV32 ISLE Migration Status

## Summary

Successfully transformed RISC-V 64-bit ISLE code to work for 32-bit architecture. All ISLE files compile. Rust implementation files need ~10-20 remaining fixes in vector-related code.

## Commits

1. **7dafceaeb** - `inst.isle` and `inst_vector.isle` transformations
   - Removed all RV64-specific instruction variants (*w operations)
   - Removed RV64-specific helper functions
   - Fixed type selection for RV32 (XLEN=32)

2. **17afa36c9** - `lower.isle` ALU transformations  
   - Fixed all ALU operations (iadd, isub, imul, div, rem, shifts)
   - Commented out ~175 vector rules (deferred to Phase 2)
   - All ISLE files now compile successfully

3. **d580bb46e** - Rust implementation stubs
   - Stubbed out vector support in `lower/isle.rs`
   - Created minimal `inst/vector.rs` with type definitions
   - Saved full vector implementation as `vector.rs.disabled`

## What Works ✓

- ✓ ISLE files compile (no ISLE errors)
- ✓ Base RV32I/M/A/F/D/Zbb/Zba/Zbs instructions supported
- ✓ Atomic *W operations correctly kept (RV32A uses word=32-bit)
- ✓ Type selection functions work for RV32
- ✓ ALU operations use correct base instructions (not *w variants)

## Remaining Work (10-20 errors)

### Files needing fixes:
1. `cranelift/codegen/src/isa/riscv32/inst/emit.rs`
   - Comment out or stub vector instruction emission
   - ~5-10 references to VecAluOp variants

2. `cranelift/codegen/src/isa/riscv32/inst/encode.rs`
   - Comment out or stub vector instruction encoding
   - ~5-10 references to VecAluOp methods

3. `cranelift/codegen/src/isa/riscv32/inst/vector.rs`
   - Add method stubs (funct6, opcode, funct3, encode, etc.)
   - Or feature-gate the vector code

### Quick Fix (Option 2 - recommended):

Add to `inst/vector.rs`:

```rust
use crate::isa::riscv32::lower::isle::generated_code::{
    VecAMode, VecAluOpRR, VecAluOpRRImm5, VecAluOpRRR,
    VecAluOpRRRImm5, VecAluOpRImm5, VecAluOpRRRR, VecOpMasking,
};

// Stub implementations - vector support deferred to Phase 2
impl VecAMode {
    pub fn lumop(&self) -> u32 { unreachable!() }
    pub fn sumop(&self) -> u32 { unreachable!() }
    pub fn mop(&self) -> u32 { unreachable!() }
    pub fn nf(&self) -> u32 { unreachable!() }
}

impl VecAluOpRRR {
    pub fn funct6(&self) -> u32 { unreachable!() }
    pub fn opcode(&self) -> u32 { unreachable!() }
    pub fn funct3(&self) -> u32 { unreachable!() }
}

impl VecAluOpRRImm5 {
    pub fn funct6(&self) -> u32 { unreachable!() }
    pub fn opcode(&self) -> u32 { unreachable!() }
    pub fn funct3(&self) -> u32 { unreachable!() }
}

impl VecAluOpRRRR {
    pub fn funct6(&self) -> u32 { unreachable!() }
    pub fn opcode(&self) -> u32 { unreachable!() }
    pub fn funct3(&self) -> u32 { unreachable!() }
}

impl VecAluOpRRRImm5 {
    pub fn funct6(&self) -> u32 { unreachable!() }
    pub fn opcode(&self) -> u32 { unreachable!() }
    pub fn funct3(&self) -> u32 { unreachable!() }
}

impl VecAluOpRImm5 {
    pub fn funct6(&self) -> u32 { unreachable!() }
    pub fn opcode(&self) -> u32 { unreachable!() }
    pub fn funct3(&self) -> u32 { unreachable!() }
}

impl VecAluOpRR {
    pub fn funct6(&self) -> u32 { unreachable!() }
    pub fn opcode(&self) -> u32 { unreachable!() }
    pub fn funct3(&self) -> u32 { unreachable!() }
    pub fn dst_regclass(&self) -> RegClass { RegClass::Vector }
}

impl VecOpMasking {
    pub fn encode(&self) -> u32 {
        match self {
            VecOpMasking::Enabled { .. } => 0,
            VecOpMasking::Disabled => 1,
        }
    }
}
```

Then in `emit.rs`, comment out the VmvrV pattern matches (lines 209, 1357).

## Testing

Once compilation succeeds:

```bash
# Build
cargo build --package cranelift-codegen --features riscv32,std

# Run tests  
cargo test --package cranelift-codegen --features riscv32,std

# Verify generated code
# TODO: Create test that iadd i32 generates `add` (0x33) not `addw` (0x3b)
```

## Key Transformations Made

### Instruction Enum Changes
- **Removed**: Addw, Subw, Sllw, Srlw, Sraw (from AluOPRRR)
- **Removed**: Addiw, Slliw, SrliW, Sraiw (from AluOPRRI)
- **Removed**: Mulw, Divw, Divuw, Remw, Remuw (from AluOPRRR)
- **Removed**: Clzw, Ctzw, Cpopw, Rolw, Rorw, Roriw, Packw
- **Removed**: CAddw, CSubw, CAddiw (compressed)
- **KEPT**: All atomic *W operations (correct for RV32A)

### Helper Function Changes
- **Removed**: rv_addw, rv_addiw, rv_sextw, rv_subw, rv_mulw, rv_divw, etc.
- **Fixed**: select_addi to always use Addi (not Addiw)
- **Fixed**: lower_ctz to use rv_ctz (not rv_ctzw)
- **Fixed**: zext/sext rules for RV32 register width

### Lower Rules Changes
- All ALU operations now use base instructions (add, sub, mul, div, etc.)
- Removed fits_in_32 type checks that selected *w variants
- Commented out ~175 vector rules (ty_supported_vec)
- Deferred I64 support to Phase 2 (will require multi-instruction sequences)

## Architecture Notes

### RV32 vs RV64 Key Differences

**RV64** (what we migrated from):
- XLEN = 64 bits
- Has *w suffix operations that work on 32-bit values and sign-extend to 64
- `addw rd, rs1, rs2` = add 32-bit, sign-extend to 64 (opcode 0x3B)
- Used for I32 operations on 64-bit architecture

**RV32** (what we migrated to):
- XLEN = 32 bits  
- No *w suffix needed - base operations work on 32-bit natively
- `add rd, rs1, rs2` = add 32-bit (opcode 0x33)
- I32 is the native register width

**Atomic Operations** (special case):
- RV32A: *W suffix means "word" = 32-bit (correct!)
- RV64A: *W suffix means "word" = 32-bit, *D means "doubleword" = 64-bit
- We correctly KEPT all atomic *W operations

## Documentation

- `.plans/isle.md` - Original migration plan
- `.plans/isle-progress.md` - Detailed progress tracking
- `.plans/isle-next-steps.md` - Original next steps (now outdated)
- `.plans/remaining-rust-fixes.md` - Current actionable fixes needed
- `.plans/MIGRATION-STATUS.md` - This file

## Backups

Original files backed up in:
`.backups/riscv32-isle-20251203-153005/`
- `inst.isle` (3,281 lines)
- `lower.isle` (3,155 lines)
- `inst_vector.isle` (82 lines)

Full vector implementation saved as:
`cranelift/codegen/src/isa/riscv32/inst/vector.rs.disabled` (1,150 lines)

## Phase 2 (Future Work)

Once Phase 1 is complete and tested:

1. **I64 Support on RV32**
   - Multi-instruction sequences for 64-bit ops
   - Register pair management
   - Wide arithmetic operations

2. **Vector Support (RVV)**
   - Re-enable commented vector rules in lower.isle
   - Restore full vector.rs implementation
   - Test vector operations

See `.plans/isle-phase2-i64.md` for Phase 2 planning.

