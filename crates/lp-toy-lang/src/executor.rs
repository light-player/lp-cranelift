//! Emulator-based executor for toy language programs.

use lpc_codegen::{compile_module_to_insts, Gpr, Riscv32Emulator, StepResult};
use lpc_lpir::{Function, FunctionBuilder, FunctionBuilderContext, Module, Signature, Value};

/// Execute a toy language function using the emulator.
///
/// Creates a module with the function, compiles it, and executes it.
/// Returns the result value from the function.
pub fn execute_function(func: Function, args: &[i32]) -> Result<i32, String> {
    // Create a module with the function
    let mut module = Module::new();
    let func_name = func.name().to_string();
    let func_sig = func.signature.clone();
    module.add_function(func_name.clone(), func);

    // Create a bootstrap function that calls the toy function and returns the result
    let sig = Signature::new(func_sig.params.clone(), func_sig.returns.clone());
    let mut bootstrap_func = Function::new(sig, String::from("bootstrap"));
    let mut bootstrap_ctx = FunctionBuilderContext::new();
    let mut bootstrap_builder = FunctionBuilder::new(&mut bootstrap_func, &mut bootstrap_ctx);
    let entry_block = bootstrap_builder.create_block();
    bootstrap_builder.append_block_params_for_function_params(entry_block);
    bootstrap_builder.switch_to_block(entry_block);
    bootstrap_builder.seal_block(entry_block);

    // Get parameter values
    let param_values: Vec<Value> = bootstrap_builder.block_params(entry_block).to_vec();

    // Call the toy function using ins() API
    let return_count = func_sig.returns.len();
    bootstrap_builder.switch_to_block(entry_block);
    let _call_results =
        bootstrap_builder
            .ins()
            .call(func_name.clone(), &param_values, return_count);
    // Halt after call - the result will be in a0 (first return register)
    bootstrap_builder.ins().halt();

    bootstrap_builder.finalize();
    module.add_function(String::from("bootstrap"), bootstrap_func);
    module.set_entry_function(String::from("bootstrap"));

    // Compile the module
    let compiled =
        compile_module_to_insts(&module).map_err(|e| format!("Compilation error: {}", e))?;
    let bytes = compiled
        .to_bytes()
        .map_err(|e| format!("Failed to convert to bytes: {}", e))?;

    // Store disassembly for error reporting (before moving compiled)
    let disassembly = compiled.disassemble();

    // Create emulator
    let ram_size = 1024 * 1024; // 1MB RAM
    let mut emu = Riscv32Emulator::new(bytes, vec![0; ram_size]);

    // Initialize stack pointer
    let sp_value = 0x80000000u32.wrapping_add(ram_size as u32).wrapping_sub(16);
    emu.set_register(Gpr::Sp, sp_value as i32);

    // Set up function arguments in registers (a0, a1, a2, ...)
    for (i, &arg) in args.iter().enumerate() {
        if i < 8 {
            // RISC-V calling convention: a0-a7 for arguments
            let reg = match i {
                0 => Gpr::A0,
                1 => Gpr::A1,
                2 => Gpr::A2,
                3 => Gpr::A3,
                4 => Gpr::A4,
                5 => Gpr::A5,
                6 => Gpr::A6,
                7 => Gpr::A7,
                _ => unreachable!(),
            };
            emu.set_register(reg, arg);
        }
    }

    // Execute until halt or error
    loop {
        match emu.step() {
            Ok(StepResult::Continue) => continue,
            Ok(StepResult::Halted) => {
                // Function completed, return value is in a0
                return Ok(emu.get_register(Gpr::A0));
            }
            Ok(StepResult::Syscall(_)) => {
                // Syscall encountered - continue execution
                continue;
            }
            Err(e) => {
                // Include disassembly in error message for debugging
                return Err(format!(
                    "Execution error: {}\n\nDisassembled code:\n{}",
                    e, disassembly
                ));
            }
        }
    }
}
