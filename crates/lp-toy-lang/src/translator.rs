//! IR translator for the toy language.

use std::collections::BTreeMap;

use lpc_lpir::{
    BlockEntity, Function, FunctionBuilder, FunctionBuilderContext, Signature, Type, Value,
    Variable,
};

use crate::frontend::{parser, Expr};

/// Translator for the toy language.
///
/// This translates toy language functions to LPIR using the new SSABuilder API.
pub struct Translator {
    /// Reusable context for function building.
    /// This preserves memory allocations between function compilations.
    context: FunctionBuilderContext,
}

impl Translator {
    /// Create a new translator.
    pub fn new() -> Self {
        Self {
            context: FunctionBuilderContext::new(),
        }
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

impl Translator {
    /// Compile a string in the toy language into LPIR Function.
    pub fn compile(&mut self, input: &str) -> Result<lpc_lpir::Function, String> {
        // Parse the string, producing AST nodes.
        let (name, params, the_return, stmts) =
            parser::function(input).map_err(|e| e.to_string())?;

        // Translate the AST nodes into LPIR.
        self.translate(name, params, the_return, stmts)
    }

    /// Translate from toy-language AST nodes into LPIR.
    fn translate(
        &mut self,
        name: String,
        params: Vec<String>,
        the_return: String,
        stmts: Vec<Expr>,
    ) -> Result<lpc_lpir::Function, String> {
        // Our toy language currently only supports I32 values.
        let int = Type::I32;

        // Create function signature
        let param_types: Vec<Type> = (0..params.len()).map(|_| int).collect();
        let return_types = vec![int];
        let signature = Signature::new(param_types, return_types);

        // Create the function
        let mut func = Function::new(signature, name);

        // Create the function builder with context
        let mut builder = FunctionBuilder::new(&mut func, &mut self.context);

        // Create the entry block (Cranelift-style pattern)
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        // Ensure entry block is in layout before sealing
        // This is required because seal_block builds a CFG that needs blocks in layout
        builder.ensure_inserted_block();

        // Seal the entry block first (matching reference: line 152)
        // Since it's the entry block, it won't have any predecessors
        builder.seal_block(entry_block);

        // Get parameter values from block params
        let entry_params: Vec<Value> = builder.block_params(entry_block).to_vec();

        // Declare all variables (parameters and implicitly declared)
        // We're already in entry_block, and seal_block doesn't change the current block
        let variables = declare_variables(
            int,
            &mut builder,
            &params,
            &the_return,
            &stmts,
            entry_block,
            &entry_params,
        );

        // Now translate the statements of the function body.
        // Match reference: current block should still be entry_block
        // (reference line 166-167 just iterates and calls translate_expr)
        let mut trans = FunctionTranslator {
            int,
            builder,
            variables,
        };

        // Translate statements
        // We're already in entry_block after declare_variables, so no need to switch
        // Control flow structures will handle their own block switching
        for expr in stmts {
            trans.translate_expr(expr)?;
            // After each statement, we may be in a different block (e.g., merge block from if-else)
            // That's fine - we'll continue from there
        }

        // Get the current block for the return instruction
        // This could be entry_block or a merge/exit block from control flow
        // We're already in the correct block after translating statements, no need to switch

        // Set up the return value
        let return_variable = trans
            .variables
            .get(&the_return)
            .ok_or_else(|| format!("Return variable '{}' not found", the_return))?;
        let return_value = trans.builder.use_var(*return_variable);

        // Emit the return instruction in the current block
        trans.builder.ins().return_(&[return_value]);

        // Finish building
        trans.builder.finalize();

        // Return the function
        Ok(func)
    }
}

/// A collection of state used for translating from toy-language AST nodes
/// into LPIR.
struct FunctionTranslator<'a> {
    int: Type,
    builder: FunctionBuilder<'a>,
    variables: BTreeMap<String, Variable>,
}

impl<'a> FunctionTranslator<'a> {
    /// Translate an expression and return its value.
    fn translate_expr(&mut self, expr: Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal(literal) => {
                let imm: i32 = literal
                    .parse()
                    .map_err(|e| format!("Invalid literal: {}", e))?;
                // Reference doesn't worry about block context - just emit instruction
                Ok(self.builder.ins().iconst(self.int, imm as i64))
            }

            Expr::Add(lhs, rhs) => {
                let lhs_val = self.translate_expr(*lhs)?;
                let rhs_val = self.translate_expr(*rhs)?;
                // Reference doesn't worry about block context - just emit instruction
                Ok(self.builder.ins().iadd(lhs_val, rhs_val))
            }

            Expr::Sub(lhs, rhs) => {
                let lhs_val = self.translate_expr(*lhs)?;
                let rhs_val = self.translate_expr(*rhs)?;
                // Reference doesn't worry about block context - just emit instruction
                Ok(self.builder.ins().isub(lhs_val, rhs_val))
            }

            Expr::Mul(lhs, rhs) => {
                let lhs_val = self.translate_expr(*lhs)?;
                let rhs_val = self.translate_expr(*rhs)?;
                // Reference doesn't worry about block context - just emit instruction
                Ok(self.builder.ins().imul(lhs_val, rhs_val))
            }

            Expr::Div(lhs, rhs) => {
                let lhs_val = self.translate_expr(*lhs)?;
                let rhs_val = self.translate_expr(*rhs)?;
                // Reference doesn't worry about block context - just emit instruction
                Ok(self.builder.ins().idiv(lhs_val, rhs_val))
            }

            Expr::Eq(lhs, rhs) => self.translate_icmp(lpc_lpir::IntCC::Equal, *lhs, *rhs),
            Expr::Ne(lhs, rhs) => self.translate_icmp(lpc_lpir::IntCC::NotEqual, *lhs, *rhs),
            Expr::Lt(lhs, rhs) => self.translate_icmp(lpc_lpir::IntCC::SignedLessThan, *lhs, *rhs),
            Expr::Le(lhs, rhs) => {
                self.translate_icmp(lpc_lpir::IntCC::SignedLessThanOrEqual, *lhs, *rhs)
            }
            Expr::Gt(lhs, rhs) => {
                self.translate_icmp(lpc_lpir::IntCC::SignedGreaterThan, *lhs, *rhs)
            }
            Expr::Ge(lhs, rhs) => {
                self.translate_icmp(lpc_lpir::IntCC::SignedGreaterThanOrEqual, *lhs, *rhs)
            }
            Expr::Call(_name, _args) => Err("Function calls not yet supported".to_string()),
            Expr::GlobalDataAddr(_name) => Err("Global data not yet supported".to_string()),
            Expr::Identifier(name) => {
                // `use_var` is used to read the value of a variable.
                // Match reference: just call use_var directly (no block switching needed)
                let variable = self
                    .variables
                    .get(&name)
                    .ok_or_else(|| format!("Variable '{}' not defined", name))?;
                Ok(self.builder.use_var(*variable))
            }
            Expr::Assign(name, expr) => self.translate_assign(name, *expr),
            Expr::IfElse(condition, then_body, else_body) => {
                self.translate_if_else(*condition, then_body, else_body)
            }
            Expr::WhileLoop(condition, loop_body) => {
                self.translate_while_loop(*condition, loop_body)
            }
        }
    }

