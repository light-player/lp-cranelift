# Phase 4: Fixed32 Transform Implementation

## Goal

Implement the fixed32 transform that converts F32 types and operations to fixed-point representation (16.16 format using I32). This builds on the transform framework from Phase 3 and adapts the existing fixed32 transform code from `backend/transform/fixed32` to work with the new `GlModule` architecture.

## Success Criteria

1. ✅ Fixed32 transform converts F32 signatures to fixed-point (I32)
2. ✅ Basic instructions supported: `fconst`, `fadd` (start small)
3. ✅ Transform preserves:
   - Block order
   - Block parameters (with type conversion)
   - Stack slots
   - Source locations
4. ✅ Non-F32 instructions fall back to base copier (`copy_instruction()`)
5. ✅ Unit tests verify correct conversion
6. ✅ End-to-end test: Build F32 function → Transform → Execute → Verify results

## Scope

### ✅ In Scope (Initial)

- Signature conversion (F32 → I32 for fixed-point)
- Basic arithmetic: `fconst`, `fadd`
- Block parameter type conversion
- Fallback to base copier for non-F32 instructions
- Unit tests for basic cases

### 🔄 Incremental Expansion (Future)

- More arithmetic: `fsub`, `fmul`, `fdiv`, `fneg`, `fabs`
- Control flow: `jump`, `brif`, `return`, `select` (with F32 handling)
- Memory: `load`, `store` (with F32 handling)
- Comparisons: `fcmp` variants
- Calls: `call` (with FuncRef remapping via local caches)
- Conversions: F32 ↔ fixed-point conversions
- Math functions: `sin`, `cos`, etc.

### ❌ Out of Scope (Future Phases)

- Full instruction set support (will be incremental)
- Performance optimizations
- Advanced fixed-point formats (32.32, etc.)

## File Structure

```
backend2/transform/fixed32/
├── mod.rs                    # Public API
├── transform.rs              # Fixed32Transform implementation - ~100 lines
├── signature.rs              # convert_signature() - ~50 lines
├── types.rs                  # FixedPointFormat, conversion utilities - ~100 lines
├── instructions.rs           # convert_all_instructions(), routing - ~200 lines
│
└── converters/              # Instruction converters (one file per category)
    ├── mod.rs
    ├── constants.rs         # F32const → fixed-point constant - ~50 lines
    ├── arithmetic.rs        # Fadd, Fsub, Fmul, Fdiv, Fneg, Fabs - ~200 lines
    ├── comparison.rs        # Fcmp variants - ~100 lines
    ├── control.rs           # Jump, Brif, Return, Select (with F32) - ~150 lines
    ├── memory.rs            # Load, Store (with F32) - ~150 lines
    ├── calls.rs             # Call (with FuncRef remapping via local caches) - ~100 lines
    ├── conversions.rs       # F32 ↔ fixed-point conversions - ~100 lines
    ├── math.rs              # Math functions - ~200 lines
    └── helpers.rs           # Shared helper functions - ~100 lines
```

## Architecture Overview

### Key Design Decisions

1. **Instruction Router Pattern**: Fixed32 transform has a `convert_instruction()` router that matches opcodes:

   - F32 instructions → Route to custom converters
   - Non-F32 instructions → Fall back to `copy_instruction()` (base copier)

2. **Local Caches for FuncRef Mapping**: Use local `ext_func_map` and `sig_map` caches (like old transform) for call instruction remapping. These are scoped to the function being transformed.

3. **Shared Utilities**: Use all shared utilities from `backend2/transform/shared` for structure copying.

4. **Type Conversion**: Convert types at signature and instruction boundaries, preserve structure using shared utilities. Signature conversion happens before `transform_function_body()`.

5. **Generic transform_function_body**: Use `transform_function_body()` with type mapping callback for block parameters.

## Implementation Plan

### Step 1: Update Shared Utilities to Support Type Mapping

**File**: `backend2/transform/shared/function.rs`

