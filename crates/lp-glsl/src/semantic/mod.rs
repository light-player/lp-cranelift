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
pub mod type_check;
pub mod builtins;
pub mod functions;

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
pub fn analyze(shader: &TranslationUnit) -> Result<TypedShader, String> {
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
            
            if func.prototype.name.0 == "main" {
                main_func = Some(typed_func);
            } else {
                user_functions.push(typed_func);
            }
        }
    }

    let main_function = main_func.ok_or("No main() function found")?;

    Ok(TypedShader {
        main_function,
        user_functions,
        function_registry: func_registry,
    })
}

fn extract_function_signature(prototype: &glsl::syntax::FunctionPrototype) -> Result<functions::FunctionSignature, String> {
    let name = prototype.name.0.clone();
    let return_type = parse_return_type(&prototype.ty)?;
    
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

fn extract_parameter(param_decl: &glsl::syntax::FunctionParameterDeclaration) -> Result<functions::Parameter, String> {
    use glsl::syntax::FunctionParameterDeclaration;

    match param_decl {
        FunctionParameterDeclaration::Named(qualifier, decl) => {
            let ty = parse_type(&decl.ty)?;
            let name = decl.name.as_ref()
                .ok_or("Parameter must have a name")?
                .0.clone();
            
            let param_qualifier = extract_param_qualifier(qualifier);
            
            Ok(functions::Parameter {
                name,
                ty,
                qualifier: param_qualifier,
            })
        }
        FunctionParameterDeclaration::Unnamed(qualifier, ty) => {
            // Unnamed parameters (allowed in prototypes)
            let param_ty = parse_type(ty)?;
            let param_qualifier = extract_param_qualifier(qualifier);
            
            Ok(functions::Parameter {
                name: String::new(), // Empty name for unnamed params
                ty: param_ty,
                qualifier: param_qualifier,
            })
        }
    }
}

fn extract_param_qualifier(qualifier: &Option<glsl::syntax::FunctionParameterQualifier>) -> functions::ParamQualifier {
    use glsl::syntax::FunctionParameterQualifier;
    
    match qualifier {
        Some(FunctionParameterQualifier::Out) => functions::ParamQualifier::Out,
        Some(FunctionParameterQualifier::InOut) => functions::ParamQualifier::InOut,
        _ => functions::ParamQualifier::In, // Default is 'in'
    }
}

fn extract_function_body(func: &glsl::syntax::FunctionDefinition) -> Result<TypedFunction, String> {
    let sig = extract_function_signature(&func.prototype)?;
    let body = func.statement.statement_list.clone();

    Ok(TypedFunction {
        name: sig.name,
        return_type: sig.return_type,
        parameters: sig.parameters,
        body,
    })
}

fn parse_type(ty: &glsl::syntax::FullySpecifiedType) -> Result<types::Type, String> {
    use glsl::syntax::TypeSpecifierNonArray;

    match &ty.ty.ty {
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
        _ => Err(format!("Type not supported yet: {:?}", ty.ty.ty)),
    }
}

fn parse_return_type(ty: &glsl::syntax::FullySpecifiedType) -> Result<types::Type, String> {
    parse_type(ty)
}

