//! Immutable CLIF module representation holding all functions before linking/compilation.

use crate::frontend::src_loc_manager::SourceLocManager;
use crate::frontend::src_loc::GlSourceMap;
use crate::backend::link::rebuild_function_for_module;
use crate::error::{ErrorCode, GlslError};
use crate::frontend::semantic::functions::FunctionRegistry;
use cranelift_codegen::CodegenError;
use cranelift_codegen::ir::{Function, SourceLoc, TrapCode};
use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_codegen::print_errors::pretty_verifier_error;
use cranelift_codegen::write_function;
use cranelift_module::{FuncId, Linkage, Module, ModuleError};
use hashbrown::HashMap;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::format as alloc_format;
#[cfg(feature = "std")]
use std::format as alloc_format;

/// Information about a trap site in compiled code.
#[derive(Clone, Debug)]
pub struct TrapInfo {
    /// Offset of the trap within the function (relative to function start).
    pub offset: u32,
    /// Type of trap that occurs at this location.
    pub code: TrapCode,
    /// Source location where this trap originates.
    pub srcloc: SourceLoc,
}

/// Immutable module representation holding CLIF IR functions before linking/compilation.
///
/// This structure holds all functions (user functions and main) in CLIF IR form,
/// along with metadata needed for later compilation or transformation.
#[derive(Clone)]
pub struct ClifModule {
    user_functions: HashMap<String, Function>,
    main_function: Function,
    function_registry: FunctionRegistry,
    source_text: String,
    isa: OwnedTargetIsa,
    // Store GLSL signatures for proper API extraction (return types, parameters)
    glsl_signatures: HashMap<String, crate::frontend::semantic::functions::FunctionSignature>,
    // Map from old FuncId (from compilation) to function name
    // This is needed when linking to remap FuncRefs to new module's FuncIds
    func_id_to_name: HashMap<u32, String>,
    // Source location manager for mapping SourceLoc to GLSL source positions
    source_loc_manager: SourceLocManager,
    // Source map for managing file locations
    source_map: GlSourceMap,
}

impl ClifModule {
    /// Create a new builder for constructing a ClifModule
    pub fn builder() -> ClifModuleBuilder {
        ClifModuleBuilder::new()
    }

    /// Get all user-defined functions
    pub fn user_functions(&self) -> &HashMap<String, Function> {
        &self.user_functions
    }

    /// Get the main function
    pub fn main_function(&self) -> &Function {
        &self.main_function
    }

    /// Get the function registry
    pub fn function_registry(&self) -> &FunctionRegistry {
        &self.function_registry
    }

    /// Get the source text
    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    /// Get the ISA as a reference
    pub fn isa(&self) -> &dyn cranelift_codegen::isa::TargetIsa {
        self.isa.as_ref()
    }

    /// Take ownership of the ISA (consumes self)
    pub fn into_isa(self) -> OwnedTargetIsa {
        self.isa
    }

    /// Get the target configuration (convenience method)
    pub fn target_config(&self) -> cranelift_codegen::isa::TargetFrontendConfig {
        self.isa.frontend_config()
    }

    /// Get GLSL signature for a function by name
    pub fn glsl_signature(
        &self,
        name: &str,
    ) -> Option<&crate::frontend::semantic::functions::FunctionSignature> {
        self.glsl_signatures.get(name)
    }

    /// Get function name for an old FuncId (from compilation)
    pub fn func_id_to_name(&self, old_func_id: u32) -> Option<&String> {
        self.func_id_to_name.get(&old_func_id)
    }

    /// Get the entire func_id_to_name mapping (for preserving during transformations)
    pub fn func_id_to_name_map(&self) -> &HashMap<u32, String> {
        &self.func_id_to_name
    }

    /// Get the source location manager
    pub fn source_loc_manager(&self) -> &SourceLocManager {
        &self.source_loc_manager
    }

    /// Get the source map
    pub fn source_map(&self) -> &GlSourceMap {
        &self.source_map
    }

