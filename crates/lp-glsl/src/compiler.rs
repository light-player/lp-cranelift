#[cfg(feature = "std")]
use crate::jit::JIT;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::String;

/// High-level compiler interface
#[cfg(feature = "std")]
pub struct Compiler {
    jit: JIT,
}

#[cfg(feature = "std")]
impl Compiler {
    pub fn new() -> Self {
        Self { jit: JIT::new() }
    }

    /// Compile GLSL shader that returns i32
    pub fn compile_int(&mut self, source: &str) -> Result<fn() -> i32, String> {
        let code_ptr = self.jit.compile(source)?;
        Ok(unsafe { std::mem::transmute(code_ptr) })
    }

    /// Compile GLSL shader that returns bool
    pub fn compile_bool(&mut self, source: &str) -> Result<fn() -> i8, String> {
        let code_ptr = self.jit.compile(source)?;
        Ok(unsafe { std::mem::transmute(code_ptr) })
    }

    /// Compile to CLIF IR for debugging/testing
    pub fn compile_to_clif(&mut self, source: &str) -> Result<String, String> {
        self.jit.compile_to_clif(source)
    }
}

#[cfg(feature = "std")]
impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

// Stub for no_std
#[cfg(not(feature = "std"))]
pub struct Compiler;

#[cfg(not(feature = "std"))]
impl Compiler {
    pub fn new() -> Self {
        Self
    }
}

