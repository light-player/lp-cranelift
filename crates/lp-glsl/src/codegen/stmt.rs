use glsl::syntax::{SimpleStatement, Statement};

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::InstBuilder;

impl<'a> CodegenContext<'a> {
    pub fn translate_statement(&mut self, stmt: &Statement) -> Result<(), GlslError> {
        match stmt {
            Statement::Simple(simple) => self.translate_simple_statement(simple),
            Statement::Compound(compound) => self.translate_compound(compound),
        }
    }

    fn translate_simple_statement(&mut self, stmt: &SimpleStatement) -> Result<(), GlslError> {
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
            _ => Err(GlslError::new(ErrorCode::E0400, format!("statement type not supported: {:?}", stmt))),
        }
    }

    fn translate_compound(
        &mut self,
        compound: &glsl::syntax::CompoundStatement,
    ) -> Result<(), GlslError> {
        // Translate all statements in the compound block
        for stmt in &compound.statement_list {
            self.translate_statement(stmt)?;
        }
        Ok(())
    }

    fn translate_selection(
        &mut self,
        selection: &glsl::syntax::SelectionStatement,
    ) -> Result<(), GlslError> {
        use glsl::syntax::SelectionRestStatement;
        use crate::error::{extract_span_from_expr, source_span_to_location};

        // Translate condition and validate type
        let (cond_vals, cond_ty) = self.translate_expr_typed(&selection.cond)?;
        // Validate that condition is bool type (GLSL spec requirement)
        if cond_ty != crate::semantic::types::Type::Bool {
            let span = extract_span_from_expr(&selection.cond);
            let error = GlslError::new(
                ErrorCode::E0107,
                "condition must be bool type"
            )
            .with_location(source_span_to_location(&span))
            .with_note(format!("condition has type `{:?}`, expected `Bool`", cond_ty));
            return Err(self.add_span_to_error(error, &span));
        }
        // Condition must be scalar, so we take the first (and only) value
        let condition_value = cond_vals.into_iter().next().ok_or_else(|| GlslError::new(ErrorCode::E0400, "condition expression produced no value"))?;

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
    ) -> Result<(), GlslError> {
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
    ) -> Result<(), GlslError> {
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
    ) -> Result<(), GlslError> {
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
    ) -> Result<(), GlslError> {
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
    ) -> Result<cranelift_codegen::ir::Value, GlslError> {
        match condition {
            glsl::syntax::Condition::Expr(expr) => {
                let (vals, ty) = self.translate_expr_typed(expr)?;
                // Validate that condition is bool type (GLSL spec requirement)
                crate::semantic::type_check::check_condition(&ty)?;
                // Condition must be scalar, so we take the first (and only) value
                Ok(vals.into_iter().next().ok_or_else(|| GlslError::new(ErrorCode::E0400, "condition expression produced no value"))?)
            }
            _ => Err(GlslError::new(ErrorCode::E0400, "only expression conditions supported")),
        }
    }

    fn translate_jump(&mut self, jump: &glsl::syntax::JumpStatement) -> Result<(), GlslError> {
        use glsl::syntax::JumpStatement;

        match jump {
            JumpStatement::Break => self.translate_break(),
            JumpStatement::Continue => self.translate_continue(),
            JumpStatement::Return(expr) => self.translate_return(expr.as_ref().map(|v| &**v)),
            _ => Err(GlslError::new(ErrorCode::E0400, format!("jump statement not supported: {:?}", jump))),
        }
    }

    fn translate_return(&mut self, expr: Option<&glsl::syntax::Expr>) -> Result<(), GlslError> {
        use crate::error::extract_span_from_expr;
        use cranelift_codegen::ir::{ArgumentPurpose, InstBuilder, MemFlags};
        
        if let Some(ret_expr) = expr {
            let span = extract_span_from_expr(ret_expr);
            let (ret_vals, ret_ty) = self.translate_expr_typed(ret_expr)?;
            
            // Validate return type matches function signature
            if let Some(expected_ty) = &self.return_type {
                // Check if function uses StructReturn
                let uses_struct_return = self.builder.func.signature.uses_special_param(ArgumentPurpose::StructReturn);
                
                if uses_struct_return {
                    // Function uses StructReturn - write values to buffer
                    // Use special_param() method (like cranelift-examples) to get the StructReturn pointer
                    let struct_ret_ptr = self.builder.func
                        .special_param(ArgumentPurpose::StructReturn)
                        .ok_or_else(|| {
                            GlslError::new(crate::error::ErrorCode::E0400, "StructReturn parameter not found (internal error)")
                        })?;
                    
                    // Coerce and write values to buffer at offsets (4 bytes per f32)
                    let expected_base = if expected_ty.is_vector() {
                        expected_ty.vector_base_type().unwrap()
                    } else {
                        crate::semantic::types::Type::Float
                    };
                    let ret_base = if ret_ty.is_vector() {
                        ret_ty.vector_base_type().unwrap()
                    } else if ret_ty.is_matrix() {
                        crate::semantic::types::Type::Float
                    } else {
                        ret_ty.clone()
                    };
                    
                    for (i, val) in ret_vals.iter().enumerate() {
                        let coerced = if ret_base == expected_base {
                            *val
                        } else {
                            self.coerce_to_type_with_location(*val, &ret_base, &expected_base, Some(span.clone()))?
                        };
                        let offset = (i * 4) as i32; // 4 bytes per f32
                        self.builder.ins().store(MemFlags::trusted(), coerced, struct_ret_ptr, offset);
                    }
                    
                    // Return void for StructReturn functions
                    self.builder.ins().return_(&[]);
                } else if expected_ty.is_vector() || expected_ty.is_matrix() {
                    // For vectors/matrices without StructReturn (shouldn't happen with this plan)
                    // Keep existing behavior as fallback
                    let expected_base = if expected_ty.is_vector() {
                        expected_ty.vector_base_type().unwrap()
                    } else {
                        crate::semantic::types::Type::Float
                    };
                    let ret_base = if ret_ty.is_vector() {
                        ret_ty.vector_base_type().unwrap()
                    } else if ret_ty.is_matrix() {
                        crate::semantic::types::Type::Float
                    } else {
                        ret_ty.clone()
                    };
                    
                    let mut coerced_vals = Vec::new();
                    for val in ret_vals {
                        let coerced = if ret_base == expected_base {
                            val
                        } else {
                            self.coerce_to_type_with_location(val, &ret_base, &expected_base, Some(span.clone()))?
                        };
                        coerced_vals.push(coerced);
                    }
                    self.builder.ins().return_(&coerced_vals);
                } else {
                    // For scalars, return single value with coercion if needed
                    let expected_base = expected_ty.clone();
                    let ret_base = ret_ty.clone();
                    
                    let return_val = if ret_base == expected_base {
                        ret_vals[0]
                    } else {
                        self.coerce_to_type_with_location(ret_vals[0], &ret_base, &expected_base, Some(span.clone()))?
                    };
                    self.builder.ins().return_(&[return_val]);
                }
            } else {
                // No return type specified, use first value as-is
                self.builder.ins().return_(&[ret_vals[0]]);
            }
        } else {
            // Void return - return empty
            self.builder.ins().return_(&[]);
        }

        // Create unreachable block for subsequent code
        let unreachable = self.builder.create_block();
        self.builder.switch_to_block(unreachable);
        self.builder.seal_block(unreachable);

        Ok(())
    }

    fn translate_break(&mut self) -> Result<(), GlslError> {
        let loop_ctx = self
            .loop_stack
            .last()
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "break statement outside of loop"))?;

        self.builder.ins().jump(loop_ctx.exit_block, &[]);

        // Create unreachable block for subsequent code
        let unreachable = self.builder.create_block();
        self.builder.switch_to_block(unreachable);
        self.builder.seal_block(unreachable);

        Ok(())
    }

    fn translate_continue(&mut self) -> Result<(), GlslError> {
        let loop_ctx = self
            .loop_stack
            .last()
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "continue statement outside of loop"))?;

        self.builder.ins().jump(loop_ctx.continue_target, &[]);

        // Create unreachable block for subsequent code
        let unreachable = self.builder.create_block();
        self.builder.switch_to_block(unreachable);
        self.builder.seal_block(unreachable);

        Ok(())
    }

    fn translate_declaration(&mut self, decl: &glsl::syntax::Declaration) -> Result<(), GlslError> {
        use glsl::syntax::Declaration;

        match decl {
            Declaration::InitDeclaratorList(list) => {
                // Get type from type specifier
                let ty = self.parse_type_specifier(&list.head.ty)?;

                // Handle the head declaration
                if let Some(name) = &list.head.name {
                    let vars = self.declare_variable(name.name.clone(), ty.clone());

                    // Handle initializer if present
                    if let Some(init) = &list.head.initializer {
                        let (init_vals, init_ty) = self.translate_initializer(init)?;
                        
                        // Type check (allows implicit conversions)
                        // Extract span from initializer for error reporting
                        let init_span = match init {
                            glsl::syntax::Initializer::Simple(expr) => {
                                crate::error::extract_span_from_expr(expr.as_ref())
                            }
                            _ => glsl::syntax::SourceSpan::unknown(),
                        };
                        match crate::semantic::type_check::check_assignment(&ty, &init_ty) {
                            Ok(()) => {}
                            Err(mut error) => {
                                if error.location.is_none() {
                                    error = error.with_location(crate::error::source_span_to_location(&init_span));
                                }
                                return Err(self.add_span_to_error(error, &init_span));
                            }
                        }

                        // Coerce initializer values to match variable type
                        let base_ty = if ty.is_vector() {
                            ty.vector_base_type().unwrap()
                        } else if ty.is_matrix() {
                            crate::semantic::types::Type::Float
                        } else {
                            ty.clone()
                        };
                        let init_base = if init_ty.is_vector() {
                            init_ty.vector_base_type().unwrap()
                        } else if init_ty.is_matrix() {
                            crate::semantic::types::Type::Float
                        } else {
                            init_ty.clone()
                        };

                        // Check component counts match
                        if vars.len() != init_vals.len() {
                            return Err(GlslError::new(
                                ErrorCode::E0400,
                                format!("component count mismatch: variable has {} components, initializer has {}", vars.len(), init_vals.len())
                            ));
                        }

                        // Assign each component with type coercion
                        for (var, val) in vars.iter().zip(&init_vals) {
                            let coerced_val = self.coerce_to_type(*val, &init_base, &base_ty)?;
                            self.builder.def_var(*var, coerced_val);
                        }
                    }
                }

                // Handle tail declarations (same type, different names)
                for declarator in &list.tail {
                    let vars = self.declare_variable(declarator.ident.ident.name.clone(), ty.clone());

                    if let Some(init) = &declarator.initializer {
                        let (init_vals, init_ty) = self.translate_initializer(init)?;
                        
                        // Type check (allows implicit conversions)
                        crate::semantic::type_check::check_assignment(&ty, &init_ty)?;

                        // Coerce initializer values to match variable type
                        let base_ty = if ty.is_vector() {
                            ty.vector_base_type().unwrap()
                        } else if ty.is_matrix() {
                            crate::semantic::types::Type::Float
                        } else {
                            ty.clone()
                        };
                        let init_base = if init_ty.is_vector() {
                            init_ty.vector_base_type().unwrap()
                        } else if init_ty.is_matrix() {
                            crate::semantic::types::Type::Float
                        } else {
                            init_ty.clone()
                        };

                        // Check component counts match
                        if vars.len() != init_vals.len() {
                            return Err(GlslError::new(
                                ErrorCode::E0400,
                                format!("component count mismatch: variable has {} components, initializer has {}", vars.len(), init_vals.len())
                            ));
                        }

                        // Assign each component with type coercion
                        for (var, val) in vars.iter().zip(&init_vals) {
                            let coerced_val = self.coerce_to_type(*val, &init_base, &base_ty)?;
                            self.builder.def_var(*var, coerced_val);
                        }
                    }
                }

                Ok(())
            }
            _ => Err(GlslError::new(ErrorCode::E0400, "only variable declarations supported")),
        }
    }

    fn parse_type_specifier(
        &self,
        type_spec: &glsl::syntax::FullySpecifiedType,
    ) -> Result<crate::semantic::types::Type, GlslError> {
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
            TypeSpecifierNonArray::Mat2 => Ok(crate::semantic::types::Type::Mat2),
            TypeSpecifierNonArray::Mat3 => Ok(crate::semantic::types::Type::Mat3),
            TypeSpecifierNonArray::Mat4 => Ok(crate::semantic::types::Type::Mat4),
            _ => Err(GlslError::unsupported_type(format!("{:?}", type_spec.ty.ty))),
        }
    }

    fn translate_initializer(
        &mut self,
        init: &glsl::syntax::Initializer,
    ) -> Result<(Vec<cranelift_codegen::ir::Value>, crate::semantic::types::Type), GlslError> {
        use glsl::syntax::Initializer;

        match init {
            Initializer::Simple(expr) => self.translate_expr_typed(expr.as_ref()),
            _ => Err(GlslError::new(ErrorCode::E0400, "only simple initializers supported")),
        }
    }
}
