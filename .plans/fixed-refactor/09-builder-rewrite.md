# Fixed Point Converter: Builder-Based Rewrite Plan

## Overview

This plan describes a complete rewrite of the fixed-point converter using a **builder-based approach** instead of in-place mutation. Rather than mutating the existing function, we will create a new function from scratch using `FunctionBuilder`, converting instructions as we traverse the original function.

## Problem Statement

The current in-place mutation approach has proven difficult to get right:

1. **Result type mismatches**: When replacing instructions, result types don't always update correctly
2. **Signature modification issues**: Modifying function signatures doesn't always reflect in call instructions
3. **Complex value mapping**: Managing `value_map` while mutating in place is error-prone
4. **Detach/reattach complexity**: `detach_inst_results()` and `replace()` interactions are subtle
5. **Verification challenges**: Checking F32 values requires understanding both old and new instruction states

## Solution: Builder-Based Rewrite

Instead of mutating the function in place, we will:

1. **Create a new function** with the converted signature
2. **Traverse the old function** block-by-block, instruction-by-instruction
3. **Convert each instruction** and emit the fixed-point equivalent using `FunctionBuilder`
4. **Map values** as we go (old value → new value)
5. **Replace the old function** with the new one

This approach:

- ✅ Avoids result type mismatches (new function has correct types from the start)
- ✅ Avoids signature modification issues (signature is correct from creation)
- ✅ Simplifies value mapping (clear old→new mapping)
- ✅ Eliminates detach/reattach complexity (no in-place mutation)
- ✅ Makes verification straightforward (only new function exists)

## Architecture

### High-Level Flow

```
1. Create new function with converted signature
2. Create FunctionBuilder for new function
3. Build block map (old_block → new_block)
4. Build value map (old_value → new_value)
5. Traverse old function:
   - For each block: create corresponding new block
   - For each instruction: convert and emit via builder
   - Map all values (operands, results, branch args)
6. Replace old function with new function
```

### Key Data Structures

```rust
struct RewriteContext {
    old_func: &Function,           // Read-only reference to original
    new_func: Function,             // New function being built
    builder: FunctionBuilder,      // Builder for new function
    builder_ctx: FunctionBuilderContext,
    block_map: HashMap<Block, Block>,  // old_block → new_block
    value_map: HashMap<Value, Value>,  // old_value → new_value
    format: FixedPointFormat,
}
```

### Instruction Conversion Pattern

For each instruction in the old function:

1. **Map operands**: `old_operand → value_map.get(old_operand) → new_operand`
2. **Convert instruction**: Determine fixed-point equivalent
3. **Emit via builder**: `builder.ins().<converted_op>(new_operands...)`
4. **Map results**: `old_result → new_result` in value_map
5. **Handle control flow**: Map branch targets through block_map

## Implementation Phases

### Phase 1: Core Rewrite Infrastructure

Create the foundational rewrite infrastructure:

- [ ] `RewriteContext` struct
- [ ] `rewrite_function` entry point
- [ ] Block creation and mapping
- [ ] Basic instruction traversal
- [ ] Value mapping infrastructure

**Files to create/modify:**

- `crates/lp-glsl/src/transform/fixed_point/rewrite.rs` (new)
- `crates/lp-glsl/src/transform/fixed_point/transform.rs` (modify entry point)

### Phase 2: Signature and Block Parameters

Handle function signature and block parameters:

- [ ] Convert function signature (F32 → I32/I64)
- [ ] Create new function with converted signature
- [ ] Map function parameters
- [ ] Create blocks and map block parameters
- [ ] Handle entry block specially

**Key functions:**

- `convert_signature()` - Convert signature types
- `create_new_function()` - Create function with converted signature
- `map_block_params()` - Convert and map block parameters

### Phase 3: Basic Instruction Conversion

Convert simple instructions (constants, arithmetic):

- [ ] F32const → iconst (fixed-point value)
- [ ] Fadd/Fsub/Fmul/Fdiv → fixed-point arithmetic
- [ ] Fneg → ineg
- [ ] Fabs → iabs (with sign handling)

**Pattern:**

