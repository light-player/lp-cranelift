# Phase 1: Proof of Concept - Direct Module Building

## Goal

Prove the core concept: build functions directly in the final Module without a linking step. This phase focuses on programmatic module building (not GLSL frontend integration yet) and validates both JIT and emulator backends.

## Success Criteria

1. ✅ Can create `TargetSpec` for both JIT and emulator targets
2. ✅ Can create `GlModule` with ObjectModule or JITModule
3. ✅ Can build functions programmatically directly in the Module
4. ✅ Cross-function calls work without linking step (FuncRefs are correct)
5. ✅ Can generate executables for both JIT and emulator
6. ✅ End-to-end tests pass for both backends

## Scope

### ✅ In Scope

- `TargetSpec` implementation
- `GlModule<M: Module>` structure (minimal)
- `GlFunc` metadata structure
- Programmatic function building utilities (for testing)
- Codegen for both JIT and emulator
- Tests demonstrating cross-function calls

### ❌ Out of Scope (Future Phases)

- GLSL frontend integration
- Transform pipeline
- Full source map support
- Complex function signatures
- Error handling edge cases

## Implementation Plan

### 1. Target Module (`backend2/target/`)

#### `target/spec.rs`

```rust
pub struct TargetSpec {
    pub kind: ModuleKind,
    pub triple: target_lexicon::Triple,
    pub flags: cranelift_codegen::settings::Flags,
}

pub enum ModuleKind {
    Jit,
    Object { name: String },
}

impl TargetSpec {
    /// Create spec for RISC-V 32 emulator
    pub fn riscv32_emulator() -> Result<Self, GlslError> {
        // Build triple, flags, ModuleKind::Object
    }
    
    /// Create spec for host JIT
    #[cfg(feature = "std")]
    pub fn host_jit() -> Result<Self, GlslError> {
        // Build triple, flags, ModuleKind::Jit
    }
}
```

#### `target/isa.rs`

```rust
/// Create ISA from TargetSpec
pub fn create_isa_from_spec(spec: &TargetSpec) -> Result<OwnedTargetIsa, GlslError> {
    let builder = isa::lookup_by_triple(spec.triple)?;
    builder.finish(spec.flags.clone())
}

/// Create Module builder from TargetSpec
pub fn create_module_builder(spec: &TargetSpec) -> Result<ModuleBuilder, GlslError> {
    let isa = create_isa_from_spec(spec)?;
    match spec.kind {
        ModuleKind::Jit => {
            Ok(JITBuilder::with_isa(isa, default_libcall_names()))
        }
        ModuleKind::Object { ref name } => {
            Ok(ObjectBuilder::new(isa, name.as_bytes(), default_libcall_names())?)
        }
    }
}
```

### 2. Module Structure (`backend2/module/`)

#### `module/gl_func.rs`

```rust
/// Function metadata (doesn't store Function IR, just metadata)
pub struct GlFunc {
    pub name: String,
    pub clif_sig: Signature,
    pub func_id: FuncId,
    // Note: GLSL signature not needed for Phase 1
}
```

#### `module/gl_module.rs`

```rust
/// GLSL Module - owns the actual Cranelift Module
pub struct GlModule<M: Module> {
    pub target: TargetSpec,
    pub fns: HashMap<String, GlFunc>,
    pub module: M, // Owned Module - functions are already defined here
    // Note: source_map not needed for Phase 1
}

impl<M: Module> GlModule<M> {
    /// Create new GlModule from TargetSpec
    pub fn new(target: TargetSpec) -> Result<Self, GlslError> {
        let builder = create_module_builder(&target)?;
        let module = match target.kind {
            ModuleKind::Jit => {
                // Create JITModule
            }
            ModuleKind::Object { .. } => {
                // Create ObjectModule
            }
        };
        Ok(Self {
            target,
            fns: HashMap::new(),
            module,
        })
    }
    
    /// Get function metadata by name
    pub fn get_func(&self, name: &str) -> Option<&GlFunc> {
        self.fns.get(name)
    }
    
    /// Get mutable reference to module (for building functions)
    pub fn module_mut(&mut self) -> &mut M {
        &mut self.module
    }
}
```

### 3. Programmatic Builder (`backend2/module/builder.rs`)

Helper utilities for building functions programmatically (used by tests):

