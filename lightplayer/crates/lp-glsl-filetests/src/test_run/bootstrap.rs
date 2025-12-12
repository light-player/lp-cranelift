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
    "float", "int", "bool", "vec2", "vec3", "vec4", "ivec2", "ivec3", "ivec4", "mat2", "mat3",
    "mat4",
];

/// Generate bootstrap GLSL code that wraps the expression in a main() function.
/// Only includes the specific function being tested, not the entire source file.
pub fn generate_bootstrap(original_source: &str, expression_str: &str) -> Result<BootstrapResult> {
    // Extract function name from expression (e.g., "divide_float(5.0, 1.0)" -> "divide_float")
    let func_name = extract_function_name(expression_str)?;

    // Extract only the function being tested from the original source
    let function_code = extract_function_definition(original_source, &func_name)?;

    // Infer return type by extracting function name from expression and looking it up in source
    let return_type = infer_return_type_from_expression(original_source, expression_str)?;

    // Count lines in the function code (for calculating main start line)
    let function_line_count = function_code.lines().count();
    
    // Build bootstrap with just the function + generated main()
    let mut bootstrap = function_code;
    let main_decl = format!("\n\n{} main() {{\n    return {};\n}}\n", return_type, expression_str);
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

/// Extract a specific function definition from GLSL source.
fn extract_function_definition(source: &str, func_name: &str) -> Result<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut function_lines = Vec::new();
    let mut in_target_function = false;
    let mut brace_depth = 0;
    let pattern = format!("{}(", func_name);

    for line in lines.iter() {
        let trimmed = line.trim();

        // Check if this line starts the target function
        if !in_target_function && trimmed.contains(&pattern) {
            in_target_function = true;
            brace_depth = 0;
            function_lines.push(*line);
            // Count opening braces on this line
            brace_depth += line.matches('{').count();
            brace_depth -= line.matches('}').count();
            continue;
        }

        if in_target_function {
            function_lines.push(*line);
            brace_depth += line.matches('{').count();
            brace_depth -= line.matches('}').count();

            // If we've closed all braces, we're done with the function
            if brace_depth == 0 {
                break;
            }
        }
    }

    if function_lines.is_empty() {
        return Err(anyhow::anyhow!(
            "Function '{}' not found in source",
            func_name
        ));
    }

    Ok(function_lines.join("\n"))
}

/// Infer the return type of an expression by looking up the called function's return type.
pub fn infer_return_type_from_expression(source: &str, expression: &str) -> Result<String> {
    // Extract function name from expression (e.g., "add_float(0.0, 0.0)" -> "add_float")
    let func_name = extract_function_name(expression)?;

    // First try: Search for function definition in source
    if let Some(return_type) = find_return_type_in_source(source, &func_name) {
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
fn find_return_type_in_source(source: &str, func_name: &str) -> Option<String> {
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
                    return Some(return_type.to_string());
                }
            }
        }
    }
    None
}

/// Infer return type using heuristics based on function name or expression content.
fn infer_type_from_heuristics(func_name: &str, expression: &str) -> String {
    // Check for float indicators
    if func_name.contains("float") || expression.contains('.') {
        return "float".to_string();
    }

    // Check for vector/matrix types in function name
    if func_name.contains("vec") || func_name.contains("mat") {
        if func_name.contains("vec2") || func_name.contains("ivec2") {
            return "vec2".to_string();
        } else if func_name.contains("vec3") || func_name.contains("ivec3") {
            return "vec3".to_string();
        } else if func_name.contains("vec4") || func_name.contains("ivec4") {
            return "vec4".to_string();
        } else if func_name.contains("mat2") {
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
