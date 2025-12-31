//! Shared emulator context for filetests
//!
//! This module provides a `SharedEmulatorContext` that manages shared emulator state
//! across multiple tests, avoiding repeated loading of the builtins executable and
//! bootstrap initialization.

#[cfg(all(feature = "std", feature = "emulator"))]
extern crate std;

#[cfg(all(feature = "std", feature = "emulator"))]
use crate::error::{ErrorCode, GlslError};
#[cfg(all(feature = "std", feature = "emulator"))]
use alloc::{string::String, vec::Vec};
#[cfg(all(feature = "std", feature = "emulator"))]
use hashbrown::HashMap;

/// Shared emulator context that manages shared state across tests.
///
/// This context:
/// - Loads the builtins executable once
/// - Maintains shared code/ram buffers that accumulate object files
/// - Runs bootstrap init once
/// - Provides methods to link object files and create emulator instances
#[cfg(all(feature = "std", feature = "emulator"))]
pub struct SharedEmulatorContext {
    /// Code/ROM region (starts at address 0) with all linked code
    code: Vec<u8>,
    /// RAM region (starts at 0x80000000) with all linked data
    ram: Vec<u8>,
    /// Symbol map (symbol name -> address) with all symbols from builtins and linked objects
    symbol_map: HashMap<String, u32>,
    /// Entry point address from the base executable
    #[allow(dead_code)] // Reserved for future use
    entry_point: u32,
    /// Whether bootstrap init has been completed
    bootstrap_done: bool,
    /// Initial state (after loading builtins, before any object files)
    /// Used to reset buffers after test files to prevent unbounded growth
    initial_code: Vec<u8>,
    initial_ram: Vec<u8>,
    initial_symbol_map: HashMap<String, u32>,
}

