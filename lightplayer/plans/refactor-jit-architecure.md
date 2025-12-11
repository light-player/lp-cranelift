# Refactor JIT Architecture for Cross-Function Fixed-Point Conversion

## Overview

Refactor the compilation architecture to separate GLSL-to-CLIF compilation from JIT/emulator backends. The key change is implementing a pure functional fixed-point transformation that operates on an immutable module representation, solving cross-function call signature mismatches.

## Problem

The current `jit.rs` mixes compilation, transformation, and JIT concerns. Function-by-function conversion causes signature mismatches when functions call each other:

1. Function signatures are declared with fixed-point types (if enabled) in `declare_function_signature`
2. Function bodies are generated with float types during codegen
3. Calls reference FuncRefs that point to fixed-point signatures, but call arguments are floats
4. Converting the caller later tries to map FuncRefs, but they already point to fixed-point signatures

This creates a fundamental mismatch that can't be resolved with the current architecture.

## Solution

Separate concerns into distinct modules:

1. **`GlslCompiler`** - Compiles GLSL to CLIF (all functions, float)
2. **`ClifModule`** - Immutable module representation holding all CLIF functions
3. **`transform_fixed32`** - Pure function: `fn(&ClifModule, FixedPointFormat) -> ClifModule`
4. **`jit_glsl`** - Thin JIT wrapper using `GlslCompiler`
5. **`emulator_glsl`** - Similar wrapper for emulator

**CRITICAL: NO BACKWARDS COMPATIBILITY**

We are completely breaking the existing API. Do not maintain any backwards compatibility. Focus on getting the basic compiler pipeline working correctly. We will clean up breaking changes later.

## Implementation Plan

### 1. Create `ClifModule` structure (`crates/lp-glsl/src/clif_module.rs`)

Immutable module representation holding CLIF IR functions before linking/compilation:

```rust
pub struct ClifModule {
    user_functions: HashMap<String, Function>,
    main_function: Function,
    function_registry: FunctionRegistry,
    source_text: String,
    // Store ISA directly - ensures consistency and simplifies later compilation
    isa: OwnedTargetIsa,
}
```

Methods:

- `fn user_functions(&self) -> &HashMap<String, Function>`
- `fn main_function(&self) -> &Function`
- `fn function_registry(&self) -> &FunctionRegistry`
- `fn source_text(&self) -> &str`
- `fn isa(&self) -> &dyn TargetIsa`
- `fn target_config(&self) -> TargetFrontendConfig` - convenience method

Builder pattern for construction:

```rust
impl ClifModule {
    pub fn builder() -> ClifModuleBuilder { ... }

    /// Link all functions from this module into a Cranelift Module (JITModule, ObjectModule, etc.)
    /// Returns a mapping of function names to their FuncIds in the target module
    pub fn link_into<M: Module>(
        &self,
        module: &mut M,
        main_linkage: Linkage,
    ) -> Result<HashMap<String, FuncId>, GlslError> {
        let mut name_to_id = HashMap::new();

        // Declare and define all user functions
        for (name, func) in self.user_functions() {
            let func_id = module.declare_function(name, Linkage::Local, &func.signature)?;
            let mut ctx = module.make_context();
            ctx.func = func.clone();
            module.define_function(func_id, &mut ctx)?;
            module.clear_context(&mut ctx);
            name_to_id.insert(name.clone(), func_id);
        }

        // Declare and define main function
        let main_id = module.declare_function("main", main_linkage, &self.main_function().signature)?;
        let mut ctx = module.make_context();
        ctx.func = self.main_function().clone();
        module.define_function(main_id, &mut ctx)?;
        module.clear_context(&mut ctx);
        name_to_id.insert("main".to_string(), main_id);

        Ok(name_to_id)
    }
}

pub struct ClifModuleBuilder { ... }
impl ClifModuleBuilder {
    pub fn add_user_function(mut self, name: String, func: Function) -> Self { ... }
    pub fn add_user_functions(mut self, functions: HashMap<String, Function>) -> Self { ... }
    pub fn set_main_function(mut self, func: Function) -> Self { ... }
    pub fn set_function_registry(mut self, registry: FunctionRegistry) -> Self { ... }
    pub fn set_source_text(mut self, text: String) -> Self { ... }
    pub fn set_isa(mut self, isa: OwnedTargetIsa) -> Self { ... }
    pub fn build(self) -> ClifModule { ... }
}
```

**Key Design Decision**: Store `OwnedTargetIsa` directly instead of reconstructing from `triple` + `pointer_type`. This:

- Ensures ISA consistency (flags, settings, etc.)
- Simplifies later compilation (no need to recreate ISA)
- Matches how Cranelift modules work (they own their ISA)

### 2. Refactor `GlslCompiler` (`crates/lp-glsl/src/compiler.rs`)

Extract core compilation logic. New primary method:

```rust
impl GlslCompiler {
    pub fn compile_to_clif_module(
        &mut self,
        source: &str,
        isa: OwnedTargetIsa,
    ) -> Result<ClifModule, GlslError> {
        // 1. Parse and analyze GLSL
        let semantic_result = CompilationPipeline::parse_and_analyze(source)?;

        // 2. Create a temporary minimal module for function declarations
        //    This is needed to get FuncIds for cross-function calls
        let mut temp_module = create_minimal_module_for_declarations(isa.as_ref())?;

        // 3. Declare all user functions with FLOAT signatures (no conversion)
        let mut func_ids: HashMap<String, FuncId> = HashMap::new();
        for user_func in &semantic_result.typed_ast.user_functions {
            let sig = SignatureBuilder::build_with_triple(
                &user_func.return_type,
                &user_func.parameters,
                isa.pointer_type(),
                isa.triple(),
            );
            let func_id = temp_module.declare_function(&user_func.name, Linkage::Local, &sig)?;
            func_ids.insert(user_func.name.clone(), func_id);
        }

        // 4. Compile all user functions to CLIF with FLOAT types
        let mut user_functions = HashMap::new();
        for user_func in &semantic_result.typed_ast.user_functions {
            let func_id = func_ids[&user_func.name];
            let func = self.compile_function_to_clif(
                user_func,
                func_id,
                &func_ids,
                &semantic_result.typed_ast.function_registry,
                &mut temp_module,
            )?;
            user_functions.insert(user_func.name.clone(), func);
        }

        // 5. Compile main function to CLIF with FLOAT types
        let main_func = self.compile_main_function_to_clif(
            &semantic_result.typed_ast.main_function,
            &func_ids,
            &semantic_result.typed_ast.function_registry,
            &mut temp_module,
        )?;

        // 6. Build and return ClifModule
        Ok(ClifModule::builder()
            .set_function_registry(semantic_result.typed_ast.function_registry)
            .set_source_text(source.to_string())
            .set_isa(isa)
            .add_user_functions(user_functions)
            .set_main_function(main_func)
            .build())
    }
}
```

Key points:

- **ALL functions compiled with float types initially**
- **NO signature conversion during declaration**
- Uses a temporary module only for FuncId/FuncRef management during compilation
- Return `ClifModule` containing all functions (immutable, ready for transformation)

Remove or completely rewrite existing methods - we don't care about breaking changes.

### 3. Create module-level fixed-point transformation (`crates/lp-glsl/src/transform/fixed32/module.rs`)

Pure functional transformation that converts all functions atomically:

```rust
pub fn transform_module(
    module: &ClifModule,
    format: FixedPointFormat,
) -> Result<ClifModule, GlslError> {
    // Create a temporary module for FuncRef management during transformation
    // This is needed because FuncRefs are scoped to a Function, but we need
    // to map old FuncRefs (pointing to float signatures) to new FuncRefs (pointing to fixed-point signatures)
    let mut temp_module = create_minimal_module_for_declarations(module.isa())?;

    // Step 1: Convert all function signatures and create new FuncRefs
    // Build mapping: function_name -> (old_signature, new_signature, new_func_ref)
    let mut func_ref_map: HashMap<String, (Signature, Signature, FuncRef)> = HashMap::new();

    // Convert user function signatures
    for (name, func) in module.user_functions() {
        let old_sig = func.signature.clone();
        let new_sig = convert_signature(&old_sig, format);

        // Create a FuncRef in the temp module for this converted function
        let func_id = temp_module.declare_function(name, Linkage::Local, &new_sig)?;
        let mut temp_func = Function::new();
        let func_ref = temp_module.declare_func_in_func(func_id, &mut temp_func);

        func_ref_map.insert(name.clone(), (old_sig, new_sig, func_ref));
    }

    // Convert main function signature
    let main_old_sig = module.main_function().signature.clone();
    let main_new_sig = convert_signature(&main_old_sig, format);
    let main_func_id = temp_module.declare_function("main", Linkage::Export, &main_new_sig)?;
    let temp_main_func = Function::new();
    let main_func_ref = temp_module.declare_func_in_func(main_func_id, &mut temp_main_func);

    // Step 2: Convert all function bodies, updating call sites to use new FuncRefs
    let mut builder = ClifModule::builder()
        .set_function_registry(module.function_registry().clone())
        .set_source_text(module.source_text().to_string())
        .set_isa(module.isa().clone());

    // Convert user functions
    for (name, func) in module.user_functions() {
        let (_, new_sig, new_func_ref) = &func_ref_map[name];

        // Create new function with converted signature
        let mut new_func = Function::with_name_signature(
            func.name.clone(),
            new_sig.clone(),
        );

        // Rewrite function body, mapping FuncRefs
        rewrite_function_with_ref_map(
            func,
            &mut new_func,
            format,
            &func_ref_map,
            *new_func_ref,
        )?;

        builder = builder.add_user_function(name.clone(), new_func);
    }

    // Convert main function
    let mut new_main_func = Function::with_name_signature(
        module.main_function().name.clone(),
        main_new_sig,
    );
    rewrite_function_with_ref_map(
        module.main_function(),
        &mut new_main_func,
        format,
        &func_ref_map,
        main_func_ref,
    )?;
    builder = builder.set_main_function(new_main_func);

    Ok(builder.build())
}

/// Helper function to rewrite a function, updating call sites to use mapped FuncRefs
fn rewrite_function_with_ref_map(
    old_func: &Function,
    new_func: &mut Function,
    format: FixedPointFormat,
    func_ref_map: &HashMap<String, (Signature, Signature, FuncRef)>,
    self_func_ref: FuncRef,
) -> Result<(), GlslError> {
    // Copy function body from old_func to new_func
    // During copy, rewrite:
    // 1. All float types to fixed-point types
    // 2. All call instructions to use mapped FuncRefs

    // Implementation details:
    // - Walk through all instructions in old_func
    // - For call instructions, look up the callee name, find its new FuncRef in func_ref_map
    // - Replace the FuncRef in the call instruction
    // - Convert all float values/operations to fixed-point

    // This is the core of the transformation - see existing rewrite::rewrite_function
    // but extended to handle FuncRef mapping across the module
    rewrite::rewrite_function_with_ref_map(old_func, new_func, format, func_ref_map, self_func_ref)
}
```

