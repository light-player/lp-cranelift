//! GLSL Module - owns the actual Cranelift Module

use crate::backend2::target::Target;
use crate::backend2::module::gl_func::GlFunc;
use crate::error::{ErrorCode, GlslError};
use cranelift_jit::JITModule;
use cranelift_object::ObjectModule;
use cranelift_module::Module;
use hashbrown::HashMap;
use alloc::string::String;

/// GLSL Module - owns the actual Cranelift Module
pub struct GlModule<M: Module> {
    pub target: Target,  // Semantic target, not technical spec
    pub fns: HashMap<String, GlFunc>,
    module: M, // PRIVATE - only accessible via internal methods
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

    /// Create new GlModule with same target (for transformations)
    pub fn new_with_target(target: Target) -> Result<Self, GlslError> {
        Self::new_jit(target)
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

    /// Create new GlModule with same target (for transformations)
    pub fn new_with_target(target: Target) -> Result<Self, GlslError> {
        Self::new_object(target)
    }
}

impl<M: Module> GlModule<M> {
    /// Get function metadata by name
    pub fn get_func(&self, name: &str) -> Option<&GlFunc> {
        self.fns.get(name)
    }

    /// Add a function to this module
    ///
    /// Declares the function in the Module and stores the Function IR.
    /// The function is NOT compiled yet - that happens in build_executable().
    ///
    /// Validates that the Function signature matches the provided signature.
    pub fn add_function(
        &mut self,
        name: &str,
        linkage: cranelift_module::Linkage,
        sig: cranelift_codegen::ir::Signature,
        func: cranelift_codegen::ir::Function,
    ) -> Result<cranelift_module::FuncId, GlslError> {
        // Validate signature matches
        if func.signature != sig {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!("Function signature mismatch for '{}'", name),
            ));
        }

        // Declare in Module
        let func_id = self.module
            .declare_function(name, linkage, &sig)
            .map_err(|e| GlslError::new(
                ErrorCode::E0400,
                format!("Failed to declare function '{}': {}", name, e),
            ))?;

        // Store Function IR
        self.fns.insert(String::from(name), GlFunc {
            name: String::from(name),
            clif_sig: sig,
            func_id,
            function: func,
        });

        Ok(func_id)
    }

    /// Declare a function without providing the body yet (forward declaration)
    ///
    /// Useful for cross-function calls where the callee is defined later.
    /// Note: The function must be defined later using `add_function` with the same name.
    pub fn declare_function(
        &mut self,
        name: &str,
        linkage: cranelift_module::Linkage,
        sig: cranelift_codegen::ir::Signature,
    ) -> Result<cranelift_module::FuncId, GlslError> {
        // Declare in Module
        let func_id = self.module
            .declare_function(name, linkage, &sig)
            .map_err(|e| GlslError::new(
                ErrorCode::E0400,
                format!("Failed to declare function '{}': {}", name, e),
            ))?;

        // Create placeholder Function with signature
        let mut placeholder_func = cranelift_codegen::ir::Function::new();
        placeholder_func.signature = sig.clone();

        // Store placeholder
        self.fns.insert(String::from(name), GlFunc {
            name: String::from(name),
            clif_sig: sig,
            func_id,
            function: placeholder_func,
        });

        Ok(func_id)
    }

    /// Internal: Get mutable access to Module
    ///
    /// **WARNING**: This is internal-only. Do not use outside of GlModule implementation.
    /// The Module should only be accessed through public builder methods.
    #[doc(hidden)]
    pub(crate) fn module_mut_internal(&mut self) -> &mut M {
        &mut self.module
    }

    /// Internal: Get immutable access to Module (for codegen)
    #[doc(hidden)]
    pub(crate) fn module_internal(&self) -> &M {
        &self.module
    }

    /// Internal helper for apply_transform - contains common logic
    fn apply_transform_impl<T: crate::backend2::transform::pipeline::Transform>(
        self,
        transform: T,
        mut new_module: Self,
    ) -> Result<Self, GlslError> {
        use crate::backend2::transform::pipeline::{Transform, TransformContext};
        use cranelift_codegen::ir::FuncRef;
        use cranelift_module::Linkage;

        // 1. Transform all function signatures and create FuncRef mappings
        let mut func_ref_map = hashbrown::HashMap::new();
        for (name, gl_func) in &self.fns {
            let new_sig = transform.transform_signature(&gl_func.clif_sig);
            // Determine linkage - for now, use Local (can be enhanced later)
            let linkage = Linkage::Local;
            let func_id = new_module.module_mut_internal()
                .declare_function(name, linkage, &new_sig)
                .map_err(|e| GlslError::new(
                    ErrorCode::E0400,
                    format!("Failed to declare function '{}' in transformed module: {}", name, e),
                ))?;
            // Create FuncRef for cross-function calls
            let mut temp_func = cranelift_codegen::ir::Function::new();
            temp_func.signature = new_sig.clone();
            let func_ref = new_module.module_mut_internal()
                .declare_func_in_func(func_id, &mut temp_func);
            func_ref_map.insert(name.clone(), func_ref);
        }

        // 2. Transform function bodies
        for (name, gl_func) in self.fns {
            let mut transform_ctx = TransformContext {
                module: &mut new_module,
                func_ref_map: func_ref_map.clone(),
            };
            let transformed_func = transform.transform_function(
                &gl_func.function,
                &mut transform_ctx,
            )?;

            // Use public API to add transformed function
            let new_sig = transform.transform_signature(&gl_func.clif_sig);
            // Determine linkage - for now, use Local (can be enhanced later)
            let linkage = Linkage::Local;
            // Remove the placeholder that was created during declaration
            new_module.fns.remove(&name);
            new_module.add_function(&name, linkage, new_sig, transformed_func)?;
        }

        Ok(new_module)
    }
}

