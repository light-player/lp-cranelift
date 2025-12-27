# Fix Linker Block Parameter Preservation

## Problem Statement

The linker (`rebuild_function_for_module` in `compiler/link.rs`) is not preserving block parameters when rebuilding functions for a new module. Specifically:

Run the test with: cargo test --color=always --test transform_exact_match test_block_params_preserved --features emulator --profile test --no-fail-fast

1. **Symptom**: After linking, blocks that should have parameters (e.g., `block2(v12: i32, v21: i32)`) end up with no parameters (`block2:`)
2. **Impact**: The `test_block_params_preserved` test fails, and jumps that should pass arguments (e.g., `jump block2(v6, v4)`) end up with no arguments (`jump block2`)
3. **Root Cause**: Unknown - parameters appear to be added by `ensure_block_params` but don't persist in the final linked function

## Test Case

Created `tests/link_block_params.rs` to isolate the issue:

- Parses a function with block parameters: `block1(v2: i32, v3: i32)` and `block2(v12: i32, v21: i32)`
- Links it via `ClifModule::build_object_module()`
- Verifies block parameters are preserved

**Current Behavior**:

- Parsed function: `block2` has 0 parameters (Cranelift infers from jumps, not declarations)
- Jumps to block2: 2 jumps with 2 arguments each (`jump block2(v6, v4)` and `jump block2(v13, v22)`)
- Linked function: `block2` has 0 parameters, jumps have no arguments

## What We've Tried

### 1. Initial Implementation (Failed)

- **Approach**: Clone function and remap FuncRefs in-place
- **Result**: Block parameters not preserved at all
- **Why**: Cloning doesn't properly handle block parameters in new module context

### 2. FunctionBuilder Approach (Current - Partially Working)

- **Approach**: Use FunctionBuilder to rebuild function block-by-block, matching fixed32's approach
- **Implementation**:
  - Use `create_blocks` to create blocks
  - Use `map_entry_block_params` to verify entry block params
  - Copy instructions block-by-block
  - Call `ensure_block_params` when processing jump/brif instructions
  - Seal all blocks and finalize
- **Result**:
  - `ensure_block_params` is called and reports adding parameters
  - Debug output shows: `ensure_block_params: old_block=block2 has 2 params, new_block=block2 has 0 params, adding 2`
  - But final linked function shows `block2` with 0 parameters
  - Jump arguments are also lost

### 3. Pre-pass Attempt (Reverted)

- **Approach**: Added a pre-pass to ensure all block params before processing instructions
- **Result**: Same issue - parameters added but don't persist
- **Why Reverted**: Fixed32 doesn't use a pre-pass, so we removed it to match exactly

### 4. Debug Output Added

- Added debug output to `ensure_block_params` to verify it's being called
- Added checks to verify jump arguments are in value_map
- **Findings**:
  - `ensure_block_params` IS being called for block2
  - Parameters ARE being added (no warnings about count mismatch)
  - Jump arguments ARE in value_map
  - But parameters don't appear in final function

## Current Code Structure

The linker now matches fixed32's structure:

1. Create blocks with `create_blocks`
2. Map entry block params with `map_entry_block_params`
3. Copy instructions block-by-block:
   - For Jump: call `ensure_block_params`, map arguments, emit jump
   - For Brif: call `ensure_block_params` for both targets, map arguments, emit brif
   - For Call: remap FuncRef, map arguments, emit call, map return values
   - For other instructions: use `copy_instruction`
4. Seal all blocks
5. Finalize builder
6. Copy value aliases

## Key Observations

1. **Parameters are added but don't persist**: `ensure_block_params` reports adding parameters, but they're gone in the final function
2. **Jump arguments are lost**: Even though arguments are in value_map and mapped correctly, the linked CLIF shows jumps with no arguments
3. **Code structure matches fixed32**: The implementation follows fixed32's approach exactly, but produces different results
4. **Parsed function has 0 params for block2**: Cranelift infers block parameters from jumps, not declarations, so the parsed function shows block2 with 0 params initially

## Hypothesis

The most likely causes:

1. **Parameters removed during sealing/finalization**: Maybe `seal_all_blocks()` or `finalize()` removes parameters that were added after instructions were processed?
2. **Jump arguments dropped**: Maybe `builder.ins().jump()` silently drops arguments if the target block doesn't have parameters at emit time?
3. **Block state issue**: Maybe blocks need to be in a specific state (pristine, sealed, etc.) for parameters to persist?
4. **Order of operations**: Maybe we need to ensure parameters BEFORE processing any instructions that reference the block, not just before emitting jumps to it?

## Next Steps to Debug

1. **Verify parameters persist after `ensure_block_params`**:

   - Add debug output immediately after `ensure_block_params` returns to check parameter count
   - Check parameter count after sealing
   - Check parameter count after finalizing
   - This will tell us WHEN parameters are being lost

2. **Check jump argument handling**:

   - Verify `old_args` contains expected values when processing jumps from block4/block5
   - Verify `new_args` is built correctly (check each mapped value)
   - Add debug output to see what's passed to `builder.ins().jump()`
   - Check if FunctionBuilder validates arguments match block params

3. **Compare execution with fixed32**:

   - Run fixed32 transform on the same function
   - Check if block params are preserved
   - Compare the exact sequence of operations
   - Look for any differences in how blocks/instructions are processed

4. **Check Cranelift FunctionBuilder requirements**:

   - Review documentation on `append_block_param` requirements
   - Check if blocks must be pristine (no instructions) when adding params
   - Verify if parameters can be added after instructions are in the block
   - Check if sealing removes parameters added after instructions

5. **Test with simpler case**:
   - Create a minimal test with just one block that has parameters
   - See if the issue reproduces with a simpler case
   - This will help isolate the problem

## Files Involved

- `lightplayer/crates/lp-glsl/src/compiler/link.rs` - Main linker implementation
- `lightplayer/crates/lp-glsl/src/util/clif_copy.rs` - `ensure_block_params` utility
- `lightplayer/crates/lp-glsl/src/util/instruction_copy.rs` - `copy_instruction` utility
- `lightplayer/crates/lp-glsl/tests/link_block_params.rs` - Targeted test
- `lightplayer/crates/lp-glsl/src/transform/fixed32/function.rs` - Working reference implementation
- `lightplayer/crates/lp-glsl/src/transform/fixed32/converters/control.rs` - Working jump/brif handling

## Success Criteria

- `test_block_params_preserved` passes
- `test_link_preserves_block_params` passes
- Block parameters are preserved in linked functions
- Jump arguments are preserved in linked functions