**FuncRef Mapping Strategy**:

The key challenge is that FuncRefs are scoped to individual Functions, but we need to map them across the entire module. Solution:

1. **Create a temporary module** to hold converted function signatures and generate FuncRefs
2. **Build a mapping**: `function_name -> new_FuncRef` for all functions
3. **During function rewriting**: When encountering a call instruction:
   - Extract the callee name from the old FuncRef (via ExternalName)
   - Look up the new FuncRef in the mapping
   - Replace the FuncRef in the call instruction
4. **Convert function signatures** first, then bodies, ensuring all references are consistent

This ensures that when function A calls function B:

- Function B's signature is converted to fixed-point
- Function A's call instruction references the new FuncRef pointing to the fixed-point signature
- All type conversions happen consistently across the module

### 4. Create `jit_glsl` module (`crates/lp-glsl/src/jit_glsl.rs`)

Thin wrapper for JIT compilation:

```rust
pub fn compile_glsl_to_jit(
    source: &str,
    fixed_point_format: Option<FixedPointFormat>,
) -> Result<*const u8, GlslError> {
    // 1. Create GlslCompiler
    let mut compiler = GlslCompiler::new();

    // 2. Compile to ClifModule (all float)
    let isa = create_host_isa()?;
    let mut module = compiler.compile_to_clif_module(source, isa)?;

    // 3. Transform to fixed-point if requested
    if let Some(format) = fixed_point_format {
        module = transform_fixed32::transform_module(&module, format)?;
    }

    // 4. Create JITModule and link all functions from ClifModule
    let mut jit_module = create_jit_module(module.isa().clone())?;
    let name_to_id = module.link_into(&mut jit_module, Linkage::Export)?;

    // 5. Finalize and return function pointer
    jit_module.finalize_definitions()?;
    let main_id = name_to_id["main"];
    Ok(jit_module.get_finalized_function(main_id))
}

fn create_jit_module(isa: OwnedTargetIsa) -> Result<JITModule, GlslError> {
    use cranelift_jit::JITBuilder;
    let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    Ok(JITModule::new(builder))
}
```

**Note**: Uses the `link_into` helper method from `ClifModule` to simplify the linking process.

### 5. Create `emulator_glsl` module (`crates/lp-glsl/src/emulator_glsl.rs`)

**IMPORTANT**: This module and all emulator-related code must be gated behind the `emulator` feature flag. Embedded use cases (like `apps/esp32c6-glsl-jit`) should not include the emulator.

Similar wrapper for emulator/binary compilation:

```rust
pub fn compile_glsl_to_binary(
    source: &str,
    isa: OwnedTargetIsa,
    fixed_point_format: Option<FixedPointFormat>,
) -> Result<Vec<u8>, GlslError> {
    // 1. Create GlslCompiler
    let mut compiler = GlslCompiler::new();

    // 2. Compile to ClifModule (all float)
    let mut module = compiler.compile_to_clif_module(source, isa.clone())?;

    // 3. Transform to fixed-point if requested
    if let Some(format) = fixed_point_format {
        module = transform_fixed32::transform_module(&module, format)?;
    }

    // 4. Compile each function to machine code
    let mut binary = Vec::new();
    let mut ctrl_plane = ControlPlane::default();
    let isa_ref = module.isa();

    // Compile user functions
    for func in module.user_functions().values() {
        let mut ctx = Context::new();
        ctx.func = func.clone();
        let compiled = ctx.compile(isa_ref, &mut ctrl_plane)?;
        binary.extend_from_slice(compiled.code_buffer());
    }

    // Compile main function
    let mut ctx = Context::new();
    ctx.func = module.main_function().clone();
    let compiled = ctx.compile(isa_ref, &mut ctrl_plane)?;
    binary.extend_from_slice(compiled.code_buffer());

    Ok(binary)
}
```

