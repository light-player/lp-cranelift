//! Test toy language compilation on the host with std to debug issues

use cranelift_codegen::{Context, ir::{AbiParam, InstBuilder, types}};
use cranelift_codegen::isa::riscv32::isa_builder;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use std::collections::HashMap;
use target_lexicon::Triple;
use lp_toy_lang::frontend::{Expr, parser};

#[test]
fn test_compile_multiply_on_host() {
    println!("\n=== Testing Toy Language Compilation on Host ===\n");
    
    let source = r#"
fn add(a, b) -> (result) {
    result = a * b
}
"#;
    
    println!("Source code:\n{}\n", source);
    
    // Parse
    println!("Step 1: Parsing...");
    let (_name, params, the_return, stmts) = parser::function(source)
        .expect("Parsing failed");
    println!("  ✓ Parsed: {} params, {} stmts\n", params.len(), stmts.len());
    
    // Create RISC-V32 ISA
    println!("Step 2: Creating RISC-V32 ISA...");
    let mut flag_builder = settings::builder();
    flag_builder.set("opt_level", "none").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    flag_builder.set("enable_verifier", "false").unwrap();
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
        .expect("ISA creation failed");
    println!("  ✓ ISA created\n");
    
    // Build IR
    println!("Step 3: Building Cranelift IR...");
    let mut ctx = Context::new();
    let int = types::I32;
    
    for _ in &params {
        ctx.func.signature.params.push(AbiParam::new(int));
    }
    ctx.func.signature.returns.push(AbiParam::new(int));
    
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);
    
    let mut variables = HashMap::new();
    for (i, name) in params.iter().enumerate() {
        let val = builder.block_params(entry_block)[i];
        let var = builder.declare_var(int);
        variables.insert(name.clone(), var);
        builder.def_var(var, val);
    }
    
    let return_var = builder.declare_var(int);
    let zero = builder.ins().iconst(int, 0);
    variables.insert(the_return.clone(), return_var);
    builder.def_var(return_var, zero);
    
    for expr in &stmts {
        translate_expr(&mut builder, &variables, expr);
    }
    
    let return_value = builder.use_var(return_var);
    builder.ins().return_(&[return_value]);
    builder.finalize();
    
    println!("  ✓ IR built\n");
    
    // Compile
    println!("Step 4: Compiling to RISC-V machine code...");
    println!("  Function: {} blocks, {} instructions", 
             ctx.func.dfg.num_blocks(), ctx.func.dfg.num_insts());
    
    let code_info = ctx
        .compile(&*isa, &mut Default::default())
        .expect("Compilation failed");
    
    println!("  ✓ Compilation succeeded!");
    println!("  Generated {} bytes of machine code\n", code_info.buffer.data().len());
    println!("  First 32 bytes: {:02x?}\n", &code_info.buffer.data()[..32.min(code_info.buffer.data().len())]);
    
    println!("=== ✅ Host Compilation Test PASSED ===\n");
}

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

