//! Dependency analysis for intrinsic functions.
//!
//! This module provides utilities to analyze function call dependencies
//! in intrinsic GLSL files and compute transitive closures.

use crate::error::GlslError;
use crate::frontend::pipeline::CompilationPipeline;
use alloc::{boxed::Box, string::String, vec::Vec};
use hashbrown::{HashMap, HashSet};

/// Dependency graph for intrinsic functions.
/// Maps function name to set of functions it calls.
pub type DependencyGraph = HashMap<String, HashSet<String>>;

/// Build a dependency graph from GLSL source.
///
/// Parses the GLSL source and extracts all function definitions and their
/// call dependencies. Returns a map from function name to set of called functions.
pub fn build_dependency_graph(glsl_source: &str) -> Result<DependencyGraph, GlslError> {
    // Parse GLSL to AST
    let parse_result = CompilationPipeline::parse(glsl_source)?;
    let ast = &parse_result.shader;

    // Extract all function definitions
    let mut function_defs: HashMap<String, &glsl::syntax::FunctionDefinition> = HashMap::new();
    let mut call_graph: DependencyGraph = HashMap::new();

    for decl in &ast.0 {
        if let glsl::syntax::ExternalDeclaration::FunctionDefinition(func) = decl {
            let func_name = func.prototype.name.name.clone();
            function_defs.insert(func_name.clone(), func);
            call_graph.insert(func_name.clone(), HashSet::new());
        }
    }

    // Build call graph by traversing function bodies
    for (func_name, func_def) in &function_defs {
        // Function body is a CompoundStatement
        let stmt = glsl::syntax::Statement::Compound(Box::new(func_def.statement.clone()));
        let callees = extract_function_calls(&stmt);
        if let Some(calls) = call_graph.get_mut(func_name) {
            *calls = callees;
        }
    }

    Ok(call_graph)
}

/// Compute transitive closure of dependencies for a function.
///
/// Given a dependency graph and a starting function name, returns all functions
/// that are transitively called (including the function itself).
pub fn compute_transitive_closure(
    graph: &DependencyGraph,
    start_func: &str,
) -> Result<HashSet<String>, GlslError> {
    let mut visited = HashSet::new();
    let mut queue = Vec::new();
    queue.push(String::from(start_func));

    while let Some(func) = queue.pop() {
        if visited.contains(&func) {
            continue;
        }
        visited.insert(func.clone());

        // Add all dependencies to queue
        if let Some(deps) = graph.get(&func) {
            for dep in deps {
                if !visited.contains(dep) {
                    queue.push(dep.clone());
                }
            }
        }
    }

    Ok(visited)
}

/// Recursively extract all function calls from a statement.
fn extract_function_calls(stmt: &glsl::syntax::Statement) -> HashSet<String> {
    let mut calls = HashSet::new();
    extract_function_calls_from_stmt(stmt, &mut calls);
    calls
}

/// Recursively traverse statements to find function calls.
fn extract_function_calls_from_stmt(stmt: &glsl::syntax::Statement, calls: &mut HashSet<String>) {
    match stmt {
        glsl::syntax::Statement::Simple(simple) => {
            extract_function_calls_from_simple_stmt(simple, calls);
        }
        glsl::syntax::Statement::Compound(compound) => {
            extract_function_calls_from_compound(compound, calls);
        }
    }
}

