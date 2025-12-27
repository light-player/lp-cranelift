# Phase 2 Pre-Transform Research: Function Storage and Transformation Architecture

## Research Questions and Findings

### 1. Function Extraction from Module

**Question**: Does Cranelift's `Module` trait allow extracting `Function` objects?

**Finding**: **No direct extraction API**, but Functions remain accessible in Context after `define_function()`.

**Key Insight**: The `Module` trait provides:

- `declare_function()` - Declare a function signature (metadata only)
- `define_function()` - Compile a function body from Context
- `make_context()` / `clear_context()` - Context management

**Important Discovery**: After `define_function()` returns, the Function IR is still accessible in `ctx.func` until `clear_context()` is called. However, this is a timing issue - by the time you'd want to transform, you've likely already cleared the context.

**Implication**: We cannot reliably extract Functions from `GlModule<M: Module>` after they've been added to the Module. We need to store Functions separately.

### 2. Declaration vs Definition Pattern

**Key Discovery**: Cranelift supports a two-phase pattern:

1. **Declaration** (`declare_function()`): Stores function signature in Module, returns `FuncId`. No compilation happens.
2. **Reference** (`declare_func_in_func()`): Creates a `FuncRef` for calling a declared function. Only requires declaration, not definition.
3. **Definition** (`define_function()`): Compiles the function body. This is when actual code generation happens.

**Implication**: We can declare functions, build IR that references them, and define them later. This enables:

- Forward declarations
- Cross-function calls during IR building
- Transformation before compilation

### 3. How ClifModule Stores Functions

**Finding**: `ClifModule` stores Functions **directly** as owned values:

```rust
pub struct ClifModule {
    user_functions: HashMap<String, Function>,  // Functions stored here
    main_function: Function,                    // Main function stored here
    // ... other fields
}
```

**Key Points**:

- Functions are built separately using `FunctionBuilder`
- Functions are stored in `ClifModule` **before** being added to a Module
- When linking, Functions are extracted from `ClifModule` and added to the target Module
- This allows transformation: Functions can be modified before being added to Module

### 4. Frontend Codegen Flow

**Finding**: The frontend builds Functions separately:

1. **Function Building**: `compile_function_to_clif()` creates a `Function` using `FunctionBuilder`
2. **Storage**: Functions are added to `ClifModule` via `add_user_function()` or `set_main_function()`
3. **Linking**: Later, `link_into()` extracts Functions from `ClifModule` and adds them to the target Module

**Flow**:

```
GLSL → FunctionBuilder → Function → ClifModule → (transform) → Module
```

**Key Insight**: Functions exist as separate objects **before** being added to Module. This enables transformation.

## Proposed Architecture

### Core Design Principles

1. **GlModule stores Functions separately**: Functions are stored in `GlFunc`, not extracted from Module
2. **Declaration-only in Module**: Functions are declared (not defined) in Module during building
3. **Definition at executable build time**: Functions are defined when building executables
4. **Encapsulation**: Module is private, accessed only through builder methods
5. **Transformation is pure**: Transform consumes GlModule, produces new GlModule

### Architecture Flow

```
Build Functions → (optional Transform) → Build Executable
     ↓                    ↓                    ↓
  Declare only      Transform IR         Define + Compile
  Store Function    Create new Module    Extract pointers
```

### Proposed Structure

```rust
pub struct GlModule<M: Module> {
    target: Target,
    fns: HashMap<String, GlFunc>,
    module: M,  // PRIVATE - only accessible via internal methods
}

pub struct GlFunc {
    pub name: String,
    pub clif_sig: Signature,
    pub func_id: FuncId,
    pub function: Function,  // Function IR stored here
}
```

### Public API

```rust
impl<M: Module> GlModule<M> {
    /// Add a function to this module
    ///
    /// Declares the function in the Module and stores the Function IR.
    /// The function is NOT compiled yet - that happens in build_executable().
    ///
    /// Validates that the Function signature matches the provided signature.
    pub fn add_function(
        &mut self,
        name: &str,
        linkage: Linkage,
        sig: Signature,
        func: Function,
    ) -> Result<FuncId, GlslError> {
        // Validate signature matches
        // Declare in Module
        // Store Function IR
    }

    /// Declare a function without providing the body yet (forward declaration)
    ///
    /// Useful for cross-function calls where the callee is defined later.
    pub fn declare_function(
        &mut self,
        name: &str,
        linkage: Linkage,
        sig: Signature,
    ) -> Result<FuncId, GlslError> {
        // Declare in Module
        // Store placeholder (or use Option<Function>)
    }

    /// Get function metadata by name
    pub fn get_func(&self, name: &str) -> Option<&GlFunc>;

    /// Apply a transform to all functions in this module
    ///
    /// Consumes this GlModule and produces a new GlModule with transformed functions.
    /// Neither module has functions defined (compiled) yet.
    pub fn apply_transform<T: Transform>(
        self,
        transform: T,
    ) -> Result<GlModule<M>, GlslError>;

    /// Internal: Get mutable access to Module
    ///
    /// **WARNING**: This is internal-only. Do not use outside of GlModule implementation.
    /// The Module should only be accessed through public builder methods.
    #[doc(hidden)]
    pub(crate) fn module_mut_internal(&mut self) -> &mut M;

    /// Internal: Get immutable access to Module (for codegen)
    #[doc(hidden)]
    pub(crate) fn module_internal(&self) -> &M;
}
```

