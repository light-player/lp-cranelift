---
name: lp-glsl-binary-compilation
overview: Add binary compilation support to lp-glsl that works in std mode, enabling compilation to machine code bytes for emulator execution without requiring JIT module allocation.
todos:
  - id: refactor_translation_logic
    content: Refactor translation logic to be shared between JIT and binary compilation
    status: pending
  - id: add_compile_to_code_bytes_std
    content: Add compile_to_code_bytes() method that works in std mode using Context::compile
    status: pending
  - id: integrate_with_filetests
    content: Update filetests binary compilation to use new lp-glsl method
    status: pending
  - id: test_binary_compilation
    content: Test binary compilation with sample GLSL code
    status: pending
---

# Binary Compilation Support for lp-glsl

## Overview

Add support for compiling GLSL to machine code bytes in std mode, enabling the filetests emulator execution backend to work without requiring JIT module allocation. This will allow compiling GLSL code for riscv32 (or other targets) and extracting the raw machine code bytes for emulator execution.

## Current State

### Existing Compilation Paths

1. **std mode (JIT)**: Uses `JIT` module, compiles to executable memory, returns function pointers
   - Location: `crates/lp-glsl/src/jit.rs`
   - Method: `JIT::compile()` → returns `*const u8` (function pointer)
   - Uses: `JITModule` from `cranelift-jit`

2. **no_std mode**: Compiles to code bytes using `Context::compile()` directly
   - Location: `crates/lp-glsl/src/compiler.rs` (lines 268-407)
   - Method: `Compiler::compile_to_code()` → returns `Vec<u8>`
   - Uses: `Context::compile()` with a `DummyModule` stub
   - Limitation: Only available when `feature = "std"` is disabled

### Problem

The filetests emulator execution backend needs to compile GLSL to machine code bytes for riscv32 in std mode, but:
- `compile_to_code()` is only available in no_std mode
- JIT compilation allocates executable memory and returns pointers, not bytes
- We need code bytes to combine with bootstrap code for emulator execution

## Proposed Solution

Add a new compilation method that:
1. Works in std mode
2. Uses `Context::compile()` directly (like no_std path)
3. Returns machine code bytes (`Vec<u8>`)
4. Supports arbitrary target ISA (not just native)
5. Shares translation logic with existing JIT path

## Architecture

### Option 1: Add `compile_to_code_bytes()` to std Compiler (Recommended)

Add a method to the std `Compiler` that uses `Context::compile()` directly, similar to the no_std path but without requiring a DummyModule.

**Pros:**
- Minimal changes to existing code
- Reuses existing translation logic from JIT
- Works in std mode
- Supports arbitrary ISA targets

**Cons:**
- Still needs to duplicate some translation logic (or refactor to share)

**Implementation:**
```rust
#[cfg(feature = "std")]
impl Compiler {
    /// Compile GLSL source to machine code bytes for a specific ISA
    /// Returns the compiled machine code that can be executed
    pub fn compile_to_code_bytes(
        &mut self,
        source: &str,
        isa: &dyn cranelift_codegen::isa::TargetIsa,
    ) -> Result<Vec<u8>, String> {
        // 1. Parse and analyze GLSL (reuse existing logic)
        // 2. Build Cranelift IR (reuse translation from JIT)
        // 3. Apply fixed-point transformation if needed
        // 4. Compile using Context::compile()
        // 5. Extract code buffer from CompiledCode
        // 6. Return Vec<u8>
    }
}
```

### Option 2: Refactor Translation Logic to Shared Module

Extract the translation logic (GLSL AST → Cranelift IR) into a shared module that both JIT and binary compilation can use.

**Pros:**
- Clean separation of concerns
- No code duplication
- Easier to maintain

**Cons:**
- Larger refactoring effort
- Need to carefully handle module dependencies

**Implementation:**
- Create `crates/lp-glsl/src/translation.rs` module
- Move `JIT::translate()` logic there (or refactor to be ISA-agnostic)
- Both JIT and binary compilation use shared translation

### Option 3: Use cranelift-object Directly

Use `cranelift-object` to compile to object files, then extract code bytes.

**Pros:**
- Produces proper ELF/object files
- Could enable linking multiple functions

**Cons:**
- More complex (object file format)
- Overkill for simple code extraction
- Still need to extract code bytes from object file

## Recommended Approach: Option 1 + Partial Option 2

Combine Option 1 (add method) with a partial Option 2 (extract translation helper):

