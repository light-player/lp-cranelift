//! Test command for running CLIF files and verifying their results
//!
//! The `run` test command compiles each function on the host machine and executes it

use crate::function_runner::{CompiledTestFile, TestFileCompiler};
use crate::object_runner::{CompiledObjectTestFile, ObjectTestFileCompiler};
use crate::runone::FileUpdate;
use crate::subtest::{Context, SubTest};
use anyhow::Context as _;
use cranelift_codegen::data_value::DataValue;
use cranelift_codegen::ir::Type;
use cranelift_codegen::isa::{OwnedTargetIsa, TargetIsa};
use cranelift_codegen::settings::{Configurable, Flags};
use cranelift_codegen::{ir, settings};
use cranelift_reader::TestCommand;
use cranelift_reader::{TestFile, parse_run_command};
use log::{info, trace};
use std::borrow::Cow;
use target_lexicon::Architecture;

struct TestRun;

pub fn subtest(parsed: &TestCommand) -> anyhow::Result<Box<dyn SubTest>> {
    assert_eq!(parsed.command, "run");
    if !parsed.options.is_empty() {
        anyhow::bail!("No options allowed on {parsed}");
    }
    Ok(Box::new(TestRun))
}

/// Builds a [TargetIsa] for the current host.
///
/// ISA Flags can be overridden by passing [Value]'s via `isa_flags`.
fn build_host_isa(
    infer_native_flags: bool,
    flags: settings::Flags,
    isa_flags: Vec<settings::Value>,
) -> anyhow::Result<OwnedTargetIsa> {
    let mut builder = cranelift_native::builder_with_options(infer_native_flags)
        .map_err(|e| anyhow::Error::msg(e))?;

    // Copy ISA Flags
    for value in isa_flags {
        builder.set(value.name, &value.value_string())?;
    }

    let isa = builder.finish(flags)?;
    Ok(isa)
}

