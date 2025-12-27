//! Compiler for intrinsic GLSL functions.

use crate::error::{ErrorCode, GlslError};
use crate::frontend::pipeline::CompilationPipeline;
use crate::frontend::src_loc::GlSourceMap;
use cranelift_codegen::ir::Function;
use cranelift_codegen::isa::TargetIsa;

use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;

use cranelift_codegen::{control::ControlPlane, Context};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage, Module, ModuleDeclarations, ModuleError, ModuleResult};

use crate::frontend::codegen::context::CodegenContext;
use crate::frontend::codegen::signature::SignatureBuilder;

/// Compile GLSL source containing intrinsic functions into Function objects.
///
/// This function parses GLSL source that may contain multiple functions,
/// compiles each function independently, and returns a map of function
/// name -> Function object.
pub fn compile_intrinsic_functions(
    glsl_source: &str,
    isa: &dyn TargetIsa,
) -> Result<hashbrown::HashMap<String, Function>, GlslError> {
    // 1. Parse and analyze GLSL
    let semantic_result = CompilationPipeline::parse_and_analyze(glsl_source)?;
    let typed_ast = semantic_result.typed_ast;

    // 1b. Create a source map for intrinsic functions
    let mut source_map = GlSourceMap::new();
    let intrinsic_file_id = source_map.add_file(
        crate::frontend::src_loc::GlFileSource::Synthetic(String::from("intrinsics.glsl")),
        String::from(glsl_source),
    );

    // 2. Create a minimal module stub for function declarations
    struct MinimalModule<'a> {
        isa: &'a dyn TargetIsa,
        func_counter: u32,
        func_ids: HashMap<String, FuncId>,
        declarations: ModuleDeclarations,
    }

    impl<'a> Module for MinimalModule<'a> {
        fn isa(&self) -> &dyn TargetIsa {
            self.isa
        }

        fn declarations(&self) -> &ModuleDeclarations {
            &self.declarations
        }

        fn declare_function(
            &mut self,
            name: &str,
            _linkage: Linkage,
            _signature: &cranelift_codegen::ir::Signature,
        ) -> ModuleResult<FuncId> {
            if let Some(&id) = self.func_ids.get(name) {
                Ok(id)
            } else {
                let id = FuncId::from_u32(self.func_counter);
                self.func_counter += 1;
                self.func_ids.insert(String::from(name), id);
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
                    message: "Data declarations not supported in intrinsic compilation",
                    err: io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Data declarations not supported",
                    ),
                })
            }
            #[cfg(not(feature = "std"))]
            {
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

        fn define_function(&mut self, _id: FuncId, _ctx: &mut Context) -> ModuleResult<()> {
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
                    message: "Data definitions not supported in intrinsic compilation",
                    err: io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Data definitions not supported",
                    ),
                })
            }
            #[cfg(not(feature = "std"))]
            {
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
                    message: "Data declarations not supported in intrinsic compilation",
                    err: io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Data declarations not supported",
                    ),
                })
            }
            #[cfg(not(feature = "std"))]
            {
                Err(ModuleError::Undeclared(
                    "Data declarations not supported".to_string(),
                ))
            }
        }

        fn define_function_with_control_plane(
            &mut self,
            _id: FuncId,
            _ctx: &mut Context,
            _ctrl_plane: &mut ControlPlane,
        ) -> ModuleResult<()> {
            Ok(())
        }
    }

    let mut module = MinimalModule {
        isa,
        func_counter: 0,
        func_ids: HashMap::new(),
        declarations: ModuleDeclarations::default(),
    };

    // 3. Declare all user functions and get their FuncIds
    let mut func_ids: HashMap<String, FuncId> = HashMap::new();
    for user_func in &typed_ast.user_functions {
        let func_id = module
            .declare_function(
                &user_func.name,
                Linkage::Local,
                &cranelift_codegen::ir::Signature::new(
                    cranelift_codegen::isa::CallConv::triple_default(isa.triple()),
                ),
            )
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to declare function: {}", e),
                )
            })?;
        func_ids.insert(user_func.name.clone(), func_id);
    }

    // 4. Compile each function
    let mut compiled_functions = hashbrown::HashMap::new();
    for user_func in &typed_ast.user_functions {
        let mut ctx = Context::new();
        let pointer_type = isa.pointer_type();
        let triple = isa.triple();
        let sig = SignatureBuilder::build_with_triple(
            &user_func.return_type,
            &user_func.parameters,
            pointer_type,
            triple,
        );
        ctx.func.signature = sig;

        let mut builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        {
            let mut codegen_ctx =
                CodegenContext::new(builder, &mut module, &source_map, intrinsic_file_id);
            codegen_ctx.set_function_ids(&func_ids);
            codegen_ctx.set_function_registry(&typed_ast.function_registry);
            codegen_ctx.set_return_type(user_func.return_type.clone());
            codegen_ctx.set_entry_block(entry_block);

            // Declare function parameters as variables
            let block_params = codegen_ctx.builder.block_params(entry_block).to_vec();
            let uses_struct_return = codegen_ctx
                .builder
                .func
                .signature
                .uses_special_param(cranelift_codegen::ir::ArgumentPurpose::StructReturn);

            let mut param_idx = if uses_struct_return { 1 } else { 0 };

            for param in &user_func.parameters {
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
            for stmt in &user_func.body {
                codegen_ctx.emit_statement(stmt)?;
            }

            // Generate default return if needed
            crate::frontend::codegen::helpers::generate_default_return(
                &mut codegen_ctx,
                &user_func.return_type,
            )?;

            codegen_ctx.builder.finalize();
        } // codegen_ctx and builder are dropped here

        // Extract the compiled function from the context (builder is dropped, so we can move)
        let compiled_func = ctx.func;
        compiled_functions.insert(user_func.name.clone(), compiled_func);
    }

    Ok(compiled_functions)
}