1. **Extract translation helper**: Create a helper function that builds Cranelift IR from GLSL AST, usable by both JIT and binary compilation
2. **Add `compile_to_code_bytes()`**: Implement the new method using the shared translation helper and `Context::compile()`

## Implementation Plan

### 1. Refactor Translation Logic

**Files to modify:**
- `crates/lp-glsl/src/jit.rs` - Extract translation logic
- `crates/lp-glsl/src/compiler.rs` - Add new method

**Changes:**
- Create helper function `build_cranelift_ir()` that takes:
  - `typed_ast: TypedShader`
  - `source_text: &str`
  - `ctx: &mut Context`
  - `module: &mut dyn Module` (for function declarations)
  - Returns: `Result<(), GlslError>`
- Refactor `JIT::translate()` to use this helper
- The helper handles:
  - Declaring user functions
  - Compiling user functions
  - Compiling main function

**Code structure:**
```rust
// In jit.rs or new translation.rs
fn build_cranelift_ir(
    typed_ast: &TypedShader,
    source_text: &str,
    ctx: &mut Context,
    module: &mut dyn Module,
    func_ids: &mut HashMap<String, FuncId>,
) -> Result<(), GlslError> {
    // Declare all user functions
    for user_func in &typed_ast.user_functions {
        let func_id = declare_function_signature(module, ...)?;
        func_ids.insert(user_func.name.clone(), func_id);
    }
    
    // Compile all user functions
    for user_func in &typed_ast.user_functions {
        compile_function(ctx, module, user_func, func_ids, ...)?;
    }
    
    // Compile main function
    compile_main_function(ctx, module, &typed_ast.main_function, func_ids, ...)?;
    
    Ok(())
}
```

### 2. Add `compile_to_code_bytes()` Method

**Files to modify:**
- `crates/lp-glsl/src/compiler.rs` - Add new method to std Compiler

**Implementation:**
```rust
#[cfg(feature = "std")]
impl Compiler {
    /// Compile GLSL source to machine code bytes for a specific ISA
    /// 
    /// This method compiles GLSL to machine code without allocating executable
    /// memory, making it suitable for emulator execution or binary generation.
    /// 
    /// # Arguments
    /// 
    /// * `source` - GLSL source code
    /// * `isa` - Target ISA to compile for (e.g., riscv32)
    /// 
    /// # Returns
    /// 
    /// Vector of machine code bytes ready for execution
    pub fn compile_to_code_bytes(
        &mut self,
        source: &str,
        isa: &dyn cranelift_codegen::isa::TargetIsa,
    ) -> Result<Vec<u8>, crate::error::GlslError> {
        use cranelift_codegen::{Context, control::ControlPlane};
        use crate::codegen::signature::SignatureBuilder;
        
        // 1. Parse and analyze GLSL
        let semantic_result = crate::pipeline::CompilationPipeline::parse_and_analyze(source)?;
        let typed_ast = semantic_result.typed_ast;
        
        // 2. Setup Cranelift context
        let mut ctx = Context::new();
        let triple = isa.triple();
        let pointer_type = isa.pointer_type();
        let mut sig = SignatureBuilder::new_with_triple(triple);
        SignatureBuilder::add_return_type(
            &mut sig,
            &typed_ast.main_function.return_type,
            pointer_type,
        );
        ctx.func.signature = sig;
        
        // 3. Build IR using shared translation helper
        // For now, use a minimal module stub (like no_std path)
        // TODO: Refactor to use shared translation helper
        struct MinimalModule {
            isa: OwnedTargetIsa,
        }
        impl cranelift_module::Module for MinimalModule {
            // ... implement minimal module interface
        }
        
        let mut module = MinimalModule { isa: isa.clone() };
        build_cranelift_ir(&typed_ast, semantic_result.source, &mut ctx, &mut module)?;
        
        // 4. Apply fixed-point transformation if needed
        if let Some(format) = self.jit.fixed_point_format {
            crate::transform::fixed_point::convert_floats_to_fixed(&mut ctx.func, format)?;
        }
        
        // 5. Verify function
        cranelift_codegen::verify_function(&ctx.func, isa)
            .map_err(|e| crate::error::GlslError::new(
                crate::error::ErrorCode::E0301,
                format!("verification error: {}", e),
            ))?;
        
        // 6. Compile to machine code
        let compiled = ctx.compile(isa, &mut ControlPlane::default())
            .map_err(|e| crate::error::GlslError::new(
                crate::error::ErrorCode::E0400,
                format!("code generation failed: {}", e),
            ))?;
        
        // 7. Extract code buffer
        Ok(compiled.code_buffer().to_vec())
    }
}
```

