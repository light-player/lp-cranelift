use glsl::syntax::TranslationUnit;
use crate::error::GlslError;

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::String;


pub mod scope;
pub mod types;
pub mod type_check;
pub mod builtins;
pub mod functions;
pub mod validator;

pub struct TypedShader {
    pub main_function: TypedFunction,
    pub user_functions: Vec<TypedFunction>,
    pub function_registry: functions::FunctionRegistry,
}

pub struct TypedFunction {
    pub name: String,
    pub return_type: types::Type,
    pub parameters: Vec<functions::Parameter>,
    pub body: Vec<glsl::syntax::Statement>,
}

/// Analyze GLSL shader and produce typed AST
pub fn analyze(shader: &TranslationUnit) -> Result<TypedShader, GlslError> {
    analyze_with_source(shader, "")
}

/// Analyze GLSL shader with source text for better error messages
pub fn analyze_with_source(shader: &TranslationUnit, source: &str) -> Result<TypedShader, GlslError> {
    let mut func_registry = functions::FunctionRegistry::new();
    let mut main_func: Option<TypedFunction> = None;
    let mut user_functions: Vec<TypedFunction> = Vec::new();

    // First pass: register all function signatures
    for decl in &shader.0 {
        if let glsl::syntax::ExternalDeclaration::FunctionDefinition(func) = decl {
            let sig = extract_function_signature(&func.prototype)?;
            func_registry.register_function(sig)?;
        }
    }

    // Second pass: extract function bodies
    for decl in &shader.0 {
        if let glsl::syntax::ExternalDeclaration::FunctionDefinition(func) = decl {
            let typed_func = extract_function_body(func)?;
            
            if func.prototype.name.name == "main" {
                main_func = Some(typed_func);
            } else {
                user_functions.push(typed_func);
            }
        }
    }

    let main_function = main_func.ok_or_else(|| {
        GlslError::no_main_function()
    })?;

    // Third pass: validate all function bodies
    for func in &user_functions {
        validator::validate_function(func, &func_registry, source)?;
    }
    validator::validate_function(&main_function, &func_registry, source)?;

    Ok(TypedShader {
        main_function,
        user_functions,
        function_registry: func_registry,
    })
}

fn extract_function_signature(prototype: &glsl::syntax::FunctionPrototype) -> Result<functions::FunctionSignature, GlslError> {
    let name = prototype.name.name.clone();
    // Extract span from function name for error reporting (fallback to type location)
    let type_span = prototype.name.span.clone();
    let return_type = parse_return_type(&prototype.ty, Some(type_span))?;
    
    let mut parameters = Vec::new();
    for param_decl in &prototype.parameters {
        let param = extract_parameter(param_decl)?;
        parameters.push(param);
    }

    Ok(functions::FunctionSignature {
        name,
        return_type,
        parameters,
    })
}

fn extract_parameter(param_decl: &glsl::syntax::FunctionParameterDeclaration) -> Result<functions::Parameter, GlslError> {
    use glsl::syntax::FunctionParameterDeclaration;

    match param_decl {
        FunctionParameterDeclaration::Named(qualifier, decl) => {
            let param_span = decl.ident.ident.span.clone();
            let ty = parse_type_specifier(&decl.ty, Some(param_span))?;
            let name = decl.ident.ident.name.clone();
            
            let param_qualifier = extract_param_qualifier(qualifier);
            
            Ok(functions::Parameter {
                name,
                ty,
                qualifier: param_qualifier,
            })
        }
        FunctionParameterDeclaration::Unnamed(qualifier, ty) => {
            // Unnamed parameters (allowed in prototypes)
            // For unnamed params, we don't have a good span, so pass None
            let param_ty = parse_type_specifier(ty, None)?;
            let param_qualifier = extract_param_qualifier(qualifier);
            
            Ok(functions::Parameter {
                name: String::new(), // Empty name for unnamed params
                ty: param_ty,
                qualifier: param_qualifier,
            })
        }
    }
}

fn extract_param_qualifier(qualifier: &Option<glsl::syntax::TypeQualifier>) -> functions::ParamQualifier {
    use glsl::syntax::{TypeQualifierSpec, StorageQualifier};
    
    if let Some(type_qual) = qualifier {
        for spec in &type_qual.qualifiers.0 {
            if let TypeQualifierSpec::Storage(storage) = spec {
                return match storage {
                    StorageQualifier::Out => functions::ParamQualifier::Out,
                    StorageQualifier::InOut => functions::ParamQualifier::InOut,
                    StorageQualifier::In => functions::ParamQualifier::In,
                    _ => functions::ParamQualifier::In, // Default for other storage qualifiers
                };
            }
        }
    }
    
    // Default is 'in'
    functions::ParamQualifier::In
}

fn extract_function_body(func: &glsl::syntax::FunctionDefinition) -> Result<TypedFunction, GlslError> {
    let sig = extract_function_signature(&func.prototype)?;
    let body = func.statement.statement_list.clone();

    Ok(TypedFunction {
        name: sig.name,
        return_type: sig.return_type,
        parameters: sig.parameters,
        body,
    })
}

fn parse_type_specifier(ty: &glsl::syntax::TypeSpecifier, span: Option<glsl::syntax::SourceSpan>) -> Result<types::Type, GlslError> {
    use glsl::syntax::TypeSpecifierNonArray;
    use crate::error::source_span_to_location;

    match &ty.ty {
        TypeSpecifierNonArray::Void => Ok(types::Type::Void),
        TypeSpecifierNonArray::Bool => Ok(types::Type::Bool),
        TypeSpecifierNonArray::Int => Ok(types::Type::Int),
        TypeSpecifierNonArray::Float => Ok(types::Type::Float),
        TypeSpecifierNonArray::Vec2 => Ok(types::Type::Vec2),
        TypeSpecifierNonArray::Vec3 => Ok(types::Type::Vec3),
        TypeSpecifierNonArray::Vec4 => Ok(types::Type::Vec4),
        TypeSpecifierNonArray::IVec2 => Ok(types::Type::IVec2),
        TypeSpecifierNonArray::IVec3 => Ok(types::Type::IVec3),
        TypeSpecifierNonArray::IVec4 => Ok(types::Type::IVec4),
        TypeSpecifierNonArray::BVec2 => Ok(types::Type::BVec2),
        TypeSpecifierNonArray::BVec3 => Ok(types::Type::BVec3),
        TypeSpecifierNonArray::BVec4 => Ok(types::Type::BVec4),
        TypeSpecifierNonArray::Mat2 => Ok(types::Type::Mat2),
        TypeSpecifierNonArray::Mat3 => Ok(types::Type::Mat3),
        TypeSpecifierNonArray::Mat4 => Ok(types::Type::Mat4),
        _ => {
            let mut error = GlslError::unsupported_type(format!("{:?}", ty.ty));
            if let Some(s) = span {
                error = error.with_location(source_span_to_location(&s));
            }
            Err(error)
        }
    }
}

fn parse_return_type(ty: &glsl::syntax::FullySpecifiedType, span: Option<glsl::syntax::SourceSpan>) -> Result<types::Type, GlslError> {
    parse_type_specifier(&ty.ty, span)
}