```rust
/// Build a simple function programmatically
pub fn build_simple_function<M: Module>(
    gl_module: &mut GlModule<M>,
    name: &str,
    linkage: Linkage,
    sig: Signature,
    body: impl FnOnce(&mut FunctionBuilder) -> Result<(), GlslError>,
) -> Result<FuncId, GlslError> {
    // 1. Declare function in module
    let func_id = gl_module.module_mut().declare_function(name, linkage, &sig)?;
    
    // 2. Create context and builder
    let mut ctx = gl_module.module_mut().make_context();
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
    
    // 3. Set signature
    ctx.func.signature = sig.clone();
    
    // 4. Build entry block
    let entry_block = builder.create_block();
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);
    
    // 5. Call user-provided body builder
    body(&mut builder)?;
    
    // 6. Finalize and define
    builder.finalize();
    gl_module.module_mut().define_function(func_id, &mut ctx)?;
    gl_module.module_mut().clear_context(&mut ctx);
    
    // 7. Store metadata
    gl_module.fns.insert(name.to_string(), GlFunc {
        name: name.to_string(),
        clif_sig: sig,
        func_id,
    });
    
    Ok(func_id)
}

/// Build a function that calls another function
pub fn build_call_function<M: Module>(
    gl_module: &mut GlModule<M>,
    name: &str,
    linkage: Linkage,
    sig: Signature,
    callee_name: &str,
    args: &[Value],
) -> Result<FuncId, GlslError> {
    build_simple_function(gl_module, name, linkage, sig, |builder| {
        let entry_block = builder.current_block().unwrap();
        
        // Get callee FuncId
        let callee_func = gl_module.get_func(callee_name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("Function '{}' not found", callee_name)))?;
        
        // Create FuncRef in this function's context
        let callee_ref = gl_module.module_mut().declare_func_in_func(callee_func.func_id, builder.func);
        
        // Call the function
        let call_result = builder.ins().call(callee_ref, args);
        let result = builder.inst_results(call_result)[0];
        
        // Return the result
        builder.ins().return_(&[result]);
        
        Ok(())
    })
}
```

### 4. Codegen (`backend2/codegen/`)

#### `codegen/jit.rs`

```rust
/// Build JIT executable from GlModule<JITModule>
pub fn build_jit_executable(
    gl_module: GlModule<JITModule>,
) -> Result<GlslJitModule, GlslError> {
    // 1. Finalize definitions
    gl_module.module.finalize_definitions()?;
    
    // 2. Extract function pointers
    let mut function_ptrs = HashMap::new();
    for (name, gl_func) in &gl_module.fns {
        let ptr = gl_module.module.get_finalized_function(gl_func.func_id);
        function_ptrs.insert(name.clone(), ptr);
    }
    
    // 3. Build signatures map (minimal for Phase 1)
    let mut signatures = HashMap::new();
    let mut cranelift_signatures = HashMap::new();
    for (name, gl_func) in &gl_module.fns {
        // For Phase 1, create minimal GLSL signature
        // Full signature support comes later
        cranelift_signatures.insert(name.clone(), gl_func.clif_sig.clone());
    }
    
    // 4. Create GlslJitModule
    Ok(GlslJitModule {
        jit_module: gl_module.module,
        function_ptrs,
        signatures,
        cranelift_signatures,
        call_conv: gl_module.target.isa().default_call_conv(),
        pointer_type: gl_module.target.isa().pointer_type(),
    })
}
```

#### `codegen/emu.rs`

```rust
/// Build emulator executable from GlModule<ObjectModule>
pub fn build_emu_executable(
    gl_module: GlModule<ObjectModule>,
    options: &EmulatorOptions,
) -> Result<GlslEmulatorModule, GlslError> {
    // 1. Finish module and get object file
    let product = gl_module.module.finish();
    let elf_bytes = product.emit()?;
    
    // 2. Load ELF and find main address
    let load_info = load_elf(&elf_bytes)?;
    let obj = object::File::parse(&elf_bytes[..])?;
    let main_address = find_symbol_address(&obj, "main", text_section_base)?;
    
    // 3. Create emulator
    let binary = load_info.code;
    let mut emulator = Riscv32Emulator::new(binary.clone(), vec![0; options.max_memory])
        .with_max_instructions(options.max_instructions);
    
    // 4. Set up stack and PC
    emulator.set_register(Gpr::Sp, options.max_memory as u32);
    emulator.set_pc(0);
    
    // 5. Build signatures (minimal for Phase 1)
    let mut signatures = HashMap::new();
    let mut cranelift_signatures = HashMap::new();
    for (name, gl_func) in &gl_module.fns {
        cranelift_signatures.insert(name.clone(), gl_func.clif_sig.clone());
        // Minimal GLSL signature for Phase 1
    }
    
    // 6. Create GlslEmulatorModule
    Ok(GlslEmulatorModule {
        emulator,
        signatures,
        cranelift_signatures,
        binary,
        main_address,
        // ... other fields with defaults for Phase 1
    })
}
```

