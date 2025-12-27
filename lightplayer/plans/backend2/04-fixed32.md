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
- Calls: `call` (with FuncRef remapping via TransformContext)
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
├── blocks.rs                 # map_function_params() - fixed32-specific - ~100 lines
├── instructions.rs           # convert_all_instructions(), routing - ~200 lines
│
└── converters/              # Instruction converters (one file per category)
    ├── mod.rs
    ├── constants.rs         # F32const → fixed-point constant - ~50 lines
    ├── arithmetic.rs        # Fadd, Fsub, Fmul, Fdiv, Fneg, Fabs - ~200 lines
    ├── comparison.rs        # Fcmp variants - ~100 lines
    ├── control.rs           # Jump, Brif, Return, Select (with F32) - ~150 lines
    ├── memory.rs            # Load, Store (with F32) - ~150 lines
    ├── calls.rs             # Call (with FuncRef remapping) - ~100 lines
    ├── conversions.rs       # F32 ↔ fixed-point conversions - ~100 lines
    ├── math.rs              # Math functions - ~200 lines
    └── helpers.rs           # Shared helper functions - ~100 lines
```

## Architecture Overview

### Key Design Decisions

1. **Instruction Router Pattern**: Fixed32 transform has a `convert_instruction()` router that matches opcodes:
   - F32 instructions → Route to custom converters
   - Non-F32 instructions → Fall back to `copy_instruction()` (base copier)

2. **TransformContext for FuncRef Mapping**: Use `TransformContext.func_ref_map` for call instruction remapping (populated by `apply_transform()`).

3. **Shared Utilities**: Use all shared utilities from `backend2/transform/shared` for structure copying.

4. **Type Conversion**: Convert types at signature and instruction boundaries, preserve structure using shared utilities.

## Implementation Plan

### Step 1: Copy and Organize Fixed32 Code

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

3. **Copy `blocks.rs`** → `fixed32/blocks.rs`
   - Keep `map_function_params()` function
   - Update imports to use `backend2::transform::shared::ensure_block_params`

4. **Copy `instructions.rs`** → `fixed32/instructions.rs`
   - Keep `convert_all_instructions()` and `convert_instruction()` routing
   - Update imports to use shared utilities
   - **Key Change**: Fall back to `copy_instruction()` for non-F32 instructions
   - Start with only `fconst` and `fadd` in the match statement

5. **Copy `converters/`** → `fixed32/converters/`
   - Copy all converter files
   - Update imports to use `backend2::transform::shared`
   - Initially only implement `constants.rs` and `arithmetic.rs` (basic version)

### Step 2: Implement Basic Fixed32 Transform

**File**: `backend2/transform/fixed32/transform.rs`

**Implementation**:

```rust
use crate::backend2::transform::fixed32::signature::convert_signature;
use crate::backend2::transform::fixed32::instructions::convert_all_instructions;
use crate::backend2::transform::fixed32::types::FixedPointFormat;
use crate::backend2::transform::pipeline::{Transform, TransformContext};
use crate::error::GlslError;
use cranelift_codegen::ir::{Block, Function, Signature};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use hashbrown::HashMap;

use crate::backend2::transform::shared::{
    copy_stack_slots, copy_value_aliases, create_blocks, ensure_block_params,
    map_entry_block_params,
};
use crate::backend2::transform::fixed32::blocks::map_function_params;

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
}

impl Transform for Fixed32Transform {
    fn transform_signature(&self, sig: &Signature) -> Signature {
        convert_signature(sig, self.format)
    }

    fn transform_function<M: cranelift_module::Module>(
        &self,
        func: &Function,
        ctx: &mut TransformContext<'_, M>,
    ) -> Result<Function, GlslError> {
        // 1. Convert signature
        let new_sig = convert_signature(&func.signature, self.format);

        // 2. Create new function
        let mut new_func = Function::with_name_signature(func.name.clone(), new_sig);

        // 3. Copy stack slots
        let stack_slot_map = copy_stack_slots(func, &mut new_func)?;

        // 4. Create builder
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut new_func, &mut builder_ctx);

        // 5. Create maps
        let mut block_map = HashMap::new();
        let mut value_map = HashMap::new();

        // 6. Create blocks and map entry params
        create_blocks(func, &mut builder, &mut block_map, &mut value_map)?;

        // 7. Get entry block
        let entry_block = func.layout.entry_block().ok_or_else(|| {
            GlslError::new(crate::error::ErrorCode::E0301, "Function has no entry block")
        })?;
        let new_entry_block = block_map[&entry_block];

