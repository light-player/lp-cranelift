//! GLSL compiler that compiles GLSL source to GlModule

use crate::backend::module::gl_module::GlModule;
use crate::backend::target::Target;
use crate::error::GlslError;
use crate::frontend::pipeline::CompilationPipeline;
use crate::frontend::src_loc::GlSourceMap;
use cranelift_codegen::ir::Function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::JITModule;
use cranelift_module::{FuncId, Linkage, Module, ModuleDeclarations, ModuleError, ModuleResult};
use cranelift_object::ObjectModule;
use hashbrown::HashMap;

use alloc::string::String;
use alloc::{boxed::Box, vec::Vec};

use alloc::format;
/// GLSL compiler that compiles GLSL source to GlModule
pub struct GlslCompiler {
    #[allow(dead_code)]
    builder_context: FunctionBuilderContext,
}

impl GlslCompiler {
    pub fn new() -> Self {
        Self {
            builder_context: FunctionBuilderContext::new(),
        }
    }

    /// Compile GLSL source to a GlModule<JITModule>
    /// All functions are compiled with float types initially (no fixed-point conversion)
    pub fn compile_to_gl_module_jit(
        &mut self,
        source: &str,
        target: Target,
    ) -> Result<GlModule<JITModule>, GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use crate::frontend::codegen::signature::SignatureBuilder;

        // 1. Parse and analyze GLSL
        let semantic_result = CompilationPipeline::parse_and_analyze(source)?;
        let typed_ast = semantic_result.typed_ast;

        // 2. Create ISA for signature building (before creating gl_module to avoid borrow conflicts)
        let mut target_for_isa = target.clone();
        let isa_ref = target_for_isa.create_isa()?;
        let pointer_type = isa_ref.pointer_type();
        let triple = isa_ref.triple();

        // 3. Create GlModule
        let mut gl_module = GlModule::new_jit(target)?;

        // 4. Create a shared source location manager for all functions
        use crate::frontend::src_loc_manager::SourceLocManager;
        let mut source_loc_manager = SourceLocManager::new();

        // 4b. Create a source map and add the main source file
        let mut source_map = GlSourceMap::new();
        let main_file_id = source_map.add_file(
            crate::frontend::src_loc::GlFileSource::Synthetic(String::from("main.glsl")),
            String::from(source),
        );

        // 5. Declare all user functions with FLOAT signatures (no conversion)
        let mut func_ids: HashMap<String, FuncId> = HashMap::new();