### 5. Tests (`tests/backend2_phase1.rs`)

#### Test 1: JIT - Simple Function Call

```rust
#[test]
fn test_jit_function_call() {
    // Create JIT target spec
    let target = TargetSpec::host_jit()?;
    
    // Create GlModule
    let mut gl_module = GlModule::new(target)?;
    
    // Build helper: add(a: i32, b: i32) -> i32
    let mut add_sig = Signature::new(CallConv::SystemV);
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.returns.push(AbiParam::new(types::I32));
    
    build_simple_function(&mut gl_module, "add", Linkage::Local, add_sig, |builder| {
        let entry = builder.current_block().unwrap();
        let a = builder.block_params(entry)[0];
        let b = builder.block_params(entry)[1];
        let sum = builder.ins().iadd(a, b);
        builder.ins().return_(&[sum]);
        Ok(())
    })?;
    
    // Build main: main() -> add(10, 20)
    let mut main_sig = Signature::new(CallConv::SystemV);
    main_sig.returns.push(AbiParam::new(types::I32));
    
    build_call_function(&mut gl_module, "main", Linkage::Export, main_sig, "add", |builder| {
        let ten = builder.ins().iconst(types::I32, 10);
        let twenty = builder.ins().iconst(types::I32, 20);
        Ok(vec![ten, twenty])
    })?;
    
    // Build executable and test
    let mut executable = build_jit_executable(gl_module)?;
    let result = executable.call_i32("main", &[])?;
    assert_eq!(result, 30);
}
```

#### Test 2: Emulator - Simple Function Call

```rust
#[test]
#[cfg(feature = "emulator")]
fn test_emu_function_call() {
    // Create emulator target spec
    let target = TargetSpec::riscv32_emulator()?;
    
    // Create GlModule
    let mut gl_module = GlModule::new(target)?;
    
    // Build same functions as JIT test
    // ... (same as above)
    
    // Build executable and test
    let options = EmulatorOptions {
        max_memory: 1024 * 1024,
        stack_size: 64 * 1024,
        max_instructions: 10000,
    };
    let mut executable = build_emu_executable(gl_module, &options)?;
    let result = executable.call_i32("main", &[])?;
    assert_eq!(result, 30);
}
```

#### Test 3: Both - Multiple Function Calls

```rust
#[test]
fn test_multiple_function_calls() {
    // Test that we can call multiple functions
    // main() -> add(multiply(2, 3), multiply(4, 5))
    // Should return 2*3 + 4*5 = 6 + 20 = 26
}
```

## File Structure

```
backend2/
├── mod.rs                 # Re-exports
├── target/
│   ├── mod.rs
│   ├── spec.rs           # TargetSpec, ModuleKind
│   └── isa.rs            # create_isa_from_spec, create_module_builder
├── module/
│   ├── mod.rs
│   ├── gl_module.rs      # GlModule<M: Module>
│   ├── gl_func.rs        # GlFunc metadata
│   └── builder.rs        # Programmatic building helpers
└── codegen/
    ├── mod.rs
    ├── jit.rs            # build_jit_executable
    └── emu.rs            # build_emu_executable
```

## Dependencies

- Will need to import from `exec/` for `GlslJitModule` and `GlslEmulatorModule` types
- Will need to import from `frontend/src_loc` for `GlSourceMap` (even if minimal)
- Will reuse existing emulator utilities from `exec/emu.rs`

## Validation

Phase 1 is complete when:

1. ✅ `TargetSpec::host_jit()` and `TargetSpec::riscv32_emulator()` work
2. ✅ Can create `GlModule<JITModule>` and `GlModule<ObjectModule>`
3. ✅ Can build functions programmatically with cross-function calls
4. ✅ `build_jit_executable()` produces working `GlslJitModule`
5. ✅ `build_emu_executable()` produces working `GlslEmulatorModule`
6. ✅ All tests pass for both backends

## Next Steps (Phase 2)

After Phase 1 is validated:

1. Add transform pipeline infrastructure
2. Port fixed32 transform to work with GlModule
3. Add GLSL frontend integration
4. Migrate existing tests to new architecture




