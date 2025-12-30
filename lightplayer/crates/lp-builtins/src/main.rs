#![cfg_attr(not(feature = "std"), no_std)]
#![no_main]

//! Dummy main for lp-builtins ELF binary.
//! This allows us to build an ELF file that the emulator can load.
//! The actual functions are in the library crate.

#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // For emulator: trap instruction (ebreak)
    #[cfg(target_arch = "riscv32")]
    unsafe {
        core::arch::asm!("ebreak", options(nomem, nostack));
    }
    
    // For other targets: infinite loop
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    // Dummy main - this ELF is just for loading symbols
    0
}