// Specific implementations for each Module type
impl GlModule<JITModule> {
    /// Build executable from JIT module
    /// Returns a boxed GlslExecutable trait object for generic code
    #[allow(unused)]
    pub fn build_executable(self) -> Result<alloc::boxed::Box<dyn crate::exec::executable::GlslExecutable>, GlslError> {
        crate::backend2::codegen::jit::build_jit_executable(self).map(|jit| alloc::boxed::Box::new(jit) as alloc::boxed::Box<dyn crate::exec::executable::GlslExecutable>)
    }

    /// Extract the module (consumes self)
    /// Internal use only - for codegen
    pub(crate) fn into_module(self) -> JITModule {
        self.module
    }

    /// Apply a transform to all functions in this module
    ///
    /// Consumes this GlModule and produces a new GlModule with transformed functions.
    /// Neither module has functions defined (compiled) yet.
    pub fn apply_transform<T: crate::backend2::transform::pipeline::Transform>(
        self,
        transform: T,
    ) -> Result<Self, GlslError> {
        let target = self.target.clone();
        let new_module = Self::new_with_target(target)?;
        self.apply_transform_impl(transform, new_module)
    }
}

impl GlModule<ObjectModule> {
    /// Build executable from Object module (for emulator)
    /// Returns a boxed GlslExecutable trait object for generic code
    #[allow(unused)]
    pub fn build_executable(self, options: &crate::backend2::codegen::emu::EmulatorOptions) -> Result<alloc::boxed::Box<dyn crate::exec::executable::GlslExecutable>, GlslError> {
        crate::backend2::codegen::emu::build_emu_executable(self, options).map(|emu| alloc::boxed::Box::new(emu) as alloc::boxed::Box<dyn crate::exec::executable::GlslExecutable>)
    }

    /// Extract the module (consumes self)
    /// Internal use only - for codegen
    pub(crate) fn into_module(self) -> ObjectModule {
        self.module
    }

    /// Apply a transform to all functions in this module
    ///
    /// Consumes this GlModule and produces a new GlModule with transformed functions.
    /// Neither module has functions defined (compiled) yet.
    pub fn apply_transform<T: crate::backend2::transform::pipeline::Transform>(
        self,
        transform: T,
    ) -> Result<Self, GlslError> {
        let target = self.target.clone();
        let new_module = Self::new_with_target(target)?;
        self.apply_transform_impl(transform, new_module)
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
