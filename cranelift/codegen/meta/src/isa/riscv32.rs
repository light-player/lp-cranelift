use crate::cdsl::isa::TargetIsa;
use crate::cdsl::settings::SettingGroupBuilder;

pub(crate) fn define() -> TargetIsa {
    let mut setting = SettingGroupBuilder::new("riscv32");

    // RV32IMAC baseline: I, M, A, C extensions
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
        true,
    );
    let _has_f = setting.add_bool(
        "has_f",
        "has extension F?",
        "Single-precision floating point",
        false,
    );
    let _has_d = setting.add_bool(
        "has_d",
        "has extension D?",
        "Double-precision floating point",
        false,
    );
    
    let _has_zicsr = setting.add_bool(
        "has_zicsr",
        "has extension Zicsr?",
        "Control and status register instructions",
        true,
    );
    let _has_zifencei = setting.add_bool(
        "has_zifencei",
        "has extension Zifencei?",
        "Instruction-fetch fence",
        true,
    );
    
    // Bit manipulation
    let _has_zba = setting.add_bool(
        "has_zba",
        "has extension Zba?",
        "Zba: Address Generation",
        false,
    );
    let _has_zbb = setting.add_bool(
        "has_zbb",
        "has extension Zbb?",
        "Zbb: Basic bit-manipulation",
        false,
    );
    let _has_zbc = setting.add_bool(
        "has_zbc",
        "has extension Zbc?",
        "Zbc: Carry-less multiplication",
        false,
    );
    let _has_zbs = setting.add_bool(
        "has_zbs",
        "has extension Zbs?",
        "Zbs: Single-bit instructions",
        false,
    );
    
    // Compressed
    let has_zca = setting.add_bool(
        "has_zca",
        "has extension Zca?",
        "Zca is the C extension without floating point loads",
        false,
    );
    let has_zcb = setting.add_bool(
        "has_zcb",
        "has extension Zcb?",
        "Zcb: Extra compressed instructions",
        false,
    );
    let has_zcd = setting.add_bool(
        "has_zcd",
        "has extension Zcd?",
        "Zcd contains only the double precision floating point loads from the C extension",
        false,
    );
    let has_zcf = setting.add_bool(
        "has_zcf",
        "has extension Zcf?",
        "Zcf contains only the single precision floating point loads from the C extension",
        false,
    );
    
    // Floating-point extensions
    let _has_zfh = setting.add_bool(
        "has_zfh",
        "has extension Zfh?",
        "Zfh: Half-Precision Floating-Point Instructions",
        false,
    );
    let _has_zfhmin = setting.add_bool(
        "has_zfhmin",
        "has extension Zfhmin?",
        "Zfhmin: Minimal Half-Precision Floating-Point",
        false,
    );
    let _has_zfa = setting.add_bool(
        "has_zfa",
        "has extension Zfa?",
        "Zfa: Extension for Additional Floating-Point Instructions",
        false,
    );
    
    // Cryptography
    let _has_zbkb = setting.add_bool(
        "has_zbkb",
        "has extension Zbkb?",
        "Zbkb: Bit-manipulation for Cryptography",
        false,
    );
    let _has_zbkc = setting.add_bool(
        "has_zbkc",
        "has extension Zbkc?",
        "Zbkc: Carry-less multiplication for Cryptography",
        false,
    );
    let _has_zbkx = setting.add_bool(
        "has_zbkx",
        "has extension Zbkx?",
        "Zbkx: Crossbar permutations for Cryptography",
        false,
    );
    let _has_zkn = setting.add_bool(
        "has_zkn",
        "has extension Zkn?",
        "Zkn: NIST Algorithm Suite",
        false,
    );
    let _has_zks = setting.add_bool(
        "has_zks",
        "has extension Zks?",
        "Zks: ShangMi Algorithm Suite",
        false,
    );
    
    // Vector
    let _has_v = setting.add_bool(
        "has_v",
        "has extension V?",
        "Vector instruction support",
        false,
    );
    let _has_zvfh = setting.add_bool(
        "has_zvfh",
        "has extension Zvfh?",
        "Zvfh: Vector Extension for Half-Precision Floating-Point",
        false,
    );
    
    // Vector length settings
    let _has_zvl32b = setting.add_bool(
        "has_zvl32b",
        "has extension Zvl32b?",
        "Zvl32b: Vector register has a minimum of 32 bits",
        false,
    );
    let _has_zvl64b = setting.add_bool(
        "has_zvl64b",
        "has extension Zvl64b?",
        "Zvl64b: Vector register has a minimum of 64 bits",
        false,
    );
    let _has_zvl128b = setting.add_bool(
        "has_zvl128b",
        "has extension Zvl128b?",
        "Zvl128b: Vector register has a minimum of 128 bits",
        false,
    );
    let _has_zvl256b = setting.add_bool(
        "has_zvl256b",
        "has extension Zvl256b?",
        "Zvl256b: Vector register has a minimum of 256 bits",
        false,
    );
    let _has_zvl512b = setting.add_bool(
        "has_zvl512b",
        "has extension Zvl512b?",
        "Zvl512b: Vector register has a minimum of 512 bits",
        false,
    );
    let _has_zvl1024b = setting.add_bool(
        "has_zvl1024b",
        "has extension Zvl1024b?",
        "Zvl1024b: Vector register has a minimum of 1024 bits",
        false,
    );
    let _has_zvl2048b = setting.add_bool(
        "has_zvl2048b",
        "has extension Zvl2048b?",
        "Zvl2048b: Vector register has a minimum of 2048 bits",
        false,
    );
    let _has_zvl4096b = setting.add_bool(
        "has_zvl4096b",
        "has extension Zvl4096b?",
        "Zvl4096b: Vector register has a minimum of 4096 bits",
        false,
    );
    let _has_zvl8192b = setting.add_bool(
        "has_zvl8192b",
        "has extension Zvl8192b?",
        "Zvl8192b: Vector register has a minimum of 8192 bits",
        false,
    );
    let _has_zvl16384b = setting.add_bool(
        "has_zvl16384b",
        "has extension Zvl16384b?",
        "Zvl16384b: Vector register has a minimum of 16384 bits",
        false,
    );
    let _has_zvl32768b = setting.add_bool(
        "has_zvl32768b",
        "has extension Zvl32768b?",
        "Zvl32768b: Vector register has a minimum of 32768 bits",
        false,
    );
    let _has_zvl65536b = setting.add_bool(
        "has_zvl65536b",
        "has extension Zvl65536b?",
        "Zvl65536b: Vector register has a minimum of 65536 bits",
        false,
    );

    Target Isa::new("riscv32", setting.build())
}
