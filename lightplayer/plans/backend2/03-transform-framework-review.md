# Phase 3 Transform Framework - Review and Design Decisions

## Overview

This document captures the questions, answers, and design decisions made during the review of the Phase 3 Transform Framework plan.

## Original Plan Issues Identified

### Critical Issues

1. **Terminator Instruction Handling**: The identity transform uses `copy_instruction()` for all instructions, but terminators (`jump`, `brif`, `return`, `br_table`) need special handling because they have no results and need block mapping.

2. **Missing block_map Parameter**: `copy_instruction()` doesn't accept `block_map`, which is needed for terminator instructions.

3. **Missing Instruction Formats**: `copy_instruction()` doesn't handle `Jump`, `Brif`, `Return`, or `BrTable` formats - it returns an error for unsupported formats.

4. **Value Map Utility Dependency**: Cannot import from old `backend` - need to copy utilities to `backend2`.

### Medium Priority Issues

5. **Test Code Import**: Missing `write_function` import in test examples.

6. **Test Helper Visibility**: Need to verify `build_simple_function` is accessible.

7. **Error Code Consistency**: Using `ErrorCode::E0301` for various errors - acceptable for now.

## Design Decisions

### 1. Transform Instruction Handling Pattern

**Question**: How should transforms handle instruction conversion?

**Answer**: **Option A** - Transform-specific instruction router (like fixed32)
- Transform has a `convert_instruction()` that matches opcodes
- Routes to custom converters or falls back to generic copier
- Identity transform: no custom converters, always use fallback

**Rationale**: Simpler, matches existing pattern in fixed32 transform. No need for trait-based approach.

### 2. InstructionCopyContext Struct

**Question**: What fields should be in the context struct?

**Answer**: 
```rust
pub struct InstructionCopyContext<'a> {
    pub old_func: &'a Function,
    pub old_inst: Inst,
    pub builder: &'a mut FunctionBuilder,
    pub value_map: &'a mut HashMap<Value, Value>,
    pub stack_slot_map: Option<&'a HashMap<StackSlot, StackSlot>>,
    pub block_map: &'a HashMap<Block, Block>,
    // check_f32 removed - transform-specific, not needed for identity
    // ext_func_map: Optional - may be needed for future transforms with calls
}
```

**Decisions**:
- ✅ `check_f32` removed - transform-specific, not needed for identity transform
- ⚠️ `ext_func_map` - Not included for now, but may be needed for future transforms that handle `Call`/`CallIndirect` instructions. Can be added later if needed.

### 3. Fallback Copier Name and Location

**Question**: What should the fallback copier be called and where should it live?

**Answer**:
- **Function name**: `copy_instruction()` (or `copy_instruction_fallback()` if we want to be explicit)
- **Location**: `backend2/transform/shared/instruction_copy.rs`

**Rationale**: Keep it simple - this is the default/generic copier that handles all instructions.

### 4. Terminator Handling in Fallback

**Question**: How should the fallback handle terminators?

**Answer**: Use `builder.ins().jump()`, `builder.ins().brif()`, `builder.ins().return_()`, etc.

**Evidence**: Fixed32 transform uses this pattern:
- `builder.ins().jump(new_dest_block, &new_args)` (line 61)
- `builder.ins().brif(condition, new_then_block, &new_then_args, new_else_block, &new_else_args)` (line 195)
- `builder.ins().return_(&new_args)` (line 306)

**Rationale**: This is the idiomatic way to create terminators using FunctionBuilder.

### 5. Value Map Utility Location

**Question**: Where should `map_value` utility live?

**Answer**: **Inline it** - it's a tiny function (~3 lines), just inline it in `instruction_copy.rs`.

**Rationale**: Avoids unnecessary file proliferation for such a small utility.

### 6. Instruction Formats to Handle

**Question**: Should we handle all InstructionData variants explicitly or use a catch-all?

**Answer**: **Handle all variants explicitly** for safety and correctness.

**Rationale**: 
- More maintainable - explicit handling makes it clear what's supported
- Safer - catch-all could miss edge cases
- Matches existing pattern in `backend/util/instruction_copy.rs`

## Updated Architecture

### Shared Utilities Structure

```
backend2/transform/shared/
├── mod.rs                    # Re-exports all shared utilities
├── stack_slots.rs            # copy_stack_slots() - ~50 lines
├── blocks.rs                 # create_blocks(), ensure_block_params(), map_entry_block_params() - ~200 lines
├── value_aliases.rs          # copy_value_aliases() - ~70 lines
└── instruction_copy.rs       # copy_instruction() with InstructionCopyContext - ~400+ lines
```

### Key Changes from Original Plan

