use glsl::syntax::TranslationUnit;

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};


pub mod scope;
pub mod types;

pub struct TypedShader {
    pub main_function: TypedFunction,
}

pub struct TypedFunction {
    pub return_type: types::Type,
    pub body: Vec<glsl::syntax::Statement>,
}

/// Analyze GLSL shader and produce typed AST
pub fn analyze(shader: &TranslationUnit) -> Result<TypedShader, String> {
    // Phase 1: Just extract main() function and basic validation
    // TODO: Full semantic analysis in later implementation

    // Find main function
    for decl in &shader.0 {
        if let glsl::syntax::ExternalDeclaration::FunctionDefinition(func) = decl {
            if func.prototype.name.0 == "main" {
                // Determine return type from function prototype
                let return_type = parse_return_type(&func.prototype.ty)?;

                // Get the body statements
                let body = func.statement.statement_list.clone();

                return Ok(TypedShader {
                    main_function: TypedFunction {
                        return_type,
                        body,
                    },
                });
            }
        }
    }

    Err("No main() function found".to_string())
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

