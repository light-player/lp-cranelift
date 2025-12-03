use crate::cdsl::isa::TargetIsa;
use crate::cdsl::settings::{PredicateNode, SettingGroup, SettingGroupBuilder};

pub fn define() -> TargetIsa {
    let mut settings = SettingGroupBuilder::new("riscv32");

    // RV32I Base ISA is always present (implied, not a setting)
    
    // RV32 Standard Extensions
    let has_m = settings.add_bool("has_m", "Has the 'M' extension (integer multiplication and division)", true);
    let has_a = settings.add_bool("has_a", "Has the 'A' extension (atomic instructions)", true);
    let has_f = settings.add_bool("has_f", "Has the 'F' extension (single-precision floating-point)", false);
    let has_d = settings.add_bool("has_d", "Has the 'D' extension (double-precision floating-point)", false);
    let has_c = settings.add_bool("has_c", "Has the 'C' extension (compressed instructions)", true);
    
    // Zicsr and Zifencei are separate in newer RISC-V specs
    let has_zicsr = settings.add_bool("has_zicsr", "Has the 'Zicsr' extension (CSR instructions)", true);
    let has_zifencei = settings.add_bool("has_zifencei", "Has the 'Zifencei' extension (instruction-fetch fence)", true);
    
    // Bit manipulation extensions
    let has_zba = settings.add_bool("has_zba", "Has the 'Zba' extension (address generation instructions)", false);
    let has_zbb = settings.add_bool("has_zbb", "Has the 'Zbb' extension (basic bit manipulation)", false);
    let has_zbc = settings.add_bool("has_zbc", "Has the 'Zbc' extension (carry-less multiplication)", false);
    let has_zbs = settings.add_bool("has_zbs", "Has the 'Zbs' extension (single-bit instructions)", false);
    
    // Compressed extension variants
    let has_zca = settings.add_bool("has_zca", "Has the 'Zca' extension (subset of C for all registers)", false);
    let has_zcb = settings.add_bool("has_zcb", "Has the 'Zcb' extension (subset of C for compressed basic ops)", false);
    let has_zcd = settings.add_bool("has_zcd", "Has the 'Zcd' extension (subset of C for double-precision FP)", false);
    let has_zcf = settings.add_bool("has_zcf", "Has the 'Zcf' extension (subset of C for single-precision FP)", false);
    
    // Half-precision floating-point
    let has_zfh = settings.add_bool("has_zfh", "Has the 'Zfh' extension (half-precision floating-point)", false);
    let has_zfhmin = settings.add_bool("has_zfhmin", "Has the 'Zfhmin' extension (minimal half-precision floating-point)", false);
    
    // Additional floating-point extensions
    let has_zfa = settings.add_bool("has_zfa", "Has the 'Zfa' extension (additional floating-point instructions)", false);
    
    // Cryptography extensions
    let has_zbkb = settings.add_bool("has_zbkb", "Has the 'Zbkb' extension (bit manipulation for crypto)", false);
    let has_zbkc = settings.add_bool("has_zbkc", "Has the 'Zbkc' extension (carry-less multiplication for crypto)", false);
    let has_zbkx = settings.add_bool("has_zbkx", "Has the 'Zbkx' extension (crossbar permutations for crypto)", false);
    let has_zkn = settings.add_bool("has_zkn", "Has the 'Zkn' extension (NIST crypto)", false);
    let has_zks = settings.add_bool("has_zks", "Has the 'Zks' extension (ShangMi crypto)", false);
    
    // Vector extension
    let has_v = settings.add_bool("has_v", "Has the 'V' extension (vector operations)", false);
    let has_zvfh = settings.add_bool("has_zvfh", "Has the 'Zvfh' extension (vector half-precision floating-point)", false);
    
    // Vector length settings (Zvl*)
    let has_zvl32b = settings.add_bool("has_zvl32b", "Minimum vector length of 32 bytes", false);
    let has_zvl64b = settings.add_bool("has_zvl64b", "Minimum vector length of 64 bytes", false);
    let has_zvl128b = settings.add_bool("has_zvl128b", "Minimum vector length of 128 bytes", false);
    let has_zvl256b = settings.add_bool("has_zvl256b", "Minimum vector length of 256 bytes", false);
    let has_zvl512b = settings.add_bool("has_zvl512b", "Minimum vector length of 512 bytes", false);
    let has_zvl1024b = settings.add_bool("has_zvl1024b", "Minimum vector length of 1024 bytes", false);
    let has_zvl2048b = settings.add_bool("has_zvl2048b", "Minimum vector length of 2048 bytes", false);
    let has_zvl4096b = settings.add_bool("has_zvl4096b", "Minimum vector length of 4096 bytes", false);
    let has_zvl8192b = settings.add_bool("has_zvl8192b", "Minimum vector length of 8192 bytes", false);
    let has_zvl16384b = settings.add_bool("has_zvl16384b", "Minimum vector length of 16384 bytes", false);
    let has_zvl32768b = settings.add_bool("has_zvl32768b", "Minimum vector length of 32768 bytes", false);
    let has_zvl65536b = settings.add_bool("has_zvl65536b", "Minimum vector length of 65536 bytes", false);
    
    settings.build()
}
