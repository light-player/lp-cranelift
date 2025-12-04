#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use cranelift_codegen::isa::riscv32::isa_builder;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::{
    ir::{types, AbiParam, InstBuilder},
    Context,
};
use cranelift_control::ControlPlane;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use defmt::info;
use embassy_executor::Spawner;
use esp_hal::{clock::CpuClock, timer::systimer::SystemTimer};
use hashbrown::HashMap;
use lp_toy_lang::frontend::{parser, Expr};
use panic_rtt_target as _;
use target_lexicon::Triple;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Allocate heap - ESP32-C6 has plenty of RAM
    esp_alloc::heap_allocator!(size: 128 * 1024); // 128KB heap for Cranelift

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    // Initialize RTT after heap setup
    rtt_target::rtt_init_defmt!();

    info!("======================================");
    info!("ESP32-C6 Toy Language JIT Test");
    info!("Testing Cranelift JIT on Real RISC-V Hardware!");
    info!("======================================\n");

    let source = r#"
fn multiply(a, b) -> (result) {
    result = a * b
}
"#;

    info!("Toy Language Source:\n{}", source);

    // Parse
    info!("Step 1: Parsing...");
    let (_name, params, the_return, stmts) = match parser::function(source) {
        Ok(p) => {
            info!("  ✓ Parsing successful");
            p
        }
        Err(_) => {
            defmt::panic!("Parsing failed");
        }
    };

    // Create ISA
    info!("Step 2: Creating RISC-V32 ISA...");
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

    let isa = match isa_builder(triple).finish(isa_flags) {
        Ok(isa) => {
            info!("  ✓ ISA created");
            isa
        }
        Err(_) => {
            defmt::panic!("ISA creation failed");
        }
    };

    // Build IR
    info!("Step 3: Building Cranelift IR and compiling...");
    let machine_code = match compile_toy_function(params, the_return, stmts, &*isa) {
        Ok(code) => {
            info!("  ✓ Compilation successful: {} bytes", code.len());
            code
        }
        Err(_) => {
            defmt::panic!("Compilation failed");
        }
    };

    info!("Step 4: Executing JIT-compiled function...");

    // Ensure instruction cache coherency
    unsafe {
        core::arch::asm!("fence.i");
    }

    // Cast to function pointer and execute
    type MultiplyFn = extern "C" fn(i32, i32) -> i32;
    let multiply_fn: MultiplyFn = unsafe { core::mem::transmute(machine_code.as_ptr()) };

    let a = 7;
    let b = 6;
    let expected = 42;

    info!("Calling multiply({}, {})", a, b);
    let result = multiply_fn(a, b);

    info!("Result: {}", result);
    info!("Expected: {}", expected);

    if result == expected {
        info!("======================================");
        info!("✅ JIT TEST SUCCESS ON REAL HARDWARE!");
        info!("======================================");
    } else {
        defmt::panic!("JIT test failed: expected {}, got {}", expected, result);
    }

    // Loop forever
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

/// Compile toy language function to RISC-V machine code
fn compile_toy_function(
    params: Vec<alloc::string::String>,
    the_return: alloc::string::String,
    stmts: Vec<Expr>,
    isa: &dyn cranelift_codegen::isa::TargetIsa,
) -> Result<Vec<u8>, alloc::string::String> {
    let mut ctx = Context::new();
    let int = types::I32;

    // Setup function signature
    for _ in &params {
        ctx.func.signature.params.push(AbiParam::new(int));
    }
    ctx.func.signature.returns.push(AbiParam::new(int));

    // Build function
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

    let return_var = builder.declare_var(int);
    let zero = builder.ins().iconst(int, 0);
    variables.insert(the_return.clone(), return_var);
    builder.def_var(return_var, zero);

    // Translate statements
    for expr in &stmts {
        translate_expr(&mut builder, &variables, expr);
    }

    // Return
    let return_value = builder.use_var(return_var);
    builder.ins().return_(&[return_value]);
    builder.finalize();

    // Compile
    let mut ctrl_plane = ControlPlane::default();
    let code_info = ctx
        .compile(isa, &mut ctrl_plane)
        .map_err(|e| alloc::format!("Codegen failed: {:?}", e))?;

    Ok(code_info.buffer.data().to_vec())
}

fn translate_expr(
    builder: &mut FunctionBuilder,
    variables: &HashMap<alloc::string::String, Variable>,
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