**Note**: For emulator/binary compilation, we compile functions directly without linking into a Module, since we're just producing raw machine code.

### 6. Update `jit.rs`

**BREAK IT. COMPLETELY REWRITE IT.**

Either:

- Delete it entirely and replace with `jit_glsl.rs`
- Or rewrite it to be a thin wrapper around `jit_glsl`

No backwards compatibility. Users will need to update their code.

### 7. Update `compiler.rs`

**BREAK IT. COMPLETELY REWRITE IT.**

Have `Compiler` use the new architecture:

- `compile_to_code_bytes` uses `emulator_glsl::compile_glsl_to_binary`
- Remove all the old JIT-specific code
- Simplify to use new modules

### 8. Update module exports (`crates/lp-glsl/src/lib.rs`)

Export new modules:

- `pub mod clif_module;`
- `pub mod executable;` - Executable trait and implementations
- `pub mod compile;` - Compilation pipeline functions
- `pub mod jit_glsl;` - (internal, may be deprecated)
- `pub mod emulator_glsl;` - (internal, may be deprecated)
- `pub mod transform::fixed32::module;` (or re-export)

Public API exports:

- `pub use executable::{GlslExecutable, GlslOptions, RunMode, DecimalFormat, GlslValue};`
- `pub use compile::{glsl_jit, glsl_emu_riscv32};`

Remove or deprecate old exports if they break.

### 9. Update `transform/fixed32/mod.rs`

Export the module transformation:

```rust
pub mod module;
pub use module::transform_module;
```

### 10. Create Executable Module API (`crates/lp-glsl/src/executable.rs`)

Design a clean, trait-based API for executing GLSL functions that abstracts away JIT vs Emulator:

```rust
// ============================================================================
// Options and Configuration
// ============================================================================

/// Execution mode for GLSL compilation
#[derive(Debug, Clone)]
pub enum RunMode {
    /// JIT compile and execute on the host architecture
    HostJit,
    /// Emulate execution (currently RISC-V 32-bit only)
    /// Requires `emulator` feature flag to be enabled
    Emulator {
        /// Maximum memory size in bytes (RAM)
        max_memory: usize,
        /// Stack size in bytes
        stack_size: usize,
        /// Maximum instruction count before timeout
        max_instructions: u64,
    },
}

/// Decimal format for floating-point operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecimalFormat {
    /// Native floating-point (f32/f64)
    Float,
    /// Fixed-point 32-bit (Q format)
    Fixed32,
    /// Fixed-point 64-bit (not yet supported)
    Fixed64,
}

/// Compilation options
#[derive(Debug, Clone)]
pub struct GlslOptions {
    pub run_mode: RunMode,
    pub decimal_format: DecimalFormat,
}

impl GlslOptions {
    pub fn validate(&self) -> Result<(), GlslError> {
        // Validate option combinations
        match (&self.run_mode, self.decimal_format) {
            (RunMode::Emulator { .. }, DecimalFormat::Float) => {
                // TODO: Float support will be added for riscv32_imafc in the future
                Err(GlslError::new(
                    ErrorCode::E0400,
                    "Float format not yet supported in emulator mode (will be supported for riscv32_imafc)",
                ))
            }
            (RunMode::HostJit, DecimalFormat::Float) => {
                // Check if host supports float
                let triple = Triple::host();
                if triple.architecture == Architecture::Riscv32 {
                    Err(GlslError::new(
                        ErrorCode::E0400,
                        "Float format not supported on RISC-V 32-bit",
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    /// Default options for JIT execution
    pub fn jit() -> Self {
        Self {
            run_mode: RunMode::HostJit,
            decimal_format: DecimalFormat::Float,
        }
    }

    /// Default options for emulator execution
    pub fn emulator(max_memory: usize, stack_size: usize) -> Self {
        Self {
            run_mode: RunMode::Emulator {
                max_memory,
                stack_size,
                max_instructions: 10_000_000,
            },
            decimal_format: DecimalFormat::Fixed32,
        }
    }

    /// Convenience method for RISC-V 32-bit IMC (Integer, Multiply, Compressed) emulator
    /// Uses 1MB RAM, 64KB stack, Fixed32 format
    /// Note: IMAFC (with Float) support will be added in the future
    pub fn emu_riscv32_imac() -> Self {
        Self {
            run_mode: RunMode::Emulator {
                max_memory: 1024 * 1024,      // 1MB RAM
                stack_size: 64 * 1024,        // 64KB stack
                max_instructions: 10_000_000,
            },
            decimal_format: DecimalFormat::Fixed32,
        }
    }

    /// Set decimal format
    pub fn with_format(mut self, format: DecimalFormat) -> Self {
        self.decimal_format = format;
        self
    }
}

// ============================================================================
// Executable Module Trait
// ============================================================================

/// Trait for executing GLSL functions with various return types
/// Abstracts away JIT vs Emulator implementations
///
/// **Current State**: Supports basic function calling with in-parameters only.
/// **Future Extensions**: Will add methods for:
///   - Setting uniform variables (uniforms are read-only, set before execution)
///   - Binding textures/samplers (opaque types accessed via built-in functions)
///   - Setting built-in variables (gl_Position, gl_FragCoord, etc.)
///   - Handling shader stage interfaces (in/out variables between stages)
pub trait GlslExecutable {
    /// Call a function that returns void
    fn call_void(&mut self, name: &str, args: &[GlslValue]) -> Result<(), GlslError>;

    /// Call a function that returns i32
    fn call_i32(&mut self, name: &str, args: &[GlslValue]) -> Result<i32, GlslError>;

    /// Call a function that returns f32 (or fixed-point equivalent)
    fn call_f32(&mut self, name: &str, args: &[GlslValue]) -> Result<f32, GlslError>;

    /// Call a function that returns bool (as i8)
    fn call_bool(&mut self, name: &str, args: &[GlslValue]) -> Result<bool, GlslError>;

    /// Call a function that returns a vector (vec2, vec3, vec4)
    fn call_vec(&mut self, name: &str, args: &[GlslValue], dim: usize) -> Result<Vec<f32>, GlslError>;

    /// Call a function that returns a matrix
    fn call_mat(&mut self, name: &str, args: &[GlslValue], rows: usize, cols: usize) -> Result<Vec<f32>, GlslError>;

    /// Get the signature of a function by name
    fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature>;

    /// List all available function names
    fn list_functions(&self) -> Vec<String>;

    // ========================================================================
    // Future extensions for full GLSL support:
    // ========================================================================

    // TODO: Uniform management
    // fn set_uniform(&mut self, name: &str, value: GlslValue) -> Result<(), GlslError>;
    // fn get_uniform(&self, name: &str) -> Option<&GlslValue>;
    // fn list_uniforms(&self) -> Vec<String>;

    // TODO: Texture/sampler binding (opaque types)
    // fn bind_texture(&mut self, name: &str, texture: TextureHandle) -> Result<(), GlslError>;
    // fn bind_sampler(&mut self, name: &str, sampler: SamplerHandle) -> Result<(), GlslError>;

    // TODO: Built-in variables (for shader stages)
    // fn set_builtin(&mut self, builtin: BuiltinVariable, value: GlslValue) -> Result<(), GlslError>;

    // TODO: Shader stage interface (in/out variables)
    // fn set_stage_input(&mut self, name: &str, value: GlslValue) -> Result<(), GlslError>;
    // fn get_stage_output(&self, name: &str) -> Option<&GlslValue>;
}

/// GLSL value types for function arguments
#[derive(Debug, Clone)]
pub enum GlslValue {
    I32(i32),
    F32(f32),
    Bool(bool),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat2x2([[f32; 2]; 2]),
    Mat3x3([[f32; 3]; 3]),
    Mat4x4([[f32; 4]; 4]),
}

// ============================================================================
// Implementations
// ============================================================================

/// JIT-compiled GLSL module (executes on host)
pub struct GlslJitModule {
    jit_module: JITModule,
    function_ptrs: HashMap<String, *const u8>,
    signatures: HashMap<String, FunctionSignature>,
    call_conv: CallConv,
    pointer_type: Type,
}

impl GlslExecutable for GlslJitModule {
    fn call_i32(&mut self, name: &str, args: &[GlslValue]) -> Result<i32, GlslError> {
        let func_ptr = self.function_ptrs.get(name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0101, format!("Function '{}' not found", name)))?;

        // Convert args to calling convention, call function, extract result
        // Implementation uses unsafe function pointer calls with proper ABI handling
        // ...
    }

    // ... other methods
}

/// Emulator-based GLSL module (executes in RISC-V emulator)
/// Requires `emulator` feature flag to be enabled
#[cfg(feature = "emulator")]
pub struct GlslEmulatorModule {
    emulator: Riscv32Emulator,
    function_entries: HashMap<String, u32>, // PC addresses
    signatures: HashMap<String, FunctionSignature>,
    binary: Vec<u8>,
}

#[cfg(feature = "emulator")]
impl GlslExecutable for GlslEmulatorModule {
    fn call_i32(&mut self, name: &str, args: &[GlslValue]) -> Result<i32, GlslError> {
        let entry = self.function_entries.get(name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0101, format!("Function '{}' not found", name)))?;

        let sig = self.signatures.get(name).unwrap();

        // Convert args to DataValues
        let data_args: Vec<DataValue> = args.iter()
            .map(|v| glsl_value_to_data_value(v))
            .collect::<Result<_, _>>()?;

        // Call via emulator
        let results = self.emulator.call_function(*entry, &data_args, &sig.to_cranelift_signature())?;

        // Extract i32 result
        if let Some(DataValue::I32(val)) = results.first() {
            Ok(*val)
        } else {
            Err(GlslError::new(ErrorCode::E0400, "Unexpected return type"))
        }
    }

    // ... other methods
}
```

