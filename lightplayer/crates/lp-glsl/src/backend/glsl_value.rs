//! GLSL value types for function arguments and return values

use crate::error::{ErrorCode, GlslError};
use glsl::syntax::{Expr, JumpStatement, SimpleStatement, Statement};

#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};
#[cfg(feature = "std")]
use std::{format, vec::Vec};

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
    /// Supports literals: integers, floats, booleans, vectors, and matrices
    /// Uses type checking to ensure valid literal syntax
    pub fn parse(literal_str: &str) -> Result<Self, GlslError> {
        // Wrap the literal in a minimal function to parse it
        // We'll try different return types to determine the literal type
        let wrappers = [
            format!("int main() {{ return {}; }}", literal_str),
            format!("float main() {{ return {}; }}", literal_str),
            format!("bool main() {{ return {}; }}", literal_str),
            format!("vec2 main() {{ return {}; }}", literal_str),
            format!("vec3 main() {{ return {}; }}", literal_str),
            format!("vec4 main() {{ return {}; }}", literal_str),
            format!("mat2 main() {{ return {}; }}", literal_str),
            format!("mat3 main() {{ return {}; }}", literal_str),
            format!("mat4 main() {{ return {}; }}", literal_str),
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
                        Expr::FunCall(func_ident, args, _) => {
                            // Handle vector and matrix constructors
                            if let glsl::syntax::FunIdentifier::Identifier(ident) = func_ident {
                                let type_name = ident.name.as_str();
                                match type_name {
                                    "vec2" => {
                                        if let Ok(v) = parse_vector_constructor(args, 2) {
                                            return Ok(GlslValue::Vec2([v[0], v[1]]));
                                        }
                                    }
                                    "vec3" => {
                                        if let Ok(v) = parse_vector_constructor(args, 3) {
                                            return Ok(GlslValue::Vec3([v[0], v[1], v[2]]));
                                        }
                                    }
                                    "vec4" => {
                                        if let Ok(v) = parse_vector_constructor(args, 4) {
                                            return Ok(GlslValue::Vec4([v[0], v[1], v[2], v[3]]));
                                        }
                                    }
                                    "mat2" => {
                                        if let Ok(m) = parse_matrix_constructor(args, 2) {
                                            return Ok(GlslValue::Mat2x2([
                                                [m[0][0], m[0][1]],
                                                [m[1][0], m[1][1]],
                                            ]));
                                        }
                                    }
                                    "mat3" => {
                                        if let Ok(m) = parse_matrix_constructor(args, 3) {
                                            return Ok(GlslValue::Mat3x3([
                                                [m[0][0], m[0][1], m[0][2]],
                                                [m[1][0], m[1][1], m[1][2]],
                                                [m[2][0], m[2][1], m[2][2]],
                                            ]));
                                        }
                                    }
                                    "mat4" => {
                                        if let Ok(m) = parse_matrix_constructor(args, 4) {
                                            return Ok(GlslValue::Mat4x4([
                                                [m[0][0], m[0][1], m[0][2], m[0][3]],
                                                [m[1][0], m[1][1], m[1][2], m[1][3]],
                                                [m[2][0], m[2][1], m[2][2], m[2][3]],
                                                [m[3][0], m[3][1], m[3][2], m[3][3]],
                                            ]));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            continue;
                        }
                        _ => {
                            // Not a literal or constructor, continue to next wrapper
                            continue;
                        }
                    }
                }
            }
        }

        Err(GlslError::new(
            ErrorCode::E0400,
            format!(
                "invalid literal: `{}` (must be an integer, float, boolean, vector, or matrix literal)",
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

/// Parse a vector constructor expression into a Vec of floats
/// Returns a Vec that can be converted to the appropriate array size
fn parse_vector_constructor(args: &[Expr], dim: usize) -> Result<Vec<f32>, GlslError> {
    let mut components = Vec::new();

    for arg in args {
        match arg {
            Expr::FloatConst(f, _) => components.push(*f),
            Expr::IntConst(n, _) => components.push(*n as f32),
            Expr::FunCall(func_ident, args, _) => {
                // Handle nested vector constructors (e.g., vec2(1.0, 2.0))
                if let glsl::syntax::FunIdentifier::Identifier(ident) = func_ident {
                    let type_name = ident.name.as_str();
                    if type_name.starts_with("vec") || type_name.starts_with("ivec") {
                        let nested_dim = match type_name {
                            "vec2" | "ivec2" => 2,
                            "vec3" | "ivec3" => 3,
                            "vec4" | "ivec4" => 4,
                            _ => continue,
                        };
                        let nested = parse_vector_constructor(args, nested_dim)?;
                        components.extend_from_slice(&nested);
                    }
                }
            }
            Expr::Unary(op, unary_expr, _) => {
                use glsl::syntax::UnaryOp;
                if let UnaryOp::Minus = *op {
                    match **unary_expr {
                        Expr::FloatConst(f, _) => components.push(-f),
                        Expr::IntConst(n, _) => components.push(-(n as f32)),
                        _ => {
                            return Err(GlslError::new(
                                ErrorCode::E0400,
                                "invalid vector constructor argument",
                            ));
                        }
                    }
                } else {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        "invalid vector constructor argument",
                    ));
                }
            }
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    "invalid vector constructor argument",
                ));
            }
        }
    }

    if components.len() != dim {
        return Err(GlslError::new(
            ErrorCode::E0400,
            format!(
                "vector constructor expects {} components, got {}",
                dim,
                components.len()
            ),
        ));
    }

    Ok(components)
}

/// Parse a matrix constructor expression into a matrix array
/// Matrices are constructed from column vectors (e.g., mat2(vec2(1.0, 2.0), vec2(3.0, 4.0)))
/// Returns a fixed-size array that will be converted to the appropriate matrix type
fn parse_matrix_constructor(args: &[Expr], dim: usize) -> Result<[[f32; 4]; 4], GlslError> {
    let mut matrix = [[0.0f32; 4]; 4];

    // Matrix constructors take column vectors
    if args.len() != dim {
        return Err(GlslError::new(
            ErrorCode::E0400,
            format!(
                "matrix constructor expects {} column vectors, got {}",
                dim,
                args.len()
            ),
        ));
    }

    for (col_idx, arg) in args.iter().enumerate() {
        match arg {
            Expr::FunCall(func_ident, args, _) => {
                if let glsl::syntax::FunIdentifier::Identifier(ident) = func_ident {
                    let type_name = ident.name.as_str();
                    if type_name == "vec2" || type_name == "vec3" || type_name == "vec4" {
                        let vec_dim = match type_name {
                            "vec2" => 2,
                            "vec3" => 3,
                            "vec4" => 4,
                            _ => {
                                return Err(GlslError::new(
                                    ErrorCode::E0400,
                                    "invalid matrix column type",
                                ));
                            }
                        };

                        if vec_dim != dim {
                            return Err(GlslError::new(
                                ErrorCode::E0400,
                                format!(
                                    "matrix column vector dimension mismatch: expected vec{}, got {}",
                                    dim, vec_dim
                                ),
                            ));
                        }

                        let vec_components = parse_vector_constructor(args, vec_dim)?;
                        // Store column vector components in the matrix (column-major order)
                        // matrix[row][col] = component from column vector at row position
                        for row_idx in 0..dim {
                            matrix[row_idx][col_idx] = vec_components[row_idx];
                        }
                    } else {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            "matrix constructor requires column vectors",
                        ));
                    }
                } else {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        "invalid matrix constructor argument",
                    ));
                }
            }
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    "matrix constructor requires column vectors",
                ));
            }
        }
    }

    Ok(matrix)
}
