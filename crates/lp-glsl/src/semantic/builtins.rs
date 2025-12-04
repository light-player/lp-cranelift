//! GLSL built-in function signatures and type checking

use crate::semantic::types::Type;

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
use std::format;
#[cfg(not(feature = "std"))]
use alloc::format;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinSignature {
    pub name: &'static str,
    pub param_types: Vec<BuiltinParamType>,
    pub return_type: BuiltinReturnType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinParamType {
    GenFType,    // float, vec2, vec3, vec4
    GenIType,    // int, ivec2, ivec3, ivec4
    Float,       // scalar float only
    Int,         // scalar int only
    Vec3,        // vec3 only (for cross product)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinReturnType {
    SameAsParam(usize),  // Return type matches parameter N
    AlwaysFloat,         // Always returns float (length, dot)
    AlwaysVec3,          // Always returns vec3 (cross)
}

/// Check if a name is a built-in function
pub fn is_builtin_function(name: &str) -> bool {
    lookup_builtin(name).is_some()
}

/// Lookup built-in function signatures by name
pub fn lookup_builtin(name: &str) -> Option<Vec<BuiltinSignature>> {
    match name {
        "dot" => Some(vec![BuiltinSignature {
            name: "dot",
            param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType],
            return_type: BuiltinReturnType::AlwaysFloat,
        }]),
        
        "cross" => Some(vec![BuiltinSignature {
            name: "cross",
            param_types: vec![BuiltinParamType::Vec3, BuiltinParamType::Vec3],
            return_type: BuiltinReturnType::AlwaysVec3,
        }]),
        
        "length" => Some(vec![BuiltinSignature {
            name: "length",
            param_types: vec![BuiltinParamType::GenFType],
            return_type: BuiltinReturnType::AlwaysFloat,
        }]),
        
        "normalize" => Some(vec![BuiltinSignature {
            name: "normalize",
            param_types: vec![BuiltinParamType::GenFType],
            return_type: BuiltinReturnType::SameAsParam(0),
        }]),
        
        "distance" => Some(vec![BuiltinSignature {
            name: "distance",
            param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType],
            return_type: BuiltinReturnType::AlwaysFloat,
        }]),
        
        "min" => Some(vec![
            BuiltinSignature {
                name: "min",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "min",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::Float],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "min",
                param_types: vec![BuiltinParamType::GenIType, BuiltinParamType::GenIType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "min",
                param_types: vec![BuiltinParamType::GenIType, BuiltinParamType::Int],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
        ]),
        
        "max" => Some(vec![
            BuiltinSignature {
                name: "max",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "max",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::Float],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "max",
                param_types: vec![BuiltinParamType::GenIType, BuiltinParamType::GenIType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "max",
                param_types: vec![BuiltinParamType::GenIType, BuiltinParamType::Int],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
        ]),
        
        "clamp" => Some(vec![
            BuiltinSignature {
                name: "clamp",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType, BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "clamp",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::Float, BuiltinParamType::Float],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "clamp",
                param_types: vec![BuiltinParamType::GenIType, BuiltinParamType::GenIType, BuiltinParamType::GenIType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "clamp",
                param_types: vec![BuiltinParamType::GenIType, BuiltinParamType::Int, BuiltinParamType::Int],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
        ]),
        
        "abs" => Some(vec![
            BuiltinSignature {
                name: "abs",
                param_types: vec![BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "abs",
                param_types: vec![BuiltinParamType::GenIType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
        ]),
        
        "sqrt" => Some(vec![BuiltinSignature {
            name: "sqrt",
            param_types: vec![BuiltinParamType::GenFType],
            return_type: BuiltinReturnType::SameAsParam(0),
        }]),
        
        "floor" => Some(vec![BuiltinSignature {
            name: "floor",
            param_types: vec![BuiltinParamType::GenFType],
            return_type: BuiltinReturnType::SameAsParam(0),
        }]),
        
        "ceil" => Some(vec![BuiltinSignature {
            name: "ceil",
            param_types: vec![BuiltinParamType::GenFType],
            return_type: BuiltinReturnType::SameAsParam(0),
        }]),
        
        "pow" => Some(vec![BuiltinSignature {
            name: "pow",
            param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType],
            return_type: BuiltinReturnType::SameAsParam(0),
        }]),
        
        "mix" => Some(vec![
            BuiltinSignature {
                name: "mix",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType, BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "mix",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType, BuiltinParamType::Float],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
        ]),
        
        "step" => Some(vec![
            BuiltinSignature {
                name: "step",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "step",
                param_types: vec![BuiltinParamType::Float, BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(1),
            },
        ]),
        
        "smoothstep" => Some(vec![
            BuiltinSignature {
                name: "smoothstep",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType, BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "smoothstep",
                param_types: vec![BuiltinParamType::Float, BuiltinParamType::Float, BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(2),
            },
        ]),
        
        "fract" => Some(vec![BuiltinSignature {
            name: "fract",
            param_types: vec![BuiltinParamType::GenFType],
            return_type: BuiltinReturnType::SameAsParam(0),
        }]),
        
        "mod" => Some(vec![
            BuiltinSignature {
                name: "mod",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "mod",
                param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::Float],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
        ]),
        
        "sign" => Some(vec![
            BuiltinSignature {
                name: "sign",
                param_types: vec![BuiltinParamType::GenFType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
            BuiltinSignature {
                name: "sign",
                param_types: vec![BuiltinParamType::GenIType],
                return_type: BuiltinReturnType::SameAsParam(0),
            },
        ]),
        
        _ => None,
    }
}

/// Type check a built-in function call and return the result type
pub fn check_builtin_call(
    name: &str,
    arg_types: &[Type],
) -> Result<Type, String> {
    let signatures = lookup_builtin(name)
        .ok_or_else(|| format!("Unknown built-in function: {}", name))?;
    
    // Try each overload
    for sig in &signatures {
        if let Ok(return_type) = try_match_signature(sig, arg_types) {
            return Ok(return_type);
        }
    }
    
    Err(format!(
        "No matching overload for {}({:?})",
        name, arg_types
    ))
}

/// Try to match a function signature with provided argument types
fn try_match_signature(
    sig: &BuiltinSignature,
    arg_types: &[Type],
) -> Result<Type, String> {
    // Check argument count
    if arg_types.len() != sig.param_types.len() {
        return Err("Argument count mismatch".to_string());
    }

    // Check each parameter type
    for (i, (param_ty, arg_ty)) in sig.param_types.iter().zip(arg_types).enumerate() {
        if !matches_param_type(param_ty, arg_ty) {
            return Err(format!("Parameter {} type mismatch", i));
        }
    }

    // Validate genType consistency (all genFType params must have same size)
    validate_gentype_consistency(&sig.param_types, arg_types)?;

    // Compute return type
    let return_type = match sig.return_type {
        BuiltinReturnType::SameAsParam(idx) => arg_types[idx].clone(),
        BuiltinReturnType::AlwaysFloat => Type::Float,
        BuiltinReturnType::AlwaysVec3 => Type::Vec3,
    };

    Ok(return_type)
}

/// Check if an argument type matches a parameter type specification
fn matches_param_type(param: &BuiltinParamType, arg: &Type) -> bool {
    match param {
        BuiltinParamType::GenFType => matches!(arg,
            Type::Float | Type::Vec2 | Type::Vec3 | Type::Vec4
        ),
        BuiltinParamType::GenIType => matches!(arg,
            Type::Int | Type::IVec2 | Type::IVec3 | Type::IVec4
        ),
        BuiltinParamType::Float => arg == &Type::Float,
        BuiltinParamType::Int => arg == &Type::Int,
        BuiltinParamType::Vec3 => arg == &Type::Vec3,
    }
}

/// Validate that all GenFType/GenIType parameters have consistent sizes
fn validate_gentype_consistency(
    param_types: &[BuiltinParamType],
    arg_types: &[Type],
) -> Result<(), String> {
    // Find the first genType parameter to establish the expected type
    let mut expected_type: Option<Type> = None;

    for (param, arg) in param_types.iter().zip(arg_types) {
        match param {
            BuiltinParamType::GenFType | BuiltinParamType::GenIType => {
                if let Some(ref expected) = expected_type {
                    if arg != expected {
                        return Err(format!(
                            "GenType parameter type mismatch: expected {:?}, got {:?}",
                            expected, arg
                        ));
                    }
                } else {
                    expected_type = Some(arg.clone());
                }
            }
            _ => {}
        }
    }

    Ok(())
}