    fn translate_assign(&mut self, name: String, expr: Expr) -> Result<Value, String> {
        let new_value = self.translate_expr(expr)?;
        let variable = self
            .variables
            .get(&name)
            .ok_or_else(|| format!("Variable '{}' not defined", name))?;
        // Reference doesn't worry about block context - just define variable
        self.builder.def_var(*variable, new_value);
        Ok(new_value)
    }

    fn translate_icmp(
        &mut self,
        cmp: lpc_lpir::IntCC,
        lhs: Expr,
        rhs: Expr,
    ) -> Result<Value, String> {
        let lhs_val = self.translate_expr(lhs)?;
        let rhs_val = self.translate_expr(rhs)?;
        // Reference doesn't worry about block context - just emit instruction
        Ok(self.builder.ins().icmp(cmp, lhs_val, rhs_val))
    }

    fn translate_if_else(
        &mut self,
        condition: Expr,
        then_body: Vec<Expr>,
        else_body: Vec<Expr>,
    ) -> Result<Value, String> {
        // Translate condition in current block
        let condition_value = self.translate_expr(condition)?;

        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        let merge_block = self.builder.create_block();

        // Test the if condition and conditionally branch.
        self.builder
            .ins()
            .brif(condition_value, then_block, &[], else_block, &[]);

        // Generate then body
        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);
        for expr in then_body {
            self.translate_expr(expr)?;
        }