### 3. Handle Module Dependencies

The translation logic needs a `Module` trait object for function declarations. Options:

**Option A: Minimal Module Stub (Like no_std)**
- Create a `DummyModule` or `MinimalModule` that implements `Module` trait
- Only implements methods needed for translation
- Unimplemented methods panic or return errors

**Option B: Use JIT Module (Temporary)**
- Create a temporary JIT module for the target ISA
- Use it for declarations, but don't allocate executable memory
- Extract code buffer before finalization

**Option C: Refactor Translation to Not Need Module**
- Make translation ISA-agnostic
- Only use module for function ID management
- Could use a simple HashMap instead

**Recommended: Option A** - Minimal module stub, similar to no_std path but works in std mode.

### 4. Update Filetests Integration

**Files to modify:**
- `crates/lp-glsl-filetests/src/execution/binary.rs`

**Changes:**
- Replace placeholder `anyhow::bail!()` with actual compilation:
```rust
pub fn compile_to_binary(
    glsl_source: &str,
    fixed_point_format: Option<FixedPointFormat>,
    bootstrap_code: &[u8],
) -> Result<Vec<u8>> {
    // Build riscv32 ISA
    let triple = Triple { ... };
    let isa = lookup(triple)?.finish(flags)?;
    
    // Compile GLSL to code bytes
    let mut compiler = lp_glsl::Compiler::new();
    compiler.set_fixed_point_format(fixed_point_format);
    let test_func_code = compiler.compile_to_code_bytes(glsl_source, isa.as_ref())
        .map_err(|e| anyhow::anyhow!("GLSL compilation failed: {}", e))?;
    
    // Combine bootstrap + test function
    let test_func_addr = bootstrap_code.len() as u32;
    let bootstrap_code = generate_bootstrap(test_func_addr, ReturnType::Float, fixed_point_format)?;
    
    let mut binary = Vec::new();
    binary.extend_from_slice(&bootstrap_code);
    
    // Align to 4-byte boundary
    while binary.len() % 4 != 0 {
        binary.push(0);
    }
    
    binary.extend_from_slice(&test_func_code);
    Ok(binary)
}
```

### 5. Testing

**Test cases:**
- Simple float return: `float main() { return 1.0; }`
- Fixed-point conversion (16.16 and 32.32)
- Functions with parameters
- User-defined functions
- Vector/matrix returns

**Test approach:**
- Compile GLSL to code bytes
- Combine with bootstrap
- Execute in riscv32 emulator
- Verify results match expected values

## Design Decisions

### Why Not Use JIT Module?

- JIT module allocates executable memory (not needed for emulator)
- JIT module is tied to native ISA (we need riscv32)
- Code extraction from JIT would require unsafe memory access

### Why Use Context::compile() Directly?

- Returns `CompiledCode` with `code_buffer()` method
- Works with any ISA (not just native)
- No executable memory allocation
- Same code path as no_std (proven to work)

### Why Share Translation Logic?

- Avoids code duplication
- Ensures consistency between JIT and binary compilation
- Easier to maintain and test
- Single source of truth for GLSL → Cranelift IR translation

## Dependencies

- `cranelift-codegen` - Already available
- `cranelift-frontend` - Already available
- No new dependencies needed

## Migration Path

1. Extract translation helper function
2. Refactor JIT to use helper
3. Add `compile_to_code_bytes()` method
4. Update filetests to use new method
5. Test with sample GLSL code
6. Remove placeholder error from filetests binary compilation

## Future Enhancements

- **Object file generation**: Use `cranelift-object` to produce proper ELF files
- **Linking support**: Link multiple GLSL functions together
- **Debug info**: Include DWARF debug information
- **Optimization levels**: Support different optimization levels for binary compilation

## Example Usage

```rust
// In filetests binary compilation
let mut compiler = lp_glsl::Compiler::new();
compiler.set_fixed_point_format(Some(FixedPointFormat::Fixed16x16));

let triple = Triple {
    architecture: Architecture::Riscv32(Riscv32Architecture::Riscv32imac),
    // ...
};
let isa = lookup(triple)?.finish(flags)?;

let code_bytes = compiler.compile_to_code_bytes(glsl_source, isa.as_ref())?;
// code_bytes is Vec<u8> ready for emulator execution
```

## Related Plans

- `.plans/lp-filetests-emulator-execution.md` - Emulator execution infrastructure (depends on this)

