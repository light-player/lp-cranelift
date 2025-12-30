//! Host function implementations for JIT mode.
//!
//! These functions delegate to `lp-glsl` macros for output.

use core::fmt::{self, Write};

/// Writer that formats to a string buffer for host functions
struct HostWriter {
    buf: alloc::string::String,
}

impl Write for HostWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buf.push_str(s);
        Ok(())
    }
}

/// Debug function implementation for JIT mode.
///
/// Delegates to `lp-glsl::debug!` macro.
#[unsafe(no_mangle)]
pub fn __host_debug(args: core::fmt::Arguments) {
    // Format the arguments to a string
    let mut writer = HostWriter {
        buf: alloc::string::String::new(),
    };
    let _ = writer.write_fmt(args);
    
    // Delegate to lp-glsl debug macro
    crate::debug!("{}", writer.buf);
}

/// Println function implementation for JIT mode.
///
/// Delegates to `std::println!`.
#[unsafe(no_mangle)]
pub fn __host_println(args: core::fmt::Arguments) {
    // Format the arguments to a string
    let mut writer = HostWriter {
        buf: alloc::string::String::new(),
    };
    let _ = writer.write_fmt(args);
    
    // Delegate to std::println!
    std::println!("{}", writer.buf);
}

