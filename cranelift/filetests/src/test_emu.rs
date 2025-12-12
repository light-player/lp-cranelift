//! Test command for running CLIF files using the RISC-V emulator.
//!
//! The `emu` test command compiles each function to ELF object format and executes it
//! using the RISC-V 32-bit emulator.

use crate::object_runner::{CompiledObjectTestFile, ObjectTestFileCompiler};
use crate::runone::FileUpdate;
use crate::subtest::{Context, SubTest};
use anyhow::Context as _;
use cranelift_codegen::data_value::DataValue;
use cranelift_codegen::isa::{OwnedTargetIsa, TargetIsa};
use cranelift_codegen::settings::{Configurable, Flags};
use cranelift_codegen::{ir, settings};
use cranelift_reader::TestCommand;
use cranelift_reader::{TestFile, parse_run_command};
use log::{info, trace};
use std::borrow::Cow;
use target_lexicon::Architecture;

struct TestEmu;

pub fn subtest(parsed: &TestCommand) -> anyhow::Result<Box<dyn SubTest>> {
    assert_eq!(parsed.command, "emu");
    if !parsed.options.is_empty() {
        anyhow::bail!("No options allowed on {parsed}");
    }
    Ok(Box::new(TestEmu))
}

/// Builds a RISC-V 32-bit [TargetIsa].
///
/// ISA Flags can be overridden by passing [Value]'s via `isa_flags`.
fn build_riscv32_isa(
    flags: settings::Flags,
    isa_flags: Vec<settings::Value>,
) -> anyhow::Result<OwnedTargetIsa> {
    use cranelift_codegen::isa::lookup;
    use target_lexicon::{
        Architecture, BinaryFormat, Environment, OperatingSystem, Triple, Vendor,
    };

    let triple = Triple {
        architecture: Architecture::Riscv32(target_lexicon::Riscv32Architecture::Riscv32),
        vendor: Vendor::Unknown,
        operating_system: OperatingSystem::None_,
        environment: Environment::Unknown,
        binary_format: BinaryFormat::Elf,
    };

    let mut builder = lookup(triple)?;

    // Copy ISA Flags
    for value in isa_flags {
        builder.set(value.name, &value.value_string())?;
    }

    let isa = builder.finish(flags)?;
    Ok(isa)
}

/// Checks if the target ISA is compatible with emulator execution.
///
/// Only riscv32 is supported for emulator execution.
fn is_emulator_compatible(requested: &dyn TargetIsa) -> Result<(), String> {
    match requested.triple().architecture {
        Architecture::Riscv32 { .. } => Ok(()),
        arch => Err(format!("emulator only supports riscv32, got {arch:?}")),
    }
}

fn compile_testfile(
    testfile: &TestFile,
    flags: &Flags,
    isa: &dyn TargetIsa,
) -> anyhow::Result<CompiledObjectTestFile> {
    let isa = build_riscv32_isa(flags.clone(), isa.isa_flags())?;
    let mut compiler = ObjectTestFileCompiler::new(isa)?;
    compiler.add_testfile(testfile)?;
    Ok(compiler.compile()?)
}

fn run_test(
    testfile: &CompiledObjectTestFile,
    func: &ir::Function,
    context: &Context,
) -> anyhow::Result<()> {
    for comment in context.details.comments.iter() {
        if let Some(command) = parse_run_command(comment.text, &func.signature)? {
            trace!("Parsed run command: {command}");

            let result = command.run(|_, run_args| {
                let mut args = Vec::with_capacity(run_args.len());
                args.extend_from_slice(run_args);

                // Create emulator executor and call the function
                let mut executor = EmulatorExecutor::new(testfile)
                    .map_err(|e| format!("Emulator setup failed: {}", e))?;
                let result = executor
                    .call_function(&func.name, &args)
                    .map_err(|e| format!("Emulator execution failed: {}", e))?;
                Ok(result)
            });

            if let Err(err_msg) = result {
                // Generate debugging information
                let debug_info = generate_debug_info(testfile, func, &err_msg);
                return Err(anyhow::anyhow!("{}\n\n{}", err_msg, debug_info));
            }
        }
    }
    Ok(())
}