        // 8. Verify entry block params (basic check)
        map_entry_block_params(func, entry_block, new_entry_block, &mut builder, &value_map)?;

        // 9. Map function params with type conversion (fixed32-specific)
        map_function_params(
            func,
            entry_block,
            new_entry_block,
            &mut builder,
            &value_map,
            self.format,
        )?;

        // 10. Convert instructions (this handles F32 → fixed-point conversion)
        convert_all_instructions(
            func,
            &mut builder,
            &mut value_map,
            self.format,
            &block_map,
            &stack_slot_map,
            ctx, // Pass context for FuncRef remapping
        )?;

        // 11. Seal all blocks
        builder.seal_all_blocks();

        // 12. Finalize builder
        builder.finalize();

        // 13. Copy value aliases
        copy_value_aliases(func, &mut new_func, &value_map)?;

        Ok(new_func)
    }
}
```

**Key Points**:
- Uses shared utilities for copying structure
- Calls fixed32-specific converters for F32 instructions
- Falls back to base copier for non-F32 instructions (handled in `convert_all_instructions`)
- Handles FuncRef remapping via context
- Preserves source locations (handled in instruction converters)

### Step 3: Update Instructions Router

**File**: `backend2/transform/fixed32/instructions.rs`

**Key Changes**:

1. **Update `convert_all_instructions()` signature**:
```rust
pub(crate) fn convert_all_instructions<M: cranelift_module::Module>(
    old_func: &Function,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
    stack_slot_map: &HashMap<StackSlot, StackSlot>,
    ctx: &mut TransformContext<'_, M>, // Add this for FuncRef remapping
) -> Result<(), GlslError>
```

2. **Update `convert_instruction()` to fall back to base copier**:
```rust
fn convert_instruction<M: cranelift_module::Module>(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
    stack_slot_map: &HashMap<StackSlot, StackSlot>,
    ctx: &mut TransformContext<'_, M>,
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
            use crate::backend2::transform::shared::instruction_copy::{
                copy_instruction, InstructionCopyContext,
            };
            let mut copy_ctx = InstructionCopyContext {
                old_func,
                old_inst,
                builder,
                value_map,
                stack_slot_map: Some(stack_slot_map),
                block_map,
            };
            copy_instruction(&mut copy_ctx)?;
        }
    }

    Ok(())
}
```

3. **Start with minimal match statement** (only `fconst` and `fadd`), expand incrementally.

### Step 4: Implement Basic Converters

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

### Step 5: Update Call Converter for FuncRef Remapping

**File**: `backend2/transform/fixed32/converters/calls.rs` (when implementing calls)

**Key Change**: Use `TransformContext.func_ref_map` instead of internal `ext_func_map`:

```rust
use crate::backend2::transform::pipeline::TransformContext;
use cranelift_module::Module;