/// Checks if the host's ISA is compatible with the one requested by the test.
fn is_isa_compatible(
    file_path: &str,
    host: Option<&dyn TargetIsa>,
    requested: &dyn TargetIsa,
) -> Result<(), String> {
    let host_triple = match host {
        Some(host) => host.triple().clone(),
        None => target_lexicon::Triple::host(),
    };
    // If this test requests to run on a completely different
    // architecture than the host platform then we skip it entirely,
    // since we won't be able to natively execute machine code.
    let host_arch = host_triple.architecture;
    let requested_arch = requested.triple().architecture;

    match (host_arch, requested_arch) {
        // If the host matches the requested target, then that's all good.
        (host, requested) if host == requested => {}

        // Allow minor differences in risc-v targets.
        (Architecture::Riscv64(_), Architecture::Riscv64(_)) => {}

        // Any host can run pulley so long as the pointer width and endianness
        // match.
        (
            _,
            Architecture::Pulley32
            | Architecture::Pulley64
            | Architecture::Pulley32be
            | Architecture::Pulley64be,
        ) if host_triple.pointer_width() == requested.triple().pointer_width()
            && host_triple.endianness() == requested.triple().endianness() => {}

        // Any host can run riscv32 via emulator (similar to pulley).
        (_, Architecture::Riscv32 { .. }) => {}

        _ => {
            return Err(format!(
                "skipped {file_path}: host can't run {requested_arch:?} programs"
            ));
        }
    }

    // For riscv32, we use an emulator so we don't need to check host ISA flags
    if matches!(requested_arch, Architecture::Riscv32 { .. }) {
        return Ok(());
    }

    // We need to check that the requested ISA does not have any flags that
    // we can't natively support on the host.
    let requested_flags = requested.isa_flags();
    for req_value in requested_flags {
        // pointer_width for pulley already validated above
        if req_value.name == "pointer_width" {
            continue;
        }
        let requested = match req_value.as_bool() {
            Some(requested) => requested,
            None => unimplemented!("ISA flag {} of kind {:?}", req_value.name, req_value.kind()),
        };
        let host_isa_flags = match host {
            Some(host) => host.isa_flags(),
            None => {
                return Err(format!(
                    "host not available on this platform for isa-specific flag"
                ));
            }
        };
        let available_in_host = host_isa_flags
            .iter()
            .find(|val| val.name == req_value.name)
            .and_then(|val| val.as_bool())
            .unwrap_or(false);

        if !requested || available_in_host {
            continue;
        }

        // The AArch64 feature `sign_return_address` is supported on all AArch64
        // hosts, regardless of whether `cranelift-native` infers it or not. The
        // instructions emitted with this feature enabled are interpreted as
        // "hint" noop instructions on CPUs which don't support address
        // authentication.
        //
        // Note that at this time `cranelift-native` will only enable
        // `sign_return_address` for macOS (notably not Linux) because of a
        // historical bug in libunwind which causes pointer address signing,
        // when run on hardware that supports it, so segfault during unwinding.
        if req_value.name == "sign_return_address" && matches!(host_arch, Architecture::Aarch64(_))
        {
            continue;
        }

        return Err(format!(
            "skipped {}: host does not support ISA flag {}",
            file_path, req_value.name
        ));
    }

    Ok(())
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

enum CompiledTestResult {
    Jit(CompiledTestFile),
    Object(CompiledObjectTestFile),
}

fn compile_testfile(
    testfile: &TestFile,
    flags: &Flags,
    isa: &dyn TargetIsa,
) -> anyhow::Result<CompiledTestResult> {
    match isa.triple().architecture {
        Architecture::Riscv32 { .. } => {
            // For riscv32, compile to object format for emulator execution
            let isa = build_riscv32_isa(flags.clone(), isa.isa_flags())?;
            let mut compiler = ObjectTestFileCompiler::new(isa)?;
            compiler.add_testfile(testfile)?;
            Ok(CompiledTestResult::Object(compiler.compile()?))
        }

        Architecture::Pulley32
        | Architecture::Pulley64
        | Architecture::Pulley32be
        | Architecture::Pulley64be => {
            // Convert `&dyn TargetIsa` to `OwnedTargetIsa` by re-making the ISA and
            // applying pulley flags/etc.
            let mut builder = cranelift_codegen::isa::lookup(isa.triple().clone())?;
            for value in isa.isa_flags() {
                builder.set(value.name, &value.value_string()).unwrap();
            }
            let isa = builder.finish(flags.clone())?;

            let mut tfc = TestFileCompiler::new(isa);
            tfc.add_testfile(testfile)?;
            Ok(CompiledTestResult::Jit(tfc.compile()?))
        }

        // We can't use the requested ISA directly since it does not contain info
        // about the operating system / calling convention / etc..
        //
        // Copy the requested ISA flags into the host ISA and use that.
        _ => {
            let isa = build_host_isa(false, flags.clone(), isa.isa_flags()).unwrap();
            let mut tfc = TestFileCompiler::new(isa);
            tfc.add_testfile(testfile)?;
            Ok(CompiledTestResult::Jit(tfc.compile()?))
        }
    }
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
            // Use the original symbol name from ELF, not the renamed internal name
            let symbol_name = &defined_func.original_symbol_name;

            // Find function address using the loaded ELF
            let address = find_symbol_address(&obj, symbol_name, text_section_base)
                .map_err(|e| format!("Failed to find symbol {}: {}", symbol_name, e))?;

            // Store using the original function name for lookup
            let lookup_key = match func_name {
                ir::UserFuncName::User(ext_name) => ext_name.to_string(),
                ir::UserFuncName::Testcase(tc) => tc.to_string(),
            };
            function_addresses.insert(lookup_key.clone(), address);
            signatures.insert(lookup_key, defined_func.signature.clone());
        }

        // Create emulator
        let ram_size = 1024 * 1024; // 1MB RAM

        let mut emulator =
            lp_riscv_tools::emu::emulator::Riscv32Emulator::new(load_info.code, vec![0; ram_size])
                .with_max_instructions(10_000)
                .with_log_level(lp_riscv_tools::emu::LogLevel::Instructions);

        // Set up stack pointer (stack starts at top of RAM, grows downward)
        // Note: call_function will override this, but we set it here for consistency
        let stack_base = lp_riscv_tools::emu::emulator::DEFAULT_RAM_START + ram_size as u32;
        let stack_pointer = stack_base - 16; // 16-byte aligned, with some space
        emulator.set_register(Gpr::Sp, stack_pointer as i32);
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
        use std::collections::HashSet;

        // Don't show emulator state if no instructions were executed
        let instruction_count = self.emulator.get_instruction_count();
        if instruction_count == 0 {
            return None;
        }

        let mut output = String::new();

        // Add register state
        output.push_str("\n=== EMULATOR STATE ===\n");
        output.push_str(&format!("PC: 0x{:08x}\n", self.emulator.get_pc()));

        // Collect registers that were written to during execution
        let mut written_registers = HashSet::new();
        for log in self.emulator.get_logs() {
            use lp_riscv_tools::InstLog::*;
            match log {
                Arithmetic { rd, .. } => {
                    written_registers.insert(*rd);
                }
                Load { rd, .. } => {
                    written_registers.insert(*rd);
                }
                Store { .. } => {} // Store doesn't modify registers
                Branch { .. } => {} // Branch doesn't modify registers
                Jump { instruction, rd_new, .. } => {
                    // Jump instructions can write to rd (like jal)
                    if rd_new.is_some() {
                        // Extract rd register from the instruction word
                        let rd_num = ((instruction >> 7) & 0x1F) as u8;
                        written_registers.insert(Gpr::new(rd_num));
                    }
                }
                Immediate { rd, .. } => {
                    written_registers.insert(*rd);
                }
                System { .. } => {} // System doesn't modify registers
            }
        }

        output.push_str("Registers:\n");

        // Show registers in a compact format: only non-zero registers and registers written during execution
        let mut shown_registers = Vec::new();

        // Always show x0 (zero) first
        shown_registers.push((Gpr::Zero, "zero".to_string()));

        // Show named registers that are non-zero or were written to during execution
        let named_regs = [
            (Gpr::Ra, "ra"),
            (Gpr::Sp, "sp"),
            (Gpr::Gp, "gp"),
            (Gpr::Tp, "tp"),
            (Gpr::T0, "t0"),
            (Gpr::T1, "t1"),
            (Gpr::T2, "t2"),
            (Gpr::S0, "s0"),
            (Gpr::S1, "s1"),
            (Gpr::A0, "a0"),
            (Gpr::A1, "a1"),
            (Gpr::A2, "a2"),
            (Gpr::A3, "a3"),
            (Gpr::A4, "a4"),
            (Gpr::A5, "a5"),
            (Gpr::A6, "a6"),
            (Gpr::A7, "a7"),
        ];

        for (reg, name) in &named_regs {
            let value = self.emulator.get_register(*reg);
            if value != 0 || written_registers.contains(reg) {
                shown_registers.push((*reg, name.to_string()));
            }
        }

        // Show other registers (s2-s11, t3-t6) that are non-zero or were written to
        for i in 18..32 {
            let reg = Gpr::new(i);
            let value = self.emulator.get_register(reg);
            if value != 0 || written_registers.contains(&reg) {
                let name = if i >= 18 && i <= 27 {
                    format!("s{}", i - 16)
                } else {
                    format!("t{}", i - 25)
                };
                shown_registers.push((reg, name));
            }
        }

        // Format the registers
        for (reg, name) in shown_registers {
            let value = self.emulator.get_register(reg);
            output.push_str(&format!(
                "  {} (x{}) = 0x{:08x} ({})\n",
                name,
                reg.num(),
                value as u32,
                value
            ));
        }

        // Add instruction count
        output.push_str(&format!("\nInstructions executed: {}\n", instruction_count));

        // Limit log entries to the number of instructions actually executed
        let log_count = instruction_count.min(50) as usize; // Cap at 50 for readability
        let debug_info = self.emulator.format_debug_info(None, log_count);
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

        let initial_instruction_count = self.emulator.get_instruction_count();
        let result = self
            .emulator
            .call_function(*address, args, signature)
            .map_err(|e| {
                format!(
                    "Emulator execution failed: {} (PC: 0x{:08x}, instructions: {})",
                    e,
                    self.emulator.get_pc(),
                    self.emulator.get_instruction_count()
                )
            })?;

        let final_instruction_count = self.emulator.get_instruction_count();
        if final_instruction_count == initial_instruction_count {
            return Err(format!(
                "Emulator did not execute any instructions for function {} (instruction count: {})",
                symbol_name, final_instruction_count
            ));
        }

        Ok(result)
    }
}