### Codegen Functions

```rust
/// Build JIT executable from GlModule<JITModule>
///
/// Consumes the GlModule. All functions are defined (compiled) during this process.
pub fn build_jit_executable(
    mut gl_module: GlModule<JITModule>,
) -> Result<GlslJitModule, GlslError> {
    // 1. Define all functions (compile them)
    for (name, gl_func) in &gl_module.fns {
        let mut ctx = gl_module.module_mut_internal().make_context();
        ctx.func = gl_func.function.clone(); // Or move if we change ownership
        gl_module.module_mut_internal().define_function(gl_func.func_id, &mut ctx)?;
        gl_module.module_mut_internal().clear_context(&mut ctx);
    }

    // 2. Finalize definitions
    gl_module.module_mut_internal().finalize_definitions()?;

    // 3. Extract function pointers
    // 4. Build GlslJitModule
}
```

### Transformation Flow

```rust
impl<M: Module> GlModule<M> {
    pub fn apply_transform<T: Transform>(
        self,
        transform: T,
    ) -> Result<GlModule<M>, GlslError> {
        // 1. Create new GlModule with same target
        let mut new_module = GlModule::new_with_target(self.target)?;

        // 2. Transform all function signatures and create FuncRef mappings
        let mut func_ref_map = HashMap::new();
        for (name, gl_func) in &self.fns {
            let new_sig = transform.transform_signature(&gl_func.clif_sig);
            let func_id = new_module.module_mut_internal()
                .declare_function(name, linkage, &new_sig)?;
            // Create FuncRef for cross-function calls
            let mut temp_func = Function::new();
            temp_func.signature = new_sig.clone();
            let func_ref = new_module.module_mut_internal()
                .declare_func_in_func(func_id, &mut temp_func);
            func_ref_map.insert(name.clone(), func_ref);
        }

        // 3. Transform function bodies
        for (name, gl_func) in self.fns {
            let transformed_func = transform.transform_function(
                &gl_func.function,
                new_module.module_mut_internal(),
                &func_ref_map,
            )?;

            // Use public API to add transformed function
            let new_sig = transform.transform_signature(&gl_func.clif_sig);
            new_module.add_function(name, linkage, new_sig, transformed_func)?;
        }

        Ok(new_module)
    }
}
```

## Test Utilities

### Current State

The following functions are currently public but only used in tests:

- `build_simple_function()` - Used in builder.rs, codegen/jit.rs, and codegen/emu.rs tests
- `build_call_function()` - Used in builder.rs tests
- `declare_function()` - Used in builder.rs tests

### Proposed Solution

**Move test utilities to a separate module**:

```rust
// backend2/module/test_helpers.rs
#[cfg(test)]
pub mod test_helpers {
    use super::*;

    /// Test helper: Build a simple function programmatically
    ///
    /// For production code, build Functions separately and use GlModule::add_function()
    pub fn build_simple_function<M: Module>(
        gl_module: &mut GlModule<M>,
        name: &str,
        linkage: Linkage,
        sig: Signature,
        body: impl FnOnce(&mut FunctionBuilder) -> Result<(), GlslError>,
    ) -> Result<FuncId, GlslError> {
        // Build Function IR
        // Use gl_module.add_function() to add it
    }

    // Similar for build_call_function and declare_function
}
```