pub(crate) fn convert_call<M: Module>(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    ctx: &mut TransformContext<'_, M>, // Use context for FuncRef mapping
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Get old FuncRef
    let old_func_ref = /* extract from instruction */;
    
    // Look up function name from old_func_ref
    let func_name = /* extract function name */;
    
    // Look up new FuncRef from context
    let new_func_ref = ctx.func_ref_map.get(&func_name)
        .ok_or_else(|| GlslError::new(
            ErrorCode::E0301,
            format!("Function '{}' not found in func_ref_map", func_name),
        ))?;
    
    // Map arguments and emit call with new FuncRef
    // ...
}
```

**Note**: This step is for future implementation when adding call support.

### Step 6: Update Fixed32 Module Exports

**File**: `backend2/transform/fixed32/mod.rs`

**Changes**:
- Re-export `Fixed32Transform`
- Re-export `FixedPointFormat` from `types.rs`
- Keep internal modules private

### Step 7: Create Unit Tests

**File**: `lightplayer/crates/lp-glsl/tests/backend2_transform.rs` (add to existing)

**Test Cases**:

1. **Simple F32 Function**
   ```rust
   #[test]
   #[cfg(feature = "std")]
   fn test_fixed32_fconst() {
       // Build: f32 test() -> 1.5f
       // Transform
       // Verify: Returns fixed-point representation of 1.5
   }
   ```

2. **F32 Addition**
   ```rust
   #[test]
   #[cfg(feature = "std")]
   fn test_fixed32_fadd() {
       // Build: f32 add(f32 a, f32 b) -> a + b
       // Transform
       // Verify: Signature converted, addition works
   }
   ```

3. **Mixed F32 and I32 Instructions**
   ```rust
   #[test]
   #[cfg(feature = "std")]
   fn test_fixed32_mixed_instructions() {
       // Build: f32 test(f32 a, i32 b) -> a + f32(b)
       // Transform
       // Verify: F32 instructions converted, I32 instructions copied as-is
   }
   ```

4. **End-to-End Execution**
   ```rust
   #[test]
   #[cfg(feature = "std")]
   fn test_fixed32_execution() {
       // Build F32 function
       // Apply fixed32 transform
       // Build executable
       // Call with F32 inputs (converted to fixed-point)
       // Verify results match expected fixed-point values
   }
   ```

### Step 8: Incrementally Add More Instructions

**Order of Implementation**:

1. ✅ `fconst`, `fadd` (Phase 4 initial)
2. `fsub`, `fmul` (next)
3. `fdiv`, `fneg`, `fabs` (next)
4. Control flow: `jump`, `brif`, `return`, `select` (with F32 handling)
5. Memory: `load`, `store` (with F32 handling)
6. Comparisons: `fcmp` variants
7. Calls: `call` (requires FuncRef remapping via TransformContext)
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

### 4. FuncRef Remapping via TransformContext

**Decision**: Use `TransformContext.func_ref_map` for call instruction remapping.

**Rationale**:
- Context already provides FuncRef mapping (populated by `apply_transform()`)
- Matches the framework design
- Clean separation of concerns
- No need for internal `ext_func_map` like old transform

**Trade-off**: Need to pass context through instruction converters, but cleaner API.

### 5. Type Conversion Strategy

**Decision**: Convert types at signature and instruction boundaries, preserve structure.

**Rationale**:
- Clear separation: structure copying vs type conversion
- Reuses shared utilities for structure
- Only converters need to know about type conversion
- Block parameter conversion handled by `ensure_block_params` with type mapping function

**Trade-off**: Slightly more complex than full rewrite, but reuses proven code.

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
   - Stack slots with F32 types
   - Verify converted appropriately

6. **Non-F32 Instruction Preservation**
   - Functions with mixed F32 and I32 instructions
   - Verify I32 instructions copied correctly via base copier
   - Verify F32 instructions converted correctly

### Integration Tests

1. **End-to-End Fixed32 Transform**
   - Build F32 function → Transform → Build executable → Execute
   - Verify runtime behavior matches expected fixed-point arithmetic

2. **Comparison with Old Transform**
   - Build same function with old and new transforms
   - Compare runtime behavior (should match)
   - CLIF output may differ slightly due to different structure, but behavior should match

## Migration Notes

### From Old Transform

**Key Differences**:
- Old: Works with `ClifModule`, uses `ext_func_map` internally
- New: Works with `GlModule`, uses `TransformContext.func_ref_map`
- Old: Custom copying for all instructions
- New: Falls back to base copier for non-F32 instructions
- Old: Uses `backend/util` utilities
- New: Uses `backend2/transform/shared` utilities

**Adaptation Required**:
- Update FuncRef handling to use `TransformContext.func_ref_map`
- Update imports to use `backend2::transform::shared`
- Add fallback to `copy_instruction()` for non-F32 instructions
- Ensure compatibility with new module structure
- Inline `map_value` utility (don't import from old backend)

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
use crate::backend2::transform::shared::copy_stack_slots;
// map_value is now inline in converters (don't import from old backend)
```

## Open Questions

1. **FuncRef Remapping**: How do we handle FuncRefs in call instructions?
   - **Answer**: Use `TransformContext.func_ref_map` which is populated during `apply_transform()`. Converters look up new FuncRef by function name from the context.

2. **Error Handling**: What should we do for unsupported F32 instructions?
   - **Answer**: Return clear error message indicating which instruction is not yet supported. Incrementally add support. Non-F32 instructions automatically fall back to base copier.

3. **Testing Strategy**: Should we test against old transform output?
   - **Answer**: Yes, compare runtime behavior. CLIF output may differ slightly due to different structure (shared utilities vs custom copying), but behavior should match.

4. **Stack Slot Type Conversion**: How do we handle stack slots with F32 types?
   - **Answer**: Stack slots are copied as-is (they're just storage). The type conversion happens when loading/storing values. If needed, we can add stack slot type conversion later.

## References

- Phase 3 plan: `lightplayer/plans/backend2/03-transform-framework.md`
- Review document: `lightplayer/plans/backend2/03-transform-framework-review.md`
- Original fixed32 transform: `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/`
- Test patterns: `lightplayer/crates/lp-glsl/tests/transform_exact_match.rs`
- Existing fixed32 tests: `lightplayer/crates/lp-glsl/tests/transform_fixed32.rs`
