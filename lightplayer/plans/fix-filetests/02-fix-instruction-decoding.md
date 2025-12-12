# Phase 2: Fix Instruction Decoding

## Goal
Add missing instruction decodings to the emulator so it recognizes all instructions that Cranelift generates.

## Prerequisites
- Phase 1 completed: You understand what instruction encodings Cranelift uses

## Affected Test Files

These tests fail with "Unknown I-type instruction" errors:

```bash
# Test the fixes:
cargo run --bin clif-util -- test filetests/filetests/runtests/arithmetic-extends.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/udiv.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/shifts.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/smulhi.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/extend.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/umulhi.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/rotl.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/fibonacci.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/popcnt.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/br_table.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/uadd_overflow_trap.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/icmp.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/rotr.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/bitrev.clif
```

## Implementation Steps

### Step 1: Update decode.rs

File: `lightplayer/crates/lp-riscv-tools/src/decode.rs`

Location: Around lines 170-260 where I-type instructions with funct3=0x5 are handled

**Current logic**:
- Checks funct6 first for SLLIUW and other funct6-encoded instructions
- Falls through to funct12 check for CLZ (0x600), CTZ (0x601), CPOP (0x602), etc.
- Doesn't handle funct12=0x20, 0x30, 0x38

**What to add**:
1. Based on Phase 1 findings, add cases for the missing funct12 values
2. If these are shift instructions with non-standard encoding, handle them appropriately
3. If these are Zbb instructions with different encodings, add them to the match statement

**Example pattern** (adjust based on Phase 1 findings):
```rust
match funct12 {
    0x600 => Ok(Inst::Clz { rd, rs1 }),
    0x601 => Ok(Inst::Ctz { rd, rs1 }),
    0x602 => Ok(Inst::Cpop { rd, rs1 }),
    // Add missing encodings here:
    0x020 => Ok(Inst::Clz { rd, rs1 }), // or whatever instruction this is
    0x030 => Ok(Inst::Ctz { rd, rs1 }), // or whatever instruction this is
    0x038 => Ok(Inst::Brev8 { rd, rs1 }), // or whatever instruction this is
    _ => Err(format!("Unknown I-type instruction: ...")),
}
```

### Step 2: Add instruction execution handlers (if needed)

File: `lightplayer/crates/lp-riscv-tools/src/emu/executor.rs`

If new instruction types were added to the `Inst` enum, add execution handlers:
- CLZ handler exists (lines 1337-1359)
- CTZ handler exists (lines 1361-1383)
- Other handlers exist for Zbb instructions

**Check**: If the decoded instructions map to existing `Inst` variants, no executor changes needed. If new variants were added, implement their execution logic.

## Testing

After making changes:

1. **Test a single file first**:
   ```bash
   cargo run --bin clif-util -- test filetests/filetests/runtests/arithmetic-extends.clif
   ```

2. **If it passes, test all affected files**:
   ```bash
   cargo run --bin clif-util -- test filetests/filetests/runtests/arithmetic-extends.clif filetests/filetests/runtests/udiv.clif filetests/filetests/runtests/shifts.clif
   ```

3. **Verify the error changes**: The "Unknown I-type instruction" error should be gone, though tests may still fail for other reasons (i64 handling, etc.)

## Common Issues

- **Wrong instruction mapping**: If tests still fail, verify the instruction bytes match what you decoded
- **Missing executor handler**: If decode succeeds but execution fails, check executor.rs
- **Encoding mismatch**: Double-check Cranelift's encoding vs emulator's decode

## Success Criteria

- All 14 tests no longer show "Unknown I-type instruction" errors
- Tests may still fail for other reasons (i64 handling, ISLE panics, etc.) - that's expected for later phases

## Next Phase

Once decoding is fixed, proceed to Phase 3 to fix i64 value handling bugs.