```rust
fn convert_instruction(
    ctx: &mut RewriteContext,
    old_inst: Inst,
    old_block: Block,
) -> Result<(), GlslError> {
    let opcode = ctx.old_func.dfg.insts[old_inst].opcode();
    let new_block = ctx.block_map[&old_block];
    ctx.builder.switch_to_block(new_block);

    match opcode {
        Opcode::F32const => {
            let old_result = ctx.old_func.dfg.first_result(old_inst);
            let f32_value = extract_f32_constant(ctx.old_func, old_inst);
            let fixed_value = convert_to_fixed(f32_value, ctx.format);
            let new_result = ctx.builder.ins().iconst(target_type, fixed_value);
            ctx.value_map.insert(old_result, new_result);
        }
        // ... other opcodes
    }
    Ok(())
}
```

### Phase 4: Control Flow Instructions

Handle branches, jumps, and control flow:

- [ ] Jump → map target block
- [ ] Brif → map condition and target blocks
- [ ] BrTable → map table and targets
- [ ] Return → map return values
- [ ] Select → convert condition and values

**Key challenge:** Mapping branch arguments through value_map

### Phase 5: Memory Operations

Convert load and store instructions:

- [ ] Load F32 → Load I32/I64 (with type conversion)
- [ ] Store F32 → Store I32/I64 (with type conversion)
- [ ] Map address operands
- [ ] Handle memory flags

### Phase 6: Function Calls

Handle call instructions:

- [ ] Map call arguments through value_map
- [ ] Convert external function signatures
- [ ] Handle return values
- [ ] Support both direct and indirect calls

**Key challenge:** External function signature conversion

### Phase 7: Complex Instructions

Handle remaining instruction types:

- [ ] Comparison instructions (Fcmp → Icmp)
- [ ] Math functions (sqrt, ceil, floor)
- [ ] Type conversions (FcvtFromSint, FcvtFromUint)
- [ ] Bitcast (if needed)
- [ ] Exception handling (try_call, etc.)

### Phase 8: Verification and Testing

Add verification and comprehensive testing:

- [ ] Verify new function has no F32 values
- [ ] Verify function is valid Cranelift IR
- [ ] Unit tests for each instruction type
- [ ] Integration tests with filetests
- [ ] Performance comparison

### Phase 9: Migration and Cleanup

Migrate to new implementation:

- [ ] Replace `convert_floats_to_fixed` to use rewrite approach
- [ ] Remove old in-place mutation code
- [ ] Update all tests
- [ ] Verify no regressions
- [ ] Clean up unused code

## Detailed Implementation Notes

### Block Mapping Strategy

```rust
// Create blocks in same order as original
let mut block_map = HashMap::new();
for old_block in old_func.layout.blocks() {
    let new_block = builder.create_block();
    block_map.insert(old_block, new_block);

    // Map block parameters
    let old_params = old_func.dfg.block_params(old_block);
    let mut new_param_types = Vec::new();
    for &old_param in old_params {
        let old_type = old_func.dfg.value_type(old_param);
        let new_type = if old_type == types::F32 {
            format.cranelift_type()
        } else {
            old_type
        };
        new_param_types.push(new_type);
    }

    // Declare block parameters
    for &param_type in &new_param_types {
        builder.append_block_param(new_block, param_type);
    }

    // Map old params to new params
    let new_params = builder.block_params(new_block);
    for (old_param, new_param) in old_params.iter().zip(new_params.iter()) {
        value_map.insert(*old_param, *new_param);
    }
}
```

### Instruction Conversion Helper

```rust
fn map_operand(ctx: &RewriteContext, old_value: Value) -> Value {
    *ctx.value_map.get(&old_value).unwrap_or(&old_value)
}

fn convert_and_emit_instruction(
    ctx: &mut RewriteContext,
    old_inst: Inst,
    old_block: Block,
) -> Result<(), GlslError> {
    let new_block = ctx.block_map[&old_block];
    ctx.builder.switch_to_block(new_block);

    // Copy source location
    if let Some(srcloc) = ctx.old_func.srclocs.get(old_inst) {
        ctx.builder.set_srcloc(*srcloc);
    }

    let opcode = ctx.old_func.dfg.insts[old_inst].opcode();

    match opcode {
        Opcode::F32const => convert_f32const(ctx, old_inst)?,
        Opcode::Fadd => convert_fadd(ctx, old_inst)?,
        // ... etc
        _ => {
            // For non-F32 instructions, copy as-is
            copy_instruction(ctx, old_inst)?;
        }
    }

    Ok(())
}
```

