//! Type inference and validation for GLSL expressions
//! Implements GLSL spec type rules for Phase 3

pub mod inference;
pub mod conversion;
pub mod constructors;
pub mod operators;
pub mod matrix;
pub mod swizzle;

// Re-export public API
pub use inference::{infer_expr_type, infer_expr_type_with_registry, infer_expr_type_in_context};
pub use conversion::{promote_numeric, can_implicitly_convert, check_assignment, check_assignment_with_span};
pub use constructors::{
    check_vector_constructor, check_vector_constructor_with_span,
    check_matrix_constructor,
    is_vector_type_name, is_matrix_type_name,
};
pub use operators::{infer_binary_result_type, infer_unary_result_type, check_condition};
pub use matrix::infer_matrix_binary_result_type;
pub use swizzle::parse_swizzle_length;

