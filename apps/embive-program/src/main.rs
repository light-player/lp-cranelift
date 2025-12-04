#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate runtime_embive;

use core::panic::PanicInfo;

use runtime_embive::{ebreak, panic_syscall};

/// Panics will report to the host and then exit
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // In no_std, we can't easily format the panic message
    // Use a static message and try to extract location info
    let msg = b"panic occurred\0";

    // Try to extract location info
    let (file_ptr, file_len, line) = if let Some(loc) = info.location() {
        let file = loc.file().as_bytes();
        (file.as_ptr(), file.len(), loc.line())
    } else {
        (core::ptr::null(), 0, 0)
    };

    // Report panic to host
    panic_syscall(msg.as_ptr(), msg.len() - 1, file_ptr, file_len, line);
}

/// Interrupt handler
/// This function is called when an interruption occurs
#[unsafe(no_mangle)]
fn interrupt_handler(_value: i32) {
    // Handle interrupts if needed
}

/// Main program - Phase 1: Simple Hello World
#[unsafe(no_mangle)]
pub extern "Rust" fn main() {
    println!("Hello from RISC-V!");
    println!("Running in no_std with Cranelift-compiled code.");
    println!("This is a test of the toolchain.");

    // Exit successfully
    ebreak()
}

