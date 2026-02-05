//! Compilation pipeline methods for Context.

use super::Context;
use crate::isa::TargetIsa;
use crate::machinst::{CompiledCode, CompiledCodeStencil};
use crate::result::{CodegenResult, CompileResult};
use crate::settings::OptLevel;
use crate::trace;
use crate::{CompileError, timing};
use cranelift_control::ControlPlane;

impl Context {
    /// Compile the function,
    ///
    /// Run the function through all the passes necessary to generate
    /// code for the target ISA represented by `isa`. The generated
    /// machine code is not relocated. Instead, any relocations can be
    /// obtained from `compiled_code.buffer.relocs()`.
    ///
    /// Performs any optimizations that are enabled, unless
    /// `optimize()` was already invoked.
    ///
    /// Returns the generated machine code as well as information about
    /// the function's code and read-only data.
    pub fn compile(
        &mut self,
        isa: &dyn TargetIsa,
        ctrl_plane: &mut ControlPlane,
    ) -> CompileResult<'_, &CompiledCode> {
        let stencil = self
            .compile_stencil(isa, ctrl_plane)
            .map_err(|error| CompileError {
                inner: error,
                func: &self.func,
            })?;
        Ok(self
            .compiled_code
            .insert(stencil.apply_params(&self.func.params)))
    }

    /// Internally compiles the function into a stencil.
    ///
    /// Public only for testing and fuzzing purposes.
    pub fn compile_stencil(
        &mut self,
        isa: &dyn TargetIsa,
        ctrl_plane: &mut ControlPlane,
    ) -> CodegenResult<CompiledCodeStencil> {
        let result;
        trace!("****** START compiling {}", self.func.display_spec());
        {
            let _tt = timing::compile();

            self.verify_if(isa)?;
            self.optimize(isa, ctrl_plane)?;
            result = isa.compile_function(&self.func, &self.domtree, self.want_disasm, ctrl_plane);
        }
        trace!("****** DONE compiling {}\n", self.func.display_spec());
        result
    }

    /// Optimize the function, performing all compilation steps up to
    /// but not including machine-code lowering and register
    /// allocation.
    ///
    /// Public only for testing purposes.
    pub fn optimize(
        &mut self,
        isa: &dyn TargetIsa,
        _ctrl_plane: &mut ControlPlane,
    ) -> CodegenResult<()> {
        log::debug!(
            "Number of CLIF instructions to optimize: {}",
            self.func.dfg.num_insts()
        );
        log::debug!(
            "Number of CLIF blocks to optimize: {}",
            self.func.dfg.num_blocks()
        );

        let opt_level = isa.flags().opt_level();
        crate::trace!(
            "Optimizing (opt level {:?}):\n{}",
            opt_level,
            self.func.display()
        );

        if isa.flags().enable_nan_canonicalization() {
            self.canonicalize_nans(isa)?;
        }

        self.legalize(isa)?;

        self.compute_cfg();
        self.compute_domtree();
        self.eliminate_unreachable_code(isa)?;
        self.remove_constant_phis(isa)?;

        self.func.dfg.resolve_all_aliases();

        if opt_level != OptLevel::None {
            #[cfg(feature = "optimizer")]
            {
                self.egraph_pass(isa, _ctrl_plane)?;
            }
            #[cfg(not(feature = "optimizer"))]
            {
                // Optimizer feature is disabled but opt_level != None was requested
                // This is a configuration error, but we continue without optimization
                // rather than panicking to maintain API compatibility
            }
        }

        Ok(())
    }
}
