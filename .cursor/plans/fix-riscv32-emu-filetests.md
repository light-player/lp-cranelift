# Fix Riscv32 Emulator Filetests

## Problem

The `test emu` command for riscv32 is failing because:

1. Symbol addresses are all 0x0 due to using `per_function_section(true)`
2. Functions can't be found at correct addresses in the loaded binary
3. Missing debugging output (VCode, disassembly, emulator state) when tests fail

## Root Cause

After comparing with `lp-glsl-filetests`:

- **lp-glsl does NOT use `per_function_section(true)`** - all functions go into one `.text` section
- **We're using `per_function_section(true)`** which creates separate sections for each function
- When functions are in separate sections, symbol addresses are section-relative (all 0x0), not absolute offsets
- The `load_elf()` function loads sections sequentially, but symbol lookup needs proper relative addresses

## Solution

### 1. Remove `per_function_section(true)` from ObjectModule

**File**: `cranelift/filetests/src/object_runner.rs`

- Remove `builder.per_function_section(true)`
- Use default behavior (all functions in one `.text` section)
- This ensures symbols have proper relative addresses within the text section

### 2. Fix symbol address lookup

**File**: `cranelift/filetests/src/test_emu.rs`

- After `load_elf()`, the code buffer contains all functions laid out sequentially starting at address 0
- Use `find_symbol_address()` which returns offsets relative to text section base
- These offsets are correct for the loaded code buffer
- Remove the manual offset calculation that was trying to work around the per-function-section issue

### 3. Add comprehensive debugging output

**File**: `cranelift/filetests/src/test_emu.rs`

When tests fail, output (following lp-glsl-filetests pattern):

- CLIF IR for the function being tested
- VCode (capture during compilation if possible)
- Disassembly (capture during compilation if possible)
- Emulator state: registers, PC, instruction count, recent instruction log (last 20 instructions)
- Function addresses found in ELF

### 4. Generate VCode and disassembly during compilation

**File**: `cranelift/filetests/src/object_runner.rs` or `test_emu.rs`

- During compilation, capture VCode and disassembly for each function
- Store in `CompiledObjectTestFile` or `EmulatorExecutor`
- Output in debug info when tests fail

## Implementation Steps

1. **Fix ObjectModule configuration**

   - Remove `per_function_section(true)` from `object_runner.rs`
   - Verify functions are in one text section

2. **Fix symbol lookup**

   - Use `find_symbol_address()` correctly after `load_elf()`
   - Remove manual offset calculation
   - Verify symbols have correct addresses

3. **Add VCode/disassembly capture**

   - Modify compilation to capture VCode and disassembly
   - Store in `CompiledObjectTestFile` or pass to `EmulatorExecutor`

4. **Enhance debug output**

   - Format emulator state nicely
   - Include VCode and disassembly in error messages
   - Match lp-glsl-filetests output format

5. **Test and verify**
   - Run `run-4-args.clif` test
   - Verify it passes
   - Verify debug output is helpful when tests fail

## Files to Modify

1. `cranelift/filetests/src/object_runner.rs`

   - Remove `per_function_section(true)`

2. `cranelift/filetests/src/test_emu.rs`
   - Fix symbol address lookup
   - Remove manual offset calculation
   - Add VCode/disassembly capture
   - Enhance debug output formatting

## Testing

- `run-4-args.clif` should pass
- Other multi-function riscv32 tests should work
- Debug output should be comprehensive when tests fail
