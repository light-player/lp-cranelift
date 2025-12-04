#![no_std]
#![no_main]

extern crate alloc;

use cranelift_codegen::isa::riscv32::isa_builder;
use cranelift_codegen::settings::{self, Configurable};
use defmt::info;
use embassy_executor::Spawner;
use esp_hal::{clock::CpuClock, timer::systimer::SystemTimer};
use lp_glsl::Compiler;
use panic_rtt_target as _;
use target_lexicon::Triple;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Allocate heap - ESP32-C6 has plenty of RAM
    esp_alloc::heap_allocator!(size: 128 * 1024);  // 128KB heap for Cranelift

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    // Initialize RTT after heap setup
    rtt_target::rtt_init_defmt!();

    info!("======================================");
    info!("ESP32-C6 GLSL JIT Test");
    info!("Testing Cranelift GLSL Compiler on Real RISC-V Hardware!");
    info!("======================================\n");

    let source = r#"
int main() {
    int a = 7;
    int b = 6;
    int result = a * b;
    return result;
}
"#;

    info!("GLSL Source:\n{}", source);

    // Create RISC-V32 ISA  
    info!("Step 1: Creating RISC-V32 ISA...");
    let mut flag_builder = settings::builder();
    flag_builder.set("opt_level", "none").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    flag_builder.set("enable_verifier", "false").unwrap();
    let isa_flags = settings::Flags::new(flag_builder);

    let triple = Triple {
        architecture: target_lexicon::Architecture::Riscv32(
            target_lexicon::Riscv32Architecture::Riscv32imac,
        ),
        vendor: target_lexicon::Vendor::Unknown,
        operating_system: target_lexicon::OperatingSystem::None_,
        environment: target_lexicon::Environment::Unknown,
        binary_format: target_lexicon::BinaryFormat::Elf,
    };

    let isa = match isa_builder(triple).finish(isa_flags) {
        Ok(isa) => {
            info!("  ✓ ISA created");
            isa
        }
        Err(_) => {
            defmt::panic!("ISA creation failed");
        }
    };

    // Compile GLSL to machine code
    info!("Step 2: Compiling GLSL to RISC-V machine code...");
    let mut compiler = Compiler::new();
    let machine_code = match compiler.compile_to_code(source, &*isa) {
        Ok(code) => {
            info!("  ✓ Compilation successful: {} bytes", code.len());
            code
        }
        Err(_) => {
            defmt::panic!("GLSL compilation failed");
        }
    };

    info!("Step 3: Executing JIT-compiled shader...");

    // Ensure instruction cache coherency
    unsafe {
        core::arch::asm!("fence.i");
    }

    // Cast to function pointer and execute
    type ShaderFn = extern "C" fn() -> i32;
    let shader_fn: ShaderFn = unsafe { core::mem::transmute(machine_code.as_ptr()) };

    let expected = 42;

    info!("Calling compiled GLSL shader");
    let result = shader_fn();

    info!("Result: {}", result);
    info!("Expected: {}", expected);

    if result == expected {
        info!("======================================");
        info!("✅ GLSL JIT TEST SUCCESS ON REAL HARDWARE!");
        info!("======================================");
    } else {
        defmt::panic!("GLSL JIT test failed: expected {}, got {}", expected, result);
    }

    // Loop forever
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

