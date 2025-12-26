# Phase 1: Proof of Concept - Direct Module Building

## Goal

Prove the core concept: build functions directly in the final Module without a linking step. This phase focuses on programmatic module building (not GLSL frontend integration yet) and validates both JIT and emulator backends with **unit tests** that test components in isolation.

## Success Criteria

1. ✅ Can create `Target::host_jit()` and `Target::riscv32_emulator()`
2. ✅ Can create `GlModule` with ObjectModule or JITModule (target handles details)
3. ✅ Can build functions programmatically directly in the Module
4. ✅ Cross-function calls work without linking step (FuncRefs are correct)
5. ✅ Can generate executables for both JIT and emulator
6. ✅ All unit tests pass for individual components

## Scope

### ✅ In Scope

- `Target` enum implementation (semantic targets: Rv32Emu, HostJit)
- `GlModule<M: Module>` structure (minimal)
- `GlFunc` metadata structure
- Programmatic function building utilities (for testing)
- Codegen for both JIT and emulator
- **Unit tests** for each component in isolation

### ❌ Out of Scope (Future Phases)

- GLSL frontend integration
- Transform pipeline
- Full source map support
- Complex function signatures
- Error handling edge cases
- Integration tests (end-to-end)

## Implementation Plan

### 1. Target Module (`backend2/target/`)

#### `target/target.rs`

