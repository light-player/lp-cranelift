#[cfg(feature = "std")]
use std::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
use std::format;
#[cfg(not(feature = "std"))]
use alloc::format;

use cranelift_codegen::ir::{AbiParam, InstBuilder};
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::Context as CodegenContext;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};

pub struct JIT {
    builder_context: FunctionBuilderContext,
    ctx: CodegenContext,
    #[allow(dead_code)] // Will be used in future phases
    data_description: DataDescription,
    module: JITModule,
    function_counter: usize,
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
        }
    }

    /// Compile GLSL source to machine code and return function pointer
    pub fn compile(&mut self, glsl_source: &str) -> Result<*const u8, String> {
        // Clear the context for a fresh compilation
        self.ctx.clear();

        // 1. Parse GLSL
        let shader = glsl::parser::Parse::parse(glsl_source)
            .map_err(|e| format!("Parse error: {:?}", e))?;

        // 2. Semantic analysis
        let typed_ast = crate::semantic::analyze(&shader)?;

        // 3. Generate Cranelift IR
        self.translate(typed_ast)?;

        // 4. Verify the function
        if let Err(e) = cranelift_codegen::verify_function(&self.ctx.func, self.module.isa()) {
            return Err(format!("Verification error: {}", e));
        }

        // 5. Declare function with unique name
        let func_name = format!("glsl_main_{}", self.function_counter);
        self.function_counter += 1;

        let id = self
            .module
            .declare_function(&func_name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        // 6. Define function
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| format!("Compilation error: {}", e))?;

        // 7. Finalize
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions().unwrap();

        // 8. Get function pointer
        let code = self.module.get_finalized_function(id);
        Ok(code)
    }

    /// Compile and return CLIF IR as string (for filetests)
    pub fn compile_to_clif(&mut self, glsl_source: &str) -> Result<String, String> {
        self.ctx.clear();

        // 1. Parse GLSL
        let shader = glsl::parser::Parse::parse(glsl_source)
            .map_err(|e| format!("Parse error: {:?}", e))?;

        // 2. Semantic analysis
        let typed_ast = crate::semantic::analyze(&shader)?;

        // 3. Generate Cranelift IR
        self.translate(typed_ast)?;

        // 4. Return as string
        Ok(format!("{}", self.ctx.func))
    }

    fn translate(&mut self, typed_ast: crate::semantic::TypedShader) -> Result<(), String> {
        // Determine return type from the main function
        let return_type = typed_ast.main_function.return_type.to_cranelift_type();
        if !matches!(typed_ast.main_function.return_type, crate::semantic::types::Type::Void) {
            self.ctx.func.signature.returns.push(AbiParam::new(return_type));
        }

        // Create function builder
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Create codegen context
        let mut ctx = crate::codegen::context::CodegenContext::new(builder, &mut self.module);

        // Translate main function body
        for stmt in &typed_ast.main_function.body {
            ctx.translate_statement(stmt)?;
        }

        // Get return value (if any)
        let return_val = if let Some(ret_expr) = &typed_ast.main_function.return_expr {
            ctx.translate_expr(ret_expr)?
        } else {
            // Default return 0
            ctx.builder.ins().iconst(return_type, 0)
        };

        // Emit return
        ctx.builder.ins().return_(&[return_val]);

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

