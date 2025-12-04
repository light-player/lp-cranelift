#[cfg(feature = "std")]
use crate::jit::JIT;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::String;

/// High-level compiler interface
#[cfg(feature = "std")]
pub struct Compiler {
    pub jit: JIT,
}

#[cfg(feature = "std")]
impl Compiler {
    pub fn new() -> Self {
        Self { jit: JIT::new() }
    }

    /// Set the fixed-point format for float-to-fixed transformation
    pub fn set_fixed_point_format(&mut self, format: Option<crate::FixedPointFormat>) {
        self.jit.fixed_point_format = format;
    }

    /// Compile GLSL shader that returns i32
    pub fn compile_int(&mut self, source: &str) -> Result<fn() -> i32, String> {
        let code_ptr = self.jit.compile(source)?;
        Ok(unsafe { std::mem::transmute(code_ptr) })
    }

    /// Compile GLSL shader that returns bool
    pub fn compile_bool(&mut self, source: &str) -> Result<fn() -> i8, String> {
        let code_ptr = self.jit.compile(source)?;
        Ok(unsafe { std::mem::transmute(code_ptr) })
    }

    /// Compile GLSL shader that returns f32
    pub fn compile_float(&mut self, source: &str) -> Result<fn() -> f32, String> {
        let code_ptr = self.jit.compile(source)?;
        Ok(unsafe { std::mem::transmute(code_ptr) })
    }

    /// Compile to CLIF IR for debugging/testing
    pub fn compile_to_clif(&mut self, source: &str) -> Result<String, String> {
        self.jit.compile_to_clif(source)
    }
}

#[cfg(feature = "std")]
impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

// no_std compiler - compiles GLSL to machine code bytes without JIT module
#[cfg(not(feature = "std"))]
pub struct Compiler {
    /// Optional fixed-point format for float-to-fixed transformation
    pub fixed_point_format: Option<crate::FixedPointFormat>,
}

#[cfg(not(feature = "std"))]
impl Compiler {
    pub fn new() -> Self {
        Self {
            fixed_point_format: None,
        }
    }
    
    /// Compile GLSL source to machine code bytes for a specific ISA
    /// Returns the compiled machine code that can be executed
    pub fn compile_to_code(
        &mut self,
        source: &str,
        isa: &dyn cranelift_codegen::isa::TargetIsa,
    ) -> Result<alloc::vec::Vec<u8>, String> {
        use cranelift_codegen::{Context, ir::AbiParam, control::ControlPlane, ir::InstBuilder};
        use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
        
        // 1. Parse GLSL
        let shader = glsl::parser::Parse::parse(source)
            .map_err(|e| alloc::format!("Parse error: {:?}", e))?;

        // 2. Semantic analysis
        let typed_ast = crate::semantic::analyze(&shader)?;

        // 3. Setup Cranelift context
        let mut ctx = Context::new();
        let return_type = typed_ast.main_function.return_type.to_cranelift_type();
        if !matches!(typed_ast.main_function.return_type, crate::semantic::types::Type::Void) {
            ctx.func.signature.returns.push(AbiParam::new(return_type));
        }

        // 4. Build IR
        let mut builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // 5. Translate GLSL - we need a minimal module stub
        struct DummyModule;
        impl cranelift_module::Module for DummyModule {
            fn isa(&self) -> &dyn cranelift_codegen::isa::TargetIsa {
                unimplemented!("DummyModule::isa should not be called")
            }
            fn declarations(&self) -> &cranelift_module::ModuleDeclarations {
                unimplemented!("DummyModule::declarations should not be called")
            }
            fn declare_function(
                &mut self,
                _name: &str,
                _linkage: cranelift_module::Linkage,
                _signature: &cranelift_codegen::ir::Signature,
            ) -> cranelift_module::ModuleResult<cranelift_module::FuncId> {
                unimplemented!("DummyModule::declare_function should not be called")
            }
            fn declare_data(
                &mut self,
                _name: &str,
                _linkage: cranelift_module::Linkage,
                _writable: bool,
                _tls: bool,
            ) -> cranelift_module::ModuleResult<cranelift_module::DataId> {
                unimplemented!("DummyModule::declare_data should not be called")
            }
            fn define_function_bytes(
                &mut self,
                _id: cranelift_module::FuncId,
                _alignment: u64,
                _bytes: &[u8],
                _relocs: &[cranelift_module::ModuleReloc],
            ) -> cranelift_module::ModuleResult<()> {
                unimplemented!("DummyModule::define_function_bytes should not be called")
            }
            fn define_function(
                &mut self,
                _id: cranelift_module::FuncId,
                _ctx: &mut cranelift_codegen::Context,
            ) -> cranelift_module::ModuleResult<()> {
                unimplemented!("DummyModule::define_function should not be called")
            }
            fn define_data(
                &mut self,
                _id: cranelift_module::DataId,
                _data: &cranelift_module::DataDescription,
            ) -> cranelift_module::ModuleResult<()> {
                unimplemented!("DummyModule::define_data should not be called")
            }
            fn declare_anonymous_function(
                &mut self,
                _signature: &cranelift_codegen::ir::Signature,
            ) -> cranelift_module::ModuleResult<cranelift_module::FuncId> {
                unimplemented!("DummyModule::declare_anonymous_function should not be called")
            }
            fn declare_anonymous_data(
                &mut self,
                _writable: bool,
                _tls: bool,
            ) -> cranelift_module::ModuleResult<cranelift_module::DataId> {
                unimplemented!("DummyModule::declare_anonymous_data should not be called")
            }
            fn define_function_with_control_plane(
                &mut self,
                _id: cranelift_module::FuncId,
                _ctx: &mut cranelift_codegen::Context,
                _ctrl_plane: &mut cranelift_codegen::control::ControlPlane,
            ) -> cranelift_module::ModuleResult<()> {
                unimplemented!("DummyModule::define_function_with_control_plane should not be called")
            }
        }
        
        let mut dummy_module = DummyModule;
        let mut codegen_ctx = crate::codegen::context::CodegenContext::new(builder, &mut dummy_module);

        // Translate main function body
        for stmt in &typed_ast.main_function.body {
            codegen_ctx.translate_statement(stmt)?;
        }

        // Add default return
        let return_val = codegen_ctx.builder.ins().iconst(return_type, 0);
        codegen_ctx.builder.ins().return_(&[return_val]);
        codegen_ctx.builder.finalize();

        // 5.5. Apply fixed-point transformation if enabled
        if let Some(format) = self.fixed_point_format {
            crate::transform::fixed_point::convert_floats_to_fixed(&mut ctx.func, format)
                .map_err(|e| alloc::format!("Fixed-point transformation error: {}", e))?;
        }

        // 6. Compile to machine code
        let mut ctrl_plane = ControlPlane::default();
        let code_info = ctx
            .compile(isa, &mut ctrl_plane)
            .map_err(|e| alloc::format!("Codegen failed: {:?}", e))?;

        Ok(code_info.buffer.data().to_vec())
    }
}