```rust
use cranelift_codegen::isa::{self, OwnedTargetIsa};
use cranelift_codegen::settings::Flags;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::Type;
use cranelift_codegen::isa::CallConv;
use target_lexicon::Architecture;
use crate::error::{ErrorCode, GlslError};

/// Semantic target enum - caller doesn't need to know implementation details
pub enum Target {
    /// RISC-V 32-bit emulator target
    Rv32Emu {
        flags: Flags,
        /// Cached ISA (created lazily)
        isa: Option<OwnedTargetIsa>,
    },
    /// Host JIT target (runs on current machine)
    HostJit {
        /// Optional architecture override (if None, detect from host)
        arch: Option<Architecture>,
        flags: Flags,
        /// Cached ISA (created lazily)
        isa: Option<OwnedTargetIsa>,
    },
}

impl Target {
    /// Create RISC-V 32 emulator target with default flags
    pub fn riscv32_emulator() -> Result<Self, GlslError> {
        Ok(Self::Rv32Emu {
            flags: default_riscv32_flags()?,
            isa: None,
        })
    }

    /// Create host JIT target (auto-detect architecture)
    #[cfg(feature = "std")]
    pub fn host_jit() -> Result<Self, GlslError> {
        Ok(Self::HostJit {
            arch: None,  // Auto-detect
            flags: default_host_flags()?,
            isa: None,
        })
    }

    /// Create host JIT with specific architecture
    pub fn host_jit_with_arch(arch: Architecture) -> Result<Self, GlslError> {
        Ok(Self::HostJit {
            arch: Some(arch),
            flags: default_host_flags()?,
            isa: None,
        })
    }

    /// Create or get cached ISA for this target
    pub fn create_isa(&mut self) -> Result<&OwnedTargetIsa, GlslError> {
        match self {
            Target::Rv32Emu { flags, isa } => {
                if isa.is_none() {
                    let triple = riscv32_triple();
                    let builder = isa::lookup_by_triple(triple)
                        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("ISA lookup failed: {}", e)))?;
                    *isa = Some(builder.finish(flags.clone())
                        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("ISA creation failed: {}", e)))?);
                }
                Ok(isa.as_ref().unwrap())
            }
            Target::HostJit { arch, flags, isa } => {
                if isa.is_none() {
                    let triple = arch.map(|a| triple_for_arch(a))
                        .unwrap_or_else(|| detect_host_triple());
                    let builder = isa::lookup_by_triple(triple)
                        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("ISA lookup failed: {}", e)))?;
                    *isa = Some(builder.finish(flags.clone())
                        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("ISA creation failed: {}", e)))?);
                }
                Ok(isa.as_ref().unwrap())
            }
        }
    }

    /// Get pointer type for this target (uses cached ISA if available)
    pub fn pointer_type(&mut self) -> Result<Type, GlslError> {
        let isa = self.create_isa()?;
        Ok(isa.pointer_type())
    }

    /// Get default calling convention for this target (uses cached ISA if available)
    pub fn default_call_conv(&mut self) -> Result<CallConv, GlslError> {
        let isa = self.create_isa()?;
        Ok(isa.default_call_conv())
    }
}

/// Helper: Create default flags for RISC-V 32-bit target
fn default_riscv32_flags() -> Result<Flags, GlslError> {
    use cranelift_codegen::settings::{self, Configurable};

    let mut flag_builder = settings::builder();
    flag_builder.set("is_pic", "false").map_err(|e| {
        GlslError::new(ErrorCode::E0400, format!("failed to set is_pic: {}", e))
    })?;
    flag_builder.set("use_colocated_libcalls", "false").map_err(|e| {
        GlslError::new(ErrorCode::E0400, format!("failed to set use_colocated_libcalls: {}", e))
    })?;
    flag_builder.set("enable_multi_ret_implicit_sret", "true").map_err(|e| {
        GlslError::new(ErrorCode::E0400, format!("failed to set enable_multi_ret_implicit_sret: {}", e))
    })?;

    Ok(settings::Flags::new(flag_builder))
}

/// Helper: Create default flags for host target
#[cfg(feature = "std")]
fn default_host_flags() -> Result<Flags, GlslError> {
    use cranelift_codegen::settings::{self, Configurable};

    let mut flag_builder = settings::builder();
    flag_builder.set("is_pic", "false").map_err(|e| {
        GlslError::new(ErrorCode::E0400, format!("failed to set is_pic: {}", e))
    })?;
    flag_builder.set("use_colocated_libcalls", "false").map_err(|e| {
        GlslError::new(ErrorCode::E0400, format!("failed to set use_colocated_libcalls: {}", e))
    })?;
    flag_builder.set("enable_multi_ret_implicit_sret", "true").map_err(|e| {
        GlslError::new(ErrorCode::E0400, format!("failed to set enable_multi_ret_implicit_sret: {}", e))
    })?;

    Ok(settings::Flags::new(flag_builder))
}

/// Helper: Get RISC-V 32-bit triple
fn riscv32_triple() -> target_lexicon::Triple {
    use target_lexicon::{Architecture, BinaryFormat, Environment, OperatingSystem, Riscv32Architecture, Triple, Vendor};

    Triple {
        architecture: Architecture::Riscv32(Riscv32Architecture::Riscv32imac),
        vendor: Vendor::Unknown,
        operating_system: OperatingSystem::None_,
        environment: Environment::Unknown,
        binary_format: BinaryFormat::Elf,
    }
}

/// Helper: Convert Architecture to Triple
fn triple_for_arch(arch: Architecture) -> target_lexicon::Triple {
    use target_lexicon::{BinaryFormat, Environment, OperatingSystem, Triple, Vendor};

    Triple {
        architecture: arch,
        vendor: Vendor::Unknown,
        operating_system: OperatingSystem::Unknown,
        environment: Environment::Unknown,
        binary_format: BinaryFormat::Elf,
    }
}

/// Helper: Detect host triple
#[cfg(feature = "std")]
fn detect_host_triple() -> target_lexicon::Triple {
    target_lexicon::Triple::host()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riscv32_emulator_creation() {
        let target = Target::riscv32_emulator();
        assert!(target.is_ok());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_host_jit_creation() {
        let target = Target::host_jit();
        assert!(target.is_ok());
    }

    #[test]
    fn test_isa_creation() {
        let mut target = Target::riscv32_emulator().unwrap();
        let isa = target.create_isa();
        assert!(isa.is_ok());
    }

    #[test]
    fn test_isa_caching() {
        let mut target = Target::riscv32_emulator().unwrap();
        let isa1 = target.create_isa().unwrap();
        let isa2 = target.create_isa().unwrap();
        // Should return same reference (cached)
        assert!(std::ptr::eq(isa1, isa2));
    }

    #[test]
    fn test_pointer_type() {
        let mut target = Target::riscv32_emulator().unwrap();
        let ptr_type = target.pointer_type();
        assert!(ptr_type.is_ok());
        // RISC-V 32-bit should have I32 pointer type
        assert_eq!(ptr_type.unwrap(), types::I32);
    }

    #[test]
    fn test_call_conv() {
        let mut target = Target::riscv32_emulator().unwrap();
        let call_conv = target.default_call_conv();
        assert!(call_conv.is_ok());
    }
}
```

