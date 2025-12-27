# Phase 3: Transform Framework - Identity Transform and Shared Utilities

## Goal

Build the foundational transform framework with shared utilities and an identity (NOP) transform that copies functions exactly. This validates the copying infrastructure before implementing type-converting transforms like fixed32.

## Success Criteria

1. ✅ Shared utilities copied from `backend/util` and organized into `backend2/transform/shared/`
2. ✅ Identity transform implemented that copies functions exactly (no modifications)
3. ✅ Identity transform preserves:
   - Block order (numeric and layout)
   - Block parameters
   - Stack slots
   - Value aliases
   - Source locations (per-instruction)
   - Value numbering (for i32-only functions)
   - Jump tables (for BrTable instructions)
   - Function calls (Call/CallIndirect)
4. ✅ Comprehensive unit tests verify exact copying behavior
5. ✅ Transform framework ready for fixed32 implementation

## Scope

### ✅ In Scope

- Copy and organize shared utilities from `backend/util` to `backend2/transform/shared/`
- Implement identity transform using shared utilities
- Unit tests for identity transform (including multi-function tests)
- Test infrastructure for verifying transform correctness
- Instruction support: All instruction formats via generic copier
- Terminator support: `jump`, `brif`, `return`, `br_table`
- Jump table support: `BrTable` with jump table copying
- Call support: `Call`/`CallIndirect` (FuncRef copied as-is)

### ❌ Out of Scope (Future Phases)

- Type-converting transforms (fixed32 - Phase 4)
- Integration with GLSL frontend
- Performance optimizations
- FuncRef mapping (not needed for identity transform)

## File Structure

```
backend2/transform/
├── mod.rs                    # Public API - re-exports
├── pipeline.rs               # Transform trait, TransformContext (already exists)
│
├── shared/                   # Shared utilities (copied from backend/util, cleaned up)
│   ├── mod.rs                # Re-exports all shared utilities
│   ├── stack_slots.rs        # copy_stack_slots() - ~50 lines
│   ├── blocks.rs             # create_blocks(), ensure_block_params(), map_entry_block_params() - ~200 lines
│   ├── value_aliases.rs      # copy_value_aliases() - ~70 lines
│   └── instruction_copy.rs  # copy_instruction() with InstructionCopyContext - ~500+ lines
│
└── identity/                 # Identity/NOP transform (copies exactly)
    ├── mod.rs                # Public API
    └── transform.rs          # IdentityTransform implementation - ~100 lines
```

## Architecture Overview

### Key Design Decisions

1. **Identity Transform = Base Copier**: The identity transform directly calls `copy_instruction()` for all instructions. No router function needed - it IS the base copier.

2. **InstructionCopyContext Struct**: All parameters bundled into a single struct for cleaner API.

3. **Complete Instruction Support**: The base copier handles ALL instruction formats explicitly, including:

   - Terminators (`Jump`, `Brif`, `Return`, `BrTable`)
   - Stack operations (`StackLoad`, `StackStore`)
   - Memory operations (`Load`, `Store`)
   - Calls (`Call`, `CallIndirect`)
   - All other instruction formats

4. **Panic on Unsupported**: If an instruction format isn't handled, panic (internal error - should never happen).

5. **No Backend Dependencies**: Cannot import from old `backend` - copy all utilities to `backend2`.

## Implementation Plan

### Step 1: Create Shared Utilities Structure

**Files**: `backend2/transform/shared/mod.rs`, `stack_slots.rs`, `blocks.rs`, `value_aliases.rs`, `instruction_copy.rs`

**Changes**:

1. **Copy from `backend/util/clif_copy.rs`** and split into focused files:

   - `stack_slots.rs`: Extract `copy_stack_slots()` function
   - `blocks.rs`: Extract `create_blocks()`, `ensure_block_params()`, `map_entry_block_params()`
   - `value_aliases.rs`: Extract `copy_value_aliases()` function

