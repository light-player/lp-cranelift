# Stage 02: Detect Internal vs External Function Calls

## Overview

Enhance the fixed-point transform to properly detect and handle internal function calls (user functions and intrinsics) vs external function calls. This is a prerequisite for converting intrinsics.

## Problem

The current `convert_call()` function assumes all function calls are external functions. It tries to access `old_func.dfg.ext_funcs[func_ref]`, which fails for internal function calls (user functions and intrinsics that are part of the module).

## Goals

- Detect whether a `Call` instruction targets an internal or external function
- Handle internal function calls correctly (convert signatures, map arguments)
- Track internal function conversions to avoid duplicate work
- Prepare infrastructure for converting called functions

## Implementation

### 1. Detect Internal vs External Calls

**File**: `crates/lp-glsl/src/transform/fixed32/converters/calls.rs`

Modify `convert_call()` to check if `FuncRef` points to an external or internal function:

```rust
pub(crate) fn convert_call(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
    // NEW: Track internal function references
    internal_func_refs: &HashSet<FuncRef>, // FuncRefs that are internal functions
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::Call { func_ref, args } = inst_data {
        // Check if this is an external or internal function call
        let is_external = old_func.dfg.ext_funcs.contains_key(*func_ref);

        if is_external {
            // External function - existing logic
            let new_func_ref = map_external_function(
                old_func, *func_ref, builder, ext_func_map, sig_map, format
            )?;

            // Map call arguments
            let old_args = args.as_slice(&old_func.dfg.value_lists);
            let new_args: Vec<Value> = old_args.iter()
                .map(|&v| map_value(value_map, v))
                .collect();

            // Emit call
            let call_inst = builder.ins().call(new_func_ref, &new_args);

            // Map return values
            let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
            let new_results: Vec<Value> = builder.inst_results(call_inst).to_vec();

            // Verify and map results
            if old_results.len() != new_results.len() {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Call return value count mismatch: old={}, new={}",
                        old_results.len(), new_results.len()
                    ),
                ));
            }

            for (old_result, new_result) in old_results.iter().zip(new_results.iter()) {
                value_map.insert(*old_result, *new_result);
            }
        } else {
            // Internal function call (user function or intrinsic)
            // Get the signature from the function reference
            // For internal calls, we need to get the signature differently

            // The signature is stored in the function's signature, not as a separate SigRef
            // We need to track which internal functions have been converted

            // For now, convert the call signature and arguments
            // The actual function conversion happens at module level (Stage 01)

            // Get signature from the called function (if we have access to it)
            // Otherwise, we'll need to track signatures separately

            // Map arguments
            let old_args = args.as_slice(&old_func.dfg.value_lists);
            let new_args: Vec<Value> = old_args.iter()
                .map(|&v| map_value(value_map, v))
                .collect();

            // For internal calls, the FuncRef should remain the same
            // because the function will be converted at module level
            // But we need to ensure the signature matches

            // This is a placeholder - actual implementation depends on Stage 01
            // For now, emit call with same FuncRef but converted args
            let call_inst = builder.ins().call(*func_ref, &new_args);

            // Map return values
            let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
            let new_results: Vec<Value> = builder.inst_results(call_inst).to_vec();

            if old_results.len() != new_results.len() {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Internal call return value count mismatch: old={}, new={}",
                        old_results.len(), new_results.len()
                    ),
                ));
            }

            for (old_result, new_result) in old_results.iter().zip(new_results.iter()) {
                value_map.insert(*old_result, *new_result);
            }
        }
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Call instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}
```

### 2. Track Internal Function References

**File**: `crates/lp-glsl/src/transform/fixed32/rewrite.rs`

Add tracking of internal function references during rewrite:

```rust
pub struct RewriteContext<'a> {
    // ... existing fields ...

    // NEW: Track internal function references
    pub internal_func_refs: HashSet<FuncRef>,
}

pub fn rewrite_function(
    old_func: &Function,
    format: FixedPointFormat,
) -> Result<Function, GlslError> {
    // ... existing setup ...

    // Collect internal function references from the function
    let internal_func_refs = collect_internal_func_refs(old_func);

    let mut ctx = RewriteContext {
        // ... existing fields ...
        internal_func_refs,
        // ...
    };

    // ... rest of rewrite ...
}
```

### 3. Helper to Collect Internal Function References

**File**: `crates/lp-glsl/src/transform/fixed32/rewrite.rs`

```rust
/// Collect all internal function references from a function.
///
/// Internal functions are those that are NOT in ext_funcs.
fn collect_internal_func_refs(func: &Function) -> HashSet<FuncRef> {
    let mut internal_refs = HashSet::new();

    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            if let InstructionData::Call { func_ref, .. } = &func.dfg.insts[inst] {
                // Check if this is an internal function (not external)
                if !func.dfg.ext_funcs.contains_key(*func_ref) {
                    internal_refs.insert(*func_ref);
                }
            }
        }
    }

    internal_refs
}
```

### 4. Pass Internal Function Refs to Converters

**File**: `crates/lp-glsl/src/transform/fixed32/rewrite.rs`

Update `convert_instruction()` to pass internal function refs:

```rust
fn convert_instruction(
    old_func: &Function,
    old_inst: Inst,
    _old_block: Block,
    _new_block: Block,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
    internal_func_refs: &HashSet<FuncRef>, // NEW
) -> Result<(), GlslError> {
    // ... existing code ...

    match opcode {
        Opcode::Call => {
            converters::calls::convert_call(
                old_func, old_inst, builder, value_map,
                ext_func_map, sig_map, format, block_map,
                internal_func_refs, // NEW
            )?;
        }
        // ... other opcodes ...
    }
}
```

## Testing

### Test File: `crates/lp-glsl-filetests/filetests/fixed32/internal_calls.glsl`

```glsl
// test compile
// test fixed32

// Test that internal function calls are detected correctly
float helper(float x) {
    return x * 2.0;
}

float main() {
    return helper(1.5);  // Should be 3.0
}

// Expected: No errors about external function calls
// Expected: Function call is handled as internal call
```

## Success Criteria

- [ ] `convert_call()` correctly detects internal vs external function calls
- [ ] Internal function calls don't try to access `ext_funcs`
- [ ] Function signatures are converted correctly for internal calls
- [ ] Arguments and return values are mapped correctly
- [ ] Tests pass without errors
- [ ] No regressions in external function call handling

## Files to Modify

1. `crates/lp-glsl/src/transform/fixed32/converters/calls.rs` - Detect internal/external
2. `crates/lp-glsl/src/transform/fixed32/rewrite.rs` - Track internal refs
3. `crates/lp-glsl-filetests/filetests/fixed32/internal_calls.glsl` - NEW test file

## Dependencies

- **Depends on**: Stage 01 (user functions must be converted at module level)
- **Enables**: Stage 03 (intrinsic conversion)

## Notes

- This stage focuses on detection and basic handling
- Full conversion of internal functions happens in Stage 01
- This stage ensures the infrastructure is in place for intrinsics
