#![cfg_attr(not(feature = "std"), no_std)]

//! Light Player builtins library.
//!
//! This crate provides low-level builtin functions for the Light Player compiler.
//! Functions are exported with `#[no_mangle] pub extern "C"` for linking.

pub mod fixed32;

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