2. **Copy from `backend/util/instruction_copy.rs`** and enhance:

   - `instruction_copy.rs`: Copy `copy_instruction()` function and enhance with:
     - `InstructionCopyContext` struct
     - Complete terminator support (`Jump`, `Brif`, `Return`, `BrTable`)
     - Jump table copying for `BrTable`
     - `Call`/`CallIndirect` support
     - Inline `map_value` utility (don't import from old backend)
     - All `InstructionData` variants handled explicitly

3. **Create `shared/mod.rs`**:
   - Re-export all utilities for easy importing
   - Add module-level documentation

**Key Points**:

- Keep the same function signatures where possible
- Update imports to use `crate::backend2::transform::shared`
- Add `#![no_std]` support where needed
- Clean up any TODO/FIXME comments
- Inline `map_value` utility (don't create separate file)

**InstructionCopyContext Struct**:

```rust
pub struct InstructionCopyContext<'a> {
    pub old_func: &'a Function,
    pub old_inst: Inst,
    pub builder: &'a mut FunctionBuilder,
    pub value_map: &'a mut HashMap<Value, Value>,
    pub stack_slot_map: Option<&'a HashMap<StackSlot, StackSlot>>,
    pub block_map: &'a HashMap<Block, Block>,
}
```

**copy_instruction() Signature**:

```rust
pub fn copy_instruction(ctx: &mut InstructionCopyContext) -> Result<(), GlslError>
```

### Step 2: Implement Instruction Copying

**File**: `backend2/transform/shared/instruction_copy.rs`

**Key Features**:

1. **Inline map_value utility**:

```rust
fn map_value(value_map: &HashMap<Value, Value>, old_value: Value) -> Value {
    *value_map.get(&old_value).unwrap_or(&old_value)
}
```

2. **Handle all instruction formats explicitly**:

   - `Unary`, `UnaryImm`, `Binary`, `Ternary`, `NullAry`
   - `IntCompare`, `FloatCompare`
   - `UnaryIeee32`, `UnaryIeee64`, `UnaryConst`
   - `Load`, `Store`
   - `StackLoad`, `StackStore`
   - `Jump`, `Brif`, `Return`, `BrTable` (terminators)
   - `Call`, `CallIndirect`
   - All other `InstructionData` variants

3. **Terminator handling**:

   - `Jump`: Use `builder.ins().jump(new_dest_block, &new_args)`
   - `Brif`: Use `builder.ins().brif(condition, then_block, &then_args, else_block, &else_args)`
   - `Return`: Use `builder.ins().return_(&args)`
   - `BrTable`: Copy jump table structure, create new table, use `builder.ins().br_table(condition, new_table)`

4. **Jump table copying** (for `BrTable`):

   - Read old jump table from `old_func.dfg.jump_tables[*table]`
   - Map all block calls in the table (map blocks via `block_map`, map values via `value_map`)
   - Create new jump table with `builder.create_jump_table(JumpTableData::new(...))`
   - Emit `builder.ins().br_table(condition, new_table)`

5. **Call instruction handling**:

   - For `Call`/`CallIndirect`: Copy FuncRef as-is (no mapping needed for identity transform)
   - Map arguments via `value_map`
   - Map results via `value_map`

6. **Error handling**: Panic on unsupported instruction formats (internal error).

**Reference Implementation**: See `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/converters/control.rs` for terminator patterns.

### Step 3: Implement Identity Transform

**File**: `backend2/transform/identity/transform.rs`

**Implementation**:

```rust
use crate::backend2::transform::pipeline::{Transform, TransformContext};
use crate::backend2::transform::shared::{
    copy_stack_slots, copy_value_aliases, create_blocks, ensure_block_params,
    map_entry_block_params, copy_instruction, InstructionCopyContext,
};
use crate::error::GlslError;
use cranelift_codegen::ir::{Block, Function, Inst, Signature};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use hashbrown::HashMap;

/// Identity transform - copies functions exactly without modification
pub struct IdentityTransform;

impl Transform for IdentityTransform {
    fn transform_signature(&self, sig: &Signature) -> Signature {
        sig.clone()
    }

    fn transform_function<M: cranelift_module::Module>(
        &self,
        func: &Function,
        _ctx: &mut TransformContext<'_, M>,
    ) -> Result<Function, GlslError> {
        // 1. Create new function with same signature
        let mut new_func = Function::with_name_signature(func.name.clone(), func.signature.clone());

        // 2. Copy stack slots
        let stack_slot_map = copy_stack_slots(func, &mut new_func)?;

        // 3. Create builder
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut new_func, &mut builder_ctx);

        // 4. Create maps
        let mut block_map = HashMap::new();
        let mut value_map = HashMap::new();

        // 5. Create blocks and map entry params
        create_blocks(func, &mut builder, &mut block_map, &mut value_map)?;

        // 6. Get entry block
        let entry_block = func.layout.entry_block().ok_or_else(|| {
            GlslError::new(crate::error::ErrorCode::E0301, "Function has no entry block")
        })?;
        let new_entry_block = block_map[&entry_block];

        // 7. Verify entry block params
        map_entry_block_params(func, entry_block, new_entry_block, &mut builder, &value_map)?;

        // 8. Copy all instructions
        let old_blocks: Vec<Block> = func.layout.blocks().collect();
        let mut block_insts: Vec<(Block, Vec<Inst>)> = Vec::new();
        for old_block in &old_blocks {
            let insts: Vec<Inst> = func.layout.block_insts(*old_block).collect();
            block_insts.push((*old_block, insts));
        }

        for (old_block, insts) in block_insts {
            let new_block = block_map[&old_block];
            builder.switch_to_block(new_block);

            // Ensure block params exist (on-demand creation)
            ensure_block_params(
                func,
                old_block,
                new_block,
                &mut builder,
                &mut value_map,
                |t| t, // Identity type mapping
            )?;

            // Copy source location for block (if first instruction has one)
            if let Some(first_inst) = insts.first() {
                let srcloc = func.srcloc(*first_inst);
                if !srcloc.is_default() {
                    builder.set_srcloc(srcloc);
                }
            }

            // Copy each instruction
            for old_inst in insts {
                // Copy source location for each instruction
                let srcloc = func.srcloc(old_inst);
                if !srcloc.is_default() {
                    builder.set_srcloc(srcloc);
                }

                // Copy instruction using base copier
                let mut copy_ctx = InstructionCopyContext {
                    old_func: func,
                    old_inst,
                    builder: &mut builder,
                    value_map: &mut value_map,
                    stack_slot_map: Some(&stack_slot_map),
                    block_map: &block_map,
                };
                copy_instruction(&mut copy_ctx)?;
            }
        }

        // 9. Seal all blocks
        builder.seal_all_blocks();

        // 10. Finalize builder
        builder.finalize();

        // 11. Copy value aliases
        copy_value_aliases(func, &mut new_func, &value_map)?;

        Ok(new_func)
    }
}
```

**Key Points**:

- Directly calls `copy_instruction()` for all instructions (no router)
- Preserves source locations per-instruction
- Handles block parameters correctly
- Copies stack slots and value aliases
- Supports all instruction types via base copier

### Step 4: Create Identity Transform Module

**File**: `backend2/transform/identity/mod.rs`

```rust
//! Identity transform - copies functions exactly without modification
//!
//! This transform is useful for:
//! - Testing the transform framework
//! - Validating that copying preserves all function structure
//! - As a base for other transforms

mod transform;

pub use transform::IdentityTransform;
```

### Step 5: Update Transform Module Exports

**File**: `backend2/transform/mod.rs`

**Changes**:

- Add `pub mod shared;`
- Add `pub mod identity;`
- Keep `pub mod fixed32;` (will be implemented in Phase 4)
- Keep `pub mod pipeline;`

### Step 6: Create Unit Test Infrastructure

**File**: `lightplayer/crates/lp-glsl/tests/backend2_transform.rs`

**Test Structure**:

```rust
//! Transform tests for backend2
//!
//! These tests verify that transforms preserve function structure correctly.

use cranelift_codegen::ir::{AbiParam, Signature, types};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::write_function;
use lp_glsl::backend2::module::gl_module::GlModule;
use lp_glsl::backend2::module::test_helpers::build_simple_function;
use lp_glsl::backend2::target::Target;
use lp_glsl::backend2::transform::identity::IdentityTransform;
use lp_glsl::backend2::transform::pipeline::Transform;

#[cfg(feature = "std")]
fn create_test_module() -> GlModule<cranelift_jit::JITModule> {
    let target = Target::host_jit().unwrap();
    GlModule::new_jit(target).unwrap()
}

/// Normalize CLIF strings for comparison
fn normalize_clif(clif: &str) -> String {
    clif.lines()
        .map(|line| {
            let line = if let Some(comment_pos) = line.find(';') {
                &line[..comment_pos]
            } else {
                line
            };
            line.trim()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_simple() {
    // Build a simple function: i32 add(i32 a, i32 b) -> a + b
    let mut gl_module = create_test_module();

    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::I32));
    sig.params.push(AbiParam::new(types::I32));
    sig.returns.push(AbiParam::new(types::I32));

    build_simple_function(&mut gl_module, "add", cranelift_module::Linkage::Local, sig.clone(), |builder| {
        let entry_block = builder.func.layout.entry_block().unwrap();
        let a = builder.block_params(entry_block)[0];
        let b = builder.block_params(entry_block)[1];
        let sum = builder.ins().iadd(a, b);
        builder.ins().return_(&[sum]);
        Ok(())
    }).unwrap();

    // Get original function
    let original_func = gl_module.get_func("add").unwrap();
    let original_func_clone = original_func.function.clone();

    // Format original
    let mut original_buf = String::new();
    write_function(&mut original_buf, &original_func_clone).unwrap();

    // Apply identity transform
    let transform = IdentityTransform;
    let transformed_module = gl_module.apply_transform(transform).unwrap();

    // Get transformed function
    let transformed_func = transformed_module.get_func("add").unwrap();
    let mut transformed_buf = String::new();
    write_function(&mut transformed_buf, &transformed_func.function).unwrap();

    // Normalize and compare
    let normalized_original = normalize_clif(&original_buf);
    let normalized_transformed = normalize_clif(&transformed_buf);

    assert_eq!(
        normalized_original, normalized_transformed,
        "Identity transform should produce identical CLIF\n\
         ORIGINAL:\n{}\n\n\
         TRANSFORMED:\n{}",
        original_buf, transformed_buf
    );
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_block_order() {
    // Test with multiple blocks to verify block order preservation
    // Function with blocks in non-sequential order
    // Verify blocks appear in same order after transform
    // ... (implementation)
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_block_params() {
    // Test with block parameters to verify they're preserved
    // Function with blocks that have parameters
    // Verify parameter counts and types match
    // ... (implementation)
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_stack_slots() {
    // Test with stack slots to verify they're copied
    // Function with stack slot declarations
    // Verify stack slots are copied
    // ... (implementation)
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_jump_tables() {
    // Test with BrTable instruction to verify jump tables are copied
    // Function with switch/br_table instruction
    // Verify jump table structure is preserved
    // ... (implementation)
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_multi_function() {
    // Test with multiple functions and calls between them
    // Build two functions: add() and multiply()
    // Call add() from multiply()
    // Verify FuncRef handling works correctly
    // ... (implementation)
}
```

**Key Test Cases**:

1. **Simple function**: Single block, basic instructions (`iconst`, `iadd`, `return`)
2. **Block order**: Multiple blocks, verify numeric and layout order preserved
3. **Block parameters**: Functions with block params, verify they're preserved
4. **Stack slots**: Functions with stack slots, verify they're copied
5. **Jump tables**: Functions with `BrTable`, verify jump tables are copied
6. **Multi-function**: Multiple functions with `Call`/`CallIndirect`, verify FuncRef handling
7. **Value aliases**: Functions with value aliases, verify they're preserved (if applicable)

### Step 7: Update Existing Tests

**Files**: Any tests that use `backend/util` imports

**Changes**:

- Update imports to use `backend2::transform::shared`
- Verify tests still pass

## Key Design Decisions

### 1. Shared Utilities Organization

**Decision**: Split `clif_copy.rs` into multiple focused files.

**Rationale**:

- Single-purpose files are easier to understand and maintain
- Clear separation of concerns
- Matches user preference for small files

**Trade-off**: More files to navigate, but clearer organization.

### 2. Identity Transform First

**Decision**: Implement identity transform before fixed32.

**Rationale**:

- Validates copying infrastructure works correctly
- Provides baseline for testing
- Easier to debug issues (no type conversion complexity)
- Can reuse exact same code structure for fixed32

**Trade-off**: Slight delay before fixed32, but reduces risk of bugs.

### 3. Instruction Copy Strategy

**Decision**: Use generic `copy_instruction()` for identity transform.

**Rationale**:

- Reuses proven code from `backend/util`
- Handles all instruction formats generically
- For identity transform, no special handling needed
- Other transforms can override specific instructions and fall back to this

**Trade-off**: For fixed32, we'll need instruction-specific converters, but identity can use generic copy.

### 4. Source Location Preservation

**Decision**: Copy source locations from old instructions to new instructions.

**Rationale**:

- Important for debugging and error reporting
- Old transform already does this
- Simple to implement (just call `builder.set_srcloc()`)
- Blocks don't have source locations in Cranelift

**Trade-off**: Slight overhead, but minimal and important for correctness.

### 5. InstructionCopyContext Struct

**Decision**: Bundle all parameters into a single struct.

**Rationale**:

- Cleaner API than many individual parameters
- Easier to extend in the future
- More readable function signatures

**Trade-off**: Slightly more verbose construction, but much cleaner overall.

### 6. Panic on Unsupported Instructions

**Decision**: Panic if instruction format not handled.

**Rationale**:

- Internal error - should never happen if we handle all formats
- Tests are thorough
- This is internal to our codebase
- Panic is appropriate for internal errors

**Trade-off**: Less graceful error handling, but appropriate for internal errors.

## Test Requirements

### Unit Tests

1. **Simple Function Copy**

   - Single block function with `iconst` and `return`
   - Verify CLIF output matches exactly

2. **Block Order Preservation**

   - Function with multiple blocks in non-sequential order
   - Verify blocks appear in same order after transform

3. **Block Parameters**

   - Function with blocks that have parameters
   - Verify parameter counts and types match

4. **Stack Slots**

   - Function with stack slot declarations
   - Verify stack slots are copied

5. **Jump Tables**

   - Function with `BrTable` instruction
   - Verify jump table structure is preserved

6. **Multi-Function Calls**

   - Multiple functions with `Call`/`CallIndirect` between them
   - Verify FuncRef handling works correctly

7. **Value Aliases** (if applicable)

   - Function with value aliases
   - Verify aliases are preserved

8. **Multi-Instruction Function**
   - Function with multiple instructions (`iconst`, `iadd`, `jump`, `brif`, `return`)
   - Verify all instructions copied correctly

### Integration Tests

1. **End-to-End Identity Transform**
   - Build function → Apply identity transform → Build executable → Execute
   - Verify runtime behavior matches original

## Migration Notes

### From `backend/util` to `backend2/transform/shared`

- `clif_copy.rs` → Split into `stack_slots.rs`, `blocks.rs`, `value_aliases.rs`
- `instruction_copy.rs` → `instruction_copy.rs` (enhanced with `InstructionCopyContext` and complete terminator support)
- `ir_utils/value_map.rs` → Inline `map_value` in `instruction_copy.rs` (don't import from old backend)

### Import Updates

**Old**:

```rust
use crate::backend::util::clif_copy::copy_stack_slots;
use crate::backend::util::instruction_copy::copy_instruction;
use crate::backend::ir_utils::value_map::map_value;
```

**New**:

```rust
use crate::backend2::transform::shared::copy_stack_slots;
use crate::backend2::transform::shared::copy_instruction;
// map_value is now inline in instruction_copy.rs
```

Or via re-exports:

```rust
use crate::backend2::transform::shared::{copy_stack_slots, copy_instruction};
```

## References

- Review document: `lightplayer/plans/backend2/03-transform-framework-review.md`
- Original shared utilities: `lightplayer/crates/lp-glsl/src/backend/util/clif_copy.rs`
- Original instruction copying: `lightplayer/crates/lp-glsl/src/backend/util/instruction_copy.rs`
- Transform trait definition: `lightplayer/crates/lp-glsl/src/backend2/transform/pipeline.rs`
- Fixed32 control converters: `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/converters/control.rs`
- Test patterns: `lightplayer/crates/lp-glsl/tests/transform_exact_match.rs`
