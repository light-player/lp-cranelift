//! Provides functionality for compiling CLIF IR to object files for emulator execution.
//!
//! Similar to `function_runner.rs` but uses `ObjectModule` to generate ELF files
//! that can be loaded and executed by the RISC-V emulator.

use anyhow::{anyhow, Result};
use core::mem;
use cranelift::prelude::Imm64;
use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::data_value::DataValue;
use cranelift_codegen::ir::{
    ExternalName, Function, InstBuilder, InstructionData, LibCall, Opcode, Signature,
    UserExternalName, UserFuncName,
};
use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_codegen::{CodegenError, Context, ir};
use cranelift_control::ControlPlane;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module, ModuleError};
use cranelift_object::{ObjectBuilder, ObjectModule};
use cranelift_reader::TestFile;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

const TESTFILE_NAMESPACE: u32 = 0;

/// Holds information about a previously defined function.
#[derive(Debug)]
pub struct DefinedFunction {
    /// This is the name that the function is internally known as.
    ///
    /// The Object module does not support linking / calling [TestcaseName]'s, so
    /// we rename every function into a [UserExternalName].
    ///
    /// By doing this we also have to rename functions that previously were using a
    /// [UserFuncName], since they may now be in conflict after the renaming that
    /// occurred.
    new_name: UserExternalName,

    /// The original symbol name used in the ELF file
    pub original_symbol_name: String,

    /// The function signature
    pub signature: ir::Signature,

    /// Object [FuncId]
    func_id: cranelift_module::FuncId,

    /// V-Code generated during compilation (for debugging)
    pub vcode: Option<String>,

    /// Disassembly generated during compilation (for debugging)
    pub disassembly: Option<String>,
}

/// Compile a test case to object format for emulator execution.
///
/// Similar to `TestFileCompiler` but generates ELF object files instead of
/// JIT-compiled code.
pub struct ObjectTestFileCompiler {
    module: ObjectModule,
    ctx: Context,

    /// Holds info about the functions that have already been defined.
    /// Use look them up by their original [UserFuncName] since that's how the caller
    /// passes them to us.
    defined_functions: HashMap<UserFuncName, DefinedFunction>,
}

impl ObjectTestFileCompiler {
    /// Build an [ObjectTestFileCompiler] from a [TargetIsa].
    pub fn new(isa: OwnedTargetIsa) -> Result<Self> {
        let mut builder = ObjectBuilder::new(
            isa,
            "cranelift-filetests".as_bytes(),
            cranelift_module::default_libcall_names(),
        )?;

        let module = ObjectModule::new(builder);
        let ctx = module.make_context();

        Ok(Self {
            module,
            ctx,
            defined_functions: HashMap::new(),
        })
    }

    /// Registers all functions in a [TestFile].
    pub fn add_testfile(&mut self, testfile: &TestFile) -> Result<()> {
        let functions = testfile
            .functions
            .iter()
            .map(|(f, _)| f)
            .cloned()
            .collect::<Vec<_>>();

        self.add_functions(&functions[..], Vec::new())?;
        Ok(())
    }

    /// Declares and compiles all functions in `functions`.
    pub fn add_functions(
        &mut self,
        functions: &[Function],
        ctrl_planes: Vec<ControlPlane>,
    ) -> Result<()> {
        // Declare all functions in the file, so that they may refer to each other.
        for func in functions {
            self.declare_function(func)?;
        }

        let ctrl_planes = ctrl_planes
            .into_iter()
            .chain(std::iter::repeat(ControlPlane::default()));

        // Define all functions
        for (func, ref mut ctrl_plane) in functions.iter().zip(ctrl_planes) {
            self.define_function(func.clone(), ctrl_plane)?;
        }

        Ok(())
    }

    /// Declares a function and registers it as a linkable and callable target internally
    pub fn declare_function(&mut self, func: &Function) -> Result<()> {
        let next_id = self.defined_functions.len() as u32;
        match self.defined_functions.entry(func.name.clone()) {
            Entry::Occupied(_) => {
                anyhow::bail!("Duplicate function with name {} found!", &func.name)
            }
            Entry::Vacant(v) => {
                let name = func.name.to_string();
                let func_id =
                    self.module
                        .declare_function(&name, Linkage::Local, &func.signature)?;

                v.insert(DefinedFunction {
                    new_name: UserExternalName::new(TESTFILE_NAMESPACE, next_id),
                    original_symbol_name: name,
                    signature: func.signature.clone(),
                    func_id,
                    vcode: None,
                    disassembly: None,
                });
            }
        };

        Ok(())
    }