#### `target/builder.rs`

```rust
use crate::error::{ErrorCode, GlslError};
use crate::backend2::target::Target;
use cranelift_module::{ModuleBuilder, default_libcall_names};
use cranelift_jit::JITBuilder;
use cranelift_object::ObjectBuilder;

impl Target {
    /// Create the appropriate Module builder for this target
    /// Internal implementation details are hidden - caller doesn't care about ModuleKind
    pub fn create_module_builder(&mut self) -> Result<ModuleBuilder, GlslError> {
        let isa = self.create_isa()?.clone(); // Clone owned ISA for builder
        match self {
            Target::Rv32Emu { .. } => {
                // Internally knows: ObjectModule, riscv32 triple, etc.
                ObjectBuilder::new(isa, b"module", default_libcall_names())
                    .map_err(|e| GlslError::new(ErrorCode::E0400, format!("ObjectBuilder creation failed: {}", e)))
                    .map(|b| ModuleBuilder::Object(b))
            }
            Target::HostJit { .. } => {
                // Internally knows: JITModule, host triple, etc.
                Ok(ModuleBuilder::JIT(JITBuilder::with_isa(isa, default_libcall_names())))
            }
        }
    }
}

/// Module builder enum (wraps different builder types)
pub enum ModuleBuilder {
    JIT(JITBuilder),
    Object(ObjectBuilder),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_jit_builder() {
        let mut target = Target::host_jit().unwrap();
        let builder = target.create_module_builder();
        assert!(builder.is_ok());
        match builder.unwrap() {
            ModuleBuilder::JIT(_) => {}
            _ => panic!("Expected JIT builder"),
        }
    }

    #[test]
    fn test_create_object_builder() {
        let mut target = Target::riscv32_emulator().unwrap();
        let builder = target.create_module_builder();
        assert!(builder.is_ok());
        match builder.unwrap() {
            ModuleBuilder::Object(_) => {}
            _ => panic!("Expected Object builder"),
        }
    }
}
```

### 2. Module Structure (`backend2/module/`)

#### `module/gl_func.rs`

```rust
use cranelift_codegen::ir::Signature;
use cranelift_module::FuncId;
use alloc::string::String;

/// Function metadata (doesn't store Function IR, just metadata)
pub struct GlFunc {
    pub name: String,
    pub clif_sig: Signature,
    pub func_id: FuncId,
    // Note: GLSL signature not needed for Phase 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use cranelift_codegen::ir::{types, AbiParam};
    use cranelift_codegen::isa::CallConv;

    #[test]
    fn test_gl_func_creation() {
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::I32));
        sig.returns.push(AbiParam::new(types::I32));

        // Note: FuncId creation requires a Module, so this is a minimal test
        let func = GlFunc {
            name: "test".to_string(),
            clif_sig: sig,
            func_id: FuncId::new(0), // Dummy ID for test
        };

        assert_eq!(func.name, "test");
        assert_eq!(func.clif_sig.params.len(), 1);
    }
}
```

#### `module/gl_module.rs`

