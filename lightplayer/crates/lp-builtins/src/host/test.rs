//! Test implementations of host functions.
//!
//! These implementations use `std` and are only available when the `test` feature is enabled.

#[cfg(feature = "test")]
extern crate std;

/// Debug function implementation for tests.
///
/// Checks `DEBUG=1` environment variable and only prints if set.
#[cfg(feature = "test")]
#[unsafe(no_mangle)]
pub fn __host_debug(args: core::fmt::Arguments) {
    if std::env::var("DEBUG").as_deref() == Ok("1") {
        std::println!("{}", args);
    }
}

/// Println function implementation for tests.
///
/// Always prints to stdout.
#[cfg(feature = "test")]
#[unsafe(no_mangle)]
pub fn __host_println(args: core::fmt::Arguments) {
    std::println!("{}", args);
}