### 11. Create Compilation Pipeline Functions (`crates/lp-glsl/src/compile.rs`)

Internal, reusable compilation functions:

```rust
/// Compile GLSL to CLIF module (internal, reusable)
/// This is the core compilation step that can be reused for different backends
pub fn compile_glsl_to_clif(
    source: &str,
    options: &GlslOptions,
) -> Result<ClifModule, GlslError> {
    options.validate()?;

    let mut compiler = GlslCompiler::new();

    // Determine ISA based on run mode
    let isa = match &options.run_mode {
        RunMode::HostJit => create_host_isa()?,
        #[cfg(feature = "emulator")]
        RunMode::Emulator { .. } => create_riscv32_isa()?,
        #[cfg(not(feature = "emulator"))]
        RunMode::Emulator { .. } => {
            return Err(GlslError::new(
                ErrorCode::E0400,
                "Emulator mode requires 'emulator' feature flag",
            ));
        }
    };

    // Compile to CLIF
    let mut module = compiler.compile_to_clif_module(source, isa)?;

    // Apply transformations
    match options.decimal_format {
        DecimalFormat::Fixed32 => {
            module = transform_fixed32::transform_module(&module, FixedPointFormat::Q16_16)?;
        }
        DecimalFormat::Fixed64 => {
            return Err(GlslError::new(ErrorCode::E0400, "Fixed64 not yet supported"));
        }
        DecimalFormat::Float => {
            // No transformation needed
        }
    }

    Ok(module)
}

/// Options for emulator execution
#[cfg(feature = "emulator")]
struct EmulatorOptions {
    max_memory: usize,
    stack_size: usize,
    max_instructions: u64,
}

/// Link CLIF module for JIT execution
fn link_glsl_for_jit(
    module: ClifModule,
) -> Result<GlslJitModule, GlslError> {
    let mut jit_module = create_jit_module(module.isa().clone())?;
    let name_to_id = module.link_into(&mut jit_module, Linkage::Export)?;

    jit_module.finalize_definitions()?;

    // Build function pointer map
    let mut function_ptrs = HashMap::new();
    for (name, func_id) in &name_to_id {
        let ptr = jit_module.get_finalized_function(*func_id);
        function_ptrs.insert(name.clone(), ptr);
    }

    // Extract signatures
    let mut signatures = HashMap::new();
    for (name, func) in module.user_functions() {
        signatures.insert(name.clone(), func.signature.clone().into());
    }
    signatures.insert("main".to_string(), module.main_function().signature.clone().into());

    Ok(GlslJitModule {
        jit_module,
        function_ptrs,
        signatures,
        call_conv: module.isa().default_call_conv(),
        pointer_type: module.isa().pointer_type(),
    })
}

/// Link CLIF module for emulator execution
/// Requires `emulator` feature flag to be enabled
#[cfg(feature = "emulator")]
fn link_glsl_for_emulator(
    module: ClifModule,
    emulator_options: &EmulatorOptions,
) -> Result<GlslEmulatorModule, GlslError> {
    // Compile to binary
    let binary = compile_clif_to_binary(&module)?;

    // Create emulator
    let ram_size = emulator_options.max_memory;
    let mut emulator = Riscv32Emulator::new(binary.clone(), vec![0; ram_size])
        .with_max_instructions(emulator_options.max_instructions);

    // Set up stack pointer (stack starts at top of RAM, grows downward)
    let stack_base = ram_size as u32;
    emulator.set_register(Gpr::Sp, stack_base as i32);
    emulator.set_pc(0);

    // Build function entry map (from symbol table or fixed addresses)
    let mut function_entries = HashMap::new();
    // ... extract function addresses from binary/symbols

    // Extract signatures
    let mut signatures = HashMap::new();
    for (name, func) in module.user_functions() {
        signatures.insert(name.clone(), func.signature.clone().into());
    }
    signatures.insert("main".to_string(), module.main_function().signature.clone().into());

    Ok(GlslEmulatorModule {
        emulator,
        function_entries,
        signatures,
        binary,
    })
}
```

### 12. Create Public API Functions (`crates/lp-glsl/src/lib.rs`)

Top-level convenience functions:

```rust
/// Compile and JIT execute GLSL on the host architecture
pub fn glsl_jit(source: &str, options: GlslOptions) -> Result<Box<dyn GlslExecutable>, GlslError> {
    let module = compile_glsl_to_clif(source, &options)?;
    let jit_module = link_glsl_for_jit(module)?;
    Ok(Box::new(jit_module))
}

/// Compile and execute GLSL in RISC-V 32-bit emulator
/// Requires `emulator` feature flag to be enabled
#[cfg(feature = "emulator")]
pub fn glsl_emu_riscv32(
    source: &str,
    options: GlslOptions,
) -> Result<Box<dyn GlslExecutable>, GlslError> {
    let module = compile_glsl_to_clif(source, &options)?;

    let emulator_options = match &options.run_mode {
        RunMode::Emulator { max_memory, stack_size, max_instructions, .. } => EmulatorOptions {
            max_memory: *max_memory,
            stack_size: *stack_size,
            max_instructions: *max_instructions,
        },
        _ => return Err(GlslError::new(ErrorCode::E0400, "Invalid run mode for emulator")),
    };

    let emu_module = link_glsl_for_emulator(module, &emulator_options)?;
    Ok(Box::new(emu_module))
}
```

**Usage Example**:

```rust
// JIT execution
let mut module = glsl_jit(
    "int main() { return 42; }",
    GlslOptions::jit(),
)?;
let result = module.call_i32("main", &[])?;
assert_eq!(result, 42);

// Emulator execution with fixed-point (using convenience method)
let mut module = glsl_emu_riscv32(
    "int main() { return 42; }",
    GlslOptions::emu_riscv32_imac(),
)?;
let result = module.call_i32("main", &[])?;
assert_eq!(result, 42);

// Or with custom memory/stack sizes
let mut module = glsl_emu_riscv32(
    "int main() { return 42; }",
    GlslOptions::emulator(1024 * 1024, 64 * 1024), // 1MB RAM, 64KB stack
)?;
let result = module.call_i32("main", &[])?;
assert_eq!(result, 42);
```

## Key Design Decisions

1. **Immutable `ClifModule`**: Enables pure functional transformation, easier to reason about, no mutation bugs
2. **Store ISA directly**: `ClifModule` owns `OwnedTargetIsa` instead of reconstructing from triple+pointer_type. Ensures consistency and simplifies compilation.
3. **Module-level transformation**: Converts all functions atomically, ensuring FuncRefs map correctly across the entire module
4. **FuncRef mapping via temporary module**: During transformation, use a temporary module to create new FuncRefs pointing to converted signatures, then map old FuncRefs to new ones
5. **Helper method for linking**: `ClifModule::link_into()` simplifies the process of linking functions into JITModule/ObjectModule
6. **Trait-based execution API**: `GlslExecutable` trait abstracts away JIT vs Emulator, providing a clean, unified interface for calling GLSL functions
7. **Extensible trait design**: `GlslExecutable` trait is designed to be extended with uniform management, texture binding, and built-in variable support without breaking existing code
8. **Emulator feature gating**: All emulator code is gated behind `#[cfg(feature = "emulator")]` to allow embedded use cases to exclude it entirely
9. **Convenience methods**: `GlslOptions::emu_riscv32_imac()` provides sensible defaults (1MB RAM, 64KB stack, Fixed32) for common emulator use cases
10. **Options validation**: `GlslOptions::validate()` ensures invalid option combinations are caught early (e.g., float on emulator currently, float on RISC-V 32-bit JIT)
11. **Clear compilation pipeline**: Separate functions for `compile_glsl_to_clif` (reusable), `link_glsl_for_jit`, and `link_glsl_for_emulator` allow flexible composition
12. **Type-safe function calling**: Methods like `call_i32`, `call_f32`, etc. handle conversions and calling conventions automatically
13. **Separation of concerns**: Compilation, transformation, and backend are completely separate
14. **NO BACKWARDS COMPATIBILITY**: Break everything. We'll fix it later.

## GLSL Spec Compatibility Notes

After reviewing the GLSL spec (`/Users/yona/dev/photomancer/glsl-spec`), the current API design is compatible with GLSL semantics:

### Currently Supported (Phase 1)

- ✅ **Function calling**: User-defined functions with `in` parameters only
- ✅ **Basic types**: int, float, bool, vectors, matrices
- ✅ **Local variables**: Variables without storage qualifiers
- ✅ **Const qualifier**: Compile-time constants

### Future Extensions Needed (Phase 2+)

- ⏳ **Uniform variables**: Read-only globals set before execution. Need `set_uniform()` methods
- ⏳ **Opaque types**: Samplers, textures, images accessed via built-in functions. Need `bind_texture()`/`bind_sampler()` methods
- ⏳ **Built-in variables**: `gl_Position`, `gl_FragCoord`, etc. Need `set_builtin()` methods
- ⏳ **Shader stage interfaces**: `in`/`out` variables for multi-stage pipelines
- ⏳ **Uniform blocks**: Structured uniform data (UBOs)
- ⏳ **Buffer variables**: `buffer` qualifier for SSBOs

### Design Considerations