#[cfg(all(feature = "std", feature = "emulator"))]
impl SharedEmulatorContext {
    /// Create a new shared emulator context by loading the builtins executable.
    ///
    /// This loads the builtins executable and runs bootstrap init once.
    /// The context can then be used to link multiple object files and create
    /// emulator instances.
    ///
    /// # Arguments
    ///
    /// * `builtins_exe_bytes` - The lp-builtins-app executable bytes
    ///
    /// # Returns
    ///
    /// A new `SharedEmulatorContext` with builtins loaded and bootstrap init completed.
    pub fn new(builtins_exe_bytes: &[u8]) -> Result<Self, GlslError> {
        use crate::backend::builtins::registry::BuiltinId;
        use lp_riscv_tools::Gpr;
        use lp_riscv_tools::StepResult;
        use lp_riscv_tools::emu::LogLevel;
        use lp_riscv_tools::emu::emulator::Riscv32Emulator;

        crate::debug!("=== Creating SharedEmulatorContext ===");

        if builtins_exe_bytes.is_empty() {
            return Err(GlslError::new(
                ErrorCode::E0400,
                "lp-builtins-app executable is empty or not available. \
                 Build it with: scripts/build-builtins.sh",
            ));
        }

        // Load the base executable
        crate::debug!("Loading base executable...");
        let load_info = lp_riscv_tools::load_elf(builtins_exe_bytes).map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!(
                    "Failed to load base executable: {}. \
                     Ensure lp-builtins-app is correctly compiled.",
                    e
                ),
            )
        })?;

        crate::debug!("Base executable loaded successfully!");

        // Verify that builtin symbols are present and defined
        let mut missing_symbols = Vec::new();
        let mut undefined_symbols = Vec::new();

        crate::debug!("Checking for builtin symbols in symbol map...");
        crate::debug!("Symbol map contains {} symbols", load_info.symbol_map.len());

        for builtin in BuiltinId::all() {
            let symbol_name = builtin.name();
            crate::debug!("Checking for builtin symbol: {}", symbol_name);

            if let Some(&address) = load_info.symbol_map.get(symbol_name) {
                if address == 0 {
                    crate::debug!(
                        "  -> Symbol {} found but address is 0 (undefined)",
                        symbol_name
                    );
                    undefined_symbols.push(symbol_name);
                } else {
                    crate::debug!(
                        "  -> Symbol {} found at address 0x{:x} (defined)",
                        symbol_name,
                        address
                    );
                }
            } else {
                crate::debug!("  -> Symbol {} not found in symbol map", symbol_name);
                missing_symbols.push(symbol_name);
            }
        }

        if !undefined_symbols.is_empty() {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "Builtin symbols are undefined: {:?}. \
                     Ensure lp-builtins library is built and linked correctly.",
                    undefined_symbols
                ),
            ));
        }

        if !missing_symbols.is_empty() {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "Builtin symbols not found: {:?}. \
                     Ensure lp-builtins library is built and contains these symbols.",
                    missing_symbols
                ),
            ));
        }

        // Run bootstrap init once
        crate::debug!("Running bootstrap init...");
        let ram_size = load_info.ram.len();
        let mut emulator = Riscv32Emulator::new(load_info.code.clone(), load_info.ram.clone())
            .with_max_instructions(10_000)
            .with_log_level(LogLevel::Instructions);

        // Set up stack pointer (sp = x2) to point to high RAM
        let sp_value = 0x80000000u32.wrapping_add((ram_size as u32).wrapping_sub(16));
        emulator.set_register(Gpr::Sp, sp_value as i32);

        // Set return address (ra = x1) to halt address so bootstrap code can return
        let halt_address = 0x80000000u32.wrapping_add(ram_size as u32);
        emulator.set_register(Gpr::Ra, halt_address as i32);

        // Set PC to entry point to start bootstrap init
        emulator.set_pc(load_info.entry_point);

        // Execute bootstrap init code until it completes
        let mut init_steps = 0;
        let max_init_steps = 10000;
        let init_address = load_info.symbol_map.get("_init").copied();

        crate::debug!(
            "Running bootstrap init (entry_point=0x{:x}, _init={:?})",
            load_info.entry_point,
            init_address
        );

        loop {
            if init_steps >= max_init_steps {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!(
                        "Bootstrap init exceeded {} steps - possible infinite loop (PC: 0x{:x})",
                        max_init_steps,
                        emulator.get_pc()
                    ),
                ));
            }

            match emulator.step() {
                Ok(step_result) => {
                    init_steps += 1;
                    let pc_after = emulator.get_pc();

                    // Handle panic result - bootstrap init failures are fatal
                    if let StepResult::Panic(panic_info) = step_result {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            format!(
                                "Bootstrap init panic: {} (PC: 0x{:x})",
                                panic_info.message, panic_info.pc
                            ),
                        ));
                    }

                    // Handle halt result - bootstrap init completed
                    if let StepResult::Halted = step_result {
                        crate::debug!(
                            "Bootstrap init completed (halted) after {} steps",
                            init_steps
                        );
                        break;
                    }

                    // Check if PC is at halt address (bootstrap code returned)
                    if pc_after == halt_address {
                        crate::debug!(
                            "Bootstrap init completed (returned) after {} steps",
                            init_steps
                        );
                        break;
                    }
                }
                Err(e) => {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "Bootstrap init error after {} steps (PC: 0x{:x}): {}",
                            init_steps,
                            emulator.get_pc(),
                            e
                        ),
                    ));
                }
            }
        }

        crate::debug!("Bootstrap init completed successfully");

        // Store initial state for reset capability
        let initial_code = load_info.code.clone();
        let initial_ram = load_info.ram.clone();
        let initial_symbol_map = load_info.symbol_map.clone();

        Ok(Self {
            code: load_info.code,
            ram: load_info.ram,
            symbol_map: load_info.symbol_map,
            entry_point: load_info.entry_point,
            bootstrap_done: true,
            initial_code,
            initial_ram,
            initial_symbol_map,
        })
    }

    /// Link an object file into the shared context.
    ///
    /// This extends the code/ram buffers with the object file's sections and
    /// merges the symbol map. The object file is appended after existing code/data.
    ///
    /// # Arguments
    ///
    /// * `elf_bytes` - The ELF object file bytes to link
    ///
    /// # Returns
    ///
    /// Information about the loaded object file, or an error if loading fails.
    pub fn link_object_file(
        &mut self,
        elf_bytes: &[u8],
    ) -> Result<lp_riscv_tools::elf_loader::ObjectLoadInfo, GlslError> {
        crate::debug!("=== Linking object file into shared context ===");
        crate::debug!("Object file size: {} bytes", elf_bytes.len());

        // Load the object file into the shared buffers
        lp_riscv_tools::elf_loader::load_object_file(
            elf_bytes,
            &mut self.code,
            &mut self.ram,
            &mut self.symbol_map,
        )
        .map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!(
                    "Failed to load object file: {}. \
                     Ensure the object file is correctly compiled.",
                    e
                ),
            )
        })
    }

    /// Create a new emulator instance from the shared context.
    ///
    /// This creates a fresh emulator instance with cloned code/ram buffers.
    /// The stack pointer is set to a safe location. Bootstrap init is skipped
    /// since it was already run when creating the context.
    ///
    /// # Arguments
    ///
    /// * `options` - Emulator options (max_memory, stack_size, max_instructions)
    /// * `traps` - Trap information from compiled code (offset -> TrapCode pairs)
    ///
    /// # Returns
    ///
    /// A new `Riscv32Emulator` instance ready for use.
    pub fn create_emulator(
        &self,
        options: &crate::backend::codegen::emu::EmulatorOptions,
        traps: &[(u32, cranelift_codegen::ir::TrapCode)],
    ) -> lp_riscv_tools::emu::emulator::Riscv32Emulator {
        use lp_riscv_tools::Gpr;
        use lp_riscv_tools::emu::LogLevel;
        use lp_riscv_tools::emu::emulator::Riscv32Emulator;

        // Create emulator with cloned buffers and trap information
        let mut emulator = Riscv32Emulator::with_traps(self.code.clone(), self.ram.clone(), traps)
            .with_max_instructions(options.max_instructions)
            .with_log_level(LogLevel::Instructions);

        // Set up stack pointer to safe location (high RAM, aligned)
        let ram_size = self.ram.len();
        let sp_value = 0x80000000u32.wrapping_add((ram_size as u32).wrapping_sub(16));
        emulator.set_register(Gpr::Sp, sp_value as i32);

        // PC will be set by function calls, so we don't set it here
        // Bootstrap init already done, so we skip it

        emulator
    }

    /// Get a reference to the symbol map.
    pub fn get_symbol_map(&self) -> &HashMap<String, u32> {
        &self.symbol_map
    }

    /// Check if bootstrap init has been completed.
    pub fn is_bootstrap_done(&self) -> bool {
        self.bootstrap_done
    }

    /// Get the code buffer size.
    pub fn code_size(&self) -> usize {
        self.code.len()
    }

    /// Get the RAM buffer size.
    pub fn ram_size(&self) -> usize {
        self.ram.len()
    }

    /// Get a reference to the code buffer (for debugging).
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// Get a reference to the RAM buffer (for debugging).
    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    /// Reset buffers to initial state (after loading builtins).
    /// This prevents unbounded growth when accumulating object files across many tests.
    /// Call this after completing a test file to free memory.
    pub fn reset_to_initial_state(&mut self) {
        self.code = self.initial_code.clone();
        self.ram = self.initial_ram.clone();
        self.symbol_map = self.initial_symbol_map.clone();
        // bootstrap_done stays true - we don't need to re-run bootstrap init
    }
}

/// Get the builtins executable bytes.
/// This is a convenience function for creating SharedEmulatorContext.
#[cfg(all(feature = "std", feature = "emulator"))]
pub fn get_builtins_executable_bytes() -> &'static [u8] {
    use crate::backend::codegen::emu::builtins_lib;
    builtins_lib::LP_BUILTINS_EXE_BYTES
}