        for user_func in &typed_ast.user_functions {
            let sig = SignatureBuilder::build_with_triple(
                &user_func.return_type,
                &user_func.parameters,
                pointer_type,
                triple,
            );
            let func_id = gl_module
                .declare_function(&user_func.name, Linkage::Local, sig)
                .map_err(|e| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!("failed to declare function '{}': {}", user_func.name, e),
                    )
                })?;
            func_ids.insert(user_func.name.clone(), func_id);
        }

        // 6. Compile all user functions to CLIF with FLOAT types
        // Collect compiled functions first to avoid borrow conflicts
        let mut compiled_user_functions: Vec<(
            String,
            Function,
            cranelift_codegen::ir::Signature,
            crate::frontend::semantic::functions::FunctionSignature,
        )> = Vec::new();
        for user_func in &typed_ast.user_functions {
            let func_id = func_ids[&user_func.name];
            let sig = SignatureBuilder::build_with_triple(
                &user_func.return_type,
                &user_func.parameters,
                pointer_type,
                triple,
            );
            let func = {
                // Borrow module only for compilation
                let module_ref = gl_module.module_mut_internal();
                self.compile_function_to_clif(
                    user_func,
                    func_id,
                    &func_ids,
                    &typed_ast.function_registry,
                    module_ref,
                    isa_ref.as_ref(),
                    &mut source_loc_manager,
                    &source_map,
                    main_file_id,
                )?
            };
            let glsl_sig = crate::frontend::semantic::functions::FunctionSignature {
                name: user_func.name.clone(),
                return_type: user_func.return_type.clone(),
                parameters: user_func.parameters.clone(),
            };
            compiled_user_functions.push((user_func.name.clone(), func, sig, glsl_sig));
        }

        // 7. Add compiled user functions to GlModule
        for (name, func, sig, glsl_sig) in compiled_user_functions {
            gl_module.add_function(&name, Linkage::Local, sig, func)?;
            gl_module.glsl_signatures.insert(name, glsl_sig);
        }

        // 8. Compile main function to CLIF with FLOAT types
        let main_sig = SignatureBuilder::build_with_triple(
            &typed_ast.main_function.return_type,
            &typed_ast.main_function.parameters,
            pointer_type,
            triple,
        );
        let main_func = {
            // Borrow module only for compilation
            let module_ref = gl_module.module_mut_internal();
            self.compile_main_function_to_clif(
                &typed_ast.main_function,
                &func_ids,
                &typed_ast.function_registry,
                module_ref,
                isa_ref.as_ref(),
                semantic_result.source,
                &mut source_loc_manager,
                &source_map,
                main_file_id,
            )?
        };

        // Add main function to GlModule
        gl_module.add_function("main", Linkage::Export, main_sig, main_func)?;

        // Store main function's GLSL signature
        gl_module.glsl_signatures.insert(
            String::from("main"),
            crate::frontend::semantic::functions::FunctionSignature {
                name: String::from("main"),
                return_type: typed_ast.main_function.return_type.clone(),
                parameters: typed_ast.main_function.parameters.clone(),
            },
        );

        // 9. Set metadata
        gl_module.function_registry = typed_ast.function_registry;
        gl_module.source_text = String::from(source);
        gl_module.source_loc_manager = source_loc_manager;
        gl_module.source_map = source_map;

        Ok(gl_module)
    }

    /// Compile GLSL source to a GlModule<ObjectModule>
    /// All functions are compiled with float types initially (no fixed-point conversion)
    pub fn compile_to_gl_module_object(
        &mut self,
        source: &str,
        target: Target,
    ) -> Result<GlModule<ObjectModule>, GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use crate::frontend::codegen::signature::SignatureBuilder;

        // 1. Parse and analyze GLSL
        let semantic_result = CompilationPipeline::parse_and_analyze(source)?;
        let typed_ast = semantic_result.typed_ast;

        // 2. Create ISA for signature building (before creating gl_module to avoid borrow conflicts)
        let mut target_for_isa = target.clone();
        let isa_ref = target_for_isa.create_isa()?;
        let pointer_type = isa_ref.pointer_type();
        let triple = isa_ref.triple();

        // 3. Create GlModule
        let mut gl_module = GlModule::new_object(target)?;

        // 4. Create a shared source location manager for all functions
        use crate::frontend::src_loc_manager::SourceLocManager;
        let mut source_loc_manager = SourceLocManager::new();

        // 4b. Create a source map and add the main source file
        let mut source_map = GlSourceMap::new();
        let main_file_id = source_map.add_file(
            crate::frontend::src_loc::GlFileSource::Synthetic(String::from("main.glsl")),
            String::from(source),
        );

        // 5. Declare all user functions with FLOAT signatures (no conversion)
        let mut func_ids: HashMap<String, FuncId> = HashMap::new();

        for user_func in &typed_ast.user_functions {
            let sig = SignatureBuilder::build_with_triple(
                &user_func.return_type,
                &user_func.parameters,
                pointer_type,
                triple,
            );
            let func_id = gl_module
                .declare_function(&user_func.name, Linkage::Local, sig)
                .map_err(|e| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!("failed to declare function '{}': {}", user_func.name, e),
                    )
                })?;
            func_ids.insert(user_func.name.clone(), func_id);
        }

        // 6. Compile all user functions to CLIF with FLOAT types
        // Collect compiled functions first to avoid borrow conflicts
        let mut compiled_user_functions: Vec<(
            String,
            Function,
            cranelift_codegen::ir::Signature,
            crate::frontend::semantic::functions::FunctionSignature,
        )> = Vec::new();
        for user_func in &typed_ast.user_functions {
            let func_id = func_ids[&user_func.name];
            let sig = SignatureBuilder::build_with_triple(
                &user_func.return_type,
                &user_func.parameters,
                pointer_type,
                triple,
            );
            let func = {
                // Borrow module only for compilation
                let module_ref = gl_module.module_mut_internal();
                self.compile_function_to_clif(
                    user_func,
                    func_id,
                    &func_ids,
                    &typed_ast.function_registry,
                    module_ref,
                    isa_ref.as_ref(),
                    &mut source_loc_manager,
                    &source_map,
                    main_file_id,
                )?
            };
            let glsl_sig = crate::frontend::semantic::functions::FunctionSignature {
                name: user_func.name.clone(),
                return_type: user_func.return_type.clone(),
                parameters: user_func.parameters.clone(),
            };
            compiled_user_functions.push((user_func.name.clone(), func, sig, glsl_sig));
        }

        // 7. Add compiled user functions to GlModule
        for (name, func, sig, glsl_sig) in compiled_user_functions {
            gl_module.add_function(&name, Linkage::Local, sig, func)?;
            gl_module.glsl_signatures.insert(name, glsl_sig);
        }

        // 8. Compile main function to CLIF with FLOAT types
        let main_sig = SignatureBuilder::build_with_triple(
            &typed_ast.main_function.return_type,
            &typed_ast.main_function.parameters,
            pointer_type,
            triple,
        );
        let main_func = {
            // Borrow module only for compilation
            let module_ref = gl_module.module_mut_internal();
            self.compile_main_function_to_clif(
                &typed_ast.main_function,
                &func_ids,
                &typed_ast.function_registry,
                module_ref,
                isa_ref.as_ref(),
                semantic_result.source,
                &mut source_loc_manager,
                &source_map,
                main_file_id,
            )?
        };

        // Add main function to GlModule
        gl_module.add_function("main", Linkage::Export, main_sig, main_func)?;

        // Store main function's GLSL signature
        gl_module.glsl_signatures.insert(
            String::from("main"),
            crate::frontend::semantic::functions::FunctionSignature {
                name: String::from("main"),
                return_type: typed_ast.main_function.return_type.clone(),
                parameters: typed_ast.main_function.parameters.clone(),
            },
        );

        // 9. Set metadata
        gl_module.function_registry = typed_ast.function_registry;
        gl_module.source_text = String::from(source);
        gl_module.source_loc_manager = source_loc_manager;
        gl_module.source_map = source_map;

        Ok(gl_module)
    }

    fn compile_function_to_clif(
        &mut self,
        func: &crate::frontend::semantic::TypedFunction,
        _func_id: FuncId,
        func_ids: &HashMap<String, FuncId>,
        func_registry: &crate::frontend::semantic::functions::FunctionRegistry,
        temp_module: &mut dyn Module,
        isa: &dyn cranelift_codegen::isa::TargetIsa,
        source_loc_manager: &mut crate::frontend::src_loc_manager::SourceLocManager,
        source_map: &crate::frontend::src_loc::GlSourceMap,
        file_id: crate::frontend::src_loc::GlFileId,
    ) -> Result<Function, GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use crate::frontend::codegen::signature::SignatureBuilder;
        use cranelift_codegen::Context;

        let mut ctx = Context::new();

        // Build signature (same as declaration) and set it on the function
        let pointer_type = isa.pointer_type();
        let triple = isa.triple();
        let sig = SignatureBuilder::build_with_triple(
            &func.return_type,
            &func.parameters,
            pointer_type,
            triple,
        );
        ctx.func.signature = sig.clone();
        use cranelift_codegen::ir::UserFuncName;
        ctx.func.name = UserFuncName::user(0, 0); // TODO: Use proper function name

        // Create function builder
        let mut func_builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_builder_context);

        // Set up entry block
        let entry_block = Self::setup_function_builder(&mut builder);

        // Create codegen context with function IDs
        let mut codegen_ctx = crate::frontend::codegen::context::CodegenContext::new(
            builder,
            temp_module,
            source_map,
            file_id,
        );
        codegen_ctx.set_function_ids(func_ids);
        codegen_ctx.set_function_registry(func_registry);
        codegen_ctx.set_return_type(func.return_type.clone());
        codegen_ctx.set_entry_block(entry_block);
        // Copy the shared SourceLocManager into the context
        codegen_ctx.source_loc_manager = source_loc_manager.clone();

        // Declare parameters as variables in the function
        let block_params = codegen_ctx.builder.block_params(entry_block).to_vec();

        // Check if function uses StructReturn
        let uses_struct_return = codegen_ctx
            .builder
            .func
            .signature
            .uses_special_param(cranelift_codegen::ir::ArgumentPurpose::StructReturn);

        // Validate parameter count
        let expected_param_count: usize = func
            .parameters
            .iter()
            .map(|p| SignatureBuilder::count_parameters(&p.ty))
            .sum::<usize>()
            + if uses_struct_return { 1 } else { 0 };

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

        // Skip StructReturn parameter if present
        let mut param_idx = if uses_struct_return { 1 } else { 0 };

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
            } else if param.ty.is_matrix() {
                let count = param.ty.matrix_element_count().unwrap();
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
            let vars = codegen_ctx.declare_variable(param.name.clone(), param.ty.clone())?;
            for (var, val) in vars.iter().zip(param_vals) {
                codegen_ctx.builder.def_var(*var, val);
            }
        }

        // Translate function body
        for stmt in &func.body {
            codegen_ctx.emit_statement(stmt)?;
        }

        // Generate default return if needed
        crate::frontend::codegen::helpers::generate_default_return(
            &mut codegen_ctx,
            &func.return_type,
        )?;

        // Seal all blocks before finalizing (safety net for any blocks not explicitly sealed)
        codegen_ctx.builder.seal_all_blocks();

        // Finalize
        codegen_ctx.builder.finalize();

        // Merge SourceLocManager back into shared one
        source_loc_manager.merge_from(&codegen_ctx.source_loc_manager);

        // Verify function
        cranelift_codegen::verify_function(&ctx.func, isa).map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!(
                    "verifier error in function '{}': {}\n\nFunction IR:\n{}",
                    func.name, e, ctx.func
                ),
            )
        })?;

        Ok(ctx.func)
    }

    fn compile_main_function_to_clif(
        &mut self,
        main_func: &crate::frontend::semantic::TypedFunction,
        func_ids: &HashMap<String, FuncId>,
        func_registry: &crate::frontend::semantic::functions::FunctionRegistry,
        temp_module: &mut dyn Module,
        isa: &dyn cranelift_codegen::isa::TargetIsa,
        source_text: &str,
        source_loc_manager: &mut crate::frontend::src_loc_manager::SourceLocManager,
        source_map: &crate::frontend::src_loc::GlSourceMap,
        file_id: crate::frontend::src_loc::GlFileId,
    ) -> Result<Function, GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use crate::frontend::codegen::signature::SignatureBuilder;
        use cranelift_codegen::Context;

        let mut ctx = Context::new();

        // Build signature with parameters (same as regular functions)
        let pointer_type = isa.pointer_type();
        let triple = isa.triple();
        let sig = SignatureBuilder::build_with_triple(
            &main_func.return_type,
            &main_func.parameters,
            pointer_type,
            triple,
        );
        ctx.func.signature = sig.clone();
        use cranelift_codegen::ir::UserFuncName;
        ctx.func.name = UserFuncName::user(0, 0); // TODO: Use "main" as function name

        // Create function builder
        let mut main_builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut main_builder_context);

        // Set up entry block
        let entry_block = Self::setup_function_builder(&mut builder);

        // Create codegen context
        let mut codegen_ctx = crate::frontend::codegen::context::CodegenContext::new(
            builder,
            temp_module,
            source_map,
            file_id,
        );
        codegen_ctx.set_function_ids(func_ids);
        codegen_ctx.set_function_registry(func_registry);
        codegen_ctx.set_source_text(source_text);
        codegen_ctx.set_return_type(main_func.return_type.clone());
        codegen_ctx.set_entry_block(entry_block);
        // Replace the default SourceLocManager with the shared one
        codegen_ctx.source_loc_manager = source_loc_manager.clone();

        // Declare parameters as variables in the function (same as compile_function_to_clif)
        let block_params = codegen_ctx.builder.block_params(entry_block).to_vec();

        // Check if function uses StructReturn
        let uses_struct_return = codegen_ctx
            .builder
            .func
            .signature
            .uses_special_param(cranelift_codegen::ir::ArgumentPurpose::StructReturn);

        // Validate parameter count
        let expected_param_count: usize = main_func
            .parameters
            .iter()
            .map(|p| SignatureBuilder::count_parameters(&p.ty))
            .sum::<usize>()
            + if uses_struct_return { 1 } else { 0 };

        if block_params.len() < expected_param_count {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "main function parameter mismatch: expected {} block parameters, got {}",
                    expected_param_count,
                    block_params.len()
                ),
            ));
        }

        // Skip StructReturn parameter if present
        let mut param_idx = if uses_struct_return { 1 } else { 0 };

        for param in &main_func.parameters {
            let param_vals: Vec<cranelift_codegen::ir::Value> = if param.ty.is_vector() {
                let count = param.ty.component_count().unwrap();
                let mut vals = Vec::new();
                for _ in 0..count {
                    if param_idx >= block_params.len() {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            format!(
                                "not enough block parameters for main parameter `{}`",
                                param.name
                            ),
                        ));
                    }
                    vals.push(block_params[param_idx]);
                    param_idx += 1;
                }
                vals
            } else if param.ty.is_matrix() {
                let count = param.ty.matrix_element_count().unwrap();
                let mut vals = Vec::new();
                for _ in 0..count {
                    if param_idx >= block_params.len() {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            format!(
                                "not enough block parameters for main parameter `{}`",
                                param.name
                            ),
                        ));
                    }
                    vals.push(block_params[param_idx]);
                    param_idx += 1;
                }
                vals
            } else {
                // Scalar parameter
                if param_idx >= block_params.len() {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "not enough block parameters for main parameter `{}`",
                            param.name
                        ),
                    ));
                }
                vec![block_params[param_idx]]
            };

            // Declare parameter as variable and initialize
            let vars = codegen_ctx.declare_variable(param.name.clone(), param.ty.clone())?;
            for (var, val) in vars.iter().zip(param_vals.iter()) {
                codegen_ctx.builder.def_var(*var, *val);
            }
            param_idx += param_vals.len();
        }

        // Translate main function body
        for stmt in &main_func.body {
            codegen_ctx.emit_statement(stmt)?;
        }

        // Generate default return if needed
        crate::frontend::codegen::helpers::generate_default_return(
            &mut codegen_ctx,
            &main_func.return_type,
        )?;

        // Seal all blocks before finalizing (safety net for any blocks not explicitly sealed)
        codegen_ctx.builder.seal_all_blocks();

        // Finalize
        codegen_ctx.builder.finalize();

        // Merge SourceLocManager back into shared one
        source_loc_manager.merge_from(&codegen_ctx.source_loc_manager);

        // Verify function
        cranelift_codegen::verify_function(&ctx.func, isa).map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!(
                    "verifier error in main function: {}\n\nFunction IR:\n{}",
                    e, ctx.func
                ),
            )
        })?;

        Ok(ctx.func)
    }

    /// Set up function builder with entry block
    fn setup_function_builder(builder: &mut FunctionBuilder) -> cranelift_codegen::ir::Block {
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        entry_block
    }
}

