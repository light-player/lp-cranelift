//! risc-v 64-bit Instruction Set Architecture.

use crate::dominator_tree::DominatorTree;
use crate::ir::{Function, Type};
use crate::isa::riscv32::settings as riscv_settings;
use crate::isa::{
    Builder as IsaBuilder, FunctionAlignment, IsaFlagsHashKey, OwnedTargetIsa, TargetIsa,
};
use crate::machinst::{
    CompiledCode, CompiledCodeStencil, MachInst, MachTextSectionBuilder, Reg, SigSet, TextSectionBuilder, VCode,
    compile,
};
use crate::result::CodegenResult;
use crate::settings::{self as shared_settings, Flags};
use crate::{CodegenError, ir};
use alloc::string::String;
use alloc::{boxed::Box, format, vec::Vec};
use core::fmt;
use cranelift_control::ControlPlane;
use target_lexicon::{Architecture, Triple};
mod abi;
pub(crate) mod inst;
mod lower;
mod settings;
#[cfg(feature = "unwind")]
use crate::isa::unwind::systemv;

use self::inst::EmitInfo;

/// An riscv32 backend.
pub struct Riscv32Backend {
    triple: Triple,
    flags: shared_settings::Flags,
    isa_flags: riscv_settings::Flags,
}

impl Riscv32Backend {
    /// Create a new riscv32 backend with the given (shared) flags.
    pub fn new_with_flags(
        triple: Triple,
        flags: shared_settings::Flags,
        isa_flags: riscv_settings::Flags,
    ) -> Riscv32Backend {
        Riscv32Backend {
            triple,
            flags,
            isa_flags,
        }
    }

    /// This performs lowering to VCode, register-allocates the code, computes block layout and
    /// finalizes branches. The result is ready for binary emission.
    fn compile_vcode(
        &self,
        func: &Function,
        domtree: &DominatorTree,
        ctrl_plane: &mut ControlPlane,
    ) -> CodegenResult<(VCode<inst::Inst>, regalloc2::Output)> {
        let emit_info = EmitInfo::new(self.flags.clone(), self.isa_flags.clone());
        let sigs = SigSet::new::<abi::Riscv32MachineDeps>(func, &self.flags)?;
        let abi = abi::Riscv32Callee::new(func, self, &self.isa_flags, &sigs)?;
        compile::compile::<Riscv32Backend>(func, domtree, self, abi, emit_info, sigs, ctrl_plane)
    }
}

impl TargetIsa for Riscv32Backend {
    fn compile_function(
        &self,
        func: &Function,
        domtree: &DominatorTree,
        want_disasm: bool,
        ctrl_plane: &mut ControlPlane,
    ) -> CodegenResult<CompiledCodeStencil> {
        let (vcode, regalloc_result) = self.compile_vcode(func, domtree, ctrl_plane)?;

        let want_disasm = want_disasm || log::log_enabled!(log::Level::Debug);
        let emit_result = vcode.emit(&regalloc_result, want_disasm, &self.flags, ctrl_plane);
        let value_labels_ranges = emit_result.value_labels_ranges;
        let buffer = emit_result.buffer;

        if let Some(disasm) = emit_result.disasm.as_ref() {
            log::debug!("disassembly:\n{disasm}");
        }

        Ok(CompiledCodeStencil {
            buffer,
            vcode: emit_result.disasm,
            value_labels_ranges,
            bb_starts: emit_result.bb_offsets,
            bb_edges: emit_result.bb_edges,
        })
    }

