//! Error handling for GLSL compilation.
//!
//! This module provides structured error types with source locations,
//! error codes, and helpful diagnostics inspired by Rust's error reporting.

#![allow(dead_code)] // Allow during development

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

use core::fmt;

/// Source location in GLSL code.
///
/// Tracks line and column information for error reporting.
/// Gracefully degrades when location information is unavailable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    /// Line number (1-indexed). 0 means unknown.
    pub line: usize,
    /// Column number (1-indexed). 0 means unknown.
    pub column: usize,
    /// Optional filename for multi-file support.
    pub filename: Option<String>,
}

impl SourceLocation {
    /// Create a new source location.
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            filename: None,
        }
    }

    /// Create a source location with a filename.
    pub fn with_file(line: usize, column: usize, filename: String) -> Self {
        Self {
            line,
            column,
            filename: Some(filename),
        }
    }

    /// Create an unknown location (for errors without source context).
    pub fn unknown() -> Self {
        Self {
            line: 0,
            column: 0,
            filename: None,
        }
    }

    /// Check if this location is unknown.
    pub fn is_unknown(&self) -> bool {
        self.line == 0 && self.column == 0
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_unknown() {
            write!(f, "<unknown>")
        } else if let Some(ref filename) = self.filename {
            write!(f, "{}:{}:{}", filename, self.line, self.column)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}

/// Error codes for GLSL compilation errors.
///
/// Organized by category:
/// - E0001-E0099: Parse errors
/// - E0100-E0299: Semantic/type errors
/// - E0300-E0399: Transform errors
/// - E0400-E0499: Codegen errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    // Parse errors (E0001-E0099)
    /// Parse error from external parser
    E0001,

    // Semantic errors (E0100-E0299)
    /// Undefined variable
    E0100,
    /// Undefined function
    E0101,
    /// Type mismatch
    E0102,
    /// Cannot implicitly convert types
    E0103,
    /// Wrong number of arguments
    E0104,
    /// Wrong argument type
    E0105,
    /// Incompatible types for operator
    E0106,
    /// Condition must be bool type
    E0107,
    /// No main() function found
    E0108,
    /// Type not supported
    E0109,
    /// Invalid vector constructor
    E0110,
    /// Component access out of range
    E0111,
    /// Cannot access component on non-vector
    E0112,
    /// Invalid swizzle
    E0113,
    /// No matching function overload
    E0114,
    /// Cannot assign to expression
    E0115,
    /// Return type mismatch
    E0116,

    // Transform errors (E0300-E0399)
    /// Fixed-point transformation error
    E0300,
    /// Verification failed after transformation
    E0301,

    // Codegen errors (E0400-E0499)
    /// Codegen error
    E0400,
    /// Verification error
    E0401,
}

impl ErrorCode {
    /// Get the error code as a string (e.g., "E0100").
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::E0001 => "E0001",
            ErrorCode::E0100 => "E0100",
            ErrorCode::E0101 => "E0101",
            ErrorCode::E0102 => "E0102",
            ErrorCode::E0103 => "E0103",
            ErrorCode::E0104 => "E0104",
            ErrorCode::E0105 => "E0105",
            ErrorCode::E0106 => "E0106",
            ErrorCode::E0107 => "E0107",
            ErrorCode::E0108 => "E0108",
            ErrorCode::E0109 => "E0109",
            ErrorCode::E0110 => "E0110",
            ErrorCode::E0111 => "E0111",
            ErrorCode::E0112 => "E0112",
            ErrorCode::E0113 => "E0113",
            ErrorCode::E0114 => "E0114",
            ErrorCode::E0115 => "E0115",
            ErrorCode::E0116 => "E0116",
            ErrorCode::E0300 => "E0300",
            ErrorCode::E0301 => "E0301",
            ErrorCode::E0400 => "E0400",
            ErrorCode::E0401 => "E0401",
        }
    }

    /// Get a short description of the error.
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::E0001 => "parse error",
            ErrorCode::E0100 => "undefined variable",
            ErrorCode::E0101 => "undefined function",
            ErrorCode::E0102 => "type mismatch",
            ErrorCode::E0103 => "cannot implicitly convert",
            ErrorCode::E0104 => "wrong argument count",
            ErrorCode::E0105 => "wrong argument type",
            ErrorCode::E0106 => "incompatible types for operator",
            ErrorCode::E0107 => "condition must be bool",
            ErrorCode::E0108 => "no main function",
            ErrorCode::E0109 => "unsupported type",
            ErrorCode::E0110 => "invalid vector constructor",
            ErrorCode::E0111 => "component out of range",
            ErrorCode::E0112 => "invalid component access",
            ErrorCode::E0113 => "invalid swizzle",
            ErrorCode::E0114 => "no matching function",
            ErrorCode::E0115 => "cannot assign",
            ErrorCode::E0116 => "return type mismatch",
            ErrorCode::E0300 => "transformation error",
            ErrorCode::E0301 => "verification failed",
            ErrorCode::E0400 => "codegen error",
            ErrorCode::E0401 => "verification error",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A structured GLSL compilation error.
