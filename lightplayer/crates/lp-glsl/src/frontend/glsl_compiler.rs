//! GLSL compiler that compiles GLSL source to ClifModule

use crate::frontend::pipeline::CompilationPipeline;
use crate::frontend::src_loc::GlSourceMap;
use crate::error::GlslError;
use crate::backend::ir::ClifModule;
use cranelift_codegen::ir::Function;
use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage, Module, ModuleDeclarations, ModuleError, ModuleResult};
use hashbrown::HashMap;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, vec::Vec};
#[cfg(feature = "std")]
use std::format;
#[cfg(feature = "std")]
use std::{boxed::Box, vec::Vec};

/// GLSL compiler that compiles GLSL source to ClifModule
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

    /// Compile GLSL source to a ClifModule
    /// All functions are compiled with float types initially (no fixed-point conversion)
    pub fn compile_to_clif_module(
        &mut self,
        source: &str,
        isa: OwnedTargetIsa,
    ) -> Result<ClifModule, GlslError> {
        use crate::frontend::codegen::signature::SignatureBuilder;
        use crate::error::{ErrorCode, GlslError};

        // 1. Parse and analyze GLSL
        let semantic_result = CompilationPipeline::parse_and_analyze(source)?;
        let typed_ast = semantic_result.typed_ast;

        // 2. Create a shared source location manager for all functions
        use crate::frontend::src_loc_manager::SourceLocManager;
        let mut source_loc_manager = SourceLocManager::new();

        // 2b. Create a source map and add the main source file
        let mut source_map = GlSourceMap::new();
        let main_file_id = source_map.add_file(
            crate::frontend::src_loc::GlFileSource::Synthetic(String::from("main.glsl")),
            String::from(source),
        );

        // 3. Create a temporary minimal module for function declarations
        //    This is needed to get FuncIds for cross-function calls
        let mut temp_module = create_minimal_module_for_declarations(isa.as_ref())?;

        // 3. Declare all user functions with FLOAT signatures (no conversion)
        let mut func_ids: HashMap<String, FuncId> = HashMap::new();
        let pointer_type = isa.pointer_type();
        let triple = isa.triple();

        for user_func in &typed_ast.user_functions {
            let sig = SignatureBuilder::build_with_triple(
                &user_func.return_type,
                &user_func.parameters,
                pointer_type,
                triple,
            );
            let func_id = temp_module
                .declare_function(&user_func.name, Linkage::Local, &sig)
                .map_err(|e| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!("failed to declare function '{}': {}", user_func.name, e),
                    )
                })?;
            func_ids.insert(user_func.name.clone(), func_id);
        }

        // 4. Compile all user functions to CLIF with FLOAT types
        let mut user_functions = HashMap::new();
        let mut glsl_signatures = HashMap::new();
        let mut func_id_to_name = HashMap::new();
        for user_func in &typed_ast.user_functions {
            let func_id = func_ids[&user_func.name];
            let func = self.compile_function_to_clif(
                user_func,
                func_id,
                &func_ids,
                &typed_ast.function_registry,
                &mut temp_module,
                isa.as_ref(),
                &mut source_loc_manager,
                &source_map,
                main_file_id,
            )?;
            user_functions.insert(user_func.name.clone(), func);

            // Store GLSL signature
            glsl_signatures.insert(
                user_func.name.clone(),
                crate::frontend::semantic::functions::FunctionSignature {
                    name: user_func.name.clone(),
                    return_type: user_func.return_type.clone(),
                    parameters: user_func.parameters.clone(),
                },
            );

            // Store FuncId -> name mapping for linking
            func_id_to_name.insert(func_id.as_u32(), user_func.name.clone());
        }

        // 5. Compile main function to CLIF with FLOAT types
        let main_func = self.compile_main_function_to_clif(
            &typed_ast.main_function,
            &func_ids,
            &typed_ast.function_registry,
            &mut temp_module,
            isa.as_ref(),
            semantic_result.source,
            &mut source_loc_manager,
            &source_map,
            main_file_id,
        )?;

        // Store main function's GLSL signature
        glsl_signatures.insert(
            String::from("main"),
            crate::frontend::semantic::functions::FunctionSignature {
                name: String::from("main"),
                return_type: typed_ast.main_function.return_type.clone(),
                parameters: typed_ast.main_function.parameters.clone(),
            },
        );

        // Store main's FuncId -> name mapping (main is declared separately, but we need to track it)
        // Note: main doesn't have a FuncId in func_ids, but we can infer it if needed
        // For now, we'll handle main separately in link_into

        // 6. Build and return ClifModule
        Ok(ClifModule::builder()
            .set_function_registry(typed_ast.function_registry)
            .set_source_text(String::from(source))
            .set_isa(isa)
            .add_user_functions(user_functions)
            .set_main_function(main_func)
            .add_glsl_signatures(glsl_signatures)
            .add_func_id_mappings(func_id_to_name)
            .set_source_loc_manager(source_loc_manager)
            .set_source_map(source_map)
            .build()?)
    }

    /// Compile GLSL source to machine code bytes
    /// This is a convenience method for embedded targets that need raw machine code
    pub fn compile_to_code(
        &mut self,
        source: &str,
        isa: &dyn cranelift_codegen::isa::TargetIsa,
    ) -> Result<Vec<u8>, GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use cranelift_codegen::Context;
        use cranelift_control::ControlPlane;

        // Compile to CLIF module
        let isa_owned = {
            let isa_builder = cranelift_codegen::isa::Builder::from_target_isa(isa);
            let flags = isa.flags().clone();
            isa_builder.finish(flags).map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to recreate ISA: {:?}", e),
                )
            })?
        };
        let module = self.compile_to_clif_module(source, isa_owned)?;

        // Compile the main function to machine code
        let main_func = module.main_function();
        let mut ctx = Context::for_function(main_func.clone());
        let mut ctrl_plane = ControlPlane::default();
        let compiled_code = ctx.compile(isa, &mut ctrl_plane).map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!("failed to compile function: {:?}", e),
            )
        })?;

        // Get the machine code bytes
        let mut code = Vec::new();
        code.extend_from_slice(compiled_code.buffer.data());
        Ok(code)
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
        use crate::frontend::codegen::signature::SignatureBuilder;
        use crate::error::{ErrorCode, GlslError};
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
        let mut codegen_ctx = crate::frontend::codegen::context::CodegenContext::new(builder, temp_module, source_map, file_id);
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
        crate::frontend::codegen::helpers::generate_default_return(&mut codegen_ctx, &func.return_type)?;

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
        use crate::frontend::codegen::signature::SignatureBuilder;
        use crate::error::{ErrorCode, GlslError};
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
        let mut codegen_ctx = crate::frontend::codegen::context::CodegenContext::new(builder, temp_module, source_map, file_id);
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
        crate::frontend::codegen::helpers::generate_default_return(&mut codegen_ctx, &main_func.return_type)?;

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
    use cranelift_codegen::entity::PrimaryMap;
    use cranelift_codegen::isa;
    use core::cell::RefCell;

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