```rust
use crate::backend2::target::Target;
use crate::backend2::module::gl_func::GlFunc;
use crate::error::{ErrorCode, GlslError};
use cranelift_jit::JITModule;
use cranelift_object::ObjectModule;
use cranelift_module::Module;
use hashbrown::HashMap;

/// GLSL Module - owns the actual Cranelift Module
pub struct GlModule<M: Module> {
    pub target: Target,  // Semantic target, not technical spec
    pub fns: HashMap<String, GlFunc>,
    pub module: M, // Owned Module - functions are already defined here
    // Note: source_map not needed for Phase 1
}

// Separate constructors for each Module type (Rust needs concrete types)
impl GlModule<JITModule> {
    /// Create new GlModule with JITModule from HostJit target
    pub fn new_jit(mut target: Target) -> Result<Self, GlslError> {
        match &target {
            Target::HostJit { .. } => {
                let builder = target.create_module_builder()?;
                let module = match builder {
                    crate::backend2::target::builder::ModuleBuilder::JIT(jit_builder) => JITModule::new(jit_builder),
                    _ => return Err(GlslError::new(ErrorCode::E0400, "Expected JIT builder")),
                };
                Ok(Self {
                    target,
                    fns: HashMap::new(),
                    module,
                })
            }
            _ => Err(GlslError::new(ErrorCode::E0400, "Target is not a JIT target")),
        }
    }
}

impl GlModule<ObjectModule> {
    /// Create new GlModule with ObjectModule from Rv32Emu target
    pub fn new_object(mut target: Target) -> Result<Self, GlslError> {
        match &target {
            Target::Rv32Emu { .. } => {
                let builder = target.create_module_builder()?;
                let module = match builder {
                    crate::backend2::target::builder::ModuleBuilder::Object(obj_builder) => ObjectModule::new(obj_builder),
                    _ => return Err(GlslError::new(ErrorCode::E0400, "Expected Object builder")),
                };
                Ok(Self {
                    target,
                    fns: HashMap::new(),
                    module,
                })
            }
            _ => Err(GlslError::new(ErrorCode::E0400, "Target is not an object target")),
        }
    }
}

impl<M: Module> GlModule<M> {
    /// Get function metadata by name
    pub fn get_func(&self, name: &str) -> Option<&GlFunc> {
        self.fns.get(name)
    }

    /// Get mutable reference to module (for building functions)
    pub fn module_mut(&mut self) -> &mut M {
        &mut self.module
    }
}

// Specific implementations for each Module type
impl GlModule<JITModule> {
    /// Build executable from JIT module
    /// Returns a boxed GlslExecutable trait object for generic code
    pub fn build_executable(self) -> Result<Box<dyn crate::exec::executable::GlslExecutable>, GlslError> {
        crate::backend2::codegen::build_jit_executable(self).map(|jit| Box::new(jit) as Box<dyn crate::exec::executable::GlslExecutable>)
    }
}

impl GlModule<ObjectModule> {
    /// Build executable from Object module (for emulator)
    /// Returns a boxed GlslExecutable trait object for generic code
    pub fn build_executable(self, options: &crate::backend2::codegen::emu::EmulatorOptions) -> Result<Box<dyn crate::exec::executable::GlslExecutable>, GlslError> {
        crate::backend2::codegen::build_emu_executable(self, options).map(|emu| Box::new(emu) as Box<dyn crate::exec::executable::GlslExecutable>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_create_jit_module() {
        let target = Target::host_jit().unwrap();
        let gl_module = GlModule::new_jit(target);
        assert!(gl_module.is_ok());
        let gl_module = gl_module.unwrap();
        assert_eq!(gl_module.fns.len(), 0);
    }

    #[test]
    fn test_create_object_module() {
        let target = Target::riscv32_emulator().unwrap();
        let gl_module = GlModule::new_object(target);
        assert!(gl_module.is_ok());
        let gl_module = gl_module.unwrap();
        assert_eq!(gl_module.fns.len(), 0);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_get_func_nonexistent() {
        let target = Target::host_jit().unwrap();
        let gl_module = GlModule::new_jit(target).unwrap();
        assert!(gl_module.get_func("nonexistent").is_none());
    }
}
```

### 3. Programmatic Builder (`backend2/module/builder.rs`)

Helper utilities for building functions programmatically (used by tests):

**Important**: Functions must be declared before they can be called. When building functions that call each other:

1. First, declare all functions (get FuncIds)
2. Then, define all functions (build bodies, can reference other FuncIds)