**Changes**: Add type mapping callback for block parameters:

```rust
pub fn transform_function_body(
    old_func: &Function,
    new_sig: Signature,
    transform_inst: impl Fn(
        &Function,
        Inst,
        &mut FunctionBuilder,
        &mut HashMap<Value, Value>,
        Option<&HashMap<StackSlot, StackSlot>>,
        &HashMap<Block, Block>,
    ) -> Result<(), GlslError>,
    map_param_type: impl Fn(Type) -> Type,  // Add this callback
) -> Result<Function, GlslError> {
    // ... existing code ...

    // Update copy_stack_slots call to use map_param_type for stack slot types
    let stack_slot_map = copy_stack_slots(old_func, &mut new_func, &map_param_type)?;

    // Update ensure_block_params calls to use map_param_type
    // In create_blocks, entry block params are created from signature (already converted)
    // In ensure_block_params calls, use map_param_type for type conversion

    // Update ensure_block_params call in instruction copying to use map_param_type
    ensure_block_params(
        old_func,
        old_dest_block,
        new_dest_block,
        builder,
        value_map,
        &map_param_type,  // Pass type mapping function
    )?;

    // ... rest of code ...
}
```

**File**: `backend2/transform/shared/stack_slots.rs`

**Changes**: Update `copy_stack_slots()` to accept type mapping function:

```rust
pub fn copy_stack_slots(
    old_func: &Function,
    new_func: &mut Function,
    map_type: impl Fn(Type) -> Type,  // Add type mapping callback
) -> Result<HashMap<StackSlot, StackSlot>, GlslError> {
    let mut stack_slot_map = HashMap::new();

    for (old_slot_idx, old_slot_data) in old_func.sized_stack_slots.iter() {
        // Convert stack slot type if needed
        let mut new_slot_data = old_slot_data.clone();
        if let Some(old_type) = old_slot_data.ty {
            new_slot_data.ty = Some(map_type(old_type));
        }

        let new_slot_idx = new_func.sized_stack_slots.push(new_slot_data);
        stack_slot_map.insert(old_slot_idx, new_slot_idx);
        // ... verification ...
    }

    Ok(stack_slot_map)
}
```

**Also update**: `ensure_block_params` calls in `instruction_copy.rs` to accept and use the type mapping function.

### Step 2: Copy and Organize Fixed32 Code

**Source**: `backend/transform/fixed32/`

**Target**: `backend2/transform/fixed32/`

**Changes**:

1. **Copy `types.rs`** → `fixed32/types.rs`

   - Keep `FixedPointFormat` enum
   - Keep conversion utilities (`float_to_fixed16x16`, etc.)
   - Update imports (no changes needed - standalone)

2. **Copy `signature.rs`** → `fixed32/signature.rs`

   - Keep `convert_signature()` function
   - Update imports (no changes needed - standalone)

3. **Copy `instructions.rs`** → `fixed32/instructions.rs`

   - Keep `convert_all_instructions()` and `convert_instruction()` routing
   - Update imports to use shared utilities
   - **Key Change**: Fall back to `copy_instruction()` for non-F32 instructions
   - **Key Change**: Use local `ext_func_map` and `sig_map` caches (not TransformContext)
   - Start with only `fconst` and `fadd` in the match statement

4. **Copy `converters/`** → `fixed32/converters/`
   - Copy all converter files
   - Update imports to use `backend2::transform::shared`
   - **Key Change**: Update `calls.rs` to use local `ext_func_map` and `sig_map` (not TransformContext)
   - Initially only implement `constants.rs` and `arithmetic.rs` (basic version)

### Step 3: Implement Basic Fixed32 Transform

**File**: `backend2/transform/fixed32/transform.rs`

**Implementation**:

