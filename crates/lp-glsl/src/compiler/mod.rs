//! GLSL compilation logic.
//!
//! This module contains the core compilation components that transform GLSL source
//! into Cranelift IR, including parsing, semantic analysis, code generation, and linking.

pub(crate) mod glsl_compiler;
pub(crate) mod link;
pub(crate) mod pipeline;

// Re-exports used by crate root; suppress unused warnings within this module.
#[allow(unused_imports)]
pub use glsl_compiler::GlslCompiler;
#[allow(unused_imports)]
pub use link::rebuild_function_for_module;
#[allow(unused_imports)]
pub use pipeline::{
    Backend, CompilationPipeline, CompiledShader, ParseResult, SemanticResult, TransformationPass,
};

// Re-export create_minimal_module_for_declarations for internal use
#[allow(unused_imports)]
pub(crate) use glsl_compiler::create_minimal_module_for_declarations;
