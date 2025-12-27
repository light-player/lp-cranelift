//! Function call graph filtering for bootstrap code generation.
//!
//! This module provides utilities to extract only the functions needed for a test
//! by tracing the call graph from a starting function.

use anyhow::Result;
use glsl::syntax::{CompoundStatement, Expr, SimpleStatement, Statement};
use std::collections::{HashMap, HashSet, VecDeque};

/// Extract GLSL source containing only the specified function and its call graph dependencies.
///
/// Given a parsed GLSL AST and source string, this function:
/// 1. Builds a call graph by traversing all function bodies
/// 2. Finds all functions reachable from the starting function
/// 3. Extracts and returns only those function definitions from the source
///
/// # Arguments
/// * `ast` - The parsed GLSL translation unit
/// * `source` - The original GLSL source code
/// * `fn_name` - The name of the function to start the call graph traversal from
///
/// # Returns
/// Filtered GLSL source containing only the reachable functions
pub fn glsl_for_fn_graph(
    ast: &glsl::syntax::TranslationUnit,
    _source: &str,
    fn_name: &str,
) -> Result<String> {
    // Step 1: Extract all function definitions and build call graph
    let mut function_defs: HashMap<String, &glsl::syntax::FunctionDefinition> = HashMap::new();
    let mut call_graph: HashMap<String, HashSet<String>> = HashMap::new();

    for decl in &ast.0 {
        if let glsl::syntax::ExternalDeclaration::FunctionDefinition(func) = decl {
            let func_name = func.prototype.name.name.clone();
            function_defs.insert(func_name.clone(), func);
            call_graph.insert(func_name.clone(), HashSet::new());
        }
    }

    // Step 2: Build call graph by traversing function bodies
    for (func_name, func_def) in &function_defs {
        // Function body is a CompoundStatement, wrap it in Statement::Compound
        let stmt = glsl::syntax::Statement::Compound(Box::new(func_def.statement.clone()));
        let callees = extract_function_calls(&stmt);
        if let Some(calls) = call_graph.get_mut(func_name) {
            *calls = callees;
        }
    }

    // Step 3: Find all reachable functions from the starting function
    let reachable = find_reachable_functions(fn_name, &call_graph, &function_defs);

    // Step 4: Extract function definitions from AST using transpiler
    extract_functions_from_source_using_ast(ast, &function_defs, &reachable)
}

/// Recursively extract all function calls from a statement.
fn extract_function_calls(stmt: &Statement) -> HashSet<String> {
    let mut calls = HashSet::new();
    extract_function_calls_from_stmt(stmt, &mut calls);
    calls
}

/// Recursively traverse statements to find function calls.
fn extract_function_calls_from_stmt(stmt: &Statement, calls: &mut HashSet<String>) {
    match stmt {
        Statement::Simple(simple) => {
            extract_function_calls_from_simple_stmt(simple, calls);
        }
        Statement::Compound(compound) => {
            extract_function_calls_from_compound(compound, calls);
        }
    }
}

