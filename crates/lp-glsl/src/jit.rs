#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::format;
use cranelift_codegen::Context as CodegenContext;
use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::settings::Configurable;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use hashbrown::HashMap;
#[cfg(feature = "std")]
use std::format;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub struct JIT {
    builder_context: FunctionBuilderContext,
    ctx: CodegenContext,
    #[allow(dead_code)] // Will be used in future phases
    data_description: DataDescription,
    module: JITModule,
    function_counter: usize,
    pub fixed_point_format: Option<crate::FixedPointFormat>,
}

impl Default for JIT {
    fn default() -> Self {
        Self::new()
    }
}

impl JIT {
    pub fn new() -> Self {
        use cranelift_codegen::settings;

        let mut flag_builder = settings::builder();
        flag_builder.set("is_pic", "false").expect("set is_pic");
        flag_builder
            .set("use_colocated_libcalls", "false")
            .expect("set use_colocated_libcalls");

        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_description: DataDescription::new(),
            module,
            function_counter: 0,
            fixed_point_format: None,
        }
    }

    /// Compile GLSL source to machine code and return function pointer
    pub fn compile(&mut self, glsl_source: &str) -> Result<*const u8, String> {
        self.compile_detailed(glsl_source)
            .map_err(|e| e.to_simple_string())
    }

    /// Compile with detailed error information
    pub fn compile_detailed(
        &mut self,
        glsl_source: &str,
    ) -> Result<*const u8, crate::error::GlslError> {
        use crate::error::{ErrorCode, GlslError};
        // Clear the context for a fresh compilation
        self.ctx.clear();

        // 1. Parse and analyze GLSL
        let semantic_result = crate::pipeline::CompilationPipeline::parse_and_analyze(glsl_source)?;

        // 2. Generate Cranelift IR
        self.translate(semantic_result.typed_ast, semantic_result.source)?;

        // 3.5. Apply fixed-point transformation if enabled
        if let Some(format) = self.fixed_point_format {
            crate::transform::fixed_point::convert_floats_to_fixed(&mut self.ctx.func, format)?;
        }

        // 4. Verify the function
        if let Err(e) = cranelift_codegen::verify_function(&self.ctx.func, self.module.isa()) {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!("verification error: {}", e),
            ));
        }

        // 5. Declare function with unique name
        let func_name = format!("glsl_main_{}", self.function_counter);
        self.function_counter += 1;

        let id = self
            .module
            .declare_function(&func_name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to declare function: {}", e),
                )
            })?;

        // 6. Define function
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| {
                GlslError::new(ErrorCode::E0400, format!("code generation failed: {}", e))
            })?;

        // 7. Finalize
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions().unwrap();

        // 8. Get function pointer
        let code = self.module.get_finalized_function(id);
        Ok(code)
    }

    /// Compile and return CLIF IR as string (for filetests)
    pub fn compile_to_clif(&mut self, glsl_source: &str) -> Result<String, String> {
        self.compile_to_clif_detailed(glsl_source)
            .map_err(|e| e.to_simple_string())
    }

    /// Compile to CLIF IR with detailed error information
    pub fn compile_to_clif_detailed(
        &mut self,
        glsl_source: &str,
    ) -> Result<String, crate::error::GlslError> {
        self.ctx.clear();

        // 1. Parse and analyze GLSL
        let semantic_result = crate::pipeline::CompilationPipeline::parse_and_analyze(glsl_source)?;

        // 2. Generate Cranelift IR
        self.translate(semantic_result.typed_ast, semantic_result.source)?;

        // 3.5. Apply fixed-point transformation if enabled
        if let Some(format) = self.fixed_point_format {
            crate::transform::fixed_point::convert_floats_to_fixed(&mut self.ctx.func, format)?;
        }

        // 4. Return as string
        Ok(format!("{}", self.ctx.func))
    }

    fn translate(
        &mut self,
        typed_ast: crate::semantic::TypedShader,
        source_text: &str,
    ) -> Result<(), crate::error::GlslError> {
        // Step 1: Declare all user functions and get their FuncIds
        let mut func_ids: HashMap<String, FuncId> = HashMap::new();

        for user_func in &typed_ast.user_functions {
            let func_id = self.declare_function_signature(
                &user_func.name,
                &user_func.return_type,
                &user_func.parameters,
            )?;
            func_ids.insert(user_func.name.clone(), func_id);
        }

        // Step 2: Compile all user functions
        for user_func in &typed_ast.user_functions {
            let func_id = func_ids[&user_func.name];
            self.compile_function(
                user_func,
                func_id,
                &func_ids,
                &typed_ast.function_registry,
                source_text,
            )?;
        }

        // Step 3: Compile main function
        self.compile_main_function(
            &typed_ast.main_function,
            &func_ids,
            &typed_ast.function_registry,
            source_text,
        )?;

        Ok(())
    }

    fn declare_function_signature(
        &mut self,
        name: &str,
        return_type: &crate::semantic::types::Type,
        parameters: &[crate::semantic::functions::Parameter],
    ) -> Result<FuncId, crate::error::GlslError> {
        use crate::codegen::signature::SignatureBuilder;
        use crate::error::{ErrorCode, GlslError};

        let sig = SignatureBuilder::build(return_type, parameters);

        // Declare function in module
        self.module
            .declare_function(name, Linkage::Local, &sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to declare function: {}", e),
                )
            })
    }

    fn compile_function(
        &mut self,
        func: &crate::semantic::TypedFunction,
        func_id: FuncId,
        func_ids: &HashMap<String, FuncId>,
        func_registry: &crate::semantic::functions::FunctionRegistry,
        _source_text: &str,
    ) -> Result<(), crate::error::GlslError> {
        use crate::codegen::signature::SignatureBuilder;
        use crate::error::{ErrorCode, GlslError};
        self.ctx.clear();

        // Build signature (same as declaration) and set it on the function
        let sig = SignatureBuilder::build(&func.return_type, &func.parameters);
        self.ctx.func.signature = sig;

        // Create a new FunctionBuilderContext for this function to avoid state pollution
        let mut func_builder_context = FunctionBuilderContext::new();
        // Create function builder
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut func_builder_context);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Create codegen context with function IDs
        let mut ctx = crate::codegen::context::CodegenContext::new(builder, &mut self.module);
        ctx.set_function_ids(func_ids);
        ctx.set_function_registry(func_registry);

        // Declare parameters as variables in the function
        let block_params = ctx.builder.block_params(entry_block).to_vec();

        // Validate that we have enough block parameters for all function parameters
        let expected_param_count: usize = func
            .parameters
            .iter()
            .map(|p| SignatureBuilder::count_parameters(&p.ty))
            .sum();
        if block_params.len() < expected_param_count {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "function parameter mismatch: expected {} block parameters, got {}",
                    expected_param_count,
                    block_params.len()
                ),
            ));
        }

        let mut param_idx = 0;
        for param in &func.parameters {
            let param_vals: Vec<cranelift_codegen::ir::Value> = if param.ty.is_vector() {
                let count = param.ty.component_count().unwrap();
                let mut vals = Vec::new();
                for _ in 0..count {
                    if param_idx >= block_params.len() {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            format!(
                                "not enough block parameters for function parameter `{}`",
                                param.name
                            ),
                        ));
                    }
                    vals.push(block_params[param_idx]);
                    param_idx += 1;
                }
                vals
            } else {
                if param_idx >= block_params.len() {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "not enough block parameters for function parameter `{}`",
                            param.name
                        ),
                    ));
                }
                let val = vec![block_params[param_idx]];
                param_idx += 1;
                val
            };

            // Declare parameter as variable and initialize
            let vars = ctx.declare_variable(param.name.clone(), param.ty.clone());
            for (var, val) in vars.iter().zip(param_vals) {
                ctx.builder.def_var(*var, val);
            }
        }

        // Translate function body
        for stmt in &func.body {
            ctx.translate_statement(stmt)?;
        }

        // Add default return if needed
        if func.return_type == crate::semantic::types::Type::Void {
            ctx.builder.ins().return_(&[]);
        } else {
            // If we're here, there was no explicit return - emit default
            let return_val = match func.return_type {
                crate::semantic::types::Type::Int => ctx
                    .builder
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I32, 0),
                crate::semantic::types::Type::Float => ctx.builder.ins().f32const(0.0),
                crate::semantic::types::Type::Bool => ctx
                    .builder
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I8, 0),
                _ => {
                    // For vectors, return zero vector
                    let base_ty = func.return_type.vector_base_type().unwrap();
                    let count = func.return_type.component_count().unwrap();
                    let mut vals = Vec::new();
                    for _ in 0..count {
                        let val = match base_ty {
                            crate::semantic::types::Type::Float => ctx.builder.ins().f32const(0.0),
                            crate::semantic::types::Type::Int => ctx
                                .builder
                                .ins()
                                .iconst(cranelift_codegen::ir::types::I32, 0),
                            crate::semantic::types::Type::Bool => ctx
                                .builder
                                .ins()
                                .iconst(cranelift_codegen::ir::types::I8, 0),
                            _ => {
                                return Err(GlslError::new(
                                    ErrorCode::E0400,
                                    format!(
                                        "unsupported return type for default return: {:?}",
                                        func.return_type
                                    ),
                                ));
                            }
                        };
                        vals.push(val);
                    }
                    ctx.builder.ins().return_(&vals);
                    return Ok(());
                }
            };
            ctx.builder.ins().return_(&[return_val]);
        }

        // Finalize
        ctx.builder.finalize();

        // Define function in module
        // Note: If verification fails, the error won't have location information
        // because it comes from Cranelift. Errors with location should be caught
        // during codegen (e.g., type coercion failures).
        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| {
                // Try to preserve any location from previous errors if available
                // For now, create error without location (will be caught during codegen)
                GlslError::new(ErrorCode::E0400, format!("code generation failed: {}", e))
            })?;
        self.module.clear_context(&mut self.ctx);

        Ok(())
    }

    fn compile_main_function(
        &mut self,
        main_func: &crate::semantic::TypedFunction,
        func_ids: &HashMap<String, FuncId>,
        func_registry: &crate::semantic::functions::FunctionRegistry,
        source_text: &str,
    ) -> Result<(), crate::error::GlslError> {
        use crate::codegen::signature::SignatureBuilder;
        self.ctx.clear();

        // Set up main signature (no parameters, just return type)
        let mut sig = SignatureBuilder::new();
        SignatureBuilder::add_return_type(&mut sig, &main_func.return_type);
        self.ctx.func.signature = sig;

        // Create a new FunctionBuilderContext for the main function to avoid state pollution
        let mut main_builder_context = FunctionBuilderContext::new();
        // Create function builder
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut main_builder_context);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Create codegen context with function IDs
        let mut ctx = crate::codegen::context::CodegenContext::new(builder, &mut self.module);
        ctx.set_function_ids(func_ids);
        ctx.set_function_registry(func_registry);
        ctx.set_source_text(source_text);
        ctx.set_return_type(main_func.return_type.clone());
        ctx.set_return_type(main_func.return_type.clone());

        // Translate main function body
        for stmt in &main_func.body {
            ctx.translate_statement(stmt)?;
        }

        // Add default return
        if main_func.return_type == crate::semantic::types::Type::Void {
            ctx.builder.ins().return_(&[]);
        } else if main_func.return_type.is_vector() {
            // For vectors, return zero for each component
            let base_ty = main_func.return_type.vector_base_type().unwrap();
            let cranelift_ty = base_ty.to_cranelift_type();
            let count = main_func.return_type.component_count().unwrap();
            let mut return_vals = Vec::new();
            for _ in 0..count {
                let val = match base_ty {
                    crate::semantic::types::Type::Float => ctx.builder.ins().f32const(0.0),
                    crate::semantic::types::Type::Int => ctx.builder.ins().iconst(cranelift_ty, 0),
                    crate::semantic::types::Type::Bool => ctx.builder.ins().iconst(cranelift_ty, 0),
                    _ => ctx.builder.ins().iconst(cranelift_ty, 0),
                };
                return_vals.push(val);
            }
            ctx.builder.ins().return_(&return_vals);
        } else {
            let return_type = main_func.return_type.to_cranelift_type();
            let return_val = match main_func.return_type {
                crate::semantic::types::Type::Float => ctx.builder.ins().f32const(0.0),
                _ => ctx.builder.ins().iconst(return_type, 0),
            };
            ctx.builder.ins().return_(&[return_val]);
        }

        // Finalize
        ctx.builder.finalize();
        Ok(())
    }
}

/// Execute a compiled function that returns i32
pub fn execute_i32(code_ptr: *const u8) -> i32 {
    let func: fn() -> i32 = unsafe { std::mem::transmute(code_ptr) };
    func()
}

/// Execute a compiled function that returns bool (as i8)
pub fn execute_bool(code_ptr: *const u8) -> bool {
    let func: fn() -> i8 = unsafe { std::mem::transmute(code_ptr) };
    func() != 0
}
