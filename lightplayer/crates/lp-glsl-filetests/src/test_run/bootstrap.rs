//! Bootstrap code generation and type inference.

use anyhow::Result;

/// Result of bootstrap code generation with span information
pub struct BootstrapResult {
    /// The generated bootstrap source code
    pub source: String,
    /// Line number where the main() function starts (1-indexed)
    pub main_start_line: usize,
    /// Line number where the main() function ends (1-indexed, inclusive)
    pub main_end_line: usize,
}

/// Known GLSL return types that can be inferred.
const KNOWN_TYPES: &[&str] = &[
    "float", "int", "uint", "bool", "vec2", "vec3", "vec4", "ivec2", "ivec3", "ivec4", "bvec2",
    "bvec3", "bvec4", "uvec2", "uvec3", "uvec4", "mat2", "mat3", "mat4",
];

/// Generate bootstrap GLSL code that wraps the expression in a main() function.
/// Includes all GLSL code that appears before the test directive line.
pub fn generate_bootstrap(
    file_lines: &[String],
    directive_line_number: usize,
    expression_str: &str,
) -> Result<BootstrapResult> {
    // Extract all GLSL code that appears before the directive line
    let function_code = extract_code_before_directive(file_lines, directive_line_number)?;

    // Infer return type by extracting function name from expression and looking it up in source
    let return_type = infer_return_type_from_expression(&function_code, expression_str)?;

    // Count lines in the function code (for calculating main start line)
    let function_line_count = function_code.lines().count();

    // Build bootstrap with all functions + generated main()
    let mut bootstrap = function_code;
    let main_decl = format!(
        "\n\n{} main() {{\n    return {};\n}}\n",
        return_type, expression_str
    );
    bootstrap.push_str(&main_decl);

    // Calculate main function span (1-indexed)
    // Main starts after function code + 2 blank lines
    let main_start_line = function_line_count + 3;
    // Main ends after main_start + main_decl lines (subtract 1 because end is inclusive)
    let main_line_count = main_decl.lines().count();
    let main_end_line = main_start_line + main_line_count - 1;

    Ok(BootstrapResult {
        source: bootstrap,
        main_start_line,
        main_end_line,
    })
}

/// Extract all GLSL code that appears before the directive line.
/// This includes all function definitions and code up to (but not including) the directive.
fn extract_code_before_directive(
    file_lines: &[String],
    directive_line_number: usize,
) -> Result<String> {
    let mut glsl_code = String::new();

    // Extract all lines from the start up to (but not including) the directive line
    // Note: directive_line_number is 1-indexed, so we take up to directive_line_number - 1
    for line in file_lines.iter().take(directive_line_number - 1) {
        let trimmed = line.trim();

        // Skip directive lines (test, target, run directives)
        if trimmed.starts_with("// test")
            || trimmed.starts_with("// target")
            || trimmed.starts_with("// #run:")
            || trimmed.starts_with("// run:")
        {
            continue;
        }

        // Include all other lines (GLSL code)
        glsl_code.push_str(line);
        glsl_code.push('\n');
    }

    Ok(glsl_code.trim_end().to_string())
}

/// Infer the return type of an expression by looking up the called function's return type.
pub fn infer_return_type_from_expression(source: &str, expression: &str) -> Result<String> {
    // Extract function name from expression (e.g., "add_float(0.0, 0.0)" -> "add_float")
    let func_name = extract_function_name(expression)?;

    // First try: Search for function definition in source
    if let Some(return_type) = find_return_type_in_source(source, &func_name)? {
        return Ok(return_type);
    }

    // Fallback: Use heuristics based on function name or expression content
    Ok(infer_type_from_heuristics(&func_name, expression))
}

/// Extract function name from an expression string.
fn extract_function_name(expression: &str) -> Result<String> {
    expression
        .split('(')
        .next()
        .ok_or_else(|| anyhow::anyhow!("invalid expression: no opening parenthesis"))
        .map(|s| s.trim().to_string())
}

/// Search for function definition in source and extract its return type.
/// Returns an error if the return type is not in KNOWN_TYPES.
fn find_return_type_in_source(source: &str, func_name: &str) -> Result<Option<String>> {
    let pattern = format!("{}(", func_name);
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find(&pattern) {
            // Extract the part before the function name
            let before_func = &trimmed[..pos].trim();
            // The return type should be the last word before the function name
            if let Some(return_type) = before_func.split_whitespace().last() {
                // Validate it's a known type
                if KNOWN_TYPES.contains(&return_type) {
                    return Ok(Some(return_type.to_string()));
                } else {
                    // Error if we found a return type that's not in KNOWN_TYPES
                    return Err(anyhow::anyhow!(
                        "unknown return type '{}' for function '{}'. Known types: {:?}. \
                         Please add '{}' to KNOWN_TYPES in bootstrap.rs",
                        return_type,
                        func_name,
                        KNOWN_TYPES,
                        return_type
                    ));
                }
            }
        }
    }
    Ok(None)
}

/// Infer return type using heuristics based on function name or expression content.
fn infer_type_from_heuristics(func_name: &str, expression: &str) -> String {
    // Check for float indicators
    if func_name.contains("float") || expression.contains('.') {
        return "float".to_string();
    }

    // Check for uint indicators
    if func_name.contains("uint") {
        return "uint".to_string();
    }

    // Check for vector/matrix types in function name
    if func_name.contains("vec") || func_name.contains("mat") {
        // Check for boolean vectors first (bvec2, bvec3, bvec4)
        if func_name.contains("bvec2") {
            return "bvec2".to_string();
        } else if func_name.contains("bvec3") {
            return "bvec3".to_string();
        } else if func_name.contains("bvec4") {
            return "bvec4".to_string();
        }
        // Check for unsigned integer vectors (uvec2, uvec3, uvec4)
        else if func_name.contains("uvec2") {
            return "uvec2".to_string();
        } else if func_name.contains("uvec3") {
            return "uvec3".to_string();
        } else if func_name.contains("uvec4") {
            return "uvec4".to_string();
        }
        // Check for signed integer vectors (ivec2, ivec3, ivec4)
        else if func_name.contains("ivec2") {
            return "ivec2".to_string();
        } else if func_name.contains("ivec3") {
            return "ivec3".to_string();
        } else if func_name.contains("ivec4") {
            return "ivec4".to_string();
        }
        // Check for float vectors (vec2, vec3, vec4)
        else if func_name.contains("vec2") {
            return "vec2".to_string();
        } else if func_name.contains("vec3") {
            return "vec3".to_string();
        } else if func_name.contains("vec4") {
            return "vec4".to_string();
        }
        // Check for matrices
        else if func_name.contains("mat2") {
            return "mat2".to_string();
        } else if func_name.contains("mat3") {
            return "mat3".to_string();
        } else if func_name.contains("mat4") {
            return "mat4".to_string();
        }
        // Default to float for generic vec/mat without specific dimension
        return "float".to_string();
    }

    // Default to int
    "int".to_string()
}
