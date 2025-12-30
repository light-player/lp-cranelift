#![cfg_attr(not(feature = "std"), no_std)]

//! Light Player builtins library.
//!
//! This crate provides low-level builtin functions for the Light Player compiler.
//! Functions are exported with `#[no_mangle] pub extern "C"` for linking.

pub mod fixed32;
// mem module only used when baremetal feature is not enabled
// When baremetal is enabled, rlibc provides memcpy/memset/memcmp
#[cfg(not(feature = "baremetal"))]
pub mod mem;
#[cfg(not(feature = "std"))]
mod panic;

// When baremetal feature is enabled, rlibc provides memcpy/memset/memcmp
// No need to re-export - rlibc exports them with #[no_mangle]

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