1. **Uniforms are read-only**: Once set, they don't change during shader execution. Perfect for `set_uniform()` API.
2. **Opaque types are handles**: Textures/samplers are opaque handles, not values. They need separate binding API.
3. **Built-ins are stage-specific**: Different built-ins for vertex vs fragment shaders. API should be stage-aware.
4. **Function parameters**: Currently only `in` parameters are supported. `out`/`inout` can be added later without breaking the trait.
5. **Emulator feature gating**: All emulator code is gated behind `#[cfg(feature = "emulator")]` to allow embedded use cases to exclude it entirely.

The trait-based design allows these extensions to be added incrementally without breaking existing code.

### Emulator Configuration

- **Current**: `emu_riscv32_imac()` - RISC-V 32-bit IMC (Integer, Multiply, Compressed) with Fixed32
  - 1MB RAM
  - 64KB stack
  - Fixed32 decimal format
- **Future**: `emu_riscv32_imafc()` - RISC-V 32-bit IMAFC (with Float) will support native float
  - Same memory/stack configuration
  - Float decimal format support

## Files to Create

- `crates/lp-glsl/src/clif_module.rs` - Immutable module representation
- `crates/lp-glsl/src/jit_glsl.rs` - JIT wrapper (internal, may be replaced by executable.rs)
- `crates/lp-glsl/src/emulator_glsl.rs` - Emulator wrapper (internal, gated behind `emulator` feature)
- `crates/lp-glsl/src/transform/fixed32/module.rs` - Module-level transformation
- `crates/lp-glsl/src/executable.rs` - Executable module trait and implementations (emulator parts gated behind `emulator` feature)
- `crates/lp-glsl/src/compile.rs` - Compilation pipeline functions (compile_glsl_to_clif, link_glsl_for_jit, etc., emulator parts gated)

## Feature Flags

- **`emulator`**: Enables emulator-based execution. When disabled, `glsl_emu_riscv32()`, `GlslEmulatorModule`, and related code are not compiled. This allows embedded use cases (like `apps/esp32c6-glsl-jit`) to exclude the emulator entirely.

**Cargo.toml configuration**:

```toml
[features]
default = []
emulator = ["lp-riscv-tools"]  # Enable emulator support
```

**Usage in embedded projects**:

```toml
# In apps/esp32c6-glsl-jit/Cargo.toml
[dependencies]
lp-glsl = { path = "../../crates/lp-glsl", default-features = false }
# No emulator feature = no emulator code compiled
```

## Files to Modify (Breaking Changes Expected)

- `crates/lp-glsl/src/compiler.rs` - Complete rewrite to use new architecture
- `crates/lp-glsl/src/jit.rs` - Complete rewrite or deletion
- `crates/lp-glsl/src/lib.rs` - Update exports
- `crates/lp-glsl/src/transform/fixed32/mod.rs` - Export module transformation

## Testing Strategy

1. Update `fn_ret_f32.glsl` test to use new API - this should now pass
2. Add tests for cross-function calls with fixed-point conversion
3. Add tests for module-level transformation
4. Verify all existing filetests still work (may need to update API calls)

## Migration Notes

**ALL EXISTING CODE USING `JIT` OR `Compiler` WILL BREAK**

Users will need to:

**Old API (breaking)**:

```rust
// Old JIT usage
let mut jit = JIT::new();
let func_ptr = jit.compile(source)?;
let result = unsafe { std::mem::transmute::<*const u8, fn() -> i32>(func_ptr)() };

// Old Compiler usage
let mut compiler = Compiler::new();
let binary = compiler.compile_to_code_bytes(source, isa)?;
```

**New API**:

```rust
// New JIT usage
use lp_glsl::{glsl_jit, GlslOptions};
let mut module = glsl_jit(source, GlslOptions::jit())?;
let result = module.call_i32("main", &[])?;

// New emulator usage
let mut module = glsl_emu_riscv32(
    source,
    GlslOptions::emulator(64 * 1024).with_format(DecimalFormat::Fixed32),
)?;
let result = module.call_i32("main", &[])?;
```

**Benefits of new API**:

- Type-safe function calling (no unsafe transmutes)
- Unified interface for JIT and emulator
- Options validation catches errors early
- Clean separation of compilation and execution
- Support for calling user-defined functions, not just main

We don't care. Fix it later.

## Implementation Order

1. Create `ClifModule` structure
2. Refactor `GlslCompiler::compile_to_clif_module` (extract from jit.rs)
3. Create `transform_fixed32::transform_module` (module-level conversion)
4. Create `compile_glsl_to_clif` function (reusable compilation pipeline)
5. Create `GlslExecutable` trait and `GlslOptions` struct
6. Implement `GlslJitModule` with `GlslExecutable` trait
7. Implement `GlslEmulatorModule` with `GlslExecutable` trait
8. Create `link_glsl_for_jit` and `link_glsl_for_emulator` functions
9. Create top-level `glsl_jit()` and `glsl_emu_riscv32()` functions
10. Update/break `jit.rs` and `compiler.rs` (may become thin wrappers)
11. Update exports and tests