```rust
use crate::backend2::transform::fixed32::signature::convert_signature;
use crate::backend2::transform::fixed32::instructions::convert_all_instructions;
use crate::backend2::transform::fixed32::types::FixedPointFormat;
use crate::backend2::transform::pipeline::{Transform, TransformContext};
use crate::backend2::transform::shared::transform_function_body;
use crate::error::GlslError;
use cranelift_codegen::ir::{Function, Signature, Type};
use hashbrown::HashMap;

/// Fixed32 transform - converts F32 to fixed-point representation
pub struct Fixed32Transform {
    format: FixedPointFormat,
}

impl Fixed32Transform {
    /// Create a new Fixed32 transform with the specified format
    pub fn new(format: FixedPointFormat) -> Self {
        Self { format }
    }

    /// Create a Fixed32 transform with default format (Fixed16x16)
    pub fn default() -> Self {
        Self::new(FixedPointFormat::Fixed16x16)
    }

    /// Map F32 type to I32 (fixed-point), other types unchanged
    fn map_type(&self, ty: Type) -> Type {
        use cranelift_codegen::ir::types;
        if ty == types::F32 {
            types::I32
        } else {
            ty
        }
    }
}

impl Transform for Fixed32Transform {
    fn transform_signature(&self, sig: &Signature) -> Signature {
        convert_signature(sig, self.format)
    }

    fn transform_function<M: cranelift_module::Module>(
        &self,
        old_func: &Function,
        _ctx: &mut TransformContext<'_, M>,
    ) -> Result<Function, GlslError> {
        // 1. Convert signature (happens before transform_function_body)
        let new_sig = convert_signature(&old_func.signature, self.format);

        // 2. Create type mapping function for block parameters
        let format = self.format;
        let map_param_type = move |ty: Type| -> Type {
            if ty == cranelift_codegen::ir::types::F32 {
                cranelift_codegen::ir::types::I32
            } else {
                ty
            }
        };

        // 3. Use shared transform_function_body with instruction converter
        transform_function_body(
            old_func,
            new_sig,
            // Instruction transformation callback
            move |old_func, old_inst, builder, value_map, stack_slot_map, block_map| {
                convert_all_instructions(
                    old_func,
                    old_inst,
                    builder,
                    value_map,
                    format,
                    block_map,
                    stack_slot_map,
                )
            },
            // Type mapping callback for block parameters
            map_param_type,
        )
    }
}
```

**Key Points**:

- Signature conversion happens before `transform_function_body()`
- Uses `transform_function_body()` with type mapping callback
- Instruction converter uses local caches (no TransformContext needed)
- Preserves source locations (handled in instruction converters)

### Step 4: Update Instructions Router

**File**: `backend2/transform/fixed32/instructions.rs`

**Key Changes**:

1. **Update `convert_all_instructions()` signature** (no TransformContext):

```rust
pub(crate) fn convert_all_instructions(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
    stack_slot_map: Option<&HashMap<StackSlot, StackSlot>>,
) -> Result<(), GlslError>
```

2. **Create local caches for FuncRef and SigRef mapping** (when needed for calls):

```rust
// These will be created in convert_instruction when handling Call/CallIndirect
// For now, we don't need them for basic instructions (fconst, fadd)
```

3. **Update `convert_instruction()` to fall back to base copier**:

```rust
fn convert_instruction(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
    stack_slot_map: Option<&HashMap<StackSlot, StackSlot>>,
) -> Result<(), GlslError> {
    // Copy source location
    let srcloc = old_func.srcloc(old_inst);
    if !srcloc.is_default() {
        builder.set_srcloc(srcloc);
    }

    let opcode = old_func.dfg.insts[old_inst].opcode();

    // Route to appropriate converter
    match opcode {
        Opcode::F32const => {
            converters::constants::convert_f32const(
                old_func, old_inst, builder, value_map, format,
            )?;
        }
        Opcode::Fadd => {
            converters::arithmetic::convert_fadd(
                old_func, old_inst, builder, value_map, format
            )?;
        }
        // ... more F32 instructions as we add them ...
        _ => {
            // For non-F32 instructions, fall back to base copier
            use crate::backend2::transform::shared::copy_instruction;
            copy_instruction(
                old_func,
                old_inst,
                builder,
                value_map,
                stack_slot_map,
                block_map,
                None, // func_ref_map not needed for non-call instructions
            )?;
        }
    }

    Ok(())
}
```

