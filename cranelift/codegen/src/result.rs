//! Result and error types representing the outcome of compiling a function.

use regalloc2::checker::CheckerErrors;

#[cfg(not(feature = "verifier"))]
use crate::ir::Function;
use crate::ir::pcc::PccError;
#[cfg(feature = "verifier")]
use crate::{ir::Function, verifier::VerifierErrors};
use alloc::string::String;

// Stub type when verifier is disabled
#[cfg(not(feature = "verifier"))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerifierErrors(pub alloc::vec::Vec<alloc::string::String>);

#[cfg(not(feature = "verifier"))]
impl VerifierErrors {
    pub fn new() -> Self {
        Self(alloc::vec::Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn has_error(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn as_result(&self) -> Result<(), ()> {
        if self.is_empty() { Ok(()) } else { Err(()) }
    }
}

#[cfg(not(feature = "verifier"))]
#[cfg(feature = "std")]
impl std::error::Error for VerifierErrors {}

#[cfg(not(feature = "verifier"))]
impl core::fmt::Display for VerifierErrors {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        for err in &self.0 {
            writeln!(f, "- {}", err)?;
        }
        Ok(())
    }
}

/// Result type for verifier operations when verifier feature is disabled.
#[cfg(not(feature = "verifier"))]
pub type VerifierResult<T> = Result<T, VerifierErrors>;

/// A compilation error.
///
/// When Cranelift fails to compile a function, it will return one of these error codes.
#[derive(Debug)]
pub enum CodegenError {
    /// A list of IR verifier errors.
    ///
    /// This always represents a bug, either in the code that generated IR for Cranelift, or a bug
    /// in Cranelift itself.
    #[cfg(feature = "verifier")]
    Verifier(VerifierErrors),
    /// A list of IR verifier errors (stub when verifier feature is disabled).
    #[cfg(not(feature = "verifier"))]
    Verifier(VerifierErrors),

    /// An implementation limit was exceeded.
    ///
    /// Cranelift can compile very large and complicated functions, but the [implementation has
    /// limits][limits] that cause compilation to fail when they are exceeded.
    ///
    /// [limits]: https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md#implementation-limits
    ImplLimitExceeded,

    /// The code size for the function is too large.
    ///
    /// Different target ISAs may impose a limit on the size of a compiled function. If that limit
    /// is exceeded, compilation fails.
    CodeTooLarge,

    /// Something is not supported by the code generator. This might be an indication that a
    /// feature is used without explicitly enabling it, or that something is temporarily
    /// unsupported by a given target backend.
    Unsupported(String),

    /// A failure to map Cranelift register representation to a DWARF register representation.
    #[cfg(feature = "unwind")]
    RegisterMappingError(crate::isa::unwind::systemv::RegisterMappingError),

    /// Register allocator internal error discovered by the symbolic checker.
    Regalloc(CheckerErrors),

    /// Proof-carrying-code validation error.
    Pcc(PccError),
}

/// A convenient alias for a `Result` that uses `CodegenError` as the error type.
pub type CodegenResult<T> = Result<T, CodegenError>;

// This is manually implementing Error and Display instead of using thiserror to reduce the amount
// of dependencies used by Cranelift.
#[cfg(feature = "std")]
#[cfg(feature = "std")]
impl std::error::Error for CodegenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "verifier")]
            CodegenError::Verifier(source) => Some(source),
            #[cfg(not(feature = "verifier"))]
            CodegenError::Verifier(_) => None,
            CodegenError::ImplLimitExceeded { .. }
            | CodegenError::CodeTooLarge { .. }
            | CodegenError::Unsupported { .. } => None,
            #[cfg(feature = "unwind")]
            CodegenError::RegisterMappingError { .. } => None,
            CodegenError::Regalloc(..) => None,
            CodegenError::Pcc(..) => None,
        }
    }
}

impl core::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            CodegenError::Verifier(_) => write!(f, "Verifier errors"),
            CodegenError::ImplLimitExceeded => write!(f, "Implementation limit exceeded"),
            CodegenError::CodeTooLarge => write!(f, "Code for function is too large"),
            CodegenError::Unsupported(feature) => write!(f, "Unsupported feature: {feature}"),
            #[cfg(feature = "unwind")]
            CodegenError::RegisterMappingError(_0) => write!(f, "Register mapping error"),
            CodegenError::Regalloc(errors) => write!(f, "Regalloc validation errors: {errors:?}"),

            // NOTE: if this is changed, please update the `is_pcc_error` function defined in
            // `wasmtime/crates/fuzzing/src/oracles.rs`
            CodegenError::Pcc(e) => write!(f, "Proof-carrying-code validation error: {e:?}"),
        }
    }
}

impl From<VerifierErrors> for CodegenError {
    fn from(source: VerifierErrors) -> Self {
        CodegenError::Verifier(source)
    }
}

/// Compilation error, with the accompanying function to help printing it.
pub struct CompileError<'a> {
    /// Underlying `CodegenError` that triggered the error.
    pub inner: CodegenError,
    /// Function we tried to compile, for display purposes.
    pub func: &'a Function,
}

// By default, have `CompileError` be displayed as the internal error, and let consumers care if
// they want to use the func field for adding details.
impl<'a> core::fmt::Debug for CompileError<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

/// A convenient alias for a `Result` that uses `CompileError` as the error type.
pub type CompileResult<'a, T> = Result<T, CompileError<'a>>;
