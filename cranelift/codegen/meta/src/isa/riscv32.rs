use crate::cdsl::isa::TargetIsa;
use crate::cdsl::settings::SettingGroupBuilder;

pub(crate) fn define() -> TargetIsa {
    let mut setting = SettingGroupBuilder::new("riscv32");

    // RISC-V32 baseline: RV32I
    // For now, we'll support a minimal configuration.
    // Extensions can be added later as needed for the Light Player backend.
    
    let _has_m = setting.add_bool(
        "has_m",
        "has extension M?",
        "Integer multiplication and division",
        true,
    );
    
    let _has_a = setting.add_bool(
        "has_a",
        "has extension A?",
        "Atomic instructions",
        false,
    );
    
    let _has_c = setting.add_bool(
        "has_c",
        "has extension C?",
        "Compressed instructions",
        true,
    );

    TargetIsa::new("riscv32", setting.build())
}

