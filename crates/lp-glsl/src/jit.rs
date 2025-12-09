#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::format;
use cranelift_codegen::Context as CodegenContext;
use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::isa::CallConv;
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

impl JIT {
    /// Get access to the underlying JIT module (for accessing ISA information)
    pub fn module(&self) -> &JITModule {
        &self.module
    }

    /// Get the calling convention for the current ISA
    pub fn call_conv(&self) -> CallConv {
        CallConv::triple_default(self.module.isa().triple())
    }

    /// Get the pointer type for the current ISA
    pub fn pointer_type(&self) -> cranelift_codegen::ir::Type {
        self.module.isa().pointer_type()
    }
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
        flag_builder
            .set("enable_multi_ret_implicit_sret", "true")
            .expect("set enable_multi_ret_implicit_sret");

        let flags = settings::Flags::new(flag_builder);

        // Use host ISA - this should always be available when running on the host
        let isa = cranelift_native::builder()
            .map_err(|e| {
                panic!(
                    "Failed to create host ISA (this should never happen): {:?}",
                    e
                );
            })
            .unwrap()
            .finish(flags)
            .unwrap();
        Self::new_with_isa(isa)
    }

    /// Create a new JIT instance with a specific ISA
    /// This allows targeting architectures other than the host
    /// The ISA should be created with appropriate flags (is_pic=false, use_colocated_libcalls=false, enable_multi_ret_implicit_sret=true)
    pub fn new_with_isa(isa: cranelift_codegen::isa::OwnedTargetIsa) -> Self {
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

        let pointer_type = self.module.isa().pointer_type();
        let triple = self.module.isa().triple();
        let sig =
            SignatureBuilder::build_with_triple(return_type, parameters, pointer_type, triple);

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

    /// Set up function builder with entry block
    fn setup_function_builder(builder: &mut FunctionBuilder) -> cranelift_codegen::ir::Block {
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        entry_block
    }

    /// Generate default return statement for a function
    /// Used when function doesn't have explicit return
    pub fn generate_default_return(
        ctx: &mut crate::codegen::context::CodegenContext,
        return_type: &crate::semantic::types::Type,
    ) -> Result<(), crate::error::GlslError> {
        use crate::semantic::types::Type;

        if return_type == &Type::Void {
            ctx.builder.ins().return_(&[]);
            return Ok(());
        }

        if return_type.is_vector() {
            Self::generate_default_vector_return(ctx, return_type)
        } else if return_type.is_matrix() {
            Self::generate_default_matrix_return(ctx, return_type)
        } else {
            Self::generate_default_scalar_return(ctx, return_type)
        }
    }

    fn generate_default_scalar_return(
        ctx: &mut crate::codegen::context::CodegenContext,
        return_type: &crate::semantic::types::Type,
    ) -> Result<(), crate::error::GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use crate::semantic::types::Type;
        use cranelift_codegen::ir::types;

        let return_val = match return_type {
            Type::Int => ctx.builder.ins().iconst(types::I32, 0),
            Type::Float => ctx.builder.ins().f32const(0.0),
            Type::Bool => ctx.builder.ins().iconst(types::I8, 0),
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!(
                        "unsupported return type for default return: {:?}",
                        return_type
                    ),
                ));
            }
        };
        ctx.builder.ins().return_(&[return_val]);
        Ok(())
    }

    /// Helper: Get the StructReturn pointer from the function signature.
    /// Returns an error if StructReturn is expected but not found.
    fn get_structreturn_pointer(
        ctx: &mut crate::codegen::context::CodegenContext,
    ) -> Result<cranelift_codegen::ir::Value, crate::error::GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use cranelift_codegen::ir::ArgumentPurpose;

        ctx.builder
            .func
            .special_param(ArgumentPurpose::StructReturn)
            .ok_or_else(|| {
                GlslError::new(
                    ErrorCode::E0400,
                    "StructReturn parameter not found (internal error)",
                )
            })
    }

    /// Helper: Write zeros to StructReturn buffer for float elements.
    /// Used for matrices and float-based vectors.
    fn write_zeros_to_structreturn_buffer(
        ctx: &mut crate::codegen::context::CodegenContext,
        struct_ret_ptr: cranelift_codegen::ir::Value,
        element_count: usize,
    ) {
        use cranelift_codegen::ir::MemFlags;

        for i in 0..element_count {
            let zero_val = ctx.builder.ins().f32const(0.0);
            let offset = (i * crate::codegen::constants::F32_SIZE_BYTES) as i32;
            ctx.builder
                .ins()
                .store(MemFlags::trusted(), zero_val, struct_ret_ptr, offset);
        }
    }

    /// Helper: Create a zero value for a given base type.
    fn create_zero_value(
        ctx: &mut crate::codegen::context::CodegenContext,
        base_ty: &crate::semantic::types::Type,
    ) -> Result<cranelift_codegen::ir::Value, crate::error::GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use crate::semantic::types::Type;
        use cranelift_codegen::ir::types;

        match base_ty {
            Type::Float => Ok(ctx.builder.ins().f32const(0.0)),
            Type::Int => Ok(ctx.builder.ins().iconst(types::I32, 0)),
            Type::Bool => Ok(ctx.builder.ins().iconst(types::I8, 0)),
            _ => Err(GlslError::new(
                ErrorCode::E0400,
                format!("unsupported base type for zero value: {:?}", base_ty),
            )),
        }
    }

    fn generate_default_vector_return(
        ctx: &mut crate::codegen::context::CodegenContext,
        return_type: &crate::semantic::types::Type,
    ) -> Result<(), crate::error::GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use crate::semantic::types::Type;
        use cranelift_codegen::ir::{ArgumentPurpose, MemFlags};

        // Check if function uses StructReturn
        let uses_struct_return = ctx
            .builder
            .func
            .signature
            .uses_special_param(ArgumentPurpose::StructReturn);

        let base_ty = return_type.vector_base_type().ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                format!("expected vector type, got: {:?}", return_type),
            )
        })?;
        let count = return_type.component_count().unwrap();

        if uses_struct_return {
            // Write zeros to StructReturn buffer
            let struct_ret_ptr = Self::get_structreturn_pointer(ctx)?;

            // For float vectors, use optimized helper
            match base_ty {
                Type::Float => {
                    Self::write_zeros_to_structreturn_buffer(ctx, struct_ret_ptr, count);
                }
                _ => {
                    // For int/bool vectors, write each component individually
                    for i in 0..count {
                        let zero_val = Self::create_zero_value(ctx, &base_ty)?;
                        let offset = (i * crate::codegen::constants::F32_SIZE_BYTES) as i32;
                        ctx.builder.ins().store(
                            MemFlags::trusted(),
                            zero_val,
                            struct_ret_ptr,
                            offset,
                        );
                    }
                }
            }

            // Return void for StructReturn functions
            ctx.builder.ins().return_(&[]);
        } else {
            // Legacy path (shouldn't happen with this plan, but kept as fallback)
            let mut vals = Vec::new();
            for _ in 0..count {
                let val = Self::create_zero_value(ctx, &base_ty)?;
                vals.push(val);
            }
            ctx.builder.ins().return_(&vals);
        }
        Ok(())
    }

    fn generate_default_matrix_return(
        ctx: &mut crate::codegen::context::CodegenContext,
        return_type: &crate::semantic::types::Type,
    ) -> Result<(), crate::error::GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use cranelift_codegen::ir::ArgumentPurpose;

        // Check if function uses StructReturn
        let uses_struct_return = ctx
            .builder
            .func
            .signature
            .uses_special_param(ArgumentPurpose::StructReturn);

        let element_count = return_type.matrix_element_count().ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                format!("expected matrix type, got: {:?}", return_type),
            )
        })?;

        if uses_struct_return {
            // Write zeros to StructReturn buffer
            let struct_ret_ptr = Self::get_structreturn_pointer(ctx)?;
            // Matrices are always float-based, use optimized helper
            Self::write_zeros_to_structreturn_buffer(ctx, struct_ret_ptr, element_count);
            // Return void for StructReturn functions
            ctx.builder.ins().return_(&[]);
        } else {
            // Legacy path (shouldn't happen with this plan, but kept as fallback)
            let mut vals = Vec::new();
            // Matrices are always float-based, return zero matrix
            for _ in 0..element_count {
                vals.push(ctx.builder.ins().f32const(0.0));
            }
            ctx.builder.ins().return_(&vals);
        }
        Ok(())
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
        let pointer_type = self.module.isa().pointer_type();
        let triple = self.module.isa().triple();
        let sig = SignatureBuilder::build_with_triple(
            &func.return_type,
            &func.parameters,
            pointer_type,
            triple,
        );
        self.ctx.func.signature = sig;

        // Create a new FunctionBuilderContext for this function to avoid state pollution
        let mut func_builder_context = FunctionBuilderContext::new();
        // Create function builder
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut func_builder_context);

        // Set up entry block (using helper)
        let entry_block = Self::setup_function_builder(&mut builder);

        // Create codegen context with function IDs
        let mut ctx = crate::codegen::context::CodegenContext::new(builder, &mut self.module);
        ctx.set_function_ids(func_ids);
        ctx.set_function_registry(func_registry);
        ctx.set_return_type(func.return_type.clone());
        ctx.set_entry_block(entry_block);

        // Declare parameters as variables in the function
        let block_params = ctx.builder.block_params(entry_block).to_vec();

        // Check if function uses StructReturn (compute once, reuse)
        let uses_struct_return = ctx
            .builder
            .func
            .signature
            .uses_special_param(cranelift_codegen::ir::ArgumentPurpose::StructReturn);

        // Validate that we have enough block parameters for all function parameters
        // Account for StructReturn parameter if present (it's FIRST)
        let expected_param_count: usize = func
            .parameters
            .iter()
            .map(|p| SignatureBuilder::count_parameters(&p.ty))
            .sum::<usize>()
            + if uses_struct_return { 1 } else { 0 }; // Add 1 for StructReturn if present
        if block_params.len() < expected_param_count {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "function parameter mismatch: expected {} block parameters (including StructReturn), got {}",
                    expected_param_count,
                    block_params.len()
                ),
            ));
        }

        // Skip StructReturn parameter if present (it's FIRST in the signature)
        let mut param_idx = if uses_struct_return {
            1 // Skip StructReturn parameter (index 0)
        } else {
            0
        };

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
            let vars = ctx.declare_variable(param.name.clone(), param.ty.clone())?;
            for (var, val) in vars.iter().zip(param_vals) {
                ctx.builder.def_var(*var, val);
            }
        }

        // Translate function body
        for stmt in &func.body {
            ctx.translate_statement(stmt)?;
        }

        // Generate default return if needed (using helper)
        Self::generate_default_return(&mut ctx, &func.return_type)?;

        // Finalize
        ctx.builder.finalize();

        // Verify function before defining to get better error messages
        if let Err(verifier_error) =
            cranelift_codegen::verify_function(&self.ctx.func, self.module.isa())
        {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "verifier error in function '{}': {}\n\nFunction IR:\n{}",
                    func.name, verifier_error, self.ctx.func
                ),
            ));
        }

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
        let triple = self.module.isa().triple();
        let mut sig = SignatureBuilder::new_with_triple(triple);
        let pointer_type = self.module.isa().pointer_type();
        SignatureBuilder::add_return_type(&mut sig, &main_func.return_type, pointer_type);
        self.ctx.func.signature = sig;

        // Create a new FunctionBuilderContext for the main function to avoid state pollution
        let mut main_builder_context = FunctionBuilderContext::new();
        // Create function builder
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut main_builder_context);

        // Create codegen context with function IDs
        let entry_block = Self::setup_function_builder(&mut builder);
        let mut ctx = crate::codegen::context::CodegenContext::new(builder, &mut self.module);
        ctx.set_function_ids(func_ids);
        ctx.set_function_registry(func_registry);
        ctx.set_source_text(source_text);
        ctx.set_return_type(main_func.return_type.clone());
        ctx.set_entry_block(entry_block);

        // Translate main function body
        for stmt in &main_func.body {
            ctx.translate_statement(stmt)?;
        }

        // Generate default return if needed (using helper)
        Self::generate_default_return(&mut ctx, &main_func.return_type)?;

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
