//! Immutable CLIF module representation holding all functions before linking/compilation.

use crate::error::GlslError;
use crate::semantic::functions::FunctionRegistry;
use cranelift_codegen::ir::Function;
use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_module::{FuncId, Linkage, Module};
use hashbrown::HashMap;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

/// Immutable module representation holding CLIF IR functions before linking/compilation.
///
/// This structure holds all functions (user functions and main) in CLIF IR form,
/// along with metadata needed for later compilation or transformation.
pub struct ClifModule {
    user_functions: HashMap<String, Function>,
    main_function: Function,
    function_registry: FunctionRegistry,
    source_text: String,
    isa: OwnedTargetIsa,
}

impl ClifModule {
    /// Create a new builder for constructing a ClifModule
    pub fn builder() -> ClifModuleBuilder {
        ClifModuleBuilder::new()
    }

    /// Get all user-defined functions
    pub fn user_functions(&self) -> &HashMap<String, Function> {
        &self.user_functions
    }

    /// Get the main function
    pub fn main_function(&self) -> &Function {
        &self.main_function
    }

    /// Get the function registry
    pub fn function_registry(&self) -> &FunctionRegistry {
        &self.function_registry
    }

    /// Get the source text
    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    /// Get the ISA as a reference
    pub fn isa(&self) -> &dyn cranelift_codegen::isa::TargetIsa {
        self.isa.as_ref()
    }

    /// Take ownership of the ISA (consumes self)
    pub fn into_isa(self) -> OwnedTargetIsa {
        self.isa
    }

    /// Get the target configuration (convenience method)
    pub fn target_config(&self) -> cranelift_codegen::isa::TargetFrontendConfig {
        self.isa.frontend_config()
    }

    /// Link all functions from this module into a Cranelift Module (JITModule, ObjectModule, etc.)
    /// Returns a mapping of function names to their FuncIds in the target module
    pub fn link_into<M: Module>(
        &self,
        module: &mut M,
        main_linkage: Linkage,
    ) -> Result<HashMap<String, FuncId>, GlslError> {
        use crate::error::{ErrorCode, GlslError};

        let mut name_to_id = HashMap::new();

        // Declare and define all user functions
        for (name, func) in &self.user_functions {
            let func_id = module
                .declare_function(name, Linkage::Local, &func.signature)
                .map_err(|e| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!("failed to declare function '{}': {}", name, e),
                    )
                })?;
            let mut ctx = module.make_context();
            ctx.func = func.clone();
            module.define_function(func_id, &mut ctx).map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to define function '{}': {}", name, e),
                )
            })?;
            module.clear_context(&mut ctx);
            name_to_id.insert(name.clone(), func_id);
        }

        // Declare and define main function
        let main_id = module
            .declare_function("main", main_linkage, &self.main_function.signature)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to declare main function: {}", e),
                )
            })?;
        let mut ctx = module.make_context();
        ctx.func = self.main_function.clone();
        module.define_function(main_id, &mut ctx).map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!("failed to define main function: {}", e),
            )
        })?;
        module.clear_context(&mut ctx);
        name_to_id.insert(String::from("main"), main_id);

        Ok(name_to_id)
    }
}

/// Builder for constructing a ClifModule
pub struct ClifModuleBuilder {
    user_functions: HashMap<String, Function>,
    main_function: Option<Function>,
    function_registry: Option<FunctionRegistry>,
    source_text: Option<String>,
    isa: Option<OwnedTargetIsa>,
}

impl ClifModuleBuilder {
    fn new() -> Self {
        Self {
            user_functions: HashMap::new(),
            main_function: None,
            function_registry: None,
            source_text: None,
            isa: None,
        }
    }

    /// Add a single user function
    pub fn add_user_function(mut self, name: String, func: Function) -> Self {
        self.user_functions.insert(name, func);
        self
    }

    /// Add multiple user functions
    pub fn add_user_functions(mut self, functions: HashMap<String, Function>) -> Self {
        self.user_functions.extend(functions);
        self
    }

    /// Set the main function
    pub fn set_main_function(mut self, func: Function) -> Self {
        self.main_function = Some(func);
        self
    }

    /// Set the function registry
    pub fn set_function_registry(mut self, registry: FunctionRegistry) -> Self {
        self.function_registry = Some(registry);
        self
    }

    /// Set the source text
    pub fn set_source_text(mut self, text: String) -> Self {
        self.source_text = Some(text);
        self
    }

    /// Set the ISA
    pub fn set_isa(mut self, isa: OwnedTargetIsa) -> Self {
        self.isa = Some(isa);
        self
    }

    /// Build the ClifModule
    pub fn build(self) -> Result<ClifModule, GlslError> {
        use crate::error::{ErrorCode, GlslError};

        let main_function = self
            .main_function
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "main function not set"))?;

        let function_registry = self
            .function_registry
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "function registry not set"))?;

        let source_text = self
            .source_text
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "source text not set"))?;

        let isa = self
            .isa
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "ISA not set"))?;

        Ok(ClifModule {
            user_functions: self.user_functions,
            main_function,
            function_registry,
            source_text,
            isa,
        })
    }
}