/// Generate debugging information when a test fails
fn generate_debug_info(
    testfile: &CompiledObjectTestFile,
    func: &ir::Function,
    _error_msg: &str,
    executor: Option<&EmulatorExecutor>,
) -> String {
    let mut debug_output = String::new();

    // Add CLIF IR
    debug_output.push_str("\n=== CLIF IR ===\n");
    debug_output.push_str(&func.display().to_string());

    // Add V-Code and Disassembly for this function
    if let Some(defined_func) = testfile.defined_functions.get(&func.name) {
        if let Some(ref vcode) = defined_func.vcode {
            debug_output.push_str("\n=== V-CODE ===\n");
            debug_output.push_str(vcode);
        }

        if let Some(ref disasm) = defined_func.disassembly {
            debug_output.push_str("\n=== DISASSEMBLY ===\n");
            debug_output.push_str(disasm);
        }
    }

    // Add ELF info
    debug_output.push_str("\n=== ELF SYMBOLS ===\n");
    for (func_name, _) in &testfile.defined_functions {
        let symbol_name = match func_name {
            ir::UserFuncName::User(ext_name) => ext_name.to_string(),
            ir::UserFuncName::Testcase(tc) => tc.to_string(),
        };
        debug_output.push_str(&format!("  {}\n", symbol_name));
    }

    // Try to get emulator state from the provided executor, or create a new one
    let emulator_state = if let Some(executor) = executor {
        executor.format_emulator_state()
    } else if let Ok(new_executor) = EmulatorExecutor::new(testfile) {
        new_executor.format_emulator_state()
    } else {
        None
    };

    if let Some(state) = emulator_state {
        debug_output.push_str(&state);
    }

    debug_output
}

