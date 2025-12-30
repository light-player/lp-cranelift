//! Code generation (Module → Executable)

#[cfg(feature = "emulator")]
pub mod emu;
#[cfg(all(feature = "std", feature = "emulator"))]
pub mod builtins_linker;
pub mod jit;