```rust
use crate::backend2::module::gl_module::GlModule;
use crate::backend2::module::gl_func::GlFunc;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{Function, Signature, FuncRef, Value};
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::AbiParam;
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::FunctionBuilder;
use cranelift_frontend::FunctionBuilderContext;
use cranelift_module::{Module, FuncId, Linkage};

/// Build a simple function programmatically
///
/// **Note**: Function must be declared before it can be called by other functions.
/// Use `declare_function` first if you need to call this function from another.
pub fn build_simple_function<M: Module>(
    gl_module: &mut GlModule<M>,
    name: &str,
    linkage: Linkage,
    sig: Signature,
    body: impl FnOnce(&mut FunctionBuilder, &mut Function) -> Result<(), GlslError>,
) -> Result<FuncId, GlslError> {
    // 1. Declare function in module
    let func_id = gl_module.module_mut().declare_function(name, linkage, &sig)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to declare function '{}': {}", name, e)))?;

    // 2. Create context and builder
    let mut ctx = gl_module.module_mut().make_context();
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    // 3. Set signature and name
    ctx.func.signature = sig.clone();
    ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32());

    // 4. Build entry block
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    // 5. Call user-provided body builder
    body(&mut builder, &mut ctx.func)?;

    // 6. Finalize and define
    builder.finalize();
    gl_module.module_mut().define_function(func_id, &mut ctx)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to define function '{}': {}", name, e)))?;
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
///
/// **Note**: The callee function must be declared before calling this function.
/// The callee should be built using `build_simple_function` or `declare_function` first.
pub fn build_call_function<M: Module>(
    gl_module: &mut GlModule<M>,
    name: &str,
    linkage: Linkage,
    sig: Signature,
    callee_name: &str,
    args_builder: impl FnOnce(&mut FunctionBuilder) -> Result<Vec<Value>, GlslError>,
) -> Result<FuncId, GlslError> {
    build_simple_function(gl_module, name, linkage, sig, |builder, func| {
        let entry_block = builder.current_block()
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "No current block"))?;

        // Get callee FuncId (must be declared already)
        let callee_func = gl_module.get_func(callee_name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("Function '{}' not found (must be declared first)", callee_name)))?;

        // Create FuncRef in this function's context
        let callee_ref = gl_module.module_mut().declare_func_in_func(callee_func.func_id, func);

        // Build arguments using user-provided builder
        let args = args_builder(builder)?;

        // Call the function
        let call_result = builder.ins().call(callee_ref, &args);
        let results = builder.inst_results(call_result);

        if results.is_empty() {
            builder.ins().return_(&[]);
        } else {
            builder.ins().return_(&[results[0]]);
        }

        Ok(())
    })
}

/// Declare a function without defining it (useful for forward declarations)
/// Returns the FuncId for later use in function calls
pub fn declare_function<M: Module>(
    gl_module: &mut GlModule<M>,
    name: &str,
    linkage: Linkage,
    sig: Signature,
) -> Result<FuncId, GlslError> {
    let func_id = gl_module.module_mut().declare_function(name, linkage, &sig)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to declare function '{}': {}", name, e)))?;

    gl_module.fns.insert(name.to_string(), GlFunc {
        name: name.to_string(),
        clif_sig: sig,
        func_id,
    });

    Ok(func_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend2::target::Target;

    #[test]
    #[cfg(feature = "std")]
    fn test_build_simple_function() {
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();

        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I32));

        let result = build_simple_function(&mut gl_module, "test", Linkage::Local, sig, |builder, _func| {
            let entry = builder.current_block().unwrap();
            let val = builder.ins().iconst(types::I32, 42);
            builder.ins().return_(&[val]);
            Ok(())
        });

        assert!(result.is_ok());
        assert!(gl_module.get_func("test").is_some());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_build_function_with_params() {
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();

        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::I32));
        sig.params.push(AbiParam::new(types::I32));
        sig.returns.push(AbiParam::new(types::I32));

        let result = build_simple_function(&mut gl_module, "add", Linkage::Local, sig, |builder, _func| {
            let entry = builder.current_block().unwrap();
            let a = builder.block_params(entry)[0];
            let b = builder.block_params(entry)[1];
            let sum = builder.ins().iadd(a, b);
            builder.ins().return_(&[sum]);
            Ok(())
        });

        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_build_call_function() {
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();

        // First, build the callee
        let mut add_sig = Signature::new(CallConv::SystemV);
        add_sig.params.push(AbiParam::new(types::I32));
        add_sig.params.push(AbiParam::new(types::I32));
        add_sig.returns.push(AbiParam::new(types::I32));

        build_simple_function(&mut gl_module, "add", Linkage::Local, add_sig, |builder, _func| {
            let entry = builder.current_block().unwrap();
            let a = builder.block_params(entry)[0];
            let b = builder.block_params(entry)[1];
            let sum = builder.ins().iadd(a, b);
            builder.ins().return_(&[sum]);
            Ok(())
        }).unwrap();

        // Then, build the caller
        let mut main_sig = Signature::new(CallConv::SystemV);
        main_sig.returns.push(AbiParam::new(types::I32));

        let result = build_call_function(&mut gl_module, "main", Linkage::Export, main_sig, "add", |builder| {
            let ten = builder.ins().iconst(types::I32, 10);
            let twenty = builder.ins().iconst(types::I32, 20);
            Ok(vec![ten, twenty])
        });

        assert!(result.is_ok());
        assert!(gl_module.get_func("main").is_some());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_declare_function() {
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();

        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I32));

        let result = declare_function(&mut gl_module, "forward", Linkage::Local, sig);
        assert!(result.is_ok());
        assert!(gl_module.get_func("forward").is_some());
    }
}
```

