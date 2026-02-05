//! Compilation backend pipeline: optimized IR to VCode / binemit.

use crate::CodegenError;
use crate::dominator_tree::DominatorTree;
use crate::ir::Function;
use crate::ir::pcc;
use crate::isa::TargetIsa;
use crate::machinst::*;
use crate::settings::RegallocAlgorithm;
use crate::timing;
use crate::trace;

use regalloc2::{Algorithm, RegallocOptions};

/// Compile the given function down to VCode with allocated registers, ready
/// for binary emission.
pub fn compile<B: LowerBackend + TargetIsa>(
    f: &Function,
    domtree: &DominatorTree,
    b: &B,
    abi: Callee<<<B as LowerBackend>::MInst as MachInst>::ABIMachineSpec>,
    emit_info: <B::MInst as MachInstEmit>::Info,
    sigs: SigSet,
    ctrl_plane: &mut ControlPlane,
) -> CodegenResult<(VCode<B::MInst>, regalloc2::Output)> {
    // Compute lowered block order.
    let block_order = BlockLoweringOrder::new(f, domtree, ctrl_plane);

    // Build the lowering context.
    let lower =
        crate::machinst::Lower::new(f, abi, emit_info, block_order, sigs, b.flags().clone())?;

    // Lower the IR.
    let mut vcode = {
        log::debug!(
            "Number of CLIF instructions to lower: {}",
            f.dfg.num_insts()
        );
        log::debug!("Number of CLIF blocks to lower: {}", f.dfg.num_blocks());

        let _tt = timing::vcode_lower();
        lower.lower(b, ctrl_plane)?
    };

    log::debug!(
        "Number of lowered vcode instructions: {}",
        vcode.num_insts()
    );
    log::debug!("Number of lowered vcode blocks: {}", vcode.num_blocks());
    trace!("vcode from lowering: \n{:?}", vcode);

    // Perform validation of proof-carrying-code facts, if requested.
    if b.flags().enable_pcc() {
        pcc::check_vcode_facts(f, &mut vcode, b).map_err(CodegenError::Pcc)?;
    }

    // Validate VCode before register allocation to catch invalid register indices.
    #[cfg(debug_assertions)]
    {
        use crate::machinst::{InsnIndex, Reg};
        use alloc::{string::String, vec::Vec};
        for iix in 0..vcode.num_insts() {
            let inst_idx = InsnIndex::new(iix);
            let mut inst = vcode[inst_idx].clone();
            let mut invalid_regs: Vec<(usize, Reg, String)> = Vec::new();
            inst.get_operands(&mut |reg: &mut Reg, _, _, _| {
                if reg.is_invalid_sentinel() {
                    invalid_regs.push((iix, reg.clone(), String::from("invalid_sentinel")));
                } else if let Some(vreg) = reg.to_virtual_reg() {
                    let index = vreg.index();
                    // Check for suspiciously large indices that indicate corruption
                    if index >= 1000000 {
                        // Much larger than typical VReg counts
                        invalid_regs.push((
                            iix,
                            reg.clone(),
                            alloc::format!("VReg index {}", index),
                        ));
                    }
                }
            });
            if !invalid_regs.is_empty() {
                log::error!("Found invalid register indices in VCode before regalloc:");
                log::error!("  Instruction {}: {:?}", iix, inst);
                for (inst_idx, reg, reason) in invalid_regs {
                    log::error!("  Inst {}: Reg {:?} - {}", inst_idx, reg, reason);
                }
                panic!("Invalid register indices detected before register allocation");
            }
        }
    }

    // Perform register allocation.
    let regalloc_result = {
        let _tt = timing::regalloc();
        let mut options = RegallocOptions::default();
        options.verbose_log = b.flags().regalloc_verbose_logs();

        if cfg!(debug_assertions) {
            options.validate_ssa = true;
        }

        options.algorithm = match b.flags().regalloc_algorithm() {
            RegallocAlgorithm::Backtracking => Algorithm::Ion,
            RegallocAlgorithm::SinglePass => Algorithm::Fastalloc,
        };

        regalloc2::run(&vcode, vcode.abi.machine_env(), &options)
            .map_err(|err| {
                log::error!(
                    "Register allocation error for vcode\n{vcode:?}\nError: {err:?}\nCLIF for error:\n{f:?}",
                );
                err
            })
            .expect("register allocation")
    };

    // Run the regalloc checker, if requested.
    if b.flags().regalloc_checker() {
        let _tt = timing::regalloc_checker();
        let mut checker = regalloc2::checker::Checker::new(&vcode, vcode.abi.machine_env());
        checker.prepare(&regalloc_result);
        checker
            .run()
            .map_err(|err| {
                log::error!("Register allocation checker errors:\n{err:?}\nfor vcode:\n{vcode:?}");
                err
            })
            .expect("register allocation checker");
    }

    Ok((vcode, regalloc_result))
}