    /// Renames the function to its new [UserExternalName], as well as any other function that
    /// it may reference.
    ///
    /// We have to do this since the Object module does not support linking / calling [TestcaseName]'s.
    fn apply_func_rename(
        &self,
        mut func: Function,
        defined_func: &DefinedFunction,
    ) -> Result<Function> {
        // First, rename the function
        let func_original_name = func.name;
        func.name = UserFuncName::User(defined_func.new_name.clone());

        // Rename any functions that it references
        // Do this in stages to appease the borrow checker
        let mut redefines = Vec::with_capacity(func.dfg.ext_funcs.len());
        for (ext_ref, ext_func) in &func.dfg.ext_funcs {
            let old_name = match &ext_func.name {
                ExternalName::TestCase(tc) => UserFuncName::Testcase(tc.clone()),
                ExternalName::User(username) => {
                    UserFuncName::User(func.params.user_named_funcs()[*username].clone())
                }
                // The other cases don't need renaming, so lets just continue...
                _ => continue,
            };

            let target_df = self.defined_functions.get(&old_name).ok_or(anyhow!(
                "Undeclared function {} is referenced by {}!",
                &old_name,
                &func_original_name
            ))?;

            redefines.push((ext_ref, target_df.new_name.clone()));
        }

        // Now register the redefines
        for (ext_ref, new_name) in redefines.into_iter() {
            // Register the new name in the func, so that we can get a reference to it.
            let new_name_ref = func.params.ensure_user_func_name(new_name);

            // Finally rename the ExtFunc
            func.dfg.ext_funcs[ext_ref].name = ExternalName::User(new_name_ref);
        }

        Ok(func)
    }

    /// Defines the body of a function
    pub fn define_function(
        &mut self,
        mut func: Function,
        ctrl_plane: &mut ControlPlane,
    ) -> Result<()> {
        Self::replace_hostcall_references(&mut func);

        // Get func_id and new_name before mutable borrow
        let original_func_name = func.name.clone();
        let func_id = {
            let defined_func = self
                .defined_functions
                .get(&original_func_name)
                .ok_or(anyhow!("Undeclared function {} found!", &original_func_name))?;
            defined_func.func_id
        };
        
        let new_name = {
            let defined_func = self
                .defined_functions
                .get(&original_func_name)
                .ok_or(anyhow!("Undeclared function {} found!", &original_func_name))?;
            defined_func.new_name.clone()
        };

        // Apply function rename (needs immutable access to self.defined_functions)
        self.ctx.func = self.apply_func_rename(func, &DefinedFunction {
            new_name,
            original_symbol_name: String::new(), // Not needed for rename
            signature: ir::Signature::new(cranelift_codegen::isa::CallConv::SystemV),
            func_id,
            vcode: None,
            disassembly: None,
        })?;
        
        // Enable disassembly for debugging
        self.ctx.set_disasm(true);
        
        // Store function params and ISA for later use
        let func_params = self.ctx.func.params.clone();
        let isa = self.module.isa();
        
        self.module.define_function_with_control_plane(
            func_id,
            &mut self.ctx,
            ctrl_plane,
        )?;
        
        // Capture V-Code and disassembly if available
        // Note: compiled_code is available after define_function_with_control_plane
        let (vcode, disassembly) = if let Some(compiled_code) = self.ctx.compiled_code() {
            // The vcode field contains disassembly when want_disasm is true
            // Capture it as disassembly
            let mut disasm = compiled_code.vcode.as_ref().map(|s| s.clone());
            
            // Try to generate disassembly using Capstone if vcode wasn't available
            #[cfg(feature = "disas")]
            {
                if disasm.is_none() {
                    if let Ok(cs) = isa.to_capstone() {
                        if let Ok(disasm_str) = compiled_code.disassemble(
                            Some(&func_params),
                            &cs,
                        ) {
                            disasm = Some(disasm_str);
                        }
                    }
                }
            }
            
            // For V-Code, we'd need to format the VCode struct before emission
            // For now, we'll use the disassembly as V-Code placeholder
            // TODO: Capture actual pre-regalloc V-Code if needed
            let vcode = disasm.clone();
            (vcode, disasm)
        } else {
            (None, None)
        };
        
        // Now update defined_func with captured vcode and disassembly
        // Use original function name since that's the key in the map
        if let Some(defined_func) = self.defined_functions.get_mut(&original_func_name) {
            defined_func.vcode = vcode;
            defined_func.disassembly = disassembly;
        }
        
        self.module.clear_context(&mut self.ctx);
        Ok(())
    }

    fn replace_hostcall_references(_func: &mut Function) {
        // For emulator execution, we don't need to replace hostcalls
        // since the emulator will handle them differently
    }

    /// Finalize this ObjectTestFileCompiler and return the compiled ELF.
    pub fn compile(mut self) -> Result<CompiledObjectTestFile, CompilationError> {
        let product = self.module.finish();
        let elf_bytes = product.emit().map_err(|e| CompilationError::ObjectWriteError(e))?;

        Ok(CompiledObjectTestFile {
            elf_bytes,
            defined_functions: self.defined_functions,
        })
    }
}

/// A finalized object file test case.
pub struct CompiledObjectTestFile {
    /// The ELF bytes containing all compiled functions
    pub elf_bytes: Vec<u8>,

    /// Holds info about the functions that have been registered.
    pub defined_functions: HashMap<UserFuncName, DefinedFunction>,
}

impl CompiledObjectTestFile {
    /// Get the function signature for a given function name
    pub fn get_signature(&self, func_name: &UserFuncName) -> Option<&Signature> {
        self.defined_functions.get(func_name).map(|df| &df.signature)
    }
}

/// Compilation Error when compiling a function to object format.
#[derive(Error, Debug)]
pub enum CompilationError {
    /// Cranelift codegen error.
    #[error("Cranelift codegen error")]
    CodegenError(#[from] CodegenError),
    /// Module Error
    #[error("Module error")]
    ModuleError(#[from] ModuleError),
    /// Object write error
    #[error("Object write error")]
    ObjectWriteError(#[from] object::write::Error),
}

