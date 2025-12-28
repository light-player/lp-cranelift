//! Compiler for intrinsic GLSL functions.

use crate::error::{ErrorCode, GlslError};
use crate::frontend::pipeline::CompilationPipeline;
use crate::frontend::src_loc::GlSourceMap;
use cranelift_codegen::ir::Function;

use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;

use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, FuncOrDataId, Linkage, Module};

use crate::frontend::codegen::context::CodegenContext;
use crate::frontend::codegen::signature::SignatureBuilder;

/// Compile GLSL source containing intrinsic functions into Function objects.
///
/// This function parses GLSL source that may contain multiple functions,
/// compiles each function independently, and returns a map of function
/// name -> Function object.
///
/// Functions are compiled directly into the provided module, ensuring
/// that function references use the correct FuncIds.
///
/// If `functions_to_compile` is Some, only those functions (and their dependencies)
/// will be compiled. If None, all functions are compiled.
pub fn compile_intrinsic_functions<M: Module>(
    glsl_source: &str,
    gl_module: &mut crate::backend::module::gl_module::GlModule<M>,
    source_map: &mut GlSourceMap,
    file_id: crate::frontend::src_loc::GlFileId,
    functions_to_compile: Option<&hashbrown::HashSet<String>>,
) -> Result<hashbrown::HashMap<String, Function>, GlslError> {
    // 1. Parse and analyze GLSL
    let semantic_result = CompilationPipeline::parse_and_analyze(glsl_source)?;
    let typed_ast = semantic_result.typed_ast;

    // 2. Filter functions to compile if specified
    let functions_to_compile_set: hashbrown::HashSet<String> =
        if let Some(set) = functions_to_compile {
            set.clone()
        } else {
            // Compile all functions
            typed_ast
                .user_functions
                .iter()
                .map(|f| f.name.clone())
                .collect()
        };

    // 3. Declare all user functions in the real module and get their FuncIds
    // We need to declare all functions (even if not compiling) so they can be called

    // First pass: Check which functions are already declared (immutable borrows only)
    let mut existing_func_ids: HashMap<String, FuncId> = HashMap::new();
    let mut functions_to_declare: Vec<&crate::frontend::semantic::TypedFunction> = Vec::new();

    for user_func in &typed_ast.user_functions {
        // Skip main() function
        if user_func.name == "main" {
            continue;
        }

        if let Some(FuncOrDataId::Func(id)) = gl_module
            .module_internal()
            .declarations()
            .get_name(&user_func.name)
        {
            // Function already declared, use existing ID
            existing_func_ids.insert(user_func.name.clone(), id);
        } else {
            // Queue for declaration
            functions_to_declare.push(user_func);
        }
    }

    // Second pass: Declare functions that need declaring (mutable borrow)
    let mut func_ids = existing_func_ids;
    for user_func in functions_to_declare {
        // Get ISA info for this function (borrow, use, then drop before mutable borrow)
        let sig = {
            let isa = gl_module.module_internal().isa();
            let pointer_type = isa.pointer_type();
            let triple = isa.triple();
            SignatureBuilder::build_with_triple(
                &user_func.return_type,
                &user_func.parameters,
                pointer_type,
                triple,
            )
        }; // isa reference dropped here

        // Declare function in real module (mutable borrow, isa reference is already dropped)
        let func_id = gl_module
            .module_mut_internal()
            .declare_function(&user_func.name, Linkage::Local, &sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to declare function {}: {}", user_func.name, e),
                )
            })?;
        func_ids.insert(user_func.name.clone(), func_id);
    }

    // 4. Compile each function (only those in functions_to_compile_set)
    let mut compiled_functions = hashbrown::HashMap::new();
    for user_func in &typed_ast.user_functions {
        // Skip if not in the set of functions to compile
        if !functions_to_compile_set.contains(&user_func.name) {
            continue;
        }

        // Build signature in a block to drop isa reference before mutable borrow
        let sig = {
            let isa = gl_module.module_internal().isa();
            let pointer_type = isa.pointer_type();
            let triple = isa.triple();
            SignatureBuilder::build_with_triple(
                &user_func.return_type,
                &user_func.parameters,
                pointer_type,
                triple,
            )
        }; // isa reference dropped here

        let mut ctx = Context::new();
        ctx.func.signature = sig;

        let mut builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        {
            let mut codegen_ctx = CodegenContext::new(builder, gl_module, source_map, file_id);
            // Use real module function IDs - these are the correct FuncIds
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
