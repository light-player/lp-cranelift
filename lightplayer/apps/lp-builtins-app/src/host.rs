use crate::{SYSCALL_ARGS, syscall};

/// Syscall number for write
const SYSCALL_WRITE: i32 = 2;

/// Debug function implementation for emulator.
///
/// This function is called by the `host_debug!` macro.
/// For now, it always prints (we can add DEBUG env var check later if needed).
#[unsafe(no_mangle)]
pub extern "C" fn __host_debug(ptr: *const u8, len: usize) {
    let ptr = ptr as usize as i32;
    let len = len as i32;

    let mut args = [0i32; SYSCALL_ARGS];
    args[0] = ptr;
    args[1] = len;
    let _ = syscall(SYSCALL_WRITE, &args);
}

/// Println function implementation for emulator.
///
/// This function is called by the `host_println!` macro.
#[unsafe(no_mangle)]
pub extern "C" fn __host_println(ptr: *const u8, len: usize) {
    // Print the message
    let ptr = ptr as usize as i32;
    let len = len as i32;

    let mut args = [0i32; SYSCALL_ARGS];
    args[0] = ptr;
    args[1] = len;
    let _ = syscall(SYSCALL_WRITE, &args);

    // Print newline
    let newline = "\n";
    let ptr = newline.as_ptr() as usize as i32;
    let len = newline.len() as i32;
    let mut args = [0i32; SYSCALL_ARGS];
    args[0] = ptr;
    args[1] = len;
    let _ = syscall(SYSCALL_WRITE, &args);
}