///
/// Provides detailed error information including:
/// - Error code (e.g., E0100)
/// - Primary error message
/// - Optional source location
/// - Optional source text snippet
/// - Additional notes/hints
/// - Optional spec reference (for future use)
#[derive(Clone, Debug)]
pub struct GlslError {
    /// Error code
    pub code: ErrorCode,
    /// Primary error message
    pub message: String,
    /// Source location where error occurred
    pub location: Option<SourceLocation>,
    /// The actual source line (if available)
    pub span_text: Option<String>,
    /// Additional notes/hints
    pub notes: Vec<String>,
    /// Optional reference to GLSL spec (for future expansion)
    pub spec_ref: Option<String>,
}

impl GlslError {
    /// Create a new error with the given code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
            span_text: None,
            notes: Vec::new(),
            spec_ref: None,
        }
    }

    /// Add a source location to this error.
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add source text to this error.
    pub fn with_span_text(mut self, text: impl Into<String>) -> Self {
        self.span_text = Some(text.into());
        self
    }

    /// Add a note/hint to this error.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a spec reference to this error.
    pub fn with_spec_ref(mut self, spec_ref: impl Into<String>) -> Self {
        self.spec_ref = Some(spec_ref.into());
        self
    }

    /// Convert to a simple string for backward compatibility.
    pub fn to_simple_string(&self) -> String {
        if let Some(ref loc) = self.location {
            format!("{}: {}", loc, self.message)
        } else {
            self.message.clone()
        }
    }
}

impl fmt::Display for GlslError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Format: error[E0100]: message
        write!(f, "error[{}]: {}", self.code, self.message)?;

        // Add location if available
        if let Some(ref loc) = self.location {
            if !loc.is_unknown() {
                write!(f, "\n --> {}", loc)?;
            }
        }

        // Add source line if available
        if let Some(ref text) = self.span_text {
            writeln!(f, "\n  |")?;
            if let Some(ref loc) = self.location {
                if !loc.is_unknown() {
                    writeln!(f, "{:>3} | {}", loc.line, text)?;
                    // Add caret indicator
                    if loc.column > 0 {
                        write!(f, "  | ")?;
                        for _ in 1..loc.column {
                            write!(f, " ")?;
                        }
                        writeln!(f, "^^^ {}", self.code.description())?;
                    }
                }
            } else {
                writeln!(f, "  | {}", text)?;
            }
            write!(f, "  |")?;
        }

        // Add notes
        for note in &self.notes {
            write!(f, "\n  = note: {}", note)?;
        }

        // Add spec reference
        if let Some(ref spec_ref) = self.spec_ref {
            write!(f, "\n  = spec: {}", spec_ref)?;
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GlslError {}

// Convenience constructors for common errors

impl GlslError {
    /// Create an undefined variable error.
    pub fn undefined_variable(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::new(
            ErrorCode::E0100,
            format!("undefined variable `{}`", name),
        )
    }

    /// Create an undefined function error.
    pub fn undefined_function(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::new(
            ErrorCode::E0101,
            format!("undefined function `{}`", name),
        )
    }

    /// Create a type mismatch error.
    pub fn type_mismatch(expected: impl fmt::Debug, found: impl fmt::Debug) -> Self {
        Self::new(
            ErrorCode::E0102,
            format!("type mismatch: expected `{:?}`, found `{:?}`", expected, found),
        )
    }

    /// Create a parse error.
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::E0001, message)
    }

    /// Create a no main function error.
    pub fn no_main_function() -> Self {
        Self::new(ErrorCode::E0108, "no `main()` function found")
            .with_note("GLSL shaders must have a main() function")
    }

    /// Create an unsupported type error.
    pub fn unsupported_type(type_name: impl Into<String>) -> Self {
        let type_name = type_name.into();
        Self::new(
            ErrorCode::E0109,
            format!("type `{}` is not supported", type_name),
        )
    }
}

