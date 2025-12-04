use glsl::syntax::{SimpleStatement, Statement};

#[cfg(feature = "std")]
use std::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

use crate::codegen::context::CodegenContext;

impl<'a> CodegenContext<'a> {
    pub fn translate_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Simple(simple) => self.translate_simple_statement(simple),
            _ => Err("Only simple statements supported in Phase 1".to_string()),
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
            _ => Err("Statement type not supported in Phase 1".to_string()),
        }
    }

    fn translate_declaration(&mut self, decl: &glsl::syntax::Declaration) -> Result<(), String> {
        use glsl::syntax::Declaration;

        match decl {
            Declaration::InitDeclaratorList(list) => {
                // Get type from type specifier
                let ty = self.parse_type_specifier(&list.head.ty)?;

                // Handle the head declaration
                if let Some(name) = &list.head.name {
                    let var = self.declare_variable(name.0.clone(), ty);

                    // Handle initializer if present
                    if let Some(init) = &list.head.initializer {
                        let init_val = self.translate_initializer(init)?;
                        self.builder.def_var(var, init_val);
                    }
                }

                // Handle tail declarations (same type, different names)
                for declarator in &list.tail {
                    let var = self.declare_variable(declarator.ident.ident.0.clone(), ty);

                    if let Some(init) = &declarator.initializer {
                        let init_val = self.translate_initializer(init)?;
                        self.builder.def_var(var, init_val);
                    }
                }

                Ok(())
            }
            _ => Err("Only variable declarations supported in Phase 1".to_string()),
        }
    }

    fn parse_type_specifier(
        &self,
        type_spec: &glsl::syntax::FullySpecifiedType,
    ) -> Result<cranelift_codegen::ir::Type, String> {
        use glsl::syntax::TypeSpecifierNonArray;

        match &type_spec.ty.ty {
            TypeSpecifierNonArray::Int => Ok(cranelift_codegen::ir::types::I32),
            TypeSpecifierNonArray::Bool => Ok(cranelift_codegen::ir::types::I8),
            _ => Err(format!(
                "Type not supported in Phase 1: {:?}",
                type_spec.ty.ty
            )),
        }
    }

    fn translate_initializer(
        &mut self,
        init: &glsl::syntax::Initializer,
    ) -> Result<cranelift_codegen::ir::Value, String> {
        use glsl::syntax::Initializer;

        match init {
            Initializer::Simple(expr) => self.translate_expr(expr.as_ref()),
            _ => Err("Only simple initializers supported in Phase 1".to_string()),
        }
    }
}

