//! Compilation pipeline for GLSL shaders.
//!
//! This module provides a unified compilation pipeline that can be used
//! by different backends (JIT, code generation, CLIF output).

use crate::error::{ErrorCode, GlslError, extract_source_line, source_span_to_location};
use crate::semantic::TypedShader;

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

/// Result of parsing GLSL source
pub struct ParseResult<'a> {
    pub shader: glsl::syntax::TranslationUnit,
    pub source: &'a str,
}

/// Result of semantic analysis
pub struct SemanticResult<'a> {
    pub typed_ast: TypedShader,
    pub source: &'a str,
}

/// Common compilation pipeline steps
pub struct CompilationPipeline;

impl CompilationPipeline {
    /// Parse GLSL source into an AST
    pub fn parse<'a>(source: &'a str) -> Result<ParseResult<'a>, GlslError> {
        let shader = glsl::parser::Parse::parse(source).map_err(|e| {
            let mut error = GlslError::new(ErrorCode::E0001, format!("parse error: {:?}", e));
            // Try to extract span from parse error if available
            if let Some(ref span) = e.span {
                error = error.with_location(source_span_to_location(span));
                if let Some(span_text) = extract_source_line(source, span) {
                    error = error.with_span_text(span_text);
                }
            }
            error
        })?;

        Ok(ParseResult {
            shader,
            source,
        })
    }

    /// Perform semantic analysis on parsed shader
    pub fn analyze<'a>(parse_result: ParseResult<'a>) -> Result<SemanticResult<'a>, GlslError> {
        let typed_ast = crate::semantic::analyze_with_source(&parse_result.shader, parse_result.source)?;

        Ok(SemanticResult {
            typed_ast,
            source: parse_result.source,
        })
    }

    /// Parse and analyze in one step
    pub fn parse_and_analyze<'a>(source: &'a str) -> Result<SemanticResult<'a>, GlslError> {
        let parse_result = Self::parse(source)?;
        Self::analyze(parse_result)
    }
}

