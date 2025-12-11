//! Shared code for RISC-V JIT testing using Cranelift.
//!
//! This crate provides common functionality for building and compiling
//! toy language code to RISC-V that can be used both in the embive VM and on real hardware.

#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use cranelift_codegen::ir::{types, AbiParam, InstBuilder};
use cranelift_codegen::isa::riscv32::isa_builder as riscv32_builder;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use lp_toy_lang::frontend::{parser, Expr};
use target_lexicon::Triple;

#[cfg(feature = "std")]
use hashbrown::HashMap;
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

mod simple_elf;

/// Result of JIT compilation
pub struct JitResult {
    /// ELF file data
    pub elf: Vec<u8>,
}

/// Compile a toy language function to RISC-V ELF.
///
/// This function takes toy language source code and compiles it to a RISC-V ELF file
/// that can be transpiled and executed by embive.
pub fn compile_toy_to_elf(source: &str) -> Result<JitResult, String> {
    // Parse the toy language source
    let (_name, params, the_return, stmts) = parser::function(source).map_err(|e| {
        #[cfg(feature = "std")]
        {
            format!("Parse error: {}", e)
        }
        #[cfg(not(feature = "std"))]
        {
            use core::fmt::Write;
            let mut s = String::new();
            let _ = write!(&mut s, "Parse error: {}", e);
            s
        }
    })?;

    // Create RISC-V 32-bit ISA with settings
    let mut flag_builder = settings::builder();
    flag_builder.set("opt_level", "speed").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    let isa_flags = settings::Flags::new(flag_builder);

    let triple = Triple {
        architecture: target_lexicon::Architecture::Riscv32(
            target_lexicon::Riscv32Architecture::Riscv32imac,
        ),
        vendor: target_lexicon::Vendor::Unknown,
        operating_system: target_lexicon::OperatingSystem::None_,
        environment: target_lexicon::Environment::Unknown,
        binary_format: target_lexicon::BinaryFormat::Elf,
    };

    let isa = riscv32_builder(triple)
        .finish(isa_flags)
        .map_err(|e| {
            #[cfg(feature = "std")]
            {
                format!("Failed to create ISA: {}", e)
            }
            #[cfg(not(feature = "std"))]
            {
                use core::fmt::Write;
                let mut s = String::new();
                let _ = write!(&mut s, "Failed to create ISA: {}", e);
                s
            }
        })?;

    // Build the function
    let mut ctx = Context::new();
    let int = types::I32; // Use I32 for RISC-V 32-bit

    // Create function signature
    for _ in &params {
        ctx.func.signature.params.push(AbiParam::new(int));
    }
    ctx.func.signature.returns.push(AbiParam::new(int));

    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);

    // Create entry block
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    // Declare variables
    let variables = declare_variables(int, &mut builder, &params, &the_return, &stmts, entry_block);

    // Translate statements
    let mut trans = FunctionTranslator {
        int,
        builder,
        variables,
    };
    for expr in stmts {
        trans.translate_expr(expr);
    }

    // Return the return variable
    let return_variable = trans.variables.get(&the_return).unwrap();
    let return_value = trans.builder.use_var(*return_variable);
    trans.builder.ins().return_(&[return_value]);

    trans.builder.finalize();

    // Compile to machine code
    let code_info = ctx.compile(&*isa, &mut Default::default()).map_err(|_| {
        #[cfg(feature = "std")]
        {
            "Failed to compile".to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            String::from("Failed to compile")
        }
    })?;

    // Extract the machine code from the buffer
    let code = code_info.buffer.data().to_vec();

    // Generate simple ELF file
    let elf = simple_elf::generate_simple_elf(&code);

    Ok(JitResult { elf })
}

/// Helper function to compile a simple add function for testing
pub fn compile_add_function() -> JitResult {
    let source = r#"
fn add(a, b) -> (result) {
    result = a + b
}
"#;
    compile_toy_to_elf(source).expect("Failed to compile add function")
}

// Function translator (simplified version from lp-toy-lang)
struct FunctionTranslator<'a> {
    int: types::Type,
    builder: FunctionBuilder<'a>,
    variables: HashMap<String, cranelift_frontend::Variable>,
}

impl<'a> FunctionTranslator<'a> {
    fn translate_expr(&mut self, expr: Expr) -> cranelift_codegen::ir::Value {
        use cranelift_codegen::ir::condcodes::IntCC;

        match expr {
            Expr::Literal(literal) => {
                let imm: i32 = literal.parse().unwrap();
                self.builder.ins().iconst(self.int, i64::from(imm))
            }
            Expr::Add(lhs, rhs) => {
                let lhs = self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                self.builder.ins().iadd(lhs, rhs)
            }
            Expr::Sub(lhs, rhs) => {
                let lhs = self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                self.builder.ins().isub(lhs, rhs)
            }
            Expr::Mul(lhs, rhs) => {
                let lhs = self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                self.builder.ins().imul(lhs, rhs)
            }
            Expr::Div(lhs, rhs) => {
                let lhs = self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                self.builder.ins().udiv(lhs, rhs)
            }
            Expr::Eq(lhs, rhs) => self.translate_icmp(IntCC::Equal, *lhs, *rhs),
            Expr::Ne(lhs, rhs) => self.translate_icmp(IntCC::NotEqual, *lhs, *rhs),
            Expr::Lt(lhs, rhs) => self.translate_icmp(IntCC::SignedLessThan, *lhs, *rhs),
            Expr::Le(lhs, rhs) => self.translate_icmp(IntCC::SignedLessThanOrEqual, *lhs, *rhs),
            Expr::Gt(lhs, rhs) => self.translate_icmp(IntCC::SignedGreaterThan, *lhs, *rhs),
            Expr::Ge(lhs, rhs) => self.translate_icmp(IntCC::SignedGreaterThanOrEqual, *lhs, *rhs),
            Expr::Identifier(name) => {
                let variable = self.variables.get(&name).expect("variable not defined");
                self.builder.use_var(*variable)
            }
            Expr::Assign(name, expr) => {
                let new_value = self.translate_expr(*expr);
                let variable = self.variables.get(&name).unwrap();
                self.builder.def_var(*variable, new_value);
                new_value
            }
            Expr::IfElse(condition, then_body, else_body) => {
                self.translate_if_else(*condition, then_body, else_body)
            }
            Expr::WhileLoop(condition, loop_body) => {
                self.translate_while_loop(*condition, loop_body)
            }
            Expr::Call(_, _) | Expr::GlobalDataAddr(_) => {
                // Not supported in this simplified version
                self.builder.ins().iconst(self.int, 0)
            }
        }
    }

