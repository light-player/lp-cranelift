//! Debug logging macro for development
//!
//! The `debug!` macro only prints output when:
//! 1. The `std` feature is enabled
//! 2. The `DEBUG=1` environment variable is set
//!
//! Usage:
//! ```rust
//! debug!("message: {:?}", value);
//! ```

#[cfg(feature = "std")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if std::env::var("DEBUG").as_deref() == Ok("1") {
            println!($($arg)*);
        }
    };
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {};
}