4. **Start with minimal match statement** (only `fconst` and `fadd`), expand incrementally.

### Step 5: Implement Basic Converters

**File**: `backend2/transform/fixed32/converters/constants.rs`

**Implementation** (copy from old, update imports):

```rust
use crate::backend2::transform::fixed32::types::{FixedPointFormat, float_to_fixed16x16};
use crate::error::GlslError;
use cranelift_codegen::ir::{Function, Inst, types};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

/// Convert F32const to fixed-point constant
pub(crate) fn convert_f32const(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Get the F32 constant value
    let f32_value = old_func.dfg.const_to_f32(old_func.dfg.inst_args(old_inst)[0])
        .ok_or_else(|| GlslError::new(
            crate::error::ErrorCode::E0301,
            "F32const instruction does not have a constant argument",
        ))?;

    // Convert to fixed-point
    let fixed_value = match format {
        FixedPointFormat::Fixed16x16 => float_to_fixed16x16(f32_value),
        FixedPointFormat::Fixed32x32 => {
            return Err(GlslError::new(
                crate::error::ErrorCode::E0301,
                "Fixed32x32 format not yet implemented",
            ));
        }
    };

    // Create I32 constant
    let new_value = builder.ins().iconst(types::I32, fixed_value as i64);

    // Map old value to new value
    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_value);

    Ok(())
}
```

**File**: `backend2/transform/fixed32/converters/arithmetic.rs`

**Implementation** (copy from old, update imports, start with `fadd` only):

```rust
use crate::backend2::transform::fixed32::types::FixedPointFormat;
use crate::error::GlslError;
use cranelift_codegen::ir::{Function, Inst, types};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

// Inline map_value utility (don't import from old backend)
fn map_value(value_map: &HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>, old_value: cranelift_codegen::ir::Value) -> cranelift_codegen::ir::Value {
    *value_map.get(&old_value).unwrap_or(&old_value)
}

/// Convert Fadd to fixed-point addition
pub(crate) fn convert_fadd(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let args = old_func.dfg.inst_args(old_inst);
    let a = map_value(value_map, args[0]);
    let b = map_value(value_map, args[1]);

    // Fixed-point addition is just integer addition (no conversion needed)
    // Both operands are already in fixed-point format
    let result = builder.ins().iadd(a, b);

    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, result);

    Ok(())
}
```

**Key Points**:

- Inline `map_value` utility (don't import from old backend)
- Update imports to use `backend2::transform::shared` where needed
- Keep same logic as old converters

### Step 6: Update Call Converter for Local Caches (Future)

**File**: `backend2/transform/fixed32/converters/calls.rs` (when implementing calls)

**Key Change**: Use local `ext_func_map` and `sig_map` caches (like old transform):

```rust
use crate::backend2::transform::fixed32::signature::convert_signature;
use crate::backend2::transform::fixed32::types::FixedPointFormat;
use cranelift_codegen::ir::{ExtFuncData, ExternalName, FuncRef, Function, Inst, SigRef, Value};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

/// Maps an external function reference to a new function reference with converted signature.
/// Uses local caches (ext_func_map, sig_map) scoped to the function being transformed.
pub(crate) fn map_external_function(
    old_func: &Function,
    old_func_ref: FuncRef,
    builder: &mut FunctionBuilder,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
) -> Result<FuncRef, GlslError> {
    // Check cache first
    if let Some(&new_func_ref) = ext_func_map.get(&old_func_ref) {
        return Ok(new_func_ref);
    }

    let old_ext_func = &old_func.dfg.ext_funcs[old_func_ref];
    let old_sig_ref = old_ext_func.signature;

    // Convert signature (with caching)
    let new_sig_ref = if let Some(&mapped_sig_ref) = sig_map.get(&old_sig_ref) {
        mapped_sig_ref
    } else {
        let old_sig = &old_func.dfg.signatures[old_sig_ref];
        let new_sig = convert_signature(old_sig, format);
        let new_sig_ref = builder.func.import_signature(new_sig);
        sig_map.insert(old_sig_ref, new_sig_ref);
        new_sig_ref
    };

    // Handle external name (same as old transform)
    let new_name = match &old_ext_func.name {
        ExternalName::User(old_user_ref) => {
            let user_name = old_func
                .params
                .user_named_funcs()
                .get(*old_user_ref)
                .cloned()
                .ok_or_else(|| {
                    GlslError::new(
                        ErrorCode::E0301,
                        format!(
                            "UserExternalNameRef {} not found in function's user_named_funcs",
                            old_user_ref
                        ),
                    )
                })?;
            let new_user_ref = builder.func.declare_imported_user_function(user_name);
            ExternalName::User(new_user_ref)
        }
        _ => old_ext_func.name.clone(),
    };

    let new_ext_func = ExtFuncData {
        name: new_name,
        signature: new_sig_ref,
        colocated: old_ext_func.colocated,
    };

    let new_func_ref = builder.func.import_function(new_ext_func);
    ext_func_map.insert(old_func_ref, new_func_ref);

    Ok(new_func_ref)
}

pub(crate) fn convert_call(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Extract FuncRef from instruction
    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Call { func_ref, args, .. } = inst_data {
        // Map FuncRef using local caches
        let new_func_ref = map_external_function(
            old_func,
            *func_ref,
            builder,
            ext_func_map,
            sig_map,
            format,
        )?;

        // Map arguments and emit call
        let old_args = args.as_slice(&old_func.dfg.value_lists);
        let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();

        let call_inst = builder.ins().call(new_func_ref, &new_args);

        // Map results
        let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
        let new_results = builder.inst_results(call_inst);

        if old_results.len() != new_results.len() {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "Call return value count mismatch: old={}, new={}",
                    old_results.len(),
                    new_results.len()
                ),
            ));
        }

        for (old_result, new_result) in old_results.iter().zip(new_results.iter()) {
            value_map.insert(*old_result, *new_result);
        }
    }

    Ok(())
}
```

**Note**: This step is for future implementation when adding call support. The caches (`ext_func_map`, `sig_map`) will be created in `convert_all_instructions()` when handling Call/CallIndirect instructions.

### Step 7: Update Fixed32 Module Exports

**File**: `backend2/transform/fixed32/mod.rs`

**Changes**:

- Re-export `Fixed32Transform`
- Re-export `FixedPointFormat` from `types.rs`
- Keep internal modules private

### Step 8: Create Unit Tests

**File**: `lightplayer/crates/lp-glsl/tests/backend2_transform.rs` (add to existing)

**Primary Smoke Test**:

The main test should be independent of the old implementation. It should:

1. Parse a CLIF function from source (using `cranelift_reader::parse_test`)
2. Apply the fixed32 transform
3. Build an executable
4. Run the function with test inputs
5. Verify the results are correct

This is a smoke test covering basic operations: add, sub, mul, div, etc. Detailed tests are in filetests.

```rust
#[test]
#[cfg(feature = "std")]
fn test_fixed32_smoke() {
    // Parse CLIF function with basic F32 operations
    // Apply fixed32 transform
    // Build executable
    // Run with test inputs
    // Verify results match expected fixed-point arithmetic
    // Should cover: fconst, fadd, fsub, fmul, fdiv
}
```

**Additional Unit Tests** (for specific functionality):

1. **Signature Conversion**

   ```rust
   #[test]
   fn test_fixed32_signature_conversion() {
       // Verify F32 params → I32 params
       // Verify F32 returns → I32 returns
       // Verify mixed signatures (F32 and I32)
   }
   ```

2. **Stack Slot Type Conversion**

   ```rust
   #[test]
   fn test_fixed32_stack_slot_conversion() {
       // Verify stack slots with F32 types are converted to I32
       // Verify stack slot mapping works correctly
   }
   ```

3. **Block Parameter Conversion**
   ```rust
   #[test]
   fn test_fixed32_block_params() {
       // Verify blocks with F32 parameters are converted to I32
       // Verify block parameter mapping works correctly
   }
   ```

### Step 9: Incrementally Add More Instructions

**Order of Implementation**:

1. ✅ `fconst`, `fadd` (Phase 4 initial)
2. `fsub`, `fmul` (next)
3. `fdiv`, `fneg`, `fabs` (next)
4. Control flow: `jump`, `brif`, `return`, `select` (with F32 handling)
5. Memory: `load`, `store` (with F32 handling)
6. Comparisons: `fcmp` variants
7. Calls: `call` (requires local `ext_func_map` and `sig_map` caches)
8. Conversions and math functions

**Strategy**: Add one instruction category at a time, with tests for each. Each new instruction goes in the match statement in `convert_instruction()`, non-F32 instructions automatically fall back to base copier.

## Key Design Decisions

### 1. Incremental Implementation

**Decision**: Start with `fconst` and `fadd` only, expand incrementally.

**Rationale**:

- Validates the framework works
- Easier to debug issues
- Can test each instruction category independently
- Matches user preference for incremental development

**Trade-off**: Not all instructions supported initially, but safer approach.

### 2. Fallback to Base Copier

**Decision**: Non-F32 instructions fall back to `copy_instruction()` (base copier).

**Rationale**:

- Reuses proven copying code from Phase 3
- No need to reimplement copying for non-F32 instructions
- Clean separation: F32 instructions → custom converters, others → base copier
- Matches the architecture: identity transform IS the base copier

**Trade-off**: Slightly more complex routing, but much less code to maintain.

### 3. Reuse Old Converters

**Decision**: Copy converters from `backend/transform/fixed32/converters/` and adapt.

**Rationale**:

- Old converters are proven and tested
- Saves implementation time
- Can clean up and improve during copy
- Focus effort on integration, not rewriting

**Trade-off**: Need to update imports and ensure compatibility, but less risk.

### 4. Local Caches for FuncRef Remapping

**Decision**: Use local `ext_func_map` and `sig_map` caches (like old transform) instead of `TransformContext.func_ref_map`.

**Rationale**:

- FuncRefs are scoped to individual functions, so module-level FuncRefs aren't valid
- Need signature conversion (F32 → I32), which requires local caching
- Matches the proven pattern from old transform
- Simpler than trying to use TransformContext

**Trade-off**: Need to create caches in instruction router, but cleaner and more correct.

### 5. Type Conversion Strategy

**Decision**: Convert types at signature and instruction boundaries, preserve structure.

**Rationale**:

- Clear separation: structure copying vs type conversion
- Reuses shared utilities for structure
- Only converters need to know about type conversion
- Block parameter conversion handled by `ensure_block_params` with type mapping function
- Signature conversion happens before `transform_function_body()`

**Trade-off**: Slightly more complex than full rewrite, but reuses proven code.

### 6. Generic transform_function_body

**Decision**: Use `transform_function_body()` with type mapping callback for block parameters.

**Rationale**:

- Reuses shared orchestration code
- Type mapping callback allows transforms to customize parameter types
- Signature conversion happens before calling `transform_function_body()`
- Matches user requirement to use shared utilities

**Trade-off**: Need to update `transform_function_body()` to support type mapping, but enables reuse.

## Test Requirements

### Unit Tests

1. **Signature Conversion**

   - F32 params → I32 params
   - F32 returns → I32 returns
   - Mixed signatures (F32 and I32)

2. **F32const Conversion**

   - Various F32 values
   - Edge cases (0.0, negative, large values)
   - Verify fixed-point representation

3. **Fadd Conversion**

   - Simple addition
   - Multiple additions in sequence
   - Verify results match expected fixed-point arithmetic

4. **Block Parameter Conversion**

   - Blocks with F32 parameters
   - Verify converted to I32
   - Verify block parameter mapping works correctly

5. **Stack Slot Conversion**

   - Stack slots with F32 types must be converted to I32
   - Verify converted appropriately
   - This requires updating `copy_stack_slots()` to accept a type mapping function

6. **Non-F32 Instruction Preservation**
   - Functions with mixed F32 and I32 instructions
   - Verify I32 instructions copied correctly via base copier
   - Verify F32 instructions converted correctly

### Integration Tests

1. **Primary Smoke Test**

   - Parse CLIF function from source → Transform → Build executable → Execute
   - Verify runtime behavior matches expected fixed-point arithmetic
   - Should cover all basic operations: add, sub, mul, div, etc.
   - This is the main test; detailed tests are in filetests

2. **Detailed Filetests**
   - Use existing filetest infrastructure for detailed test cases
   - These provide comprehensive coverage of edge cases and specific scenarios

## Migration Notes

### From Old Transform

**Key Differences**:

- Old: Works with `ClifModule`, uses `ext_func_map` internally
- New: Works with `GlModule`, uses local `ext_func_map` and `sig_map` caches (same pattern)
- Old: Custom copying for all instructions
- New: Falls back to base copier for non-F32 instructions
- Old: Uses `backend/util` utilities
- New: Uses `backend2/transform/shared` utilities
- Old: Custom function transformation orchestration
- New: Uses `transform_function_body()` with type mapping callback

**Adaptation Required**:

- Update FuncRef handling to use local caches (same pattern as old)
- Update imports to use `backend2::transform::shared`
- Add fallback to `copy_instruction()` for non-F32 instructions
- Ensure compatibility with new module structure
- Inline `map_value` utility (don't import from old backend)
- Use `transform_function_body()` instead of custom orchestration
- Update `copy_stack_slots()` to support type conversion (F32 → I32 for stack slots)

### Import Updates

**Old**:

```rust
use crate::backend::transform::fixed32::convert_signature;
use crate::backend::util::clif_copy::copy_stack_slots;
use crate::backend::ir_utils::value_map::map_value;
```

**New**:

```rust
use crate::backend2::transform::fixed32::signature::convert_signature;
use crate::backend2::transform::shared::transform_function_body;
use crate::backend2::transform::shared::copy_stack_slots;
// map_value is now inline in converters (don't import from old backend)
```

## Open Questions

1. **FuncRef Remapping**: How do we handle FuncRefs in call instructions?

   - **Answer**: Use local `ext_func_map` and `sig_map` caches (like old transform). These are created in the instruction router when handling Call/CallIndirect instructions.

2. **Error Handling**: What should we do for unsupported F32 instructions?

   - **Answer**: Return clear error message indicating which instruction is not yet supported. Incrementally add support. Non-F32 instructions automatically fall back to base copier.

3. **Testing Strategy**: Should we test against old transform output?

   - **Answer**: Tests should be independent of old implementation. Primary test should be a test that parses a CLIF function from source, transforms it, runs it, and verifies that the result is correct. It's just a smoke test of the functionality, we have detailed filetests. It should do all the basic operations: add, sub, mul, div, etc.

4. **Stack Slot Type Conversion**: How do we handle stack slots with F32 types?
   - **Answer**: We need to convert them, because the type of data is changing, and we don't want to confuse the lowering code. This may require slight refactoring.

## References

- Phase 3 plan: `lightplayer/plans/backend2/03-transform-framework.md`
- Review document: `lightplayer/plans/backend2/03-transform-framework-review.md`
- Original fixed32 transform: `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/`
- Test patterns: `lightplayer/crates/lp-glsl/tests/transform_exact_match.rs`
- Existing fixed32 tests: `lightplayer/crates/lp-glsl/tests/transform_fixed32.rs`
