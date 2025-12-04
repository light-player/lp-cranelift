use glsl::syntax::{SimpleStatement, Statement};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(feature = "std")]
use std::string::{String, ToString};

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

use crate::codegen::context::CodegenContext;
use cranelift_codegen::ir::InstBuilder;

impl<'a> CodegenContext<'a> {
    pub fn translate_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Simple(simple) => self.translate_simple_statement(simple),
            Statement::Compound(compound) => self.translate_compound(compound),
        }
    }

    fn translate_simple_statement(&mut self, stmt: &SimpleStatement) -> Result<(), String> {
        match stmt {
            SimpleStatement::Declaration(decl) => self.translate_declaration(decl),
            SimpleStatement::Expression(Some(expr)) => {
                self.translate_expr(expr)?;
                Ok(())
            }
            SimpleStatement::Expression(None) => Ok(()), // Empty statement
            SimpleStatement::Selection(selection) => self.translate_selection(selection),
            SimpleStatement::Iteration(iteration) => self.translate_iteration(iteration),
            SimpleStatement::Jump(jump) => self.translate_jump(jump),
            _ => Err(format!("Statement type not supported: {:?}", stmt)),
        }
    }

    fn translate_compound(
        &mut self,
        compound: &glsl::syntax::CompoundStatement,
    ) -> Result<(), String> {
        // Translate all statements in the compound block
        for stmt in &compound.statement_list {
            self.translate_statement(stmt)?;
        }
        Ok(())
    }

    fn translate_selection(
        &mut self,
        selection: &glsl::syntax::SelectionStatement,
    ) -> Result<(), String> {
        use glsl::syntax::SelectionRestStatement;

        let condition_value = self.translate_expr(&selection.cond)?;

        let then_block = self.builder.create_block();
        let merge_block = self.builder.create_block();

        match &selection.rest {
            SelectionRestStatement::Statement(then_stmt) => {
                // No else: branch to then or merge
                self.builder
                    .ins()
                    .brif(condition_value, then_block, &[], merge_block, &[]);

                // Then branch
                self.builder.switch_to_block(then_block);
                self.builder.seal_block(then_block);
                self.translate_statement(then_stmt)?;
                self.builder.ins().jump(merge_block, &[]);

                // Merge
                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);
            }
            SelectionRestStatement::Else(then_stmt, else_stmt) => {
                let else_block = self.builder.create_block();

                // Branch to then or else
                self.builder
                    .ins()
                    .brif(condition_value, then_block, &[], else_block, &[]);

                // Then branch
                self.builder.switch_to_block(then_block);
                self.builder.seal_block(then_block);
                self.translate_statement(then_stmt)?;
                self.builder.ins().jump(merge_block, &[]);

                // Else branch
                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);
                self.translate_statement(else_stmt)?;
                self.builder.ins().jump(merge_block, &[]);

                // Merge
                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);
            }
        }

        Ok(())
    }

    fn translate_iteration(
        &mut self,
        iteration: &glsl::syntax::IterationStatement,
    ) -> Result<(), String> {
        use glsl::syntax::IterationStatement;

        match iteration {
            IterationStatement::While(condition, body) => {
                self.translate_while_loop(condition, body)
            }
            IterationStatement::DoWhile(body, expr) => self.translate_do_while_loop(body, expr),
            IterationStatement::For(init, rest, body) => self.translate_for_loop(init, rest, body),
        }
    }

    fn translate_while_loop(
        &mut self,
        condition: &glsl::syntax::Condition,
        body: &Statement,
    ) -> Result<(), String> {
        let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit_block = self.builder.create_block();

        // Push loop context for break/continue
        self.loop_stack.push(crate::codegen::context::LoopContext {
            continue_target: header_block,
            exit_block,
        });

        // Jump to header
        self.builder.ins().jump(header_block, &[]);

        // Header: evaluate condition
        self.builder.switch_to_block(header_block);
        let condition_value = self.translate_condition(condition)?;
        self.builder
            .ins()
            .brif(condition_value, body_block, &[], exit_block, &[]);

        // Body
        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);
        self.translate_statement(body)?;
        self.builder.ins().jump(header_block, &[]); // Loop back

        // Exit
        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(header_block);
        self.builder.seal_block(exit_block);

        // Pop loop context
        self.loop_stack.pop();

        Ok(())
    }

    fn translate_do_while_loop(
        &mut self,
        body: &Statement,
        condition: &glsl::syntax::Expr,
    ) -> Result<(), String> {
        let body_block = self.builder.create_block();
        let header_block = self.builder.create_block();
        let exit_block = self.builder.create_block();

        self.loop_stack.push(crate::codegen::context::LoopContext {
            continue_target: header_block,
            exit_block,
        });

        // Jump directly to body (do-while always executes once)
        self.builder.ins().jump(body_block, &[]);

        // Body
        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);
        self.translate_statement(body)?;
        self.builder.ins().jump(header_block, &[]);

        // Header: evaluate condition
        self.builder.switch_to_block(header_block);
        let condition_value = self.translate_expr(condition)?;
        self.builder
            .ins()
            .brif(condition_value, body_block, &[], exit_block, &[]);

        // Exit
        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(header_block);
        self.builder.seal_block(exit_block);

        self.loop_stack.pop();

        Ok(())
    }

    fn translate_for_loop(
        &mut self,
        init: &glsl::syntax::ForInitStatement,
        rest: &glsl::syntax::ForRestStatement,
        body: &Statement,
    ) -> Result<(), String> {
        // Translate init
        match init {
            glsl::syntax::ForInitStatement::Expression(Some(expr)) => {
                self.translate_expr(expr)?;
            }
            glsl::syntax::ForInitStatement::Declaration(decl) => {
                self.translate_declaration(decl)?;
            }
            glsl::syntax::ForInitStatement::Expression(None) => {
                // Empty init
            }
        }

        // Create blocks: header, body, update (for continue), exit
        let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let update_block = self.builder.create_block();
        let exit_block = self.builder.create_block();

        // For loops: continue should jump to update block, not header
        self.loop_stack.push(crate::codegen::context::LoopContext {
            continue_target: update_block,
            exit_block,
        });

        self.builder.ins().jump(header_block, &[]);

        // Header: evaluate condition
        self.builder.switch_to_block(header_block);
        let condition_value = if let Some(condition) = &rest.condition {
            self.translate_condition(condition)?
        } else {
            // No condition means infinite loop (while(true))
            self.builder
                .ins()
                .iconst(cranelift_codegen::ir::types::I8, 1)
        };
        self.builder
            .ins()
            .brif(condition_value, body_block, &[], exit_block, &[]);

        // Body
        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);
        self.translate_statement(body)?;
        self.builder.ins().jump(update_block, &[]);

        // Update block
        self.builder.switch_to_block(update_block);
        self.builder.seal_block(update_block);
        if let Some(update_expr) = &rest.post_expr {
            self.translate_expr(update_expr)?;
        }
        self.builder.ins().jump(header_block, &[]);

        // Exit
        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(header_block);
        self.builder.seal_block(exit_block);

        self.loop_stack.pop();

        Ok(())
    }

    fn translate_condition(
        &mut self,
        condition: &glsl::syntax::Condition,
    ) -> Result<cranelift_codegen::ir::Value, String> {
        match condition {
            glsl::syntax::Condition::Expr(expr) => {
                let (val, ty) = self.translate_expr_typed(expr)?;
                // Validate that condition is bool type (GLSL spec requirement)
                crate::semantic::type_check::check_condition(&ty)?;
                Ok(val)
            }
            _ => Err("Only expression conditions supported".to_string()),
        }
    }

    fn translate_jump(&mut self, jump: &glsl::syntax::JumpStatement) -> Result<(), String> {
        use glsl::syntax::JumpStatement;

        match jump {
            JumpStatement::Break => self.translate_break(),
            JumpStatement::Continue => self.translate_continue(),
            JumpStatement::Return(expr) => self.translate_return(expr.as_ref().map(|v| &**v)),
            _ => Err(format!("Jump statement not supported: {:?}", jump)),
        }
    }

    fn translate_return(&mut self, expr: Option<&glsl::syntax::Expr>) -> Result<(), String> {
        let return_val = if let Some(ret_expr) = expr {
            self.translate_expr(ret_expr)?
        } else {
            // Void return - return 0 as placeholder
            self.builder
                .ins()
                .iconst(cranelift_codegen::ir::types::I32, 0)
        };

        self.builder.ins().return_(&[return_val]);

        // Create unreachable block for subsequent code
        let unreachable = self.builder.create_block();
        self.builder.switch_to_block(unreachable);
        self.builder.seal_block(unreachable);

        Ok(())
    }

    fn translate_break(&mut self) -> Result<(), String> {
        let loop_ctx = self
            .loop_stack
            .last()
            .ok_or_else(|| "break statement outside of loop".to_string())?;

        self.builder.ins().jump(loop_ctx.exit_block, &[]);

        // Create unreachable block for subsequent code
        let unreachable = self.builder.create_block();
        self.builder.switch_to_block(unreachable);
        self.builder.seal_block(unreachable);

        Ok(())
    }

    fn translate_continue(&mut self) -> Result<(), String> {
        let loop_ctx = self
            .loop_stack
            .last()
            .ok_or_else(|| "continue statement outside of loop".to_string())?;

        self.builder.ins().jump(loop_ctx.continue_target, &[]);

        // Create unreachable block for subsequent code
        let unreachable = self.builder.create_block();
        self.builder.switch_to_block(unreachable);
        self.builder.seal_block(unreachable);

        Ok(())
    }

    fn translate_declaration(&mut self, decl: &glsl::syntax::Declaration) -> Result<(), String> {
        use glsl::syntax::Declaration;

        match decl {
            Declaration::InitDeclaratorList(list) => {
                // Get type from type specifier
                let ty = self.parse_type_specifier(&list.head.ty)?;

                // Handle the head declaration
                if let Some(name) = &list.head.name {
                    let vars = self.declare_variable(name.0.clone(), ty.clone());

                    // Handle initializer if present
                    if let Some(init) = &list.head.initializer {
                        let (init_vals, init_ty) = self.translate_initializer(init)?;
                        
                        // Type check
                        if init_ty != ty {
                            return Err(format!(
                                "Type mismatch in initialization: expected {:?}, got {:?}",
                                ty, init_ty
                            ));
                        }

                        // Check component counts match
                        if vars.len() != init_vals.len() {
                            return Err(format!(
                                "Component count mismatch: variable has {} components, initializer has {}",
                                vars.len(), init_vals.len()
                            ));
                        }

                        // Assign each component
                        for (var, val) in vars.iter().zip(&init_vals) {
                            self.builder.def_var(*var, *val);
                        }
                    }
                }

                // Handle tail declarations (same type, different names)
                for declarator in &list.tail {
                    let vars = self.declare_variable(declarator.ident.ident.0.clone(), ty.clone());

                    if let Some(init) = &declarator.initializer {
                        let (init_vals, init_ty) = self.translate_initializer(init)?;
                        
                        // Type check
                        if init_ty != ty {
                            return Err(format!(
                                "Type mismatch in initialization: expected {:?}, got {:?}",
                                ty, init_ty
                            ));
                        }

                        // Check component counts match
                        if vars.len() != init_vals.len() {
                            return Err(format!(
                                "Component count mismatch: variable has {} components, initializer has {}",
                                vars.len(), init_vals.len()
                            ));
                        }

                        // Assign each component
                        for (var, val) in vars.iter().zip(&init_vals) {
                            self.builder.def_var(*var, *val);
                        }
                    }
                }

                Ok(())
            }
            _ => Err("Only variable declarations supported".to_string()),
        }
    }

    fn parse_type_specifier(
        &self,
        type_spec: &glsl::syntax::FullySpecifiedType,
    ) -> Result<crate::semantic::types::Type, String> {
        use glsl::syntax::TypeSpecifierNonArray;

        match &type_spec.ty.ty {
            TypeSpecifierNonArray::Int => Ok(crate::semantic::types::Type::Int),
            TypeSpecifierNonArray::Bool => Ok(crate::semantic::types::Type::Bool),
            TypeSpecifierNonArray::Float => Ok(crate::semantic::types::Type::Float),
            TypeSpecifierNonArray::Vec2 => Ok(crate::semantic::types::Type::Vec2),
            TypeSpecifierNonArray::Vec3 => Ok(crate::semantic::types::Type::Vec3),
            TypeSpecifierNonArray::Vec4 => Ok(crate::semantic::types::Type::Vec4),
            TypeSpecifierNonArray::IVec2 => Ok(crate::semantic::types::Type::IVec2),
            TypeSpecifierNonArray::IVec3 => Ok(crate::semantic::types::Type::IVec3),
            TypeSpecifierNonArray::IVec4 => Ok(crate::semantic::types::Type::IVec4),
            TypeSpecifierNonArray::BVec2 => Ok(crate::semantic::types::Type::BVec2),
            TypeSpecifierNonArray::BVec3 => Ok(crate::semantic::types::Type::BVec3),
            TypeSpecifierNonArray::BVec4 => Ok(crate::semantic::types::Type::BVec4),
            _ => Err(format!(
                "Type not supported yet: {:?}",
                type_spec.ty.ty
            )),
        }
    }

    fn translate_initializer(
        &mut self,
        init: &glsl::syntax::Initializer,
    ) -> Result<(alloc::vec::Vec<cranelift_codegen::ir::Value>, crate::semantic::types::Type), String> {
        use glsl::syntax::Initializer;

        match init {
            Initializer::Simple(expr) => self.translate_expr_typed(expr.as_ref()),
            _ => Err("Only simple initializers supported".to_string()),
        }
    }
}