### 4. Codegen (`backend2/codegen/`)

The codegen layer provides functions to build executables from `GlModule`. These are called by `GlModule::build_executable()` methods.

#### `codegen/jit.rs`

```rust
use crate::backend2::module::gl_module::GlModule;
use crate::exec::jit::GlslJitModule;
use crate::error::{ErrorCode, GlslError};
use cranelift_jit::JITModule;
use hashbrown::HashMap;

/// Build JIT executable from GlModule<JITModule>
/// Called by GlModule<JITModule>::build_executable()
pub fn build_jit_executable(
    mut gl_module: GlModule<JITModule>,
) -> Result<GlslJitModule, GlslError> {
    // 1. Finalize definitions
    gl_module.module.finalize_definitions()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to finalize definitions: {}", e)))?;

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

    // 4. Get target properties (requires mutable reference for ISA caching)
    let call_conv = gl_module.target.default_call_conv()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to get call conv: {}", e)))?;
    let pointer_type = gl_module.target.pointer_type()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to get pointer type: {}", e)))?;

    // 5. Create GlslJitModule
    Ok(GlslJitModule {
        jit_module: gl_module.module,
        function_ptrs,
        signatures,
        cranelift_signatures,
        call_conv,
        pointer_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend2::target::Target;
    use crate::backend2::module::builder::build_simple_function;
    use cranelift_codegen::ir::{types, AbiParam, Signature};
    use cranelift_codegen::isa::CallConv;
    use cranelift_module::Linkage;

    #[test]
    #[cfg(feature = "std")]
    fn test_build_jit_executable() {
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();

        // Build a simple function
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I32));

        build_simple_function(&mut gl_module, "main", Linkage::Export, sig, |builder, _func| {
            let val = builder.ins().iconst(types::I32, 42);
            builder.ins().return_(&[val]);
            Ok(())
        }).unwrap();

        // Build executable
        let executable = build_jit_executable(gl_module);
        assert!(executable.is_ok());
        let executable = executable.unwrap();
        assert!(executable.function_ptrs.contains_key("main"));
    }
}
```

#### `codegen/emu.rs`