/// Extract function calls from a simple statement.
fn extract_function_calls_from_simple_stmt(stmt: &SimpleStatement, calls: &mut HashSet<String>) {
    match stmt {
        SimpleStatement::Expression(Some(expr)) => {
            extract_function_calls_from_expr(expr, calls);
        }
        SimpleStatement::Selection(selection) => {
            extract_function_calls_from_expr(&selection.cond, calls);
            match &selection.rest {
                glsl::syntax::SelectionRestStatement::Statement(then_stmt) => {
                    extract_function_calls_from_stmt(then_stmt, calls);
                }
                glsl::syntax::SelectionRestStatement::Else(then_stmt, else_stmt) => {
                    extract_function_calls_from_stmt(then_stmt, calls);
                    extract_function_calls_from_stmt(else_stmt, calls);
                }
            }
        }
        SimpleStatement::Iteration(iteration) => match iteration {
            glsl::syntax::IterationStatement::While(condition, body) => {
                extract_function_calls_from_condition(condition, calls);
                extract_function_calls_from_stmt(body, calls);
            }
            glsl::syntax::IterationStatement::DoWhile(body, expr) => {
                extract_function_calls_from_stmt(body, calls);
                extract_function_calls_from_expr(expr, calls);
            }
            glsl::syntax::IterationStatement::For(init, rest, body) => {
                extract_function_calls_from_for_init(init, calls);
                extract_function_calls_from_for_rest(rest, calls);
                extract_function_calls_from_stmt(body, calls);
            }
        },
        SimpleStatement::Jump(jump) => {
            if let glsl::syntax::JumpStatement::Return(Some(expr)) = jump {
                extract_function_calls_from_expr(expr, calls);
            }
        }
        SimpleStatement::Declaration(decl) => {
            // Check for initializers in declarations
            // decl is Box<Declaration>, dereference it
            match decl {
                glsl::syntax::Declaration::InitDeclaratorList(list) => {
                    if let Some(init) = &list.head.initializer {
                        extract_function_calls_from_initializer(init, calls);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// Extract function calls from a compound statement.
fn extract_function_calls_from_compound(compound: &CompoundStatement, calls: &mut HashSet<String>) {
    for stmt in &compound.statement_list {
        extract_function_calls_from_stmt(stmt, calls);
    }
}

/// Recursively extract function calls from an expression.
fn extract_function_calls_from_expr(expr: &Expr, calls: &mut HashSet<String>) {
    match expr {
        Expr::FunCall(func_ident, args, _) => {
            // Extract function name
            if let glsl::syntax::FunIdentifier::Identifier(ident) = func_ident {
                let func_name = ident.name.clone();
                // Filter out built-ins and type constructors (simple heuristic)
                if !is_builtin_or_constructor(&func_name) {
                    calls.insert(func_name);
                }
            }
            // Also check arguments for nested function calls
            for arg in args {
                extract_function_calls_from_expr(arg, calls);
            }
        }
        Expr::Binary(_, left, right, _) => {
            extract_function_calls_from_expr(left, calls);
            extract_function_calls_from_expr(right, calls);
        }
        Expr::Unary(_, operand, _) => {
            extract_function_calls_from_expr(operand, calls);
        }
        Expr::Ternary(cond, then_expr, else_expr, _) => {
            extract_function_calls_from_expr(cond, calls);
            extract_function_calls_from_expr(then_expr, calls);
            extract_function_calls_from_expr(else_expr, calls);
        }
        Expr::Assignment(_, _lvalue, rvalue, _) => {
            extract_function_calls_from_expr(rvalue, calls);
        }
        Expr::Bracket(expr, _, _) => {
            extract_function_calls_from_expr(expr, calls);
            // ArraySpecifier doesn't contain function calls, skip it
        }
        Expr::Dot(expr, _, _) => {
            extract_function_calls_from_expr(expr, calls);
        }
        Expr::PostInc(expr, _) | Expr::PostDec(expr, _) => {
            extract_function_calls_from_expr(expr, calls);
        }
        _ => {}
    }
}

/// Extract function calls from an initializer.
fn extract_function_calls_from_initializer(
    init: &glsl::syntax::Initializer,
    calls: &mut HashSet<String>,
) {
    match init {
        glsl::syntax::Initializer::Simple(expr) => {
            extract_function_calls_from_expr(expr, calls);
        }
        glsl::syntax::Initializer::List(list) => {
            for item in list {
                extract_function_calls_from_initializer(item, calls);
            }
        }
    }
}

/// Extract function calls from a condition.
fn extract_function_calls_from_condition(
    condition: &glsl::syntax::Condition,
    calls: &mut HashSet<String>,
) {
    match condition {
        glsl::syntax::Condition::Expr(expr) => {
            extract_function_calls_from_expr(expr, calls);
        }
        glsl::syntax::Condition::Assignment(_, _, initializer) => {
            extract_function_calls_from_initializer(initializer, calls);
        }
    }
}

/// Extract function calls from a for init statement.
fn extract_function_calls_from_for_init(
    init: &glsl::syntax::ForInitStatement,
    calls: &mut HashSet<String>,
) {
    match init {
        glsl::syntax::ForInitStatement::Expression(Some(expr)) => {
            extract_function_calls_from_expr(expr, calls);
        }
        glsl::syntax::ForInitStatement::Declaration(decl) => {
            // decl is Box<Declaration>, dereference it
            match &**decl {
                glsl::syntax::Declaration::InitDeclaratorList(list) => {
                    if let Some(init) = &list.head.initializer {
                        extract_function_calls_from_initializer(init, calls);
                    }
                }
                _ => {}
            }
        }
        glsl::syntax::ForInitStatement::Expression(None) => {}
    }
}

/// Extract function calls from a for rest statement.
fn extract_function_calls_from_for_rest(
    rest: &glsl::syntax::ForRestStatement,
    calls: &mut HashSet<String>,
) {
    if let Some(ref cond) = rest.condition {
        extract_function_calls_from_condition(cond, calls);
    }
    if let Some(ref post_expr) = rest.post_expr {
        extract_function_calls_from_expr(post_expr, calls);
    }
}

/// Check if a function name is a built-in or type constructor.
fn is_builtin_or_constructor(name: &str) -> bool {
    // Type constructors
    matches!(
        name,
        "vec2"
            | "vec3"
            | "vec4"
            | "ivec2"
            | "ivec3"
            | "ivec4"
            | "uvec2"
            | "uvec3"
            | "uvec4"
            | "bvec2"
            | "bvec3"
            | "bvec4"
            | "mat2"
            | "mat3"
            | "mat4"
            | "float"
            | "int"
            | "uint"
            | "bool"
    ) || name.starts_with("gl_") // Built-in GLSL functions/variables
}

/// Find all functions reachable from a starting function using BFS.
fn find_reachable_functions(
    start_func: &str,
    call_graph: &HashMap<String, HashSet<String>>,
    function_defs: &HashMap<String, &glsl::syntax::FunctionDefinition>,
) -> HashSet<String> {
    let mut reachable = HashSet::new();

    // If the starting function doesn't exist, return empty set
    if !function_defs.contains_key(start_func) {
        return reachable;
    }

    let mut queue = VecDeque::new();
    queue.push_back(start_func.to_string());
    reachable.insert(start_func.to_string());

    while let Some(current) = queue.pop_front() {
        if let Some(callees) = call_graph.get(&current) {
            for callee in callees {
                if !reachable.contains(callee) && function_defs.contains_key(callee) {
                    reachable.insert(callee.clone());
                    queue.push_back(callee.clone());
                }
            }
        }
    }

    reachable
}

/// Extract function definitions from AST using the transpiler.
/// This converts AST nodes back to GLSL source.
fn extract_functions_from_source_using_ast(
    ast: &glsl::syntax::TranslationUnit,
    _function_defs: &HashMap<String, &glsl::syntax::FunctionDefinition>,
    function_names: &HashSet<String>,
) -> Result<String> {
    if function_names.is_empty() {
        return Ok(String::new());
    }

    // Collect function definitions in order they appear in the AST
    let mut functions_to_include: Vec<&glsl::syntax::FunctionDefinition> = Vec::new();
    let mut seen = HashSet::new();

    // First, collect all functions we need in the order they appear
    for decl in &ast.0 {
        if let glsl::syntax::ExternalDeclaration::FunctionDefinition(func) = decl {
            let func_name = func.prototype.name.name.clone();
            if function_names.contains(&func_name) && !seen.contains(&func_name) {
                functions_to_include.push(func);
                seen.insert(func_name);
            }
        }
    }

    // Convert each function definition back to GLSL source using the transpiler
    let mut result = String::new();
    for func in functions_to_include {
        glsl::transpiler::glsl::show_function_definition(&mut result, func);
        result.push_str("\n\n");
    }

    Ok(result.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lp_glsl::frontend::CompilationPipeline;

    #[test]
    fn test_simple_call_graph() {
        let source = r#"
float add(float a, float b) {
    return a + b;
}

float multiply(float a, float b) {
    return a * b;
}

float test() {
    return multiply(add(1.0, 2.0), 3.0);
}
"#;

        let parse_result = CompilationPipeline::parse(source).unwrap();
        let filtered = glsl_for_fn_graph(&parse_result.shader, source, "test").unwrap();

        // Should include test, multiply, and add
        assert!(filtered.contains("test"));
        assert!(filtered.contains("multiply"));
        assert!(filtered.contains("add"));
    }

    #[test]
    fn test_nested_calls() {
        let source = r#"
float a() { return 1.0; }
float b() { return a() + 1.0; }
float c() { return b() + 1.0; }
float d() { return 4.0; }
float test() { return c(); }
"#;

        let parse_result = CompilationPipeline::parse(source).unwrap();
        let filtered = glsl_for_fn_graph(&parse_result.shader, source, "test").unwrap();

        // Should include test, c, b, a (but not d)
        assert!(filtered.contains("test"));
        assert!(filtered.contains("c"));
        assert!(filtered.contains("b"));
        assert!(filtered.contains("a"));
        assert!(!filtered.contains("d"));
    }

    #[test]
    fn test_missing_function() {
        let source = r#"
float test() { return 1.0; }
"#;

        let parse_result = CompilationPipeline::parse(source).unwrap();
        let filtered = glsl_for_fn_graph(&parse_result.shader, source, "nonexistent").unwrap();

        // Should return empty or just the function itself
        assert!(filtered.is_empty() || !filtered.contains("test"));
    }
}