### Handling Non-F32 Instructions

For instructions that don't involve F32, we can copy them directly:

```rust
fn copy_instruction(
    ctx: &mut RewriteContext,
    old_inst: Inst,
) -> Result<(), GlslError> {
    // Map operands
    let old_operands: Vec<Value> = ctx.old_func.dfg.inst_args(old_inst)
        .iter()
        .map(|&v| map_operand(ctx, v))
        .collect();

    // Emit same instruction with mapped operands
    // (This requires instruction-specific handling)

    // Map results
    let old_results: Vec<Value> = ctx.old_func.dfg.inst_results(old_inst)
        .iter()
        .copied()
        .collect();
    let new_results: Vec<Value> = /* get from builder */;

    for (old_result, new_result) in old_results.iter().zip(new_results.iter()) {
        ctx.value_map.insert(*old_result, *new_result);
    }

    Ok(())
}
```

## Key Advantages

1. **Type Safety**: New function has correct types from the start
2. **Simplicity**: No need to manage detach/reattach
3. **Clarity**: Clear separation between old and new function
4. **Debuggability**: Can inspect both old and new functions
5. **Correctness**: Less error-prone than in-place mutation
6. **Maintainability**: Easier to understand and modify

## Challenges and Solutions

### Challenge 1: Instruction Copying

**Problem**: Copying non-F32 instructions requires instruction-specific handling.

**Solution**: Use pattern matching on `InstructionData` to reconstruct instructions, or use a helper that handles common cases.

### Challenge 2: External Function Signatures

**Problem**: External function signatures need to be converted.

**Solution**: Convert signatures when creating external function references in the new function.

### Challenge 3: Jump Tables

**Problem**: Jump tables need to be recreated with mapped block references.

**Solution**: Create new jump tables and map entries through block_map.

### Challenge 4: Exception Tables

**Problem**: Exception tables reference blocks and values that need mapping.

**Solution**: Recreate exception tables with mapped blocks and values.

### Challenge 5: Performance

**Problem**: Creating a new function might be slower than in-place mutation.

**Solution**: Profile and optimize hot paths. The correctness benefits likely outweigh minor performance costs.

## Testing Strategy

1. **Unit Tests**: Test each instruction converter in isolation
2. **Integration Tests**: Test complete function conversion
3. **Filetests**: Run full filetest suite to catch regressions
4. **Comparison Tests**: Compare output of old vs new approach (where possible)

## Migration Path

1. Implement new rewrite approach alongside old approach
2. Add feature flag to switch between approaches
3. Test new approach thoroughly
4. Switch default to new approach
5. Remove old approach after validation period

## Success Criteria

- [ ] All instruction types converted correctly
- [ ] No F32 values in output function
- [ ] All filetests pass
- [ ] Performance is acceptable (within 2x of old approach)
- [ ] Code is cleaner and more maintainable
- [ ] No regressions in functionality

## Files Structure

```
crates/lp-glsl/src/transform/fixed_point/
├── mod.rs                    # Module exports
├── types.rs                  # FixedPointFormat (unchanged)
├── transform.rs              # Entry point (modified)
├── rewrite.rs                # NEW: Builder-based rewrite
├── converters/
│   ├── constants.rs         # F32const conversion
│   ├── arithmetic.rs         # Arithmetic operations
│   ├── comparison.rs         # Comparison operations
│   ├── memory.rs             # Load/Store
│   ├── calls.rs              # Function calls
│   ├── control.rs            # Control flow
│   ├── math.rs               # Math functions
│   └── conversions.rs        # Type conversions
└── transform_test.rs         # Unit tests
```

## Next Steps

1. Review and approve this plan
2. Start Phase 1: Core rewrite infrastructure
3. Implement incrementally, testing as we go
4. Migrate when ready