/// Generate debugging information when a test fails
fn generate_debug_info(
    testfile: &CompiledObjectTestFile,
    func: &ir::Function,
    _error_msg: &str,
) -> String {
    let mut debug_output = String::new();

    // Add CLIF IR
    debug_output.push_str("\n=== CLIF IR ===\n");
    debug_output.push_str(&func.display().to_string());

    // Add ELF info
    debug_output.push_str("\n=== ELF SYMBOLS ===\n");
    for (func_name, _) in &testfile.defined_functions {
        let symbol_name = match func_name {
            ir::UserFuncName::User(ext_name) => ext_name.to_string(),
            ir::UserFuncName::Testcase(tc) => tc.to_string(),
        };
        debug_output.push_str(&format!("  {}\n", symbol_name));
    }

    // Try to create emulator and get state (this will have the failed execution state)
    if let Ok(executor) = EmulatorExecutor::new(testfile) {
        if let Some(emulator_state) = executor.format_emulator_state() {
            debug_output.push_str(&emulator_state);
        }
    }

    debug_output
}

/// Executor that manages the RISC-V emulator and ELF loading.
struct EmulatorExecutor {
    emulator: lp_riscv_tools::emu::emulator::Riscv32Emulator,
    function_addresses: std::collections::HashMap<String, u32>,
    signatures: std::collections::HashMap<String, ir::Signature>,
    #[allow(dead_code)] // Reserved for future debugging output
    vcode: Option<String>,
    #[allow(dead_code)] // Reserved for future debugging output
    disassembly: Option<String>,
}

impl EmulatorExecutor {
    fn new(compiled_testfile: &CompiledObjectTestFile) -> Result<Self, String> {
        use lp_riscv_tools::Gpr;
        use lp_riscv_tools::elf_loader::{find_symbol_address, load_elf};
        use object::{Object, ObjectSection};

        // Load ELF and apply relocations
        let load_info = load_elf(&compiled_testfile.elf_bytes)
            .map_err(|e| format!("ELF load failed: {}", e))?;

        // Parse ELF to find function addresses
        let obj = object::File::parse(&compiled_testfile.elf_bytes[..])
            .map_err(|e| format!("Failed to parse ELF: {:?}", e))?;

        // Find text section base for symbol address calculation
        let mut text_section_base = 0u64;
        for section in obj.sections() {
            if section.kind() == object::SectionKind::Text {
                text_section_base = section.address();
                break;
            }
        }

        // Build function address and signature maps
        let mut function_addresses = std::collections::HashMap::new();
        let mut signatures = std::collections::HashMap::new();

        for (func_name, defined_func) in &compiled_testfile.defined_functions {
            let symbol_name = match func_name {
                ir::UserFuncName::User(ext_name) => ext_name.to_string(),
                ir::UserFuncName::Testcase(tc) => tc.to_string(),
            };

            // Find function address using the loaded ELF
            let address = find_symbol_address(&obj, &symbol_name, text_section_base)
                .map_err(|e| format!("Failed to find symbol {}: {}", symbol_name, e))?;

            function_addresses.insert(symbol_name.clone(), address);
            signatures.insert(symbol_name, defined_func.signature.clone());
        }

        // Create emulator
        let ram_size = 1024 * 1024; // 1MB RAM
        let mut emulator =
            lp_riscv_tools::emu::emulator::Riscv32Emulator::new(load_info.code, vec![0; ram_size])
                .with_max_instructions(10_000)
                .with_log_level(lp_riscv_tools::emu::LogLevel::Instructions);

        // Set up stack pointer (stack starts at top of RAM, grows downward)
        let stack_base = ram_size as u32;
        emulator.set_register(Gpr::Sp, stack_base as i32);
        emulator.set_pc(0);

        Ok(Self {
            emulator,
            function_addresses,
            signatures,
            vcode: None,       // TODO: Generate VCode during compilation for debugging
            disassembly: None, // TODO: Generate disassembly during compilation for debugging
        })
    }

