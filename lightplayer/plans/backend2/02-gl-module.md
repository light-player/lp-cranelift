# GlModule Architecture

## Overview

Replace the current `ClifModule` + linking approach with a new `GlModule` architecture that builds functions directly into the target Module, eliminating the need for a separate linking step.

## Current Problems

1. **Multiple rebuilds**: Functions are rebuilt 2-3 times:
   - Initial compilation (GLSL → Function IR with temp Module)
   - Fixed32 transform (rebuild with type conversion)
   - Linking (rebuild to remap FuncRefs to final Module)
2. **FuncRef remapping complexity**: Moving functions between Modules requires complex FuncRef remapping logic
3. **Block params bugs**: The copying/rebuilding process introduces bugs (e.g., block parameters not preserved)
4. **Architecture mismatch**: Doesn't align with Cranelift's intended pattern of building functions directly in the target Module

## Proposed Solution

### Core Structures

```rust
// Module specification - encodes what Module to build
pub struct ModuleSpec {
    pub kind: ModuleKind,
    pub triple: target_lexicon::Triple,
    pub flags: Flags, // ISA flags
}

pub enum ModuleKind {
    Jit,
    Object { name: String },
}

// Function metadata (doesn't store Function IR, just metadata)
pub struct GlFunc {
    pub name: String,
    pub glsl_sig: FunctionSignature, // GLSL-level signature
    pub clif_sig: Signature,         // Cranelift signature
    pub func_id: FuncId,              // In the Module
}

// Main module structure - owns the actual Cranelift Module
pub struct GlModule<M: Module> {
    pub spec: ModuleSpec,
    pub source_map: GlSourceMap,
    pub fns: HashMap<String, GlFunc>,
    pub module: M, // Owned Module - functions are already defined here
}
```

### Key Design Decisions

1. **GlModule owns the Module**: Functions are built directly into the Module during compilation, not copied later
2. **No linking step**: Since functions are built in the final Module, no FuncRef remapping is needed
3. **Separate transform functions**: `from_glsl` and `transform_fixed32` are NOT methods on GlModule - they are separate functions that create/transform GlModules
4. **Generic over Module type**: `GlModule<M: Module>` allows JITModule, ObjectModule, etc.

### Compilation Flow

**Current**:
```
GLSL → ClifModule → (optional transform) → link → JIT/Object
```

**Proposed**:
```
GLSL + ModuleSpec → GlModule → (optional transform) → Done
```

### ModuleSpec to Module Builder

```rust
fn create_module_builder(spec: &ModuleSpec) -> Result<ModuleBuilder, GlslError> {
    match spec.kind {
        ModuleKind::Jit => {
            let isa = create_isa_from_spec(spec)?;
            Ok(JITBuilder::with_isa(isa, default_libcall_names()))
        }
        ModuleKind::Object { name } => {
            let isa = create_isa_from_spec(spec)?;
            Ok(ObjectBuilder::new(isa, name, default_libcall_names())?)
        }
    }
}

fn create_isa_from_spec(spec: &ModuleSpec) -> Result<OwnedTargetIsa, GlslError> {
    // Build ISA from triple and flags
    let builder = isa::lookup_by_triple(spec.triple)?;
    builder.finish(spec.flags.clone())
}
```

### Compilation Functions (NOT methods on GlModule)

```rust
// Compile GLSL source into a GlModule
pub fn compile_glsl_to_module<M: Module>(
    source: &str,
    spec: ModuleSpec,
) -> Result<GlModule<M>, GlslError> {
    // 1. Parse and analyze GLSL
    // 2. Create Module from spec
    // 3. Build functions directly in Module (no temp Module needed!)
    // 4. Return GlModule
}

// Transform GlModule for fixed32 (creates new GlModule)
pub fn transform_fixed32<M: Module>(
    module: &GlModule<M>,
    format: FixedPointFormat,
) -> Result<GlModule<M>, GlslError> {
    // 1. Create new GlModule with same spec
    // 2. Rebuild functions with type conversion
    // 3. This rebuild is unavoidable (type changes)
    // 4. But FuncRefs are already correct (same Module type)
}
```

### Benefits

1. **Eliminates linking step**: Functions built directly in final Module
2. **Reduces rebuilds**: Only one rebuild for fixed32 (unavoidable due to type changes)
3. **Aligns with Cranelift pattern**: Build functions in the Module you'll use
4. **Clearer architecture**: Explicit target upfront (JIT vs Object)
5. **Fewer bugs**: Less copying/rebuilding = fewer opportunities for errors

### Trade-offs

1. **Less flexibility**: Can't compile once and use multiple backends (probably fine for GLSL)
2. **ModuleSpec upfront**: Caller must know target upfront (usually true anyway)
3. **Transform still rebuilds**: Fixed32 transform still needs to rebuild functions (necessary due to type changes)

### Implementation Notes

- `GlSourceMap`: TBD - likely source location information for debugging/error reporting
- `GlFunc`: Stores metadata only, not Function IR (functions are in the Module)
- Module extraction: If needed, can add `into_module()` to extract components
- Finalization: For JIT, need to finalize and extract function pointers (separate function)

### Migration Path

1. Implement `ModuleSpec` and `create_module_builder`
2. Implement `GlModule` structure
3. Implement `compile_glsl_to_module` (replaces `compile_to_clif_module`)
4. Implement `transform_fixed32` for GlModule (replaces current transform)
5. Update callers to use new API
6. Remove old `ClifModule` + linking code