**Usage in tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend2::module::test_helpers::build_simple_function;
    // ...
}
```

## Implementation Plan

### Step 1: Update GlFunc to Store Function

**File**: `lightplayer/crates/lp-glsl/src/backend2/module/gl_func.rs`

**Changes**:

- Add `function: Function` field to `GlFunc`
- Update constructor to require Function
- Update tests

### Step 2: Make Module Private and Add Builder Methods

**File**: `lightplayer/crates/lp-glsl/src/backend2/module/gl_module.rs`

**Changes**:

- Make `module` field private
- Add `add_function()` method (validates signature, declares, stores Function)
- Add `declare_function()` method (forward declarations)
- Add `module_mut_internal()` and `module_internal()` for internal access
- Update constructors if needed

### Step 3: Update Function Building

**File**: `lightplayer/crates/lp-glsl/src/backend2/module/builder.rs` → Move to `test_helpers.rs`

**Changes**:

- Move `build_simple_function`, `build_call_function`, `declare_function` to test helpers
- Update them to use `GlModule::add_function()` instead of direct Module access
- Update all test imports

### Step 4: Update Codegen Functions

**Files**:

- `lightplayer/crates/lp-glsl/src/backend2/codegen/jit.rs`
- `lightplayer/crates/lp-glsl/src/backend2/codegen/emu.rs`

**Changes**:

- Update `build_jit_executable()` to define all functions before finalizing
- Update `build_emu_executable()` similarly
- Use `module_mut_internal()` for Module access
- Update tests to use test helpers

### Step 5: Create Transformer Trait

**File**: `lightplayer/crates/lp-glsl/src/backend2/transform/pipeline.rs`

**Changes**:

- Define `Transform` trait with `transform_signature()` and `transform_function()`
- Define `TransformContext` struct for shared transform state (FuncRef maps, etc.)
- Add helper methods for common transform operations

### Step 6: Implement Fixed32 Transform

**File**: `lightplayer/crates/lp-glsl/src/backend2/transform/fixed32/mod.rs`

**Changes**:

- Implement `Transform` trait for Fixed32
- Reuse existing fixed32 transform code from `backend/transform/fixed32/`
- Adapt to work with new `GlModule` API

### Step 7: Add Module Transformation API

**File**: `lightplayer/crates/lp-glsl/src/backend2/module/gl_module.rs`

**Changes**:

- Add `apply_transform<T: Transform>()` method
- Handle FuncRef remapping for cross-function calls
- Create new GlModule with transformed functions

### Step 8: Unit Tests

**Files**:

- `lightplayer/crates/lp-glsl/src/backend2/transform/pipeline.rs` (trait tests)
- `lightplayer/crates/lp-glsl/src/backend2/transform/fixed32/mod.rs` (fixed32 tests)
- `lightplayer/crates/lp-glsl/tests/backend2_transform.rs` (integration tests)

**Test Requirements**:

1. **CLIF Structure Preservation**: Adapt `transform_fixed32.rs` test to verify:

   - Block parameters preserved
   - SSA form maintained
   - Block order preserved
   - Value numbering consistent

2. **End-to-End Sanity Test**:
   - Build simple float function (e.g., `f32 add(f32 a, f32 b) -> a + b`)
   - Apply fixed32 transform
   - Build executable (both JIT and emulator)
   - Call function and verify results

## Key Design Decisions

### 1. Function Storage

**Decision**: Store Functions in `GlFunc` as owned values.

**Rationale**:

- Enables transformation before compilation
- Matches proven `ClifModule` pattern
- Functions are needed at IR stage anyway

**Trade-off**: More memory usage, but temporary and acceptable.

### 2. Declaration-Only Pattern

**Decision**: Only declare functions in Module during building, define them at executable build time.

**Rationale**:

- Enables transformation before compilation
- Clear separation: IR stage vs compilation stage
- Matches Cranelift's natural pattern

### 3. Module Encapsulation

**Decision**: Make Module private, access only through builder methods.

**Rationale**:

- Type safety: Builder methods ensure signature consistency
- Encapsulation: Module is implementation detail
- Clear API boundaries
- Easier maintenance

### 4. Test Utilities Separation

**Decision**: Move test helpers to separate module.

**Rationale**:

- Clear separation: test utilities vs production API
- No risk of accidental production use
- Easier to find test helpers
- Matches Rust conventions

## Open Questions Resolved

1. **Function Extraction Timing**: Extract Function from Context before `define_function()` or store separately?

   - **Answer**: Store separately in `GlFunc`. This is cleaner and enables transformation.

2. **Module Cleanup**: Can we remove functions from a Module if transformation fails partway?

   - **Answer**: No - Cranelift doesn't provide removal API. We create a new Module.

3. **FuncRef Handling**: How do we handle FuncRefs during transformation?

   - **Answer**: Create new FuncRefs in the new Module, build a mapping from old names to new FuncRefs, update call instructions during transformation.

4. **Linkage Preservation**: How do we preserve function linkage (Export vs Local)?

   - **Answer**: Store linkage in `GlFunc` or pass it through the transform context.

5. **Source Location Preservation**: How do we preserve source locations during transformation?
   - **Answer**: The existing fixed32 transform already handles this via `builder.set_srcloc()`. We can reuse this pattern.

## References

- `lightplayer/crates/lp-glsl/src/backend/ir/clif_module.rs` - ClifModule structure
- `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/module.rs` - Existing fixed32 transform
- `lightplayer/crates/lp-glsl/src/frontend/glsl_compiler.rs` - Frontend function building
- `lightplayer/crates/lp-glsl/tests/transform_fixed32.rs` - Existing transform tests
- `cranelift/module/src/module.rs` - Module trait documentation