    /// Link all functions from this module into a Cranelift Module (JITModule, ObjectModule, etc.)
    /// Returns a mapping of function names to their FuncIds in the target module,
    /// along with the formatted CLIF IR string for all functions
    pub fn link_into<M: Module>(
        &self,
        module: &mut M,
        main_linkage: Linkage,
    ) -> Result<(HashMap<String, FuncId>, String, Vec<(String, Vec<TrapInfo>)>), GlslError> {
        use crate::error::{ErrorCode, GlslError};

        let mut name_to_id = HashMap::new();
        let mut clif_functions = Vec::new(); // Store (name, clif_text) pairs in declaration order
        let mut traps = Vec::new(); // Store (function_name, trap_info) pairs

        // Declare all functions first to get their FuncIds
        for (name, func) in &self.user_functions {
            let func_id = module
                .declare_function(name, Linkage::Local, &func.signature)
                .map_err(|e| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!("failed to declare function '{}': {}", name, e),
                    )
                })?;
            name_to_id.insert(name.clone(), func_id);
        }
        let main_id = module
            .declare_function("main", main_linkage, &self.main_function.signature)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to declare main function: {}", e),
                )
            })?;
        name_to_id.insert(String::from("main"), main_id);

        // Helper function to remap FuncRefs in a function
        fn remap_func_refs<M: Module>(
            func: &mut cranelift_codegen::ir::Function,
            module: &mut M,
            func_id_to_name: &HashMap<u32, String>,
            name_to_id: &HashMap<String, FuncId>,
        ) -> Result<(), GlslError> {
            use cranelift_codegen::ir::{ExternalName, FuncRef, InstructionData};

            // Collect all old FuncRefs and extract FuncIds
            // If user_named_funcs is empty, we'll match by signature
            // Collect all FuncRefs and their IDs first to avoid borrow conflicts
            let user_named_funcs = func.params.user_named_funcs();
            // Collect ExternalName conversions upfront to avoid borrow conflicts
            let mut ext_name_conversions: Vec<(FuncRef, String)> = Vec::new();
            let mut old_func_ref_to_old_func_id: Vec<(FuncRef, u32)> = Vec::new();
            let mut func_ids_to_add: Vec<u32> = Vec::new();
            for (old_func_ref, old_ext_func) in func.dfg.ext_funcs.iter() {
                if let ExternalName::User(user_name_ref) = old_ext_func.name {
                    // Try to look up the UserExternalName from user_named_funcs first
                    if let Some(user_name) = user_named_funcs.get(user_name_ref) {
                        old_func_ref_to_old_func_id.push((old_func_ref, user_name.index));
                        // Also collect for ExternalName conversion
                        if let Some(func_name) = func_id_to_name.get(&user_name.index) {
                            ext_name_conversions.push((old_func_ref, func_name.clone()));
                        }
                    } else {
                        // user_named_funcs is empty - match by signature
                        let old_sig = &func.dfg.signatures[old_ext_func.signature];
                        let mut found = false;
                        for (func_id_val, func_name) in func_id_to_name.iter() {
                            if let Some(new_func_id) = name_to_id.get(func_name) {
                                let decl = module.declarations().get_function_decl(*new_func_id);
                                // Compare signatures - they should match exactly
                                if decl.signature.params.len() == old_sig.params.len()
                                    && decl.signature.returns.len() == old_sig.returns.len()
                                {
                                    let params_match = decl
                                        .signature
                                        .params
                                        .iter()
                                        .zip(old_sig.params.iter())
                                        .all(|(new_param, old_param)| {
                                            new_param.value_type == old_param.value_type
                                                && new_param.purpose == old_param.purpose
                                        });
                                    let returns_match = decl
                                        .signature
                                        .returns
                                        .iter()
                                        .zip(old_sig.returns.iter())
                                        .all(|(new_ret, old_ret)| {
                                            new_ret.value_type == old_ret.value_type
                                                && new_ret.purpose == old_ret.purpose
                                        });

                                    if params_match && returns_match {
                                        // Match found - store func_id to add later
                                        func_ids_to_add.push(*func_id_val);
                                        old_func_ref_to_old_func_id
                                            .push((old_func_ref, *func_id_val));
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if !found {
                            // Provide more detailed error message with available signatures
                            let available_sigs: Vec<String> = func_id_to_name
                                .iter()
                                .filter_map(|(func_id_val, func_name)| {
                                    name_to_id.get(func_name).map(|new_func_id| {
                                        let decl =
                                            module.declarations().get_function_decl(*new_func_id);
                                        format!(
                                            "  {} (old FuncId {}, new FuncId {}): {:?}",
                                            func_name,
                                            func_id_val,
                                            new_func_id.as_u32(),
                                            decl.signature
                                        )
                                    })
                                })
                                .collect();

                            let func_id_to_name_debug: Vec<String> = func_id_to_name
                                .iter()
                                .map(|(func_id_val, func_name)| {
                                    format!("  FuncId {} -> '{}'", func_id_val, func_name)
                                })
                                .collect();

                            let name_to_id_debug: Vec<String> = name_to_id
                                .iter()
                                .map(|(name, func_id)| {
                                    format!("  '{}' -> FuncId {}", name, func_id.as_u32())
                                })
                                .collect();

                            let available_sigs_str = if available_sigs.is_empty() {
                                String::from("  (none)")
                            } else {
                                available_sigs.join("\n")
                            };
                            return Err(GlslError::new(
                                ErrorCode::E0400,
                                format!(
                                    "Could not match FuncRef to FuncId - signature matching failed.\n\
                                    Looking for signature: {:?}\n\
                                    func_id_to_name mappings ({} entries):\n{}\n\
                                    name_to_id mappings ({} entries):\n{}\n\
                                    Available signatures:\n{}",
                                    old_sig,
                                    func_id_to_name.len(),
                                    func_id_to_name_debug.join("\n"),
                                    name_to_id.len(),
                                    name_to_id_debug.join("\n"),
                                    available_sigs_str
                                ),
                            ));
                        }
                    }
                }
            }

            // Now add user_named_funcs entries (after we're done reading from user_named_funcs)
            for func_id_val in &func_ids_to_add {
                use cranelift_codegen::ir::UserExternalName;
                let user_name = UserExternalName::new(0, *func_id_val);
                let _ = func.params.ensure_user_func_name(user_name);
            }

            // Build mapping from old FuncRef to new FuncRef
            let mut func_ref_map: HashMap<FuncRef, FuncRef> = HashMap::new();
            let mut new_func_refs_to_convert: Vec<(FuncRef, String)> = Vec::new();
            for (old_func_ref, old_func_id) in &old_func_ref_to_old_func_id {
                let callee_name = func_id_to_name.get(old_func_id).ok_or_else(|| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "Could not find function name for old FuncId {}",
                            old_func_id
                        ),
                    )
                })?;
                let new_callee_func_id = name_to_id.get(callee_name).ok_or_else(|| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!("Could not find new FuncId for function '{}'", callee_name),
                    )
                })?;
                let new_func_ref = module.declare_func_in_func(*new_callee_func_id, func);
                func_ref_map.insert(*old_func_ref, new_func_ref);
                // Track new FuncRefs for ExternalName conversion
                new_func_refs_to_convert.push((new_func_ref, callee_name.clone()));
            }

            // Now replace FuncRefs in all call instructions
            // Collect blocks and instructions first to avoid borrow conflicts
            let blocks: Vec<_> = func.layout.blocks().collect();
            let mut insts_to_update: Vec<(cranelift_codegen::ir::Inst, FuncRef)> = Vec::new();
            for block in &blocks {
                for inst in func.layout.block_insts(*block) {
                    let inst_data = &func.dfg.insts[inst];
                    if let InstructionData::Call { func_ref, .. } = inst_data {
                        if let Some(&new_func_ref) = func_ref_map.get(func_ref) {
                            insts_to_update.push((inst, new_func_ref));
                        }
                    }
                }
            }

            // Now update the instructions
            for (inst, new_func_ref) in insts_to_update {
                let inst_data = &mut func.dfg.insts[inst];
                if let InstructionData::Call { opcode, args, .. } = inst_data.clone() {
                    *inst_data = InstructionData::Call {
                        opcode,
                        func_ref: new_func_ref,
                        args,
                    };
                }
            }

            // Note: ObjectModule doesn't support ExternalName::TestCase, so we keep ExternalName::User
            // The symbol resolution works via user_named_funcs entries which are already set up correctly

            Ok(())
        }

        /// Find the source location for a trap at the given offset.
        ///
        /// Uses an improved lookup algorithm that:
        /// 1. First tries to find a source location range containing the trap offset
        /// 2. If not found, searches backwards to find the nearest source location before the trap
        /// 3. If still not found, searches forwards to find the nearest source location after the trap
        /// 4. Only falls back to entry block if no source locations exist in the function
        fn find_trap_source_location(
            trap_offset: u32,
            srclocs: &[cranelift_codegen::MachSrcLoc<cranelift_codegen::Final>],
            func: &Function,
        ) -> SourceLoc {
            // First, try to find a source location range containing the trap offset
            if let Some(loc) = srclocs
                .iter()
                .find(|loc| loc.start <= trap_offset && trap_offset < loc.end)
            {
                return loc.loc;
            }

            // If no exact match, search backwards to find the nearest source location before the trap
            if let Some(loc) = srclocs
                .iter()
                .rev()
                .find(|loc| loc.end <= trap_offset)
            {
                return loc.loc;
            }

            // If still not found, search forwards to find the nearest source location after the trap
            if let Some(loc) = srclocs
                .iter()
                .find(|loc| loc.start > trap_offset)
            {
                return loc.loc;
            }

            // Only fall back to entry block if no source locations exist in the function
            // This is better than the previous approach which always fell back to entry block
            if srclocs.is_empty() {
                if let Some(entry_block) = func.layout.entry_block() {
                    if let Some(first_inst) = func.layout.block_insts(entry_block).next() {
                        return func.srcloc(first_inst);
                    }
                }
            }

            // Last resort: default source location
            SourceLoc::default()
        }

        // Define all user functions, remapping FuncRefs
        for (name, old_func) in &self.user_functions {
            let new_func_id = name_to_id[name];
            
            // Rebuild function with proper block params and function name preservation
            let rebuilt_func = rebuild_function_for_module(
                old_func,
                module,
                &self.func_id_to_name,
                &name_to_id,
                new_func_id,
            )?;

            let mut ctx = module.make_context();
            ctx.func = rebuilt_func;
            module.define_function(new_func_id, &mut ctx).map_err(|e| {
                extract_and_format_module_error(&e, &ctx.func, &format!("function '{}'", name))
            })?;

            // Collect trap information from compiled code
            let mut func_traps = Vec::new();
            if let Some(compiled_code) = ctx.compiled_code() {
                let buffer = &compiled_code.buffer;
                let srclocs = buffer.get_srclocs_sorted();

                for trap in buffer.traps() {
                    // Find the source location for this trap offset using improved lookup
                    let srcloc = find_trap_source_location(trap.offset, srclocs, &ctx.func);

                    func_traps.push(TrapInfo {
                        offset: trap.offset,
                        code: trap.code,
                        srcloc,
                    });
                }
            }

            if !func_traps.is_empty() {
                traps.push((name.clone(), func_traps));
            }

            // Capture CLIF with correct function name before clearing context
            let clif_text = format_function_clif(&ctx.func)?;
            clif_functions.push((name.clone(), clif_text));

            module.clear_context(&mut ctx);
        }

        // Define main function, remapping FuncRefs
        // Rebuild function with proper block params and function name preservation
        let rebuilt_main_func = rebuild_function_for_module(
            &self.main_function,
            module,
            &self.func_id_to_name,
            &name_to_id,
            main_id,
        )?;

        let mut ctx = module.make_context();
        ctx.func = rebuilt_main_func;
        module
            .define_function(main_id, &mut ctx)
            .map_err(|e| extract_and_format_module_error(&e, &ctx.func, "main function"))?;

        // Collect trap information from compiled code for main function
        let mut main_traps = Vec::new();
        if let Some(compiled_code) = ctx.compiled_code() {
            let buffer = &compiled_code.buffer;
            let srclocs = buffer.get_srclocs_sorted();

            for trap in buffer.traps() {
                // Find the source location for this trap offset using improved lookup
                let srcloc = find_trap_source_location(trap.offset, srclocs, &ctx.func);

                main_traps.push(TrapInfo {
                    offset: trap.offset,
                    code: trap.code,
                    srcloc,
                });
            }
        }

        if !main_traps.is_empty() {
            traps.push((String::from("main"), main_traps));
        }

        // Capture CLIF with correct function name before clearing context
        let clif_text = format_function_clif(&ctx.func)?;
        clif_functions.push((String::from("main"), clif_text));

        module.clear_context(&mut ctx);

        // Format CLIF IR for all functions in declaration order
        let mut clif_ir = String::new();

        for (name, clif_text) in &clif_functions {
            clif_ir.push_str(&format!("; function {}:\n", name));
            clif_ir.push_str(clif_text);
            clif_ir.push('\n');
        }

        Ok((name_to_id, clif_ir, traps))
    }

    /// Build an ObjectModule from this CLIF module and extract both ELF bytes and CLIF IR
    /// Returns (ELF bytes, formatted CLIF IR string, trap information)
    #[cfg(feature = "emulator")]
    pub fn build_object_module(&self) -> Result<(Vec<u8>, String, Vec<(String, Vec<TrapInfo>)>), GlslError> {
        use cranelift_module::Linkage;
        use cranelift_object::{ObjectBuilder, ObjectModule};

        // Create ObjectModule for proper linking with relocations
        let isa = self.isa();
        let isa_builder = cranelift_codegen::isa::Builder::from_target_isa(isa);
        let flags = isa.flags().clone();
        let owned_isa = isa_builder.finish(flags).map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                alloc_format!("failed to recreate ISA: {:?}", e),
            )
        })?;

        let builder = ObjectBuilder::new(
            owned_isa,
            "glsl_module",
            cranelift_module::default_libcall_names(),
        )
        .map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                alloc_format!("failed to create ObjectBuilder: {:?}", e),
            )
        })?;

        let mut object_module = ObjectModule::new(builder);

        // Link all functions into the ObjectModule (handles relocations)
        // This returns (name_to_id, clif_ir) where clif_ir contains the formatted CLIF
        let (_name_to_id, clif_ir, traps) = self.link_into(&mut object_module, Linkage::Export)?;

        // Finish the module and get the object file
        let object_product = object_module.finish();
        let object_bytes = object_product.emit().map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                alloc_format!("failed to emit object: {:?}", e),
            )
        })?;

        Ok((object_bytes, clif_ir, traps))
    }
}