/// Extract function calls from a simple statement.
fn extract_function_calls_from_simple_stmt(
    stmt: &glsl::syntax::SimpleStatement,
    calls: &mut HashSet<String>,
) {
    match stmt {
        glsl::syntax::SimpleStatement::Expression(Some(expr)) => {
            extract_function_calls_from_expr(expr, calls);
        }
        glsl::syntax::SimpleStatement::Selection(selection) => {
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
        glsl::syntax::SimpleStatement::Iteration(iteration) => match iteration {
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
        glsl::syntax::SimpleStatement::Jump(jump) => {
            if let glsl::syntax::JumpStatement::Return(Some(expr)) = jump {
                extract_function_calls_from_expr(expr, calls);
            }
        }
        glsl::syntax::SimpleStatement::Declaration(decl) => {
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
fn extract_function_calls_from_compound(
    compound: &glsl::syntax::CompoundStatement,
    calls: &mut HashSet<String>,
) {
    for stmt in &compound.statement_list {
        extract_function_calls_from_stmt(stmt, calls);
    }
}

/// Recursively extract function calls from an expression.
fn extract_function_calls_from_expr(expr: &glsl::syntax::Expr, calls: &mut HashSet<String>) {
    match expr {
        glsl::syntax::Expr::FunCall(func_ident, args, _) => {
            // Extract function name
            if let glsl::syntax::FunIdentifier::Identifier(ident) = func_ident {
                let func_name = ident.name.clone();
                // Only track intrinsic functions (those starting with __lp_)
                // or other user-defined functions, but skip built-ins
                if func_name.starts_with("__lp_") || !is_builtin_or_constructor(&func_name) {
                    calls.insert(func_name);
                }
            }
            // Also check arguments for nested function calls
            for arg in args {
                extract_function_calls_from_expr(arg, calls);
            }
        }
        glsl::syntax::Expr::Binary(_, left, right, _) => {
            extract_function_calls_from_expr(left, calls);
            extract_function_calls_from_expr(right, calls);
        }
        glsl::syntax::Expr::Unary(_, operand, _) => {
            extract_function_calls_from_expr(operand, calls);
        }
        glsl::syntax::Expr::Ternary(cond, then_expr, else_expr, _) => {
            extract_function_calls_from_expr(cond, calls);
            extract_function_calls_from_expr(then_expr, calls);
            extract_function_calls_from_expr(else_expr, calls);
        }
        glsl::syntax::Expr::Assignment(_, _lvalue, rvalue, _) => {
            extract_function_calls_from_expr(rvalue, calls);
        }
        glsl::syntax::Expr::Bracket(expr, _, _) => {
            extract_function_calls_from_expr(expr, calls);
        }
        glsl::syntax::Expr::Dot(expr, _, _) => {
            extract_function_calls_from_expr(expr, calls);
        }
        glsl::syntax::Expr::PostInc(expr, _) | glsl::syntax::Expr::PostDec(expr, _) => {
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

/// Extract function calls from a for loop initialization.
fn extract_function_calls_from_for_init(
    init: &glsl::syntax::ForInitStatement,
    calls: &mut HashSet<String>,
) {
    match init {
        glsl::syntax::ForInitStatement::Expression(Some(expr)) => {
            extract_function_calls_from_expr(expr, calls);
        }
        glsl::syntax::ForInitStatement::Declaration(decl) => match decl.as_ref() {
            glsl::syntax::Declaration::InitDeclaratorList(list) => {
                if let Some(init) = &list.head.initializer {
                    extract_function_calls_from_initializer(init, calls);
                }
            }
            _ => {}
        },
        glsl::syntax::ForInitStatement::Expression(None) => {}
    }
}

/// Extract function calls from a for loop rest (condition and increment).
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

/// Extract function calls from a condition expression.
fn extract_function_calls_from_condition(
    cond: &glsl::syntax::Condition,
    calls: &mut HashSet<String>,
) {
    match cond {
        glsl::syntax::Condition::Expr(expr) => {
            extract_function_calls_from_expr(expr, calls);
        }
        glsl::syntax::Condition::Assignment(_type_spec, _identifier, initializer) => {
            extract_function_calls_from_initializer(initializer, calls);
        }
    }
}

/// Check if a function name is a built-in or type constructor.
fn is_builtin_or_constructor(name: &str) -> bool {
    // GLSL built-in functions and type constructors
    // This is a simplified heuristic - we track __lp_ functions and skip obvious built-ins
    matches!(
        name,
        "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "sinh"
            | "cosh"
            | "tanh"
            | "asinh"
            | "acosh"
            | "atanh"
            | "exp"
            | "log"
            | "exp2"
            | "log2"
            | "pow"
            | "sqrt"
            | "abs"
            | "mod"
            | "min"
            | "max"
            | "clamp"
            | "mix"
            | "step"
            | "smoothstep"
            | "floor"
            | "ceil"
            | "fract"
            | "sign"
            | "length"
            | "distance"
            | "dot"
            | "cross"
            | "normalize"
            | "reflect"
            | "refract"
            | "transpose"
            | "determinant"
            | "inverse"
            | "vec2"
            | "vec3"
            | "vec4"
            | "mat2"
            | "mat3"
            | "mat4"
            | "float"
            | "int"
            | "bool"
    )
}