    fn translate_icmp(
        &mut self,
        cmp: cranelift_codegen::ir::condcodes::IntCC,
        lhs: Expr,
        rhs: Expr,
    ) -> cranelift_codegen::ir::Value {
        let lhs = self.translate_expr(lhs);
        let rhs = self.translate_expr(rhs);
        self.builder.ins().icmp(cmp, lhs, rhs)
    }

    fn translate_if_else(
        &mut self,
        condition: Expr,
        then_body: Vec<Expr>,
        else_body: Vec<Expr>,
    ) -> cranelift_codegen::ir::Value {
        use cranelift_codegen::ir::BlockArg;

        let condition_value = self.translate_expr(condition);

        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        let merge_block = self.builder.create_block();

        self.builder.append_block_param(merge_block, self.int);

        self.builder
            .ins()
            .brif(condition_value, then_block, &[], else_block, &[]);

        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);
        let mut then_return = self.builder.ins().iconst(self.int, 0);
        for expr in then_body {
            then_return = self.translate_expr(expr);
        }
        self.builder
            .ins()
            .jump(merge_block, &[BlockArg::Value(then_return)]);

        self.builder.switch_to_block(else_block);
        self.builder.seal_block(else_block);
        let mut else_return = self.builder.ins().iconst(self.int, 0);
        for expr in else_body {
            else_return = self.translate_expr(expr);
        }
        self.builder
            .ins()
            .jump(merge_block, &[BlockArg::Value(else_return)]);

        self.builder.switch_to_block(merge_block);
        self.builder.seal_block(merge_block);

        self.builder.block_params(merge_block)[0]
    }

    fn translate_while_loop(
        &mut self,
        condition: Expr,
        loop_body: Vec<Expr>,
    ) -> cranelift_codegen::ir::Value {
        let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit_block = self.builder.create_block();

        self.builder.ins().jump(header_block, &[]);
        self.builder.switch_to_block(header_block);

        let condition_value = self.translate_expr(condition);
        self.builder
            .ins()
            .brif(condition_value, body_block, &[], exit_block, &[]);

        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);

        for expr in loop_body {
            self.translate_expr(expr);
        }
        self.builder.ins().jump(header_block, &[]);

        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(header_block);
        self.builder.seal_block(exit_block);

        self.builder.ins().iconst(self.int, 0)
    }
}

fn declare_variables(
    int: types::Type,
    builder: &mut FunctionBuilder,
    params: &[String],
    the_return: &str,
    stmts: &[Expr],
    entry_block: cranelift_codegen::ir::Block,
) -> HashMap<String, cranelift_frontend::Variable> {
    let mut variables = HashMap::new();
    for (i, name) in params.iter().enumerate() {
        let val = builder.block_params(entry_block)[i];
        let var = declare_variable(int, builder, &mut variables, name);
        builder.def_var(var, val);
    }
    let zero = builder.ins().iconst(int, 0);
    let return_variable = declare_variable(int, builder, &mut variables, the_return);
    builder.def_var(return_variable, zero);
    for expr in stmts {
        declare_variables_in_stmt(int, builder, &mut variables, expr);
    }

    variables
}

fn declare_variables_in_stmt(
    int: types::Type,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, cranelift_frontend::Variable>,
    expr: &Expr,
) {
    match *expr {
        Expr::Assign(ref name, _) => {
            declare_variable(int, builder, variables, name);
        }
        Expr::IfElse(ref _condition, ref then_body, ref else_body) => {
            for stmt in then_body {
                declare_variables_in_stmt(int, builder, variables, stmt);
            }
            for stmt in else_body {
                declare_variables_in_stmt(int, builder, variables, stmt);
            }
        }
        Expr::WhileLoop(ref _condition, ref loop_body) => {
            for stmt in loop_body {
                declare_variables_in_stmt(int, builder, variables, stmt);
            }
        }
        _ => (),
    }
}

fn declare_variable(
    int: types::Type,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, cranelift_frontend::Variable>,
    name: &str,
) -> cranelift_frontend::Variable {
    use hashbrown::hash_map::Entry;
    match variables.entry(name.into()) {
        Entry::Occupied(e) => *e.get(),
        Entry::Vacant(e) => {
            let var = builder.declare_var(int);
            *e.insert(var)
        }
    }
}