// Span extraction helpers

/// Convert glsl::syntax::SourceSpan to our SourceLocation
pub fn source_span_to_location(span: &glsl::syntax::SourceSpan) -> SourceLocation {
    SourceLocation::new(span.line, span.column)
}

// Re-export SourceSpan for convenience
pub use glsl::syntax::SourceSpan;

/// Extract span from an expression
pub fn extract_span_from_expr(expr: &glsl::syntax::Expr) -> glsl::syntax::SourceSpan {
    use glsl::syntax::Expr;
    match expr {
        Expr::Variable(_, span) => span.clone(),
        Expr::IntConst(_, span) => span.clone(),
        Expr::UIntConst(_, span) => span.clone(),
        Expr::FloatConst(_, span) => span.clone(),
        Expr::DoubleConst(_, span) => span.clone(),
        Expr::BoolConst(_, span) => span.clone(),
        Expr::Unary(_, _, span) => span.clone(),
        Expr::Binary(_, _, _, span) => span.clone(),
        Expr::Ternary(_, _, _, span) => span.clone(),
        Expr::Assignment(_, _, _, span) => span.clone(),
        Expr::Bracket(_, _, span) => span.clone(),
        Expr::FunCall(_, _, span) => span.clone(),
        Expr::Dot(_, _, span) => span.clone(),
        Expr::PostInc(_, span) => span.clone(),
        Expr::PostDec(_, span) => span.clone(),
        Expr::Comma(_, _, span) => span.clone(),
    }
}

/// Extract span from an identifier
pub fn extract_span_from_identifier(ident: &glsl::syntax::Identifier) -> glsl::syntax::SourceSpan {
    ident.span.clone()
}

/// Extract source line text from a span
pub fn extract_source_line(source: &str, span: &glsl::syntax::SourceSpan) -> Option<String> {
    if span.is_unknown() {
        return None;
    }
    source.lines().nth(span.line.saturating_sub(1)).map(|s| s.into())
}

/// Helper to add span_text to an error if source is available
pub fn add_span_text_to_error(
    mut error: GlslError,
    source: Option<&str>,
    span: &glsl::syntax::SourceSpan,
) -> GlslError {
    if let Some(source_text) = source {
        if let Some(span_text) = extract_source_line(source_text, span) {
            error = error.with_span_text(span_text);
        }
    }
    error
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new(5, 10);
        assert_eq!(loc.to_string(), "5:10");

        let loc = SourceLocation::with_file(5, 10, "shader.glsl".into());
        assert_eq!(loc.to_string(), "shader.glsl:5:10");

        let loc = SourceLocation::unknown();
        assert_eq!(loc.to_string(), "<unknown>");
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::E0100.as_str(), "E0100");
        assert_eq!(ErrorCode::E0100.description(), "undefined variable");
    }

    #[test]
    fn test_glsl_error_simple() {
        let err = GlslError::undefined_variable("foo");
        assert_eq!(err.code, ErrorCode::E0100);
        assert!(err.message.contains("foo"));
    }

    #[test]
    fn test_glsl_error_with_location() {
        let err = GlslError::undefined_variable("foo")
            .with_location(SourceLocation::new(5, 10));
        
        let display = err.to_string();
        assert!(display.contains("E0100"));
        assert!(display.contains("5:10"));
    }

    #[test]
    fn test_glsl_error_with_note() {
        let err = GlslError::undefined_variable("foo")
            .with_note("did you mean `bar`?");
        
        assert_eq!(err.notes.len(), 1);
        let display = err.to_string();
        assert!(display.contains("note:"));
    }
}
