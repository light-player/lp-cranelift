# Backend2

A rewrite of the glsl compiler backend structured around a new `GlModule` architecture.

It will go into `lightplayer/crates/lp-glsl/src/backend2` and completely replace `lightplayer/crates/lp-glsl/src/backend`.

## Directory Structure

```
backend2/
├── mod.rs
├── target/              # Target architecture and codegen options
│   ├── mod.rs
│   ├── target.rs        # Target enum: Rv32Emu, HostJit (semantic targets)
│   └── builder.rs       # create_module_builder, create_isa (internal details)
├── module/              # GLSL Module (wraps Cranelift Module)
│   ├── mod.rs
│   ├── gl_module.rs     # GlModule<M: Module>
│   └── gl_func.rs       # GlFunc metadata
├── transform/           # Transform pipeline
│   ├── mod.rs
│   ├── pipeline.rs      # Transform trait, pipeline
│   ├── utils/           # Shared transform utilities
│   └── fixed32/         # Fixed32 implementation
└── codegen/             # Code generation (Module → Executable)
    ├── mod.rs
    ├── jit.rs           # build_jit_executable(GlModule<JITModule>)
    └── emu.rs           # build_emu_executable(GlModule<ObjectModule>)
```

## Terminology

- **Target**: Semantic target enum (Rv32Emu, HostJit) - hides implementation details
- **GlModule**: GLSL compilation unit (one shader source compiled to a Module)
- **codegen/**: Code generation layer (Module → Executable)

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
// Semantic target enum - caller doesn't need to know implementation details
pub enum Target {
    /// RISC-V 32-bit emulator target
    Rv32Emu {
        flags: Flags,
    },
    /// Host JIT target (runs on current machine)
    HostJit {
        /// Optional architecture override (if None, detect from host)
        arch: Option<Architecture>,
        flags: Flags,
    },
}

impl Target {
    /// Create RISC-V 32 emulator target with default flags
    pub fn riscv32_emulator() -> Self {
        Self::Rv32Emu {
            flags: default_riscv32_flags(),
        }
    }

    /// Create host JIT target (auto-detect architecture)
    #[cfg(feature = "std")]
    pub fn host_jit() -> Self {
        Self::HostJit {
            arch: None,  // Auto-detect
            flags: default_host_flags(),
        }
    }

    /// Create the appropriate Module builder for this target
    /// (Internal: knows whether to create JITModule or ObjectModule)
    pub fn create_module_builder(&self) -> Result<ModuleBuilder, GlslError> {
        // Implementation details hidden - caller doesn't care about ModuleKind
    }
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
    pub target: Target,  // Semantic target, not technical spec
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
GLSL + Target → GlModule → (optional transform) → Done
```

### Target to Module Builder

```rust
impl Target {
    /// Create the appropriate Module builder for this target
    /// Internal implementation details are hidden
    pub fn create_module_builder(&self) -> Result<ModuleBuilder, GlslError> {
        match self {
            Target::Rv32Emu { flags } => {
                // Internally knows: ObjectModule, riscv32 triple, etc.
                let triple = riscv32_triple();
                let isa = create_isa(triple, flags)?;
                Ok(ObjectBuilder::new(isa, b"module", default_libcall_names())?)
            }
            Target::HostJit { arch, flags } => {
                // Internally knows: JITModule, host triple, etc.
                let triple = arch.map(|a| triple_for_arch(a))
                    .unwrap_or_else(|| detect_host_triple());
                let isa = create_isa(triple, flags)?;
                Ok(JITBuilder::with_isa(isa, default_libcall_names()))
            }
        }
    }

    /// Create ISA for this target (internal helper)
    fn create_isa(&self) -> Result<OwnedTargetIsa, GlslError> {
        // Implementation details hidden
    }
}
```

### Compilation Functions (NOT methods on GlModule)

```rust
// Compile GLSL source into a GlModule
pub fn compile_glsl_to_module<M: Module>(
    source: &str,
    target: Target,
) -> Result<GlModule<M>, GlslError> {
    // 1. Parse and analyze GLSL
    // 2. Create Module from target (target knows how to create the right Module type)
    // 3. Build functions directly in Module (no temp Module needed!)
    // 4. Return GlModule
}

// Transform GlModule for fixed32 (creates new GlModule)
pub fn transform_fixed32<M: Module>(
    module: &GlModule<M>,
    format: FixedPointFormat,
) -> Result<GlModule<M>, GlslError> {
    // 1. Create new GlModule with same target
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
2. **Target upfront**: Caller must know target upfront (usually true anyway)
3. **Transform still rebuilds**: Fixed32 transform still needs to rebuild functions (necessary due to type changes)

### Implementation Notes

- `GlSourceMap`: TBD - likely source location information for debugging/error reporting
- `GlFunc`: Stores metadata only, not Function IR (functions are in the Module)
- Module extraction: If needed, can add `into_module()` to extract components
- Finalization: For JIT, need to finalize and extract function pointers (separate function)

### Migration Path

1. Implement `Target` enum and `create_module_builder` in `target/`
2. Implement `GlModule` structure in `module/`
3. Implement `compile_glsl_to_module` (replaces `compile_to_clif_module`)
4. Implement `transform_fixed32` for GlModule in `transform/` (replaces current transform)
5. Implement codegen functions in `codegen/` (build executables from GlModule)
6. Update callers to use new API
7. Remove old `ClifModule` + linking code
