---
name: Debug riscv32 stack corruption
overview: Continue debugging the riscv32 stack corruption issue by reviewing current changes, cleaning up experimental code, creating a minimal test case, and systematically comparing with riscv64 to identify word-size related bugs.
todos: []
---

# Debug riscv32 Stack Corruption Issue

## Current State

- Baseline commit exists: `lpc: DEBUG - riscv32 ABI baseline with debugging code` (b453ccf9f)
- Debugging code is in place (lines 499-525, 543-572, 765-795 in `cranelift/codegen/src/isa/riscv32/abi.rs`)
- Test file exists: `cranelift/filetests/filetests/isa/riscv32/abi-stack-args.clif`
- Word size functions are correct: `word_bits() = 32`, `word_bytes() = 4`

## Step 1: Review and Clean Current Changes

### 1.1 Verify Current Implementation

- Review `cranelift/codegen/src/isa/riscv32/abi.rs` for correctness:
- Lines 83-89: `word_bits() = 32`, `word_bytes() = 4` ✓ (correct)
- Line 177: `core::cmp::max(size, Self::word_bytes())` ✓ (uses correct 4 bytes)
- Line 352: `-(word_bytes * 2)` ✓ (correctly uses 8 bytes for FP+RA)
- Line 536: `RegClass::Int => I32` ✓ (correct type)
- Line 892: `clobbered_size += 4` ✓ (correct for riscv32)
- Check for any experimental changes that modify ABI behavior (not just logging)
- Verify debugging code is properly gated behind `#[cfg(feature = "std")]`

### 1.2 Remove Experimental Functionality (if any)

- Search for any temporary workarounds or experimental ABI modifications
- Keep only debugging/logging code (lines 499-525, 543-572, 765-795)
- Ensure no functional changes remain that aren't intended fixes

### 1.3 Commit Current State

- If changes are clean, commit with: `lpc: DEBUG - riscv32 ABI baseline verified`
- Document that debugging code is present and may need removal later

## Step 2: Compare riscv32 vs riscv64 ABI

### 2.1 Systematic Comparison

Compare critical sections between `cranelift/codegen/src/isa/riscv32/abi.rs` and `cranelift/codegen/src/isa/riscv64/abi.rs`:

**Stack argument alignment** (`compute_arg_locs`):

- riscv64 line 168: `std::cmp::max(size, 8)` (hardcoded 8 bytes)
- riscv32 line 177: `core::cmp::max(size, Self::word_bytes())` (uses 4 bytes) ✓

**Frame setup** (`gen_prologue_frame_setup`):

- riscv64 line 343: Hardcoded `-16` (2 * 8 bytes)
- riscv32 line 352: `-(word_bytes * 2)` (2 * 4 = 8 bytes) ✓

**Register save types** (`gen_clobber_save`):

- riscv64 line 496: `RegClass::Int => I64`
- riscv32 line 536: `RegClass::Int => I32` ✓

**Clobber size calculation** (`compute_clobber_size`):

- riscv64 line 763: `clobbered_size += 8` for Int
- riscv32 line 892: `clobbered_size += 4` for Int ✓

**Incoming args handling** (`gen_clobber_save` lines 464-478):

- riscv64: Uses `I64` type for loads/stores
- riscv32: Uses `I32` type ✓

### 2.2 Document Findings

- Create notes on all word-size dependent locations
- Verify all differences are intentional and correct for 32-bit vs 64-bit

## Step 3: Create Minimal Reproducible Test Case

### 3.1 Verify Existing Test File

- Test file exists: `cranelift/filetests/filetests/isa/riscv32/abi-stack-args.clif`
- Verify it compiles and reproduces the issue
- Run: `cargo test --package cranelift-filetests --test runone -- abi-stack-args`

### 3.2 Minimize Test Case

- Start with current 33-argument test (32 args + sret)
- Systematically reduce arguments to find minimum failing case
- Goal: smallest test that reproduces stack corruption
- Document the threshold (e.g., "fails with N+ args on stack")

### 3.3 Iterate Until Minimal

- Each iteration: reduce args, verify failure still occurs
- Test should fail with same error: invalid memory write at address 0x00160000
- Keep both full and minimal versions for reference

## Step 4: Additional Debugging Strategies (if needed)

### 4.1 Reduce Register Usage

- Modify `compute_arg_locs` in riscv32 ABI (lines 114-117)
- Change `(x_start, x_end, f_start, f_end)` from `(10, 17, 10, 17)` to smaller range (e.g., `(10, 12, 10, 12)`)
- This forces more args onto stack earlier, may reveal issue with fewer args
- File: `cranelift/codegen/src/isa/riscv32/abi.rs` lines 114-117

### 4.2 Test on riscv64

- Verify the same test case works correctly on riscv64
- May require riscv64 emulator setup
- If works on rv64, confirms word-size issue

### 4.3 Instrument riscv64 Code

- Add similar logging to riscv64 ABI code
- Compare stack layouts and offsets between rv32 and rv64
- Look for differences in:
- Stack argument offsets
- Frame pointer calculations  
- Register save/restore offsets

## Implementation Order

1. **First**: Review and verify current changes (Step 1)
2. **Then**: Systematic comparison with riscv64 (Step 2)
3. **Next**: Create and minimize test case (Step 3)
4. **If still failing**: Apply additional debugging strategies (Step 4)

## Files to Modify

- `cranelift/codegen/src/isa/riscv32/abi.rs` - Main ABI implementation (review/verify)
- `cranelift/filetests/filetests/isa/riscv32/abi-stack-args.clif` - Test file (minimize)
- Potentially: `cranelift/codegen/src/isa/riscv64/abi.rs` - For comparison/instrumentation

## Key Verification Points

- Word size functions return correct values (32/4 for riscv32)
- Stack argument alignment uses correct word size
- Frame setup uses correct word size (8 bytes = 2 * 4)
- Register types are correct (I32 for Int registers)
- Clobber size calculation uses correct sizes (4 bytes per Int reg)