```rust
use crate::backend2::module::gl_module::GlModule;
use crate::exec::emu::GlslEmulatorModule;
use crate::error::{ErrorCode, GlslError};
use crate::frontend::src_loc::GlSourceMap;
use crate::frontend::src_loc_manager::SourceLocManager;
use cranelift_object::ObjectModule;
use hashbrown::HashMap;

/// Emulator execution options
#[derive(Debug, Clone)]
pub struct EmulatorOptions {
    /// Maximum memory size in bytes (RAM)
    pub max_memory: usize,
    /// Stack size in bytes
    pub stack_size: usize,
    /// Maximum instruction count before timeout
    pub max_instructions: u64,
}

/// Build emulator executable from GlModule<ObjectModule>
/// Called by GlModule<ObjectModule>::build_executable()
#[cfg(feature = "emulator")]
pub fn build_emu_executable(
    gl_module: GlModule<ObjectModule>,
    options: &EmulatorOptions,
) -> Result<GlslEmulatorModule, GlslError> {
    use lp_riscv_tools::emu::emulator::{Riscv32Emulator, Gpr};

    // 1. Finish module and get object file
    let product = gl_module.module.finish();
    let elf_bytes = product.emit()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to emit ELF: {}", e)))?;

    // 2. Load ELF and find main address
    // Note: These functions should be imported from existing emulator utilities
    // See exec/emu.rs for reference implementation
    let load_info = load_elf(&elf_bytes)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to load ELF: {}", e)))?;
    let obj = object::File::parse(&elf_bytes[..])
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to parse ELF: {}", e)))?;

    // Find main function address (implementation depends on ELF loading utilities)
    // For Phase 1, assume main is at a known offset or use symbol lookup
    let main_address = find_symbol_address(&obj, "main", load_info.text_section_base)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to find main address: {}", e)))?;

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
    // Note: Some fields are required by GlslEmulatorModule but not needed for Phase 1
    // Use minimal/default values for now
    Ok(GlslEmulatorModule {
        emulator,
        signatures,
        cranelift_signatures,
        binary,
        main_address,
        transformed_clif: None,  // Phase 1: not needed
        original_clif: None,     // Phase 1: not needed
        vcode: None,             // Phase 1: not needed
        disassembly: None,       // Phase 1: not needed
        trap_source_info: Vec::new(),  // Phase 1: empty
        source_text: None,       // Phase 1: not needed
        source_file_path: None,  // Phase 1: not needed
        source_loc_manager: SourceLocManager::new(),
        source_map: GlSourceMap::new(),
        next_buffer_addr: 0x80000000,  // Default RAM start
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend2::target::Target;
    use crate::backend2::module::builder::build_simple_function;
    use cranelift_codegen::ir::{types, AbiParam, Signature};
    use cranelift_codegen::isa::CallConv;
    use cranelift_module::Linkage;

    #[test]
    #[cfg(feature = "emulator")]
    fn test_build_emu_executable() {
        let target = Target::riscv32_emulator().unwrap();
        let mut gl_module = GlModule::new_object(target).unwrap();

        // Build a simple function
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I32));

        build_simple_function(&mut gl_module, "main", Linkage::Export, sig, |builder, _func| {
            let val = builder.ins().iconst(types::I32, 42);
            builder.ins().return_(&[val]);
            Ok(())
        }).unwrap();

        // Build executable
        let options = EmulatorOptions {
            max_memory: 1024 * 1024,
            stack_size: 64 * 1024,
            max_instructions: 10000,
        };

        let executable = build_emu_executable(gl_module, &options);
        assert!(executable.is_ok());
        let executable = executable.unwrap();
        assert_eq!(executable.main_address, 0); // Will be set by find_symbol_address
    }
}
```

## File Structure

```
backend2/
├── mod.rs                 # Re-exports
├── target/
│   ├── mod.rs
│   ├── target.rs         # Target enum: Rv32Emu, HostJit (semantic)
│   └── builder.rs        # create_module_builder (internal)
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

## Unit Test Strategy

All tests are **unit tests** that test components in isolation:

1. **Target tests**: Test Target creation, ISA creation, caching, pointer types, call conventions
2. **GlModule tests**: Test module creation, function metadata storage
3. **Builder tests**: Test function building utilities independently
4. **Codegen tests**: Test executable building with minimal functions

Tests use `#[cfg(test)]` modules within each source file, not separate integration test files.

## Validation

Phase 1 is complete when:

1. ✅ `Target::host_jit()` and `Target::riscv32_emulator()` work
2. ✅ Can create `GlModule<JITModule>` and `GlModule<ObjectModule>` (target handles details)
3. ✅ Can build functions programmatically with cross-function calls
4. ✅ `build_jit_executable()` produces working `GlslJitModule`
5. ✅ `build_emu_executable()` produces working `GlslEmulatorModule`
6. ✅ All unit tests pass for both backends

## Next Steps (Phase 2)

After Phase 1 is validated:

1. Add transform pipeline infrastructure
2. Port fixed32 transform to work with GlModule
3. Add GLSL frontend integration
4. Migrate existing tests to new architecture
5. Add integration tests for end-to-end validation