/// Format a single function as raw CLIF text (without // prefix)
fn format_function_clif(func: &Function) -> Result<String, GlslError> {
    let mut buf = String::new();
    write_function(&mut buf, func).map_err(|e| {
        GlslError::new(ErrorCode::E0400, format!("failed to write function: {}", e))
    })?;
    Ok(buf)
}

/// Extract and format verifier errors from ModuleError if present
fn extract_and_format_module_error(
    error: &ModuleError,
    func: &Function,
    context: &str,
) -> GlslError {
    let base_message = format!("failed to define {}: {}", context, error);

    // Check if this is a compilation error with verifier errors
    if let ModuleError::Compilation(codegen_error) = error {
        if let CodegenError::Verifier(verifier_errors) = codegen_error {
            // Format verifier errors with function context
            let formatted_errors = pretty_verifier_error(func, None, verifier_errors.clone());

            // Use E0401 (verification error) instead of E0400 when we have verifier details
            return GlslError::new(ErrorCode::E0401, base_message)
                .with_note(format!("Detailed verifier errors:\n{}", formatted_errors));
        }
    }

    // For non-verifier errors or if extraction failed, use generic error
    GlslError::new(ErrorCode::E0400, base_message)
}

/// Builder for constructing a ClifModule
pub struct ClifModuleBuilder {
    user_functions: HashMap<String, Function>,
    main_function: Option<Function>,
    function_registry: Option<FunctionRegistry>,
    source_text: Option<String>,
    isa: Option<OwnedTargetIsa>,
    glsl_signatures: HashMap<String, crate::frontend::semantic::functions::FunctionSignature>,
    func_id_to_name: HashMap<u32, String>,
    source_loc_manager: Option<SourceLocManager>,
    source_map: Option<GlSourceMap>,
}

