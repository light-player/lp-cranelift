use glsl::syntax::TranslationUnit;

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
use std::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

pub mod scope;
pub mod types;

pub struct TypedShader {
    pub main_function: TypedFunction,
}

pub struct TypedFunction {
    pub return_type: types::Type,
    pub body: Vec<glsl::syntax::Statement>,
    pub return_expr: Option<Box<glsl::syntax::Expr>>,
}

/// Analyze GLSL shader and produce typed AST
pub fn analyze(shader: &TranslationUnit) -> Result<TypedShader, String> {
    // Phase 1: Just extract main() function and basic validation
    // TODO: Full semantic analysis in later implementation

    // Find main function
    for decl in &shader.0 {
        if let glsl::syntax::ExternalDeclaration::FunctionDefinition(func) = decl {
            if func.prototype.name.0 == "main" {
                // Extract body statements and return expression
                let (body, return_expr) = extract_function_body(&func.statement);

                // Determine return type from function prototype
                let return_type = parse_return_type(&func.prototype.ty)?;

                return Ok(TypedShader {
                    main_function: TypedFunction {
                        return_type,
                        body,
                        return_expr,
                    },
                });
            }
        }
    }

    Err("No main() function found".to_string())
}

fn extract_function_body(
    compound: &glsl::syntax::CompoundStatement,
) -> (Vec<glsl::syntax::Statement>, Option<Box<glsl::syntax::Expr>>) {
    use glsl::syntax::Statement;

    let mut body = Vec::new();
    let mut return_expr = None;

    for stmt in &compound.statement_list {
        // Check if this is a return statement
        if let Statement::Simple(simple) = stmt {
            if let glsl::syntax::SimpleStatement::Jump(jump) = simple.as_ref() {
                if let glsl::syntax::JumpStatement::Return(expr) = jump {
                    return_expr = expr.clone();
                    continue;
                }
            }
        }
        body.push(stmt.clone());
    }

    (body, return_expr)
}

fn parse_return_type(ty: &glsl::syntax::FullySpecifiedType) -> Result<types::Type, String> {
    use glsl::syntax::TypeSpecifierNonArray;

    match &ty.ty.ty {
        TypeSpecifierNonArray::Void => Ok(types::Type::Void),
        TypeSpecifierNonArray::Bool => Ok(types::Type::Bool),
        TypeSpecifierNonArray::Int => Ok(types::Type::Int),
        _ => Err(format!("Return type not supported in Phase 1: {:?}", ty.ty.ty)),
    }
}