fn run_emulator_test(
    testfile: &CompiledObjectTestFile,
    func: &ir::Function,
    context: &Context,
) -> anyhow::Result<()> {
    for comment in context.details.comments.iter() {
        if let Some(command) = parse_run_command(comment.text, &func.signature)? {
            trace!("Parsed run command: {command}");

            // Create emulator executor outside the closure so we can use it for error reporting
            let mut executor = EmulatorExecutor::new(testfile)
                .map_err(|e| anyhow::anyhow!("Emulator setup failed: {}", e))?;

            let result = {
                let executor_ref = &mut executor;
                command.run(|_, run_args| {
                    let mut args = Vec::with_capacity(run_args.len());
                    args.extend_from_slice(run_args);

                    // Call the function using the executor
                    executor_ref
                        .call_function(&func.name, &args)
                        .map_err(|e| format!("Emulator execution failed: {}", e))
                })
            };

            if let Err(err_msg) = result {
                // Generate debugging information using the executor that ran the code
                let debug_info = generate_debug_info(testfile, func, &err_msg, Some(&executor));
                return Err(anyhow::anyhow!("{}\n\n{}", err_msg, debug_info));
            }
        }
    }
    Ok(())
}

fn run_test(
    compiled_result: &CompiledTestResult,
    func: &ir::Function,
    context: &Context,
) -> anyhow::Result<()> {
    match compiled_result {
        CompiledTestResult::Jit(testfile) => {
            // Native JIT execution
            for comment in context.details.comments.iter() {
                if let Some(command) = parse_run_command(comment.text, &func.signature)? {
                    trace!("Parsed run command: {command}");

                    command
                        .run(|_, run_args| {
                            let (_ctx_struct, _vmctx_ptr) =
                                build_vmctx_struct(context.isa.unwrap().pointer_type());

                            let mut args = Vec::with_capacity(run_args.len());
                            args.extend_from_slice(run_args);

                            let trampoline = testfile.get_trampoline(func).unwrap();
                            Ok(trampoline.call(&testfile, &args))
                        })
                        .map_err(|s| anyhow::anyhow!("{s}"))?;
                }
            }
        }
        CompiledTestResult::Object(testfile) => {
            // Emulator execution
            run_emulator_test(testfile, func, context)?;
        }
    }
    Ok(())
}

impl SubTest for TestRun {
    fn name(&self) -> &'static str {
        "run"
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
        // Disable runtests with pinned reg enabled.
        // We've had some abi issues that the trampoline isn't quite ready for.
        if flags.enable_pinned_reg() {
            return Err(anyhow::anyhow!(
                [
                    "Cannot run runtests with pinned_reg enabled.",
                    "See https://github.com/bytecodealliance/wasmtime/issues/4376 for more info"
                ]
                .join("\n")
            ));
        }

        // Check that the host machine can run this test case (i.e. has all extensions)
        let host_isa = build_host_isa(true, flags.clone(), vec![]).ok();
        if let Err(e) = is_isa_compatible(file_path, host_isa.as_deref(), isa.unwrap()) {
            log::info!("{e}");
            return Ok(());
        }

        let compiled_result = compile_testfile(&testfile, flags, isa.unwrap())?;

        for (func, details) in &testfile.functions {
            info!(
                "Test: {}({}) {}",
                self.name(),
                func.name,
                isa.map_or("-", TargetIsa::name)
            );

            let context = Context {
                preamble_comments: &testfile.preamble_comments,
                details,
                flags,
                isa,
                file_path: file_path.as_ref(),
                file_update,
            };

            run_test(&compiled_result, &func, &context).context(self.name())?;
        }

        Ok(())
    }

    fn run(&self, _func: Cow<ir::Function>, _context: &Context) -> anyhow::Result<()> {
        unreachable!()
    }
}

/// Build a VMContext struct with the layout described in docs/testing.md.
pub fn build_vmctx_struct(ptr_ty: Type) -> (Vec<u64>, DataValue) {
    let context_struct: Vec<u64> = Vec::new();

    let ptr = context_struct.as_ptr() as usize as i128;
    let ptr_dv =
        DataValue::from_integer(ptr, ptr_ty).expect("Failed to cast pointer to native target size");

    // Return all these to make sure we don't deallocate the heaps too early
    (context_struct, ptr_dv)
}