    fn name(&self) -> &'static str {
        "riscv32"
    }
    fn dynamic_vector_bytes(&self, _dynamic_ty: ir::Type) -> u32 {
        16
    }

    fn triple(&self) -> &Triple {
        &self.triple
    }

    fn flags(&self) -> &shared_settings::Flags {
        &self.flags
    }

    fn isa_flags(&self) -> Vec<shared_settings::Value> {
        self.isa_flags.iter().collect()
    }

    fn isa_flags_hash_key(&self) -> IsaFlagsHashKey<'_> {
        IsaFlagsHashKey(self.isa_flags.hash_key())
    }

    #[cfg(feature = "unwind")]
    fn emit_unwind_info(
        &self,
        result: &CompiledCode,
        kind: crate::isa::unwind::UnwindInfoKind,
    ) -> CodegenResult<Option<crate::isa::unwind::UnwindInfo>> {
        use crate::isa::unwind::UnwindInfo;
        use crate::isa::unwind::UnwindInfoKind;
        Ok(match kind {
            UnwindInfoKind::SystemV => {
                let mapper = self::inst::unwind::systemv::RegisterMapper;
                Some(UnwindInfo::SystemV(
                    crate::isa::unwind::systemv::create_unwind_info_from_insts(
                        &result.buffer.unwind_info[..],
                        result.buffer.data().len(),
                        &mapper,
                    )?,
                ))
            }
            UnwindInfoKind::Windows => None,
            _ => None,
        })
    }

    #[cfg(feature = "unwind")]
    fn create_systemv_cie(&self) -> Option<gimli::write::CommonInformationEntry> {
        Some(inst::unwind::systemv::create_cie())
    }

    fn text_section_builder(&self, num_funcs: usize) -> Box<dyn TextSectionBuilder> {
        Box::new(MachTextSectionBuilder::<inst::Inst>::new(num_funcs))
    }

    #[cfg(feature = "unwind")]
    fn map_regalloc_reg_to_dwarf(&self, reg: Reg) -> Result<u16, systemv::RegisterMappingError> {
        inst::unwind::systemv::map_reg(reg).map(|reg| reg.0)
    }

    fn function_alignment(&self) -> FunctionAlignment {
        inst::Inst::function_alignment()
    }

    fn page_size_align_log2(&self) -> u8 {
        debug_assert_eq!(1 << 12, 0x1000);
        12
    }

    #[cfg(feature = "disas")]
    fn to_capstone(&self) -> Result<capstone::Capstone, capstone::Error> {
        use capstone::prelude::*;
        let mut cs_builder = Capstone::new().riscv().mode(arch::riscv::ArchMode::RiscV32);

        // Enable C instruction decoding if we have compressed instructions enabled.
        //
        // We can't enable this unconditionally because it will cause Capstone to
        // emit weird instructions and generally mess up when it encounters unknown
        // instructions, such as any Zba,Zbb,Zbc or Vector instructions.
        //
        // This causes the default disassembly to be quite unreadable, so enable
        // it only when we are actually going to be using them.
        let uses_compressed = self
            .isa_flags()
            .iter()
            .filter(|f| ["has_zca", "has_zcb", "has_zcd"].contains(&f.name))
            .any(|f| f.as_bool().unwrap_or(false));
        if uses_compressed {
            cs_builder = cs_builder.extra_mode([arch::riscv::ArchExtraMode::RiscVC].into_iter());
        }

        let mut cs = cs_builder.build()?;

        // Similar to AArch64, RISC-V uses inline constants rather than a separate
        // constant pool. We want to skip disassembly over inline constants instead
        // of stopping on invalid bytes.
        cs.set_skipdata(true)?;
        Ok(cs)
    }

    fn pretty_print_reg(&self, reg: Reg, _size: u8) -> String {
        // TODO-RISC-V: implement proper register pretty-printing.
        format!("{reg:?}")
    }

    fn has_native_fma(&self) -> bool {
        true
    }

    fn has_round(&self) -> bool {
        true
    }

    fn has_x86_blendv_lowering(&self, _: Type) -> bool {
        false
    }

    fn has_x86_pshufb_lowering(&self) -> bool {
        false
    }

    fn has_x86_pmulhrsw_lowering(&self) -> bool {
        false
    }

    fn has_x86_pmaddubsw_lowering(&self) -> bool {
        false
    }

    fn default_argument_extension(&self) -> ir::ArgumentExtension {
        // According to https://riscv.org/wp-content/uploads/2024/12/riscv-calling.pdf
        // it says:
        //
        // > In RV64, 32-bit types, such as int, are stored in integer
        // > registers as proper sign extensions of their 32-bit values; that
        // > is, bits 63..31 are all equal. This restriction holds even for
        // > unsigned 32-bit types.
        //
        // leading to `sext` here.
        ir::ArgumentExtension::Sext
    }
}

impl fmt::Display for Riscv32Backend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MachBackend")
            .field("name", &self.name())
            .field("triple", &self.triple())
            .field("flags", &format!("{}", self.flags()))
            .finish()
    }
}

/// Create a new `isa::Builder`.
pub fn isa_builder(triple: Triple) -> IsaBuilder {
    match triple.architecture {
        Architecture::Riscv32(..) => {}
        _ => unreachable!(),
    }
    IsaBuilder {
        triple,
        setup: riscv_settings::builder(),
        constructor: isa_constructor,
    }
}

fn isa_constructor(
    triple: Triple,
    shared_flags: Flags,
    builder: &shared_settings::Builder,
) -> CodegenResult<OwnedTargetIsa> {
    let isa_flags = riscv_settings::Flags::new(&shared_flags, builder);

    // Unlike riscv64, we don't require the G extension (which includes F and D).
    // For RV32, we support various configurations:
    // - RV32I: Base integer instruction set (always required)
    // - RV32M: Integer multiplication and division (typically enabled)
    // - RV32A: Atomic instructions (optional)
    // - RV32C: Compressed instructions (optional)
    // - RV32F: Single-precision floating-point (optional)
    // - RV32D: Double-precision floating-point (optional, requires F)
    //
    // This allows for configurations like RV32IMAC (no floating point).

    // Verify D extension doesn't appear without F extension
    if isa_flags.has_d() && !isa_flags.has_f() {
        return Err(CodegenError::Unsupported(
            "RISC-V D extension requires F extension to be enabled".into(),
        ));
    }

    let backend = Riscv32Backend::new_with_flags(triple, shared_flags, isa_flags);
    Ok(backend.wrapped())
}
