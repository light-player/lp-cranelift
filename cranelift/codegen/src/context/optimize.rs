//! Optimization pass methods for Context.

use super::Context;
use crate::alias_analysis::AliasAnalysis;
use crate::egraph::EgraphPass;
use crate::inline::{Inline, do_inlining};
use crate::isa::TargetIsa;
use crate::legalizer::simple_legalize;
use crate::nan_canonicalization::do_nan_canonicalization;
use crate::remove_constant_phis::do_remove_constant_phis;
use crate::result::CodegenResult;
use crate::settings::FlagsOrIsa;
use crate::timing;
use crate::trace;
use crate::unreachable_code::eliminate_unreachable_code;
#[cfg(feature = "souper-harvest")]
use alloc::string::String;
use cranelift_control::ControlPlane;
use target_lexicon::Architecture;

#[cfg(feature = "souper-harvest")]
use crate::souper_harvest::do_souper_harvest;

impl Context {
    /// Perform function call inlining.
    ///
    /// Returns `true` if any function call was inlined, `false` otherwise.
    pub fn inline(&mut self, inliner: impl Inline) -> CodegenResult<bool> {
        do_inlining(&mut self.func, inliner)
    }

    /// Perform constant-phi removal on the function.
    pub fn remove_constant_phis<'a, FOI: Into<FlagsOrIsa<'a>>>(
        &mut self,
        fisa: FOI,
    ) -> CodegenResult<()> {
        do_remove_constant_phis(&mut self.func, &mut self.domtree);
        self.verify_if(fisa)?;
        Ok(())
    }

    /// Perform NaN canonicalizing rewrites on the function.
    pub fn canonicalize_nans(&mut self, isa: &dyn TargetIsa) -> CodegenResult<()> {
        // Currently only RiscV64 is the only arch that may not have vector support.
        let has_vector_support = match isa.triple().architecture {
            Architecture::Riscv64(_) => match isa.isa_flags().iter().find(|f| f.name == "has_v") {
                Some(value) => value.as_bool().unwrap_or(false),
                None => false,
            },
            _ => true,
        };
        do_nan_canonicalization(&mut self.func, has_vector_support);
        self.verify_if(isa)
    }

    /// Run the legalizer for `isa` on the function.
    pub fn legalize(&mut self, isa: &dyn TargetIsa) -> CodegenResult<()> {
        // Legalization invalidates the domtree and loop_analysis by mutating the CFG.
        // TODO: Avoid doing this when legalization doesn't actually mutate the CFG.
        self.domtree.clear();
        self.loop_analysis.clear();
        self.cfg.clear();

        // Run some specific legalizations only.
        simple_legalize(&mut self.func, isa);
        self.verify_if(isa)
    }

    /// Perform unreachable code elimination.
    pub fn eliminate_unreachable_code<'a, FOI>(&mut self, fisa: FOI) -> CodegenResult<()>
    where
        FOI: Into<FlagsOrIsa<'a>>,
    {
        eliminate_unreachable_code(&mut self.func, &mut self.cfg, &self.domtree);
        self.verify_if(fisa)
    }

    /// Replace all redundant loads with the known values in
    /// memory. These are loads whose values were already loaded by
    /// other loads earlier, as well as loads whose values were stored
    /// by a store instruction to the same instruction (so-called
    /// "store-to-load forwarding").
    pub fn replace_redundant_loads(&mut self) -> CodegenResult<()> {
        let mut analysis = AliasAnalysis::new(&self.func, &self.domtree);
        analysis.compute_and_update_aliases(&mut self.func);
        Ok(())
    }

    /// Harvest candidate left-hand sides for superoptimization with Souper.
    #[cfg(feature = "souper-harvest")]
    pub fn souper_harvest(
        &mut self,
        out: &mut std::sync::mpsc::Sender<String>,
    ) -> CodegenResult<()> {
        do_souper_harvest(&self.func, out);
        Ok(())
    }

    /// Run optimizations via the egraph infrastructure.
    pub fn egraph_pass<'a, FOI>(
        &mut self,
        fisa: FOI,
        ctrl_plane: &mut ControlPlane,
    ) -> CodegenResult<()>
    where
        FOI: Into<FlagsOrIsa<'a>>,
    {
        let _tt = timing::egraph();

        trace!(
            "About to optimize with egraph phase:\n{}",
            self.func.display()
        );
        let fisa = fisa.into();
        self.compute_loop_analysis();
        let mut alias_analysis = AliasAnalysis::new(&self.func, &self.domtree);
        let mut pass = EgraphPass::new(
            &mut self.func,
            &self.domtree,
            &self.loop_analysis,
            &mut alias_analysis,
            &fisa.flags,
            ctrl_plane,
        );
        pass.run();
        log::debug!("egraph stats: {:?}", pass.stats);
        trace!("After egraph optimization:\n{}", self.func.display());

        self.verify_if(fisa)
    }
}
