# ISLE Migration - Next Steps

## Status: ISLE files compile successfully ✓

Commits:
- 7dafceaeb: inst.isle/inst_vector.isle transformations  
- [latest]: lower.isle ALU transformations

## Current Issue: Rust Compilation Errors (529 errors)

The ISLE files now compile, but the generated Rust code has compilation errors in:
- `cranelift/codegen/src/isa/riscv32/inst/emit.rs` 
- `cranelift/codegen/src/isa/riscv32/inst/encode.rs`
- Other inst/*.rs files

These errors are because we removed RV64 instruction enum variants, but the Rust code still references them.

## Next Actions

1. **Fix instruction encoding/emission** (est. 100-200 lines of changes):
   - Remove RV64 *w instruction encoding from `inst/encode.rs`
   - Remove RV64 *w instruction emission from `inst/emit.rs`  
   - Update pattern matches to remove *w variants
   - Keep atomic *W variants (correct for RV32A)

2. **Fix instruction printing/display**:
   - Update `inst/mod.rs` display/debug implementations
   - Remove *w variants from pretty-printing

3. **Run tests**:
   ```bash
   cargo build --package cranelift-codegen --features riscv32,std
   cargo test --package cranelift-codegen --features riscv32,std
   ```

4. **Verify generated code**:
   - Check that iadd i32 generates `add` (opcode 0x33) not `addw` (0x3b)
   - Encoding for `add a0, a0, a1` should be `0x00b50533`

## Files to Update

- `cranelift/codegen/src/isa/riscv32/inst/encode.rs` - Remove *w encodings
- `cranelift/codegen/src/isa/riscv32/inst/emit.rs` - Remove *w emissions  
- `cranelift/codegen/src/isa/riscv32/inst/mod.rs` - Update display/debug
- `cranelift/codegen/src/isa/riscv32/inst/args.rs` - May need AluOPRRR/AluOPRRI updates

## Reference

RV32I base operations use opcode 0x33 (OP):
- `add rd, rs1, rs2` = opcode 0x33, funct3=0x0, funct7=0x00
- `sub rd, rs1, rs2` = opcode 0x33, funct3=0x0, funct7=0x20  
- `sll rd, rs1, rs2` = opcode 0x33, funct3=0x1, funct7=0x00
- etc.

RV64I *w operations use opcode 0x3B (OP-32) - NOT USED ON RV32

