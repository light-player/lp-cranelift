//! Code generation (Module → Executable)

#[cfg(all(feature = "std", feature = "emulator"))]
pub mod builtins_linker;
#[cfg(feature = "emulator")]
pub mod emu;
pub mod jit;
#[cfg(all(feature = "std", feature = "emulator"))]
pub mod shared_emulator;
