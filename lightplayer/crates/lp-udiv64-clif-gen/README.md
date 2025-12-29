# u64 Division CLIF Generation

This crate generates Cranelift IR (CLIF) for 64-bit unsigned division by 32-bit divisor on 32-bit targets (specifically RISC-V 32-bit).

## Source

The division algorithm is adapted from **Rust's `compiler-builtins` crate**, specifically the `u64_div_rem` delegate algorithm for 32-bit targets. The algorithm performs binary long division to reduce the high 32 bits of the dividend to zero, then delegates to 32-bit hardware division for the remainder.

Reference: `rust-lang/compiler-builtins` crate, `src/udiv.rs` (delegate algorithm)

## Algorithm Overview

The algorithm handles the case where `dividend_hi < divisor` (ensuring the quotient fits in 32 bits):

1. **Normalization**: Calculate a shift amount to align the most significant bits of dividend and divisor
2. **Binary Long Division Loop**: 
   - Shift the divisor left by the normalization amount
   - Repeatedly subtract the shifted divisor from the dividend
   - Accumulate quotient bits
   - Right-shift divisor and continue until high part of dividend is zero
3. **Delegation**: Once the high 32 bits are zero, delegate to 32-bit hardware division

## Debugging Utility: `debug.sh`

The `debug.sh` script automates the process of building CLIF from Rust, cleaning it, and creating filetests:

```bash
# Test a specific file
./debug.sh 13-isub_icmp_no_loop.rs

# Use lib.rs as-is
./debug.sh
```

### Features

- **Automatic file copying**: Copies a test file over `lib.rs` for testing
- **Function signature detection**: Automatically extracts function name and parameter count from CLIF
- **Test expectation extraction**: Reads `// run:` comments from Rust source and converts them to CLIF test format
- **Named output files**: Creates test files named after the source file (e.g., `13-isub_icmp_no_loop.clif`)
- **Backup management**: Backs up original `lib.rs` before overwriting

### Usage

```bash
# Test a specific file
./debug.sh 05-isub_only.rs

# The script will:
# 1. Copy 05-isub_only.rs to lib.rs
# 2. Build CLIF IR
# 3. Clean the CLIF
# 4. Extract function signature and test expectations
# 5. Create cranelift/filetests/filetests/32bit/udiv/debug/05-isub_only.clif
# 6. Run the test
```

## The Bug: i64 Operations on RISC-V 32-bit

During debugging, we discovered a **critical bug in the RISC-V 32-bit lowering code** for i64 operations, specifically in how i64 subtraction results are used in comparisons.

### Bug Location

The bug appears to be in `cranelift/codegen/src/isa/riscv32/lower.isle`:

- **i64 subtraction** (line 678-689): The `isub` lowering for i64 uses `rv_sltu x_lo diff_lo` for borrow detection, which may be incorrect
- **i64 comparison** (line 2952-2953): The `icmp sle` lowering for i64 may not properly handle the high part when comparing with constants

### Symptoms

1. **Infinite loops**: The division algorithm gets stuck in an infinite loop
2. **Wrong comparison results**: `icmp sle 0, (isub result)` returns incorrect values
3. **Register corruption**: The high part of i64 values becomes corrupted (e.g., `-1` instead of `0`)

### Test Cases Created

We created multiple test files to isolate the bug:

- `00-failing_main_loop.rs`: The full division algorithm (fails with infinite loop)
- `05-isub_only.rs`: Just i64 subtraction (passes)
- `13-isub_icmp_no_loop.rs`: Subtraction + comparison without loop (fails - comparison wrong)
- `15-isub_then_update.rs`: Subtraction, update variable, then check (passes)

### Key Finding

**Test 13 isolated the bug**: The i64 subtraction itself works correctly, but when the result is used in a signed comparison (`icmp sle 0, result`), the comparison only checks the low 32 bits and ignores the high part.

### Execution Log Evidence

From the execution log:
```
[9966] 0x00000178: sub s5, a0, a3  ; s5: 32 -> -1 (rs1=-1, rs2=0)
```

This shows `a0` (high part) is `-1` when it should be `0`, suggesting the high part of the i64 subtraction result is wrong when used in subsequent operations.

### CLIF Pattern That Fails

```clif
block5(v31: i64, v32: i64, ...):
    v33 = isub v31, v32        ; i64 subtraction
    v34 = iconst.i64 0
    v35 = icmp sle v34, v33    ; Comparison - THIS IS WHERE IT FAILS
    brif v35, block6, block9
```

The comparison `icmp sle 0, v33` should check both high and low parts of the i64 value, but appears to only check the low part.

## Test Files

### Numbered Test Files

- `00-failing_main_loop.rs`: Full division algorithm (currently failing)
- `01-minimal.rs`: Minimal subtraction + comparison
- `02-minimal_divide.rs`: Minimal division loop
- `05-isub_only.rs`: Just i64 subtraction (isolates subtraction)
- `06-icmp_only.rs`: Just signed comparison
- `07-loop_one_iteration.rs`: Loop with one iteration
- `08-exact_failing_values.rs`: Uses exact values from failing test
- `09-isub_with_high_zero.rs`: Subtraction when high parts are zero
- `10-isub_then_icmp.rs`: Subtraction followed by comparison
- `11-isub_in_loop.rs`: Subtraction in a loop
- `12-minimal_loop.rs`: Minimal loop pattern
- `13-isub_icmp_no_loop.rs`: **KEY TEST** - Subtraction + comparison, no loop (reproduces bug)
- `14-loop_with_block_arg.rs`: Loop with i64 block arguments
- `15-isub_then_update.rs`: Subtraction, update, then check

### Test Expectations

Each test file includes `// run:` comments that define expected results. The `debug.sh` script automatically extracts these and converts them to CLIF test format.

## Current Status

- ✅ **Rust implementation**: Correct and tested (all unit tests pass)
- ✅ **CLIF generation**: Works correctly
- ❌ **RISC-V 32-bit lowering**: Has bugs in i64 operations
- ⚠️ **Workaround**: Using reciprocal multiplication for fixed32 division instead

## Next Steps (If Revisiting)

1. **Fix the i64 subtraction lowering**: Investigate `riscv32/lower.isle` line 686 - the borrow detection may be wrong
2. **Fix the i64 comparison lowering**: Investigate `riscv32/lower.isle` line 2952 - `icmp sle` may not handle high parts correctly
3. **Add more test cases**: Create tests that specifically target the failing patterns
4. **Check register allocation**: The bug may be in how i64 values are passed as block arguments

## Related Files

- `src/lib.rs`: Main division implementation (from Rust compiler-builtins)
- `src/working.rs`: More complete version with additional edge cases
- `src/00-failing_main_loop.rs`: The failing version we're debugging
- `debug.sh`: Main debugging utility script (builds, cleans, and tests)
- `build.sh`: Builds CLIF from Rust source
- `clean_clif.sh`: Cleans generated CLIF
- `format_clif.sh`: Formats CLIF output
- `cranelift/filetests/filetests/32bit/udiv/debug/`: Generated test files

## References

- Rust compiler-builtins: https://github.com/rust-lang/compiler-builtins
- Cranelift lowering rules: `cranelift/codegen/src/isa/riscv32/lower.isle`
- RISC-V 32-bit instruction definitions: `cranelift/codegen/src/isa/riscv32/inst.isle`
