# Phase 5: JIT Module Cleanup

## Current State

`jit.rs` has 474 lines with significant code duplication:

1. **Duplicate default return logic** (70+ lines duplicated between `compile_function` and `compile_main_function`)
2. **Duplicate function setup** (entry block creation, builder setup)
3. **Main function treated as special case** rather than abstraction

## Issues Identified

### Issue 1: Duplicate Default Return Generation

Both `compile_function` (lines 322-369) and `compile_main_function` (lines 429-455) have nearly identical default return logic:

```rust
// In compile_function:
if func.return_type == Type::Void {
    ctx.builder.ins().return_(&[]);
} else {
    // Generate default return value (scalar or vector)
    // ... 40+ lines of logic
}

// In compile_main_function:
if main_func.return_type == Type::Void {
    ctx.builder.ins().return_(&[]);
} else if main_func.return_type.is_vector() {
    // Generate default return value for vector
    // ... 25+ lines of logic
} else {
    // Generate default return value for scalar
    // ... 10+ lines of logic
}
```

### Issue 2: Duplicate Function Setup

Both functions create entry blocks and set up function builders:
```rust
// Duplicated in both:
let entry_block = builder.create_block();
builder.append_block_params_for_function_params(entry_block);
builder.switch_to_block(entry_block);
builder.seal_block(entry_block);
```

### Issue 3: Main Function Special-Casing

Main function is handled separately but is really just a function with:
- No parameters
- Same return type handling
- Same body translation

## Target Architecture

### Proposed Structure
```
jit.rs (refactored)
├── Function compilation abstraction
├── Default return generation (shared utility)
└── Main function compilation (uses shared abstraction)
```

## Refactoring Plan

### Step 1: Extract Default Return Generation

**Create helper function** in `jit.rs`:

```rust
impl JIT {
    /// Generate default return statement for a function
    /// Used when function doesn't have explicit return
    fn generate_default_return(
        ctx: &mut CodegenContext,
        return_type: &Type,
    ) -> Result<(), GlslError> {
        if return_type == &Type::Void {
            ctx.builder.ins().return_(&[]);
            return Ok(());
        }
        
        if return_type.is_vector() {
            Self::generate_default_vector_return(ctx, return_type)
        } else {
            Self::generate_default_scalar_return(ctx, return_type)
        }
    }
    
    fn generate_default_scalar_return(
        ctx: &mut CodegenContext,
        return_type: &Type,
    ) -> Result<(), GlslError> {
        let return_val = match return_type {
            Type::Int => ctx.builder.ins().iconst(types::I32, 0),
            Type::Float => ctx.builder.ins().f32const(0.0),
            Type::Bool => ctx.builder.ins().iconst(types::I8, 0),
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!("unsupported return type for default return: {:?}", return_type),
                ));
            }
        };
        ctx.builder.ins().return_(&[return_val]);
        Ok(())
    }
    
    fn generate_default_vector_return(
        ctx: &mut CodegenContext,
        return_type: &Type,
    ) -> Result<(), GlslError> {
        let base_ty = return_type.vector_base_type()
            .ok_or_else(|| GlslError::new(
                ErrorCode::E0400,
                format!("expected vector type, got: {:?}", return_type),
            ))?;
        let count = return_type.component_count().unwrap();
        let mut vals = Vec::new();
        
        for _ in 0..count {
            let val = match base_ty {
                Type::Float => ctx.builder.ins().f32const(0.0),
                Type::Int => ctx.builder.ins().iconst(types::I32, 0),
                Type::Bool => ctx.builder.ins().iconst(types::I8, 0),
                _ => {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!("unsupported vector base type: {:?}", base_ty),
                    ));
                }
            };
            vals.push(val);
        }
        
        ctx.builder.ins().return_(&vals);
        Ok(())
    }
}
```

**Benefits**:
- Single implementation of default return logic
- Easier to maintain and extend
- Can be tested independently

### Step 2: Extract Function Setup Logic

**Create helper function**:

```rust
impl JIT {
    /// Set up function builder with entry block
    fn setup_function_builder(
        builder: &mut FunctionBuilder,
    ) -> Block {
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        entry_block
    }
}
```

**Benefits**:
- DRY principle
- Consistent function setup
- Easier to modify setup logic

### Step 3: Refactor compile_function to Use Helpers

**Updated `compile_function`**:

```rust
fn compile_function(
    &mut self,
    func: &TypedFunction,
    func_id: FuncId,
    func_ids: &HashMap<String, FuncId>,
    func_registry: &FunctionRegistry,
    _source_text: &str,
) -> Result<(), GlslError> {
    self.ctx.clear();
    
    // Build signature
    let sig = SignatureBuilder::build(&func.return_type, &func.parameters);
    self.ctx.func.signature = sig;
    
    // Create function builder
    let mut func_builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut func_builder_context);
    
    // Set up entry block (now using helper)
    let entry_block = Self::setup_function_builder(&mut builder);
    
    // Create codegen context
    let mut ctx = CodegenContext::new(builder, &mut self.module);
    ctx.set_function_ids(func_ids);
    ctx.set_function_registry(func_registry);
    
    // Declare parameters (existing logic)
    // ... parameter handling ...
    
    // Translate function body
    for stmt in &func.body {
        ctx.translate_statement(stmt)?;
    }
    
    // Generate default return if needed (now using helper)
    Self::generate_default_return(&mut ctx, &func.return_type)?;
    
    // Finalize
    ctx.builder.finalize();
    
    // Define function in module
    self.module
        .define_function(func_id, &mut self.ctx)
        .map_err(|e| {
            GlslError::new(ErrorCode::E0400, format!("code generation failed: {}", e))
        })?;
    self.module.clear_context(&mut self.ctx);
    
    Ok(())
}
```

**Lines reduced**: ~50 lines (from 223 to ~173)

### Step 4: Refactor compile_main_function to Reuse compile_function Logic

**Option A: Make main function a special case of compile_function**

```rust
fn compile_main_function(
    &mut self,
    main_func: &TypedFunction,
    func_ids: &HashMap<String, FuncId>,
    func_registry: &FunctionRegistry,
    source_text: &str,
) -> Result<(), GlslError> {
    // Main function is just a function with no parameters
    // Create a dummy FuncId (or handle specially in module)
    
    // We can't easily reuse compile_function because:
    // 1. Main function uses different module linkage (Export vs Local)
    // 2. Main function needs source_text for error reporting
    // 3. Main function is the entry point, not a user function
    
    // So we keep it separate but use shared helpers
    self.ctx.clear();
    
    // Set up main signature (no parameters, just return type)
    let mut sig = SignatureBuilder::new();
    SignatureBuilder::add_return_type(&mut sig, &main_func.return_type);
    self.ctx.func.signature = sig;
    
    // Create function builder
    let mut main_builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut main_builder_context);
    
    // Set up entry block (using helper)
    let entry_block = Self::setup_function_builder(&mut builder);
    
    // Create codegen context
    let mut ctx = CodegenContext::new(builder, &mut self.module);
    ctx.set_function_ids(func_ids);
    ctx.set_function_registry(func_registry);
    ctx.set_source_text(source_text);
    ctx.set_return_type(main_func.return_type.clone());
    
    // Translate main function body
    for stmt in &main_func.body {
        ctx.translate_statement(stmt)?;
    }
    
    // Generate default return if needed (using helper)
    Self::generate_default_return(&mut ctx, &main_func.return_type)?;
    
    // Finalize
    ctx.builder.finalize();
    Ok(())
}
```

**Lines reduced**: ~30 lines (from 71 to ~41)

**Option B: Extract common function compilation logic**

Create a shared function that both use:

```rust
fn compile_function_impl(
    &mut self,
    func: &TypedFunction,
    parameters: &[Parameter],
    func_ids: &HashMap<String, FuncId>,
    func_registry: &FunctionRegistry,
    source_text: Option<&str>,
) -> Result<(), GlslError> {
    // Shared logic for both user functions and main
    // - Function setup
    // - Body translation
    // - Default return generation
}
```

**Benefits**:
- More code reuse
- Single place to modify function compilation logic

**Drawbacks**:
- More complex function signature
- Need to handle parameter differences

**Recommendation**: Use Option A (shared helpers) for now, consider Option B if more duplication emerges.

## Code Reduction Summary

### Before
- `compile_function`: 223 lines
- `compile_main_function`: 71 lines
- **Total**: 294 lines

### After
- `compile_function`: ~173 lines (-50)
- `compile_main_function`: ~41 lines (-30)
- Helper functions: ~80 lines (new, but reusable)
- **Net reduction**: ~30 lines + improved maintainability

## Testing Strategy

- Unit tests for `generate_default_return`
- Unit tests for `setup_function_builder`
- Integration tests for function compilation (existing tests should still pass)
- Test edge cases (void return, vector return, etc.)

## Benefits

1. **DRY**: Eliminated 70+ lines of duplication
2. **Maintainability**: Single place to modify default return logic
3. **Testability**: Helper functions can be tested independently
4. **Consistency**: Same logic used everywhere
5. **Extensibility**: Easy to add new return type handling

## Future Improvements

1. **Consider extracting function compilation to separate module**: `codegen/function.rs`
2. **Parameter handling could be extracted**: Common pattern of declaring parameters
3. **Function signature building**: Could be moved to codegen layer

