use core::fmt::{self, Write};

use crate::{SYSCALL_ARGS, syscall};

/// Syscall number for write
const SYSCALL_WRITE: i32 = 2;

/// Writer that sends output to the host via syscall
///
/// Syscall 2: Write string to host
/// - args[0] = pointer to string (as i32)
/// - args[1] = length of string
pub struct HostWriter;

impl Write for HostWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Skip empty strings (formatting artifacts)
        if s.is_empty() {
            return Ok(());
        }

        // Syscall 2: Write string to host
        // args[0] = pointer to string (as i32)
        // args[1] = length of string
        let ptr = s.as_ptr() as usize as i32;
        let len = s.len() as i32;

        let mut args = [0i32; SYSCALL_ARGS];
        args[0] = ptr;
        args[1] = len;
        let _ = syscall(SYSCALL_WRITE, &args);
        Ok(())
    }
}

/// Global writer instance for host functions
static mut HOST_WRITER: HostWriter = HostWriter;

/// Debug function implementation for emulator.
///
/// This function is called by the `host_debug!` macro.
/// For now, it always prints (we can add DEBUG env var check later if needed).
#[unsafe(no_mangle)]
#[allow(static_mut_refs)] // Safe: HOST_WRITER is only accessed from this single-threaded function
pub fn __host_debug(args: core::fmt::Arguments) {
    unsafe {
        // Use addr_of_mut! to safely get a pointer to the mutable static
        // This avoids creating a mutable reference directly, which is unsafe in Rust 2024
        match (*core::ptr::addr_of_mut!(HOST_WRITER)).write_fmt(args) {
            Ok(()) => {}
            Err(_) => {
                // If formatting fails, we can't do much in no_std
                // But at least we tried
            }
        }
    }
}

/// Println function implementation for emulator.
///
/// This function is called by the `host_println!` macro.
#[unsafe(no_mangle)]
#[allow(static_mut_refs)] // Safe: HOST_WRITER is only accessed from this single-threaded function
pub fn __host_println(args: core::fmt::Arguments) {
    unsafe {
        // Use addr_of_mut! to safely get a pointer to the mutable static
        // This avoids creating a mutable reference directly, which is unsafe in Rust 2024
        match (*core::ptr::addr_of_mut!(HOST_WRITER)).write_fmt(args) {
            Ok(()) => {}
            Err(_) => {
                // If formatting fails, we can't do much in no_std
                // But at least we tried
            }
        }
    }
    // Print newline
    let newline = "\n";
    let ptr = newline.as_ptr() as usize as i32;
    let len = newline.len() as i32;
    let mut args = [0i32; SYSCALL_ARGS];
    args[0] = ptr;
    args[1] = len;
    let _ = syscall(SYSCALL_WRITE, &args);
}