impl ClifModuleBuilder {
    fn new() -> Self {
        Self {
            user_functions: HashMap::new(),
            main_function: None,
            function_registry: None,
            source_text: None,
            isa: None,
            glsl_signatures: HashMap::new(),
            func_id_to_name: HashMap::new(),
            source_loc_manager: None,
            source_map: None,
        }
    }

    /// Set the source location manager
    pub fn set_source_loc_manager(mut self, manager: SourceLocManager) -> Self {
        self.source_loc_manager = Some(manager);
        self
    }

    /// Set the source map
    pub fn set_source_map(mut self, source_map: GlSourceMap) -> Self {
        self.source_map = Some(source_map);
        self
    }

    /// Add a GLSL signature for a function
    pub fn add_glsl_signature(
        mut self,
        name: String,
        signature: crate::frontend::semantic::functions::FunctionSignature,
    ) -> Self {
        self.glsl_signatures.insert(name, signature);
        self
    }

    /// Add multiple GLSL signatures
    pub fn add_glsl_signatures(
        mut self,
        signatures: HashMap<String, crate::frontend::semantic::functions::FunctionSignature>,
    ) -> Self {
        self.glsl_signatures.extend(signatures);
        self
    }

    /// Add a FuncId to name mapping
    pub fn add_func_id_mapping(mut self, func_id: u32, name: String) -> Self {
        self.func_id_to_name.insert(func_id, name);
        self
    }