impl Default for GlslCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a minimal module for function declarations during compilation
/// This is used to get FuncIds for cross-function calls
pub fn create_minimal_module_for_declarations(
    isa: &dyn cranelift_codegen::isa::TargetIsa,
) -> Result<Box<dyn Module>, GlslError> {
    use crate::error::{ErrorCode, GlslError};
    use core::cell::RefCell;
    use cranelift_codegen::entity::PrimaryMap;
    use cranelift_codegen::isa;

    // Recreate OwnedTargetIsa from the reference
    let isa_builder = isa::Builder::from_target_isa(isa);
    let flags = isa.flags().clone();
    let owned_isa = isa_builder.finish(flags).map_err(|e| {
        GlslError::new(ErrorCode::E0400, format!("failed to recreate ISA: {:?}", e))
    })?;

    struct MinimalModule {
        isa: cranelift_codegen::isa::OwnedTargetIsa,
        func_counter: u32,
        func_ids: HashMap<String, FuncId>,
        functions: RefCell<PrimaryMap<FuncId, cranelift_module::FunctionDeclaration>>,
        names: RefCell<HashMap<String, cranelift_module::FuncOrDataId>>,
        #[cfg(not(feature = "std"))]
        declarations: ModuleDeclarations,
    }

    impl Module for MinimalModule {
        fn isa(&self) -> &dyn cranelift_codegen::isa::TargetIsa {
            self.isa.as_ref()
        }

        fn declarations(&self) -> &ModuleDeclarations {
            #[cfg(feature = "std")]
            {
                use std::sync::OnceLock;
                static EMPTY: OnceLock<ModuleDeclarations> = OnceLock::new();
                EMPTY.get_or_init(|| ModuleDeclarations::default())
            }
            #[cfg(not(feature = "std"))]
            {
                &self.declarations
            }
        }

        fn declare_function(
            &mut self,
            name: &str,
            linkage: Linkage,
            signature: &cranelift_codegen::ir::Signature,
        ) -> ModuleResult<FuncId> {
            if let Some(&id) = self.func_ids.get(name) {
                Ok(id)
            } else {
                let id = FuncId::from_u32(self.func_counter);
                self.func_counter += 1;
                self.func_ids.insert(String::from(name), id);

                let decl = cranelift_module::FunctionDeclaration {
                    name: Some(String::from(name)),
                    linkage,
                    signature: signature.clone(),
                };
                self.functions.borrow_mut().push(decl);
                self.names
                    .borrow_mut()
                    .insert(String::from(name), cranelift_module::FuncOrDataId::Func(id));

                Ok(id)
            }
        }

        fn declare_data(
            &mut self,
            _name: &str,
            _linkage: Linkage,
            _writable: bool,
            _tls: bool,
        ) -> ModuleResult<cranelift_module::DataId> {
            #[cfg(feature = "std")]
            {
                use std::io;
                Err(ModuleError::Allocation {
                    message: "Data declarations not supported",
                    err: io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Data declarations not supported",
                    ),
                })
            }
            #[cfg(not(feature = "std"))]
            {
                use alloc::string::ToString;
                Err(ModuleError::Undeclared(
                    "Data declarations not supported".to_string(),
                ))
            }
        }

        fn define_function_bytes(
            &mut self,
            _id: FuncId,
            _alignment: u64,
            _bytes: &[u8],
            _relocs: &[cranelift_module::ModuleReloc],
        ) -> ModuleResult<()> {
            Ok(())
        }

        fn define_function(
            &mut self,
            _id: FuncId,
            _ctx: &mut cranelift_codegen::Context,
        ) -> ModuleResult<()> {
            Ok(())
        }

        fn define_data(
            &mut self,
            _id: cranelift_module::DataId,
            _data: &cranelift_module::DataDescription,
        ) -> ModuleResult<()> {
            #[cfg(feature = "std")]
            {
                use std::io;
                Err(ModuleError::Allocation {
                    message: "Data definitions not supported",
                    err: io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Data definitions not supported",
                    ),
                })
            }
            #[cfg(not(feature = "std"))]
            {
                use alloc::string::ToString;
                Err(ModuleError::Undeclared(
                    "Data definitions not supported".to_string(),
                ))
            }
        }

        fn declare_anonymous_function(
            &mut self,
            _signature: &cranelift_codegen::ir::Signature,
        ) -> ModuleResult<FuncId> {
            let id = FuncId::from_u32(self.func_counter);
            self.func_counter += 1;
            Ok(id)
        }

        fn declare_anonymous_data(
            &mut self,
            _writable: bool,
            _tls: bool,
        ) -> ModuleResult<cranelift_module::DataId> {
            #[cfg(feature = "std")]
            {
                use std::io;
                Err(ModuleError::Allocation {
                    message: "Data declarations not supported",
                    err: io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Data declarations not supported",
                    ),
                })
            }
            #[cfg(not(feature = "std"))]
            {
                use alloc::string::ToString;
                Err(ModuleError::Undeclared(
                    "Data declarations not supported".to_string(),
                ))
            }
        }

        fn define_function_with_control_plane(
            &mut self,
            _id: FuncId,
            _ctx: &mut cranelift_codegen::Context,
            _ctrl_plane: &mut cranelift_codegen::control::ControlPlane,
        ) -> ModuleResult<()> {
            Ok(())
        }

        fn declare_func_in_func(
            &mut self,
            func_id: FuncId,
            func: &mut cranelift_codegen::ir::Function,
        ) -> cranelift_codegen::ir::FuncRef {
            // Get the function declaration from our stored functions
            let functions = self.functions.borrow();
            let decl = &functions[func_id];
            let signature = func.import_signature(decl.signature.clone());
            let user_name_ref =
                func.declare_imported_user_function(cranelift_codegen::ir::UserExternalName {
                    namespace: 0,
                    index: func_id.as_u32(),
                });
            let colocated = decl.linkage.is_final();
            func.import_function(cranelift_codegen::ir::ExtFuncData {
                name: cranelift_codegen::ir::ExternalName::user(user_name_ref),
                signature,
                colocated,
            })
        }
    }

    Ok(Box::new(MinimalModule {
        isa: owned_isa,
        func_counter: 0,
        func_ids: HashMap::new(),
        functions: RefCell::new(PrimaryMap::new()),
        names: RefCell::new(HashMap::new()),
        #[cfg(not(feature = "std"))]
        declarations: ModuleDeclarations::default(),
    }))
}
