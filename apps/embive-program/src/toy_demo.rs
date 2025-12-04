//! Toy language JIT demonstration - TRUE runtime compilation!
//!
//! This demonstrates Cranelift compiling toy language source code at runtime
//! in a no_std RISC-V environment.

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use cranelift_codegen::Context;
use cranelift_codegen::ir::{AbiParam, InstBuilder, types};
use cranelift_codegen::isa::riscv32::isa_builder;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use hashbrown::HashMap;
use lp_toy_lang::frontend::{Expr, parser};
use target_lexicon::Triple;

/// Run the toy language JIT demonstration.
///
/// This performs REAL JIT compilation:
/// 1. Parse toy language source code
/// 2. Compile with Cranelift to RISC-V machine code
/// 3. Execute the compiled function
/// All happening at runtime in no_std on RISC-V!
pub fn run_toy_demo() -> i32 {
    println!("");
    println!("=== Toy Language Runtime JIT Demo ===");

    let source = r#"
fn add(a, b) -> (result) {
    result = a + b
}
"#;

    println!("Source code:");
    println!("  fn add(a, b) -> (result) {{");
    println!("      result = a + b");
    println!("  }}");
    println!("");

    // PARSE at runtime in no_std!
    println!("Parsing toy language...");
    let (_name, params, the_return, stmts) = match parser::function(source) {
        Ok(parsed) => {
            println!("✓ Parse successful");
            parsed
        }
        Err(_e) => {
            println!("✗ Parse failed");
            return -1;
        }
    };

    // COMPILE with Cranelift at runtime in no_std!
    println!("Compiling with Cranelift...");
    let machine_code = match compile_to_code(&params, &the_return, &stmts) {
        Ok(code) => {
            println!("✓ Compilation successful: {} bytes", code.len());
            println!("  Code: {:02x?}", &code[0..code.len().min(32)]);
            code
        }
        Err(e) => {
            println!("✗ Compilation failed: {:?}", e);
            return -1;
        }
    };

    // EXECUTE the JIT-compiled function!
    println!("");
    println!("Executing: add(5, 3)");
    type AddFn = extern "C" fn(i32, i32) -> i32;
    let add_fn: AddFn = unsafe { core::mem::transmute(machine_code.as_ptr()) };
    let result = add_fn(5, 3);
    println!("✓ Result: {}", result);
    println!("");

    result
}

/// Compile toy language AST to RISC-V machine code.
fn compile_to_code(params: &[String], the_return: &str, stmts: &[Expr]) -> Result<Vec<u8>, String> {
    // Create RISC-V ISA
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

    let isa = isa_builder(triple)
        .finish(isa_flags)
        .map_err(|_| String::from("ISA creation failed"))?;

    // Build function
    let mut ctx = Context::new();
    let int = types::I32;

    // Signature
    for _ in params {
        ctx.func.signature.params.push(AbiParam::new(int));
    }
    ctx.func.signature.returns.push(AbiParam::new(int));

    // Build IR from AST
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    // Declare variables
    let mut variables = HashMap::new();
    for (i, name) in params.iter().enumerate() {
        let val = builder.block_params(entry_block)[i];
        let var = builder.declare_var(int);
        variables.insert(name.clone(), var);
        builder.def_var(var, val);
    }

    let zero = builder.ins().iconst(int, 0);
    let return_var = builder.declare_var(int);
    variables.insert(the_return.to_string(), return_var);
    builder.def_var(return_var, zero);

    // Translate statements
    for expr in stmts {
        translate_expr(&mut builder, &variables, expr);
    }

    // Return
    let return_value = builder.use_var(return_var);
    builder.ins().return_(&[return_value]);
    builder.finalize();

    // Compile to machine code
    let code_info = ctx
        .compile(&*isa, &mut Default::default())
        .map_err(|_| String::from("Codegen failed"))?;

    Ok(code_info.buffer.data().to_vec())
}

/// Translate a toy language expression to Cranelift IR.
fn translate_expr(
    builder: &mut FunctionBuilder,
    variables: &HashMap<String, Variable>,
    expr: &Expr,
) -> cranelift_codegen::ir::Value {
    match expr {
        Expr::Literal(lit) => {
            let imm: i32 = lit.parse().unwrap_or(0);
            builder.ins().iconst(types::I32, i64::from(imm))
        }
        Expr::Identifier(name) => {
            let var = variables.get(name).unwrap();
            builder.use_var(*var)
        }
        Expr::Assign(name, expr) => {
            let value = translate_expr(builder, variables, expr);
            let var = variables.get(name).unwrap();
            builder.def_var(*var, value);
            value
        }
        Expr::Add(lhs, rhs) => {
            let l = translate_expr(builder, variables, lhs);
            let r = translate_expr(builder, variables, rhs);
            builder.ins().iadd(l, r)
        }
        Expr::Sub(lhs, rhs) => {
            let l = translate_expr(builder, variables, lhs);
            let r = translate_expr(builder, variables, rhs);
            builder.ins().isub(l, r)
        }
        Expr::Mul(lhs, rhs) => {
            let l = translate_expr(builder, variables, lhs);
            let r = translate_expr(builder, variables, rhs);
            builder.ins().imul(l, r)
        }
        _ => builder.ins().iconst(types::I32, 0),
    }
}