    /// Add multiple FuncId to name mappings
    pub fn add_func_id_mappings(mut self, mappings: HashMap<u32, String>) -> Self {
        self.func_id_to_name.extend(mappings);
        self
    }

    /// Add a single user function
    pub fn add_user_function(mut self, name: String, func: Function) -> Self {
        self.user_functions.insert(name, func);
        self
    }

    /// Add multiple user functions
    pub fn add_user_functions(mut self, functions: HashMap<String, Function>) -> Self {
        self.user_functions.extend(functions);
        self
    }

    /// Set the main function
    pub fn set_main_function(mut self, func: Function) -> Self {
        self.main_function = Some(func);
        self
    }

    /// Set the function registry
    pub fn set_function_registry(mut self, registry: FunctionRegistry) -> Self {
        self.function_registry = Some(registry);
        self
    }

    /// Set the source text
    pub fn set_source_text(mut self, text: String) -> Self {
        self.source_text = Some(text);
        self
    }

    /// Set the ISA
    pub fn set_isa(mut self, isa: OwnedTargetIsa) -> Self {
        self.isa = Some(isa);
        self
    }

    /// Build the ClifModule
    pub fn build(self) -> Result<ClifModule, GlslError> {
        use crate::error::{ErrorCode, GlslError};

        let main_function = self
            .main_function
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "main function not set"))?;

        let function_registry = self
            .function_registry
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "function registry not set"))?;

        let source_text = self
            .source_text
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "source text not set"))?;

        let isa = self
            .isa
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "ISA not set"))?;

        let source_map = self
            .source_map
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "source map not set"))?;

        Ok(ClifModule {
            user_functions: self.user_functions,
            main_function,
            function_registry,
            source_text,
            isa,
            glsl_signatures: self.glsl_signatures,
            func_id_to_name: self.func_id_to_name,
            source_loc_manager: self.source_loc_manager.unwrap_or_else(SourceLocManager::new),
            source_map,
        })
    }
}
