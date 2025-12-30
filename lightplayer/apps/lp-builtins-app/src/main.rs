#![no_std]
#![no_main]

use core::{
    arch::global_asm,
    mem::zeroed,
    ptr::{addr_of_mut, read, write_volatile},
};

use lp_builtins::fixed32::{__lp_fixed32_div, __lp_fixed32_mul, __lp_fixed32_sqrt};

/// Panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // For emulator: trap instruction (ebreak)
    #[cfg(target_arch = "riscv32")]
    unsafe {
        core::arch::asm!("ebreak", options(nomem, nostack, noreturn));
    }
    
    // For other targets: infinite loop
    #[cfg(not(target_arch = "riscv32"))]
    loop {}
}

// Binary entry point
// Initializes the global, stack, and frame pointers; and then calls the _code_entry function
global_asm! {
    ".section .text.init.entry, \"ax\"",
    ".global _entry",
    "_entry:",
    ".option push",
    ".option norelax",
    ".option norvc",  // Disable compressed instructions
    // Initialize global pointer
    "la gp, __global_pointer$",
    // Initialize stack and frame pointers
    "la t1, __stack_start",
    "andi sp, t1, -16",
    "add s0, sp, zero",
    ".option pop",
    // Call _code_entry using long-range jump (la pseudo-instruction expands to auipc + addi)
    "la t0, _code_entry",
    "jalr ra, 0(t0)",
}

/// This code is responsible for initializing the .bss and .data sections, and calling the placeholder main function.
#[unsafe(no_mangle)]
unsafe extern "C" fn _code_entry() -> ! {
    unsafe extern "C" {
        // These symbols come from `memory.ld`
        static mut __bss_target_start: u32; // Start of .bss target
        static mut __bss_target_end: u32; // End of .bss target
        static mut __data_target_start: u32; // Start of .data target
        static mut __data_target_end: u32; // End of .data target
        static __data_source_start: u32; // Start of .data source
    }

    // Initialize (Zero) BSS
    let mut sbss: *mut u32 = addr_of_mut!(__bss_target_start);
    let ebss: *mut u32 = addr_of_mut!(__bss_target_end);

    while sbss < ebss {
        unsafe {
            write_volatile(sbss, zeroed());
            sbss = sbss.offset(1);
        }
    }

    // Initialize Data
    let mut sdata: *mut u32 = addr_of_mut!(__data_target_start);
    let edata: *mut u32 = addr_of_mut!(__data_target_end);
    let mut sdatas: *const u32 = unsafe { &__data_source_start };

    while sdata < edata {
        unsafe {
            write_volatile(sdata, read(sdatas));
            sdata = sdata.offset(1);
            sdatas = sdatas.offset(1);
        }
    }

    // Call placeholder main function
    unsafe extern "C" {
        fn main();
    }

    unsafe {
        main();
    }

    // If main returns (shouldn't happen), loop forever
    loop {}
}

/// User main pointer - will be overwritten by linker to point to actual user main()
/// Initialized to sentinel value 0xDEADBEEF to make it obvious if relocation isn't applied
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static mut __USER_MAIN_PTR: u32 = 0xDEADBEEF;

/// Placeholder main function that references all builtin functions to prevent dead code elimination.
/// 
/// This function:
/// 1. References all __lp_* functions explicitly (prevents dead code elimination)
/// 2. Reads __user_main_ptr from .data section
/// 3. Jumps to that address if non-zero, otherwise panics
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Reference all builtin functions to prevent dead code elimination
    // These references ensure the functions are included in the executable
    unsafe {
        // Create function pointers (never actually called, just referenced)
        let _sqrt_fn: extern "C" fn(i32) -> i32 = __lp_fixed32_sqrt;
        let _mul_fn: extern "C" fn(i32, i32) -> i32 = __lp_fixed32_mul;
        let _div_fn: extern "C" fn(i32, i32) -> i32 = __lp_fixed32_div;
        
        // Force these to be included by using them in a way that can't be optimized away
        // We'll use volatile reads to prevent optimization
        let _ = core::ptr::read_volatile(&_sqrt_fn as *const _);
        let _ = core::ptr::read_volatile(&_mul_fn as *const _);
        let _ = core::ptr::read_volatile(&_div_fn as *const _);
    }

    // Read user main pointer
    let user_main_ptr = unsafe { core::ptr::read_volatile(&raw const __USER_MAIN_PTR as *const u32) };
    
    if user_main_ptr == 0 || user_main_ptr == 0xDEADBEEF {
        // No user main set - panic
        panic!("__user_main_ptr not set (value: 0x{:x})", user_main_ptr);
    }

    // Jump to user main
    unsafe {
        let user_main: extern "C" fn() -> ! = core::mem::transmute(user_main_ptr);
        user_main();
    }
}

