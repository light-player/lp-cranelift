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
    #[allow(unused)]
    pub fn build_executable(self) -> Result<alloc::boxed::Box<dyn crate::exec::executable::GlslExecutable>, GlslError> {
        crate::backend2::codegen::jit::build_jit_executable(self).map(|jit| alloc::boxed::Box::new(jit) as alloc::boxed::Box<dyn crate::exec::executable::GlslExecutable>)
    }
}

impl GlModule<ObjectModule> {
    /// Build executable from Object module (for emulator)
    /// Returns a boxed GlslExecutable trait object for generic code
    #[allow(unused)]
    pub fn build_executable(self, options: &crate::backend2::codegen::emu::EmulatorOptions) -> Result<alloc::boxed::Box<dyn crate::exec::executable::GlslExecutable>, GlslError> {
        crate::backend2::codegen::emu::build_emu_executable(self, options).map(|emu| alloc::boxed::Box::new(emu) as alloc::boxed::Box<dyn crate::exec::executable::GlslExecutable>)
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
