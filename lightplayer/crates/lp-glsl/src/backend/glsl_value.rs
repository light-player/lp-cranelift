//! GLSL value types for function arguments and return values

use crate::error::{ErrorCode, GlslError};
use glsl::syntax::{Expr, JumpStatement, SimpleStatement, Statement};

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

/// GLSL value types for function arguments
#[derive(Debug, Clone)]
pub enum GlslValue {
    I32(i32),
    F32(f32),
    Bool(bool),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat2x2([[f32; 2]; 2]),
    Mat3x3([[f32; 3]; 3]),
    Mat4x4([[f32; 4]; 4]),
}

impl GlslValue {
    /// Parse a literal value string into GlslValue using GLSL parser
    /// Only supports literals: integers, floats, booleans
    /// Uses type checking to ensure valid literal syntax
    pub fn parse(literal_str: &str) -> Result<Self, GlslError> {
        // Wrap the literal in a minimal function to parse it
        // We'll try different return types to determine the literal type
        let wrappers = [
            format!("int main() {{ return {}; }}", literal_str),
            format!("float main() {{ return {}; }}", literal_str),
            format!("bool main() {{ return {}; }}", literal_str),
        ];

        for wrapper in &wrappers {
            if let Ok(shader) = glsl::parser::Parse::parse(wrapper) {
                // Extract the return statement expression
                if let Some(expr) = extract_return_expression(&shader) {
                    match expr {
                        Expr::IntConst(n, _) => {
                            return Ok(GlslValue::I32(*n));
                        }
                        Expr::FloatConst(f, _) => {
                            return Ok(GlslValue::F32(*f));
                        }
                        Expr::BoolConst(b, _) => {
                            return Ok(GlslValue::Bool(*b));
                        }
                        Expr::Unary(op, unary_expr, _) => {
                            // Handle unary minus for negative numbers
                            use glsl::syntax::UnaryOp;
                            if let UnaryOp::Minus = *op {
                                match **unary_expr {
                                    Expr::IntConst(n, _) => {
                                        return Ok(GlslValue::I32(-n));
                                    }
                                    Expr::FloatConst(f, _) => {
                                        return Ok(GlslValue::F32(-f));
                                    }
                                    _ => {
                                        // Not a negated literal, continue
                                        continue;
                                    }
                                }
                            } else {
                                // Not a minus operator, continue
                                continue;
                            }
                        }
                        _ => {
                            // Not a literal, continue to next wrapper
                            continue;
                        }
                    }
                }
            }
        }

        Err(GlslError::new(
            ErrorCode::E0400,
            format!(
                "invalid literal: `{}` (must be an integer, float, or boolean literal)",
                literal_str
            ),
        ))
    }

    /// Exact equality comparison (==)
    /// For integers and booleans: exact match required
    /// For floats: exact match required (use `approx_eq` for tolerance-based comparison)
    /// For vectors/matrices: exact match for all components
    pub fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (GlslValue::I32(a), GlslValue::I32(b)) => a == b,
            (GlslValue::F32(a), GlslValue::F32(b)) => a == b, // Exact equality
            (GlslValue::Bool(a), GlslValue::Bool(b)) => a == b,
            (GlslValue::Vec2(a), GlslValue::Vec2(b)) => a == b,
            (GlslValue::Vec3(a), GlslValue::Vec3(b)) => a == b,
            (GlslValue::Vec4(a), GlslValue::Vec4(b)) => a == b,
            (GlslValue::Mat2x2(a), GlslValue::Mat2x2(b)) => a == b,
            (GlslValue::Mat3x3(a), GlslValue::Mat3x3(b)) => a == b,
            (GlslValue::Mat4x4(a), GlslValue::Mat4x4(b)) => a == b,
            _ => false, // Type mismatch
        }
    }

    /// Approximate equality comparison (~=) with tolerance
    /// For floats: checks if values are within tolerance
    /// For integers and booleans: falls back to exact equality
    /// For vectors/matrices: checks each component within tolerance
    pub fn approx_eq(&self, other: &Self, tolerance: f32) -> bool {
        match (self, other) {
            (GlslValue::I32(a), GlslValue::I32(b)) => a == b, // Exact for ints
            (GlslValue::F32(a), GlslValue::F32(b)) => (a - b).abs() <= tolerance,
            (GlslValue::Bool(a), GlslValue::Bool(b)) => a == b, // Exact for bools
            (GlslValue::Vec2(a), GlslValue::Vec2(b)) => a
                .iter()
                .zip(b.iter())
                .all(|(x, y)| (x - y).abs() <= tolerance),
            (GlslValue::Vec3(a), GlslValue::Vec3(b)) => a
                .iter()
                .zip(b.iter())
                .all(|(x, y)| (x - y).abs() <= tolerance),
            (GlslValue::Vec4(a), GlslValue::Vec4(b)) => a
                .iter()
                .zip(b.iter())
                .all(|(x, y)| (x - y).abs() <= tolerance),
            (GlslValue::Mat2x2(a), GlslValue::Mat2x2(b)) => a
                .iter()
                .flatten()
                .zip(b.iter().flatten())
                .all(|(x, y)| (x - y).abs() <= tolerance),
            (GlslValue::Mat3x3(a), GlslValue::Mat3x3(b)) => a
                .iter()
                .flatten()
                .zip(b.iter().flatten())
                .all(|(x, y)| (x - y).abs() <= tolerance),
            (GlslValue::Mat4x4(a), GlslValue::Mat4x4(b)) => a
                .iter()
                .flatten()
                .zip(b.iter().flatten())
                .all(|(x, y)| (x - y).abs() <= tolerance),
            _ => false, // Type mismatch
        }
    }

    /// Default tolerance for float comparisons (1e-4)
    pub const DEFAULT_TOLERANCE: f32 = 1e-4;

    /// Approximate equality with default tolerance
    pub fn approx_eq_default(&self, other: &Self) -> bool {
        self.approx_eq(other, Self::DEFAULT_TOLERANCE)
    }
}

/// Extract the return expression from a parsed shader
/// Assumes the shader has a single function with a single return statement
fn extract_return_expression(shader: &glsl::syntax::TranslationUnit) -> Option<&Expr> {
    for decl in &shader.0 {
        if let glsl::syntax::ExternalDeclaration::FunctionDefinition(func) = decl {
            // The function body is a CompoundStatement with a statement_list
            for stmt in &func.statement.statement_list {
                if let Statement::Simple(simple_stmt) = stmt {
                    if let SimpleStatement::Jump(JumpStatement::Return(Some(ref expr))) =
                        **simple_stmt
                    {
                        return Some(expr);
                    }
                }
            }
        }
    }
    None
}