    /// Generate formatted emulator state for debugging
    fn format_emulator_state(&self) -> Option<String> {
        use lp_riscv_tools::Gpr;

        let mut output = String::new();

        // Add register state
        output.push_str("\n=== EMULATOR STATE ===\n");
        output.push_str(&format!("PC: 0x{:08x}\n", self.emulator.get_pc()));

        output.push_str("Registers:\n");
        for i in 0..32 {
            let reg_name = match i {
                0 => "zero",
                1 => "ra",
                2 => "sp",
                3 => "gp",
                4 => "tp",
                5..=7 => &format!("t{}", i - 5),
                8..=9 => &format!("s{}", i - 8),
                10..=17 => &format!("a{}", i - 10),
                18..=27 => &format!("s{}", i - 16),
                28..=31 => &format!("t{}", i - 25),
                _ => unreachable!(),
            };
            let value = self.emulator.get_register(Gpr::new(i as u8));
            output.push_str(&format!(
                "  {:>3} ({:>4}): 0x{:08x} ({})\n",
                format!("x{}", i),
                reg_name,
                value,
                value
            ));
        }

        // Add instruction count
        output.push_str(&format!(
            "\nInstructions executed: {}\n",
            self.emulator.get_instruction_count()
        ));

        // Use the built-in debug info formatter
        let debug_info = self.emulator.format_debug_info(None, 20);
        if !debug_info.is_empty() {
            output.push_str(&format!("\n=== EXECUTION LOG ===\n{}", debug_info));
        }

        Some(output)
    }

    fn call_function(
        &mut self,
        func_name: &ir::UserFuncName,
        args: &[DataValue],
    ) -> Result<Vec<DataValue>, String> {
        let symbol_name = match func_name {
            ir::UserFuncName::User(ext_name) => ext_name.to_string(),
            ir::UserFuncName::Testcase(tc) => tc.to_string(),
        };

        let address = self
            .function_addresses
            .get(&symbol_name)
            .ok_or_else(|| format!("Function {} not found", symbol_name))?;

        let signature = self
            .signatures
            .get(&symbol_name)
            .ok_or_else(|| format!("Signature for function {} not found", symbol_name))?;

        self.emulator
            .call_function(*address, args, signature)
            .map_err(|e| format!("Emulator execution failed: {}", e))
    }
}

impl SubTest for TestEmu {
    fn name(&self) -> &'static str {
        "emu"
    }

    fn is_mutating(&self) -> bool {
        false
    }

    fn needs_isa(&self) -> bool {
        true
    }

    /// Runs the entire subtest for a given target, invokes [Self::run] for running
    /// individual tests.
    fn run_target<'a>(
        &self,
        testfile: &TestFile,
        file_update: &mut FileUpdate,
        file_path: &'a str,
        flags: &'a Flags,
        isa: Option<&'a dyn TargetIsa>,
    ) -> anyhow::Result<()> {
        let isa = isa.unwrap();

        // Check that this target is compatible with emulator execution
        if let Err(_e) = is_emulator_compatible(isa) {
            return Ok(());
        }

        let compiled_testfile = compile_testfile(&testfile, flags, isa)?;

        for (func, details) in &testfile.functions {
            info!("Test: {}({}) {}", self.name(), func.name, isa.name());

            let context = Context {
                preamble_comments: &testfile.preamble_comments,
                details,
                flags,
                isa: Some(isa),
                file_path: file_path.as_ref(),
                file_update,
            };

            run_test(&compiled_testfile, &func, &context).context(self.name())?;
        }

        Ok(())
    }

    fn run(&self, _func: Cow<ir::Function>, _context: &Context) -> anyhow::Result<()> {
        unreachable!()
    }
}
