#[cfg(feature = "std")]
use crate::jit::JIT;

#[cfg(feature = "std")]
use std::{boxed::Box, string::String};
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};

#[cfg(feature = "std")]
use lp_jit_util::call_structreturn;

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

    /// Compile GLSL shader that returns vec2 (2 f32s)
    pub fn compile_vec2(&mut self, source: &str) -> Result<Box<dyn Fn() -> (f32, f32)>, String> {
        // Get calling convention and pointer type before compilation
        let call_conv = self.jit.call_conv();
        let pointer_type = self.jit.pointer_type();
        let code_ptr = self.jit.compile(source)?;
        Ok(Box::new(move || {
            let mut buffer = [0.0f32; 2];
            unsafe {
                call_structreturn(
                    code_ptr,
                    buffer.as_mut_ptr(),
                    2,
                    call_conv,
                    pointer_type,
                ).unwrap_or_else(|e| {
                    panic!("Internal error: StructReturn call failed for vec2: {}. This indicates a codegen bug.", e);
                });
            }
            (buffer[0], buffer[1])
        }))
    }

    /// Compile GLSL shader that returns vec3 (3 f32s)
    pub fn compile_vec3(&mut self, source: &str) -> Result<Box<dyn Fn() -> (f32, f32, f32)>, String> {
        // Get calling convention and pointer type before compilation
        let call_conv = self.jit.call_conv();
        let pointer_type = self.jit.pointer_type();
        let code_ptr = self.jit.compile(source)?;
        Ok(Box::new(move || {
            let mut buffer = [0.0f32; 3];
            unsafe {
                call_structreturn(
                    code_ptr,
                    buffer.as_mut_ptr(),
                    3,
                    call_conv,
                    pointer_type,
                ).unwrap_or_else(|e| {
                    panic!("Internal error: StructReturn call failed for vec3: {}. This indicates a codegen bug.", e);
                });
            }
            (buffer[0], buffer[1], buffer[2])
        }))
    }

    /// Compile GLSL shader that returns vec4 (4 f32s)
    pub fn compile_vec4(&mut self, source: &str) -> Result<Box<dyn Fn() -> (f32, f32, f32, f32)>, String> {
        // Get calling convention and pointer type before compilation
        let call_conv = self.jit.call_conv();
        let pointer_type = self.jit.pointer_type();
        let code_ptr = self.jit.compile(source)?;
        Ok(Box::new(move || {
            let mut buffer = [0.0f32; 4];
            unsafe {
                call_structreturn(
                    code_ptr,
                    buffer.as_mut_ptr(),
                    4,
                    call_conv,
                    pointer_type,
                ).expect("StructReturn call failed");
            }
            (buffer[0], buffer[1], buffer[2], buffer[3])
        }))
    }

    /// Compile GLSL shader that returns mat2 (4 f32s, column-major)
    pub fn compile_mat2(&mut self, source: &str) -> Result<Box<dyn Fn() -> (f32, f32, f32, f32)>, String> {
        // Get calling convention and pointer type before compilation
        let call_conv = self.jit.call_conv();
        let pointer_type = self.jit.pointer_type();
        let code_ptr = self.jit.compile(source)?;
        Ok(Box::new(move || {
            let mut buffer = [0.0f32; 4];
            unsafe {
                call_structreturn(
                    code_ptr,
                    buffer.as_mut_ptr(),
                    4,
                    call_conv,
                    pointer_type,
                ).unwrap_or_else(|e| {
                    panic!("Internal error: StructReturn call failed for mat2: {}. This indicates a codegen bug.", e);
                });
            }
            (buffer[0], buffer[1], buffer[2], buffer[3])
        }))
    }

    /// Compile GLSL shader that returns mat3 (9 f32s, column-major)
    pub fn compile_mat3(&mut self, source: &str) -> Result<Box<dyn Fn() -> (f32, f32, f32, f32, f32, f32, f32, f32, f32)>, String> {
        // Get calling convention and pointer type before compilation
        let call_conv = self.jit.call_conv();
        let pointer_type = self.jit.pointer_type();
        let code_ptr = self.jit.compile(source)?;
        Ok(Box::new(move || {
            let mut buffer = [0.0f32; 9];
            unsafe {
                call_structreturn(
                    code_ptr,
                    buffer.as_mut_ptr(),
                    9,
                    call_conv,
                    pointer_type,
                ).unwrap_or_else(|e| {
                    panic!("Internal error: StructReturn call failed for mat3: {}. This indicates a codegen bug.", e);
                });
            }
            (buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7], buffer[8])
        }))
    }

    /// Compile GLSL shader that returns mat4 (16 f32s, column-major)
    pub fn compile_mat4(&mut self, source: &str) -> Result<Box<dyn Fn() -> (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32)>, String> {
        // Get calling convention and pointer type before compilation
        let call_conv = self.jit.call_conv();
        let pointer_type = self.jit.pointer_type();
        let code_ptr = self.jit.compile(source)?;
        Ok(Box::new(move || {
            let mut buffer = [0.0f32; 16];
            unsafe {
                call_structreturn(
                    code_ptr,
                    buffer.as_mut_ptr(),
                    16,
                    call_conv,
                    pointer_type,
                ).unwrap_or_else(|e| {
                    panic!("Internal error: StructReturn call failed for mat4: {}. This indicates a codegen bug.", e);
                });
            }
            (
                buffer[0], buffer[1], buffer[2], buffer[3],
                buffer[4], buffer[5], buffer[6], buffer[7],
                buffer[8], buffer[9], buffer[10], buffer[11],
                buffer[12], buffer[13], buffer[14], buffer[15],
            )
        }))
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
        self.compile_to_code_detailed(source, isa).map_err(|e| String::from(e))
    }
    
    /// Compile GLSL source to machine code with detailed error information
    pub fn compile_to_code_detailed(
        &mut self,
        source: &str,
        isa: &dyn cranelift_codegen::isa::TargetIsa,
    ) -> Result<alloc::vec::Vec<u8>, crate::error::GlslError> {
        use cranelift_codegen::{Context, ir::AbiParam, control::ControlPlane, ir::InstBuilder};
        use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
        use crate::error::GlslError;
        
        // 1. Parse and analyze GLSL
        let semantic_result = crate::pipeline::CompilationPipeline::parse_and_analyze(source)?;
        let typed_ast = semantic_result.typed_ast;

        // 3. Setup Cranelift context
        use crate::codegen::signature::SignatureBuilder;
        let mut ctx = Context::new();
        let triple = isa.triple();
        let mut sig = SignatureBuilder::new_with_triple(triple);
        let pointer_type = isa.pointer_type();
        SignatureBuilder::add_return_type(&mut sig, &typed_ast.main_function.return_type, pointer_type);
        ctx.func.signature = sig;

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
            crate::transform::fixed_point::convert_floats_to_fixed(&mut ctx.func, format)?;
        }

        // 6. Compile to machine code
        let mut ctrl_plane = ControlPlane::default();
        let code_info = ctx
            .compile(isa, &mut ctrl_plane)
            .map_err(|e| GlslError::new(ErrorCode::E0400, alloc::format!("code generation failed: {:?}", e)))?;

        Ok(code_info.buffer.data().to_vec())
    }
}