        // Jump to the merge block (no arguments - variables will create phi nodes as needed)
        self.builder.ins().jump(merge_block, &[]);

        // Generate else body
        self.builder.switch_to_block(else_block);
        self.builder.seal_block(else_block);
        for expr in else_body {
            self.translate_expr(expr)?;
        }

        // Jump to the merge block (no arguments - variables will create phi nodes as needed)
        self.builder.ins().jump(merge_block, &[]);

        // Switch to the merge block for subsequent statements.
        self.builder.switch_to_block(merge_block);

        // We've now seen all the predecessors of the merge block.
        self.builder.seal_block(merge_block);

        // If-else used as statement doesn't return a value - return 0
        Ok(self.builder.ins().iconst(self.int, 0))
    }

    fn translate_while_loop(
        &mut self,
        condition: Expr,
        loop_body: Vec<Expr>,
    ) -> Result<Value, String> {
        let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit_block = self.builder.create_block();

        // Jump to header from current block
        self.builder.ins().jump(header_block, &[]);
        self.builder.switch_to_block(header_block);

        // Generate condition in header
        let condition_value = self.translate_expr(condition)?;
        self.builder
            .ins()
            .brif(condition_value, body_block, &[], exit_block, &[]);

        // Generate body
        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);

        for expr in loop_body {
            self.translate_expr(expr)?;
        }
        // Jump back to header
        self.builder.ins().jump(header_block, &[]);

        // Switch to exit block
        self.builder.switch_to_block(exit_block);

        // We've reached the bottom of the loop, so there will be no
        // more backedges to the header to exits to the bottom.
        self.builder.seal_block(header_block);
        self.builder.seal_block(exit_block);

        // Just return 0 for now.
        Ok(self.builder.ins().iconst(self.int, 0))
    }
}

fn declare_variables(
    int: Type,
    builder: &mut FunctionBuilder,
    params: &[String],
    the_return: &str,
    stmts: &[Expr],
    _entry_block: BlockEntity,
    entry_params: &[Value],
) -> BTreeMap<String, Variable> {
    let mut variables = BTreeMap::new();

    // Declare parameter variables
    // Match reference (line 407-413): rely on caller having set current block to entry_block
    // The reference doesn't switch blocks - it just uses builder.block_params and def_var
    for (i, name) in params.iter().enumerate() {
        let var = builder.declare_var(int);
        variables.insert(name.clone(), var);
        // Define variable with parameter value (current block should be entry_block)
        builder.def_var(var, entry_params[i]);
    }

    // Declare return variable with initial value 0
    let zero = builder.ins().iconst(int, 0);
    let return_variable = builder.declare_var(int);
    variables.insert(the_return.to_string(), return_variable);
    builder.def_var(return_variable, zero);

    // Declare variables used in statements
    for expr in stmts {
        declare_variables_in_stmt(int, builder, &mut variables, expr);
    }

    variables
}

/// Recursively descend through the AST, translating all implicit
/// variable declarations.
fn declare_variables_in_stmt(
    int: Type,
    builder: &mut FunctionBuilder,
    variables: &mut BTreeMap<String, Variable>,
    expr: &Expr,
) {
    match expr {
        Expr::Assign(name, _) => {
            if !variables.contains_key(name) {
                let var = builder.declare_var(int);
                variables.insert(name.clone(), var);
            }
        }
        Expr::IfElse(_condition, then_body, else_body) => {
            for stmt in then_body {
                declare_variables_in_stmt(int, builder, variables, stmt);
            }
            for stmt in else_body {
                declare_variables_in_stmt(int, builder, variables, stmt);
            }
        }
        Expr::WhileLoop(_condition, loop_body) => {
            for stmt in loop_body {
                declare_variables_in_stmt(int, builder, variables, stmt);
            }
        }
        _ => (),
    }
}