1. **InstructionCopyContext struct**: Cleaner API with all parameters in one struct
2. **Complete terminator support**: Base copier handles `Jump`, `Brif`, `Return`, `BrTable`
3. **No check_f32**: Removed from context (transform-specific)
4. **Inline map_value**: No separate file needed
5. **All instruction formats**: Explicit handling of all InstructionData variants
6. **No router in identity transform**: Directly calls `copy_instruction()` - it IS the base copier
7. **Panic on unsupported**: Internal error if instruction format not handled
8. **Per-instruction source locations only**: Blocks don't have source locations in Cranelift
9. **Jump table support**: Must handle `BrTable` with jump table copying
10. **Call/CallIndirect support**: Must handle for multi-function tests (FuncRef copied as-is)

### Identity Transform Flow

1. Create new function with same signature
2. Copy stack slots
3. Create blocks and map entry params
4. For each instruction:
   - Copy per-instruction source location (`func.srcloc(inst)`)
   - Directly call `copy_instruction()` with `InstructionCopyContext`
   - No router needed - identity transform IS the base copier
5. Seal all blocks
6. Finalize builder
7. Copy value aliases

**Key Insight**: The identity transform directly uses `copy_instruction()` for all instructions. Other transforms (like fixed32) will have custom converters that override specific instructions and fall back to `copy_instruction()` for the rest.

**Important**: Must support `Call`/`CallIndirect` for multi-function tests. FuncRef copied as-is (no mapping needed since signatures don't change).

## Remaining Questions

### 1. Identity Transform Instruction Handling

**Question**: Should the identity transform directly call `copy_instruction()` or have a router function?

**Answer**: **Directly call `copy_instruction()`** - no router function needed. This is elegant because the identity transform IS the fallback. Other transforms (like fixed32) can have custom converters that call `copy_instruction()` as their fallback.

**Rationale**: Simpler architecture - identity transform directly uses the base copier, other transforms override specific instructions and fall back to the base copier.

### 2. InstructionCopyContext Construction

**Question**: Should we provide a constructor function or construct inline?

**Answer**: **Inline construction is fine** - no constructor needed for now. Can be added later if needed.

### 3. Error Handling for Unsupported Instructions

**Question**: Should the fallback return detailed errors or panic for unsupported instructions?

**Answer**: **Panic** - it should handle everything. If it doesn't handle an instruction, that's an internal error. Tests are thorough and this is internal to our codebase.

**Rationale**: Since `copy_instruction()` should handle all instruction formats, encountering an unhandled format indicates a bug in our code, not user error. Panic is appropriate for internal errors.

### 4. Source Location Handling

**Question**: Should we copy block-level source locations in addition to per-instruction?

**Answer**: **Per-instruction only** - blocks don't have source locations in Cranelift. Source locations are per-instruction (`func.srcloc(inst)`), so we only need to copy per-instruction source locations.

**Rationale**: Cranelift doesn't have block-level source locations, so no need to handle them.

### 5. Jump Tables for BrTable

**Question**: How should the fallback handle `BrTable` instructions with jump tables?

**Answer**: **Handle the same way as fixed32** - copy jump table data structure:
- Read old jump table from `old_func.dfg.jump_tables[*table]`
- Map all block calls in the table (map blocks and values)
- Create new jump table with `builder.create_jump_table(JumpTableData::new(...))`
- Emit `builder.ins().br_table(condition, new_table)`

**Rationale**: Jump tables are data structures that need to be copied, not just instructions.

### 6. Call/CallIndirect Instructions

**Question**: Should the fallback handle `Call`/`CallIndirect` instructions?

**Answer**: **Yes, must support them** - important to have at least one unit test with multiple functions. For identity transform, copy FuncRef as-is (no mapping needed since signatures don't change).

**Rationale**: Even though identity transform is i32-only, we need to support calls for multi-function tests and future use.

### 7. ext_func_map for Future Transforms

**Question**: Should `ext_func_map` be in `InstructionCopyContext` now, or added later?

**Status**: Not needed for identity transform (FuncRef copied as-is). Can be added later for transforms that need FuncRef mapping.

### 8. Function Name for Base Copier

**Question**: `copy_instruction()` or `copy_instruction_fallback()`?

**Answer**: **`copy_instruction()`** - it's the base/default copier. The identity transform directly calls it, and other transforms use it as their fallback.

### 9. Other Instruction Formats

**Question**: Are there any other special instruction formats beyond terminators, stack ops, memory ops, and calls?

**Answer**: **Not that we know of** - handle all `InstructionData` variants explicitly. Panic if we encounter something unexpected (internal error).

## Implementation Checklist

- [x] Review original plan
- [x] Identify critical issues
- [x] Make design decisions
- [x] Document decisions
- [ ] Update plan document with new architecture
- [ ] Implement shared utilities
- [ ] Implement identity transform
- [ ] Write tests

## References

- Original plan: `lightplayer/plans/backend2/03-transform-framework.md`
- Fixed32 transform pattern: `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/`
- Existing instruction copy: `lightplayer/crates/lp-glsl/src/backend/util/instruction_copy.rs`
- Control flow converters: `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/converters/control.rs`

