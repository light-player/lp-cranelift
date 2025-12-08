use cranelift_codegen::ir::Block;
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::{FuncId, Module};
use hashbrown::HashMap;

use crate::semantic::functions::FunctionRegistry;
use crate::semantic::types::Type as GlslType;
use crate::semantic::types::Type;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub struct VarInfo {
    pub cranelift_vars: Vec<Variable>, // Changed from single Variable to support vectors
    pub glsl_type: GlslType,
}

pub struct CodegenContext<'a> {
    pub builder: FunctionBuilder<'a>,
    pub module: &'a mut dyn Module,
    pub variables: HashMap<String, VarInfo>,

    // Control flow tracking for break/continue
    pub loop_stack: Vec<LoopContext>,

    // User-defined function support
    pub function_ids: Option<HashMap<String, FuncId>>,
    pub function_registry: Option<&'a FunctionRegistry>,

    // Source text for span extraction
    pub source_text: Option<&'a str>,

    // Current function return type (for return statement validation)
    pub return_type: Option<GlslType>,

    // Entry block for accessing function parameters (including StructReturn)
    pub entry_block: Option<Block>,
}

pub struct LoopContext {
    pub continue_target: Block, // Target for continue (might be header or update block)
    pub exit_block: Block,      // Target for break
}

impl<'a> CodegenContext<'a> {
    pub fn new(builder: FunctionBuilder<'a>, module: &'a mut dyn Module) -> Self {
        Self {
            builder,
            module,
            variables: HashMap::new(),
            loop_stack: Vec::new(),
            function_ids: None,
            function_registry: None,
            source_text: None,
            return_type: None,
            entry_block: None,
        }
    }

    pub fn set_entry_block(&mut self, entry_block: Block) {
        self.entry_block = Some(entry_block);
    }

    pub fn set_return_type(&mut self, return_type: GlslType) {
        self.return_type = Some(return_type);
    }

    pub fn set_source_text(&mut self, source: &'a str) {
        self.source_text = Some(source);
    }

    pub fn set_function_ids(&mut self, func_ids: &HashMap<String, FuncId>) {
        self.function_ids = Some(func_ids.clone());
    }

    pub fn set_function_registry(&mut self, registry: &'a FunctionRegistry) {
        self.function_registry = Some(registry);
    }

    /// Add span_text to an error if source is available
    pub fn add_span_to_error(&self, error: crate::error::GlslError, span: &glsl::syntax::SourceSpan) -> crate::error::GlslError {
        use crate::error::add_span_text_to_error;
        add_span_text_to_error(error, self.source_text, span)
    }

    pub fn declare_variable(&mut self, name: String, glsl_ty: GlslType) -> Vec<Variable> {
        let component_count = if glsl_ty.is_vector() {
            glsl_ty.component_count().unwrap()
        } else if glsl_ty.is_matrix() {
            glsl_ty.matrix_element_count().unwrap()
        } else {
            1
        };

        let base_ty = if glsl_ty.is_vector() {
            glsl_ty.vector_base_type().unwrap()
        } else if glsl_ty.is_matrix() {
            Type::Float // Matrices are always float
        } else {
            glsl_ty.clone()
        };

        let cranelift_ty = base_ty.to_cranelift_type();

        let mut vars = Vec::new();
        for _ in 0..component_count {
            vars.push(self.builder.declare_var(cranelift_ty));
        }

        self.variables.insert(
            name,
            VarInfo {
                cranelift_vars: vars.clone(),
                glsl_type: glsl_ty,
            },
        );

        vars
    }

    pub fn lookup_variable(&self, name: &str) -> Option<Variable> {
        // Legacy method: returns first component (for scalars)
        self.variables
            .get(name)
            .and_then(|info| info.cranelift_vars.first().copied())
    }

    pub fn lookup_variables(&self, name: &str) -> Option<&[Variable]> {
        self.variables
            .get(name)
            .map(|info| info.cranelift_vars.as_slice())
    }

    pub fn lookup_variable_type(&self, name: &str) -> Option<&GlslType> {
        self.variables.get(name).map(|info| &info.glsl_type)
    }

    /// Store a value to a matrix element at (row, col)
    /// Matrix is stored in column-major order: element (row, col) = vars[col * rows + row]
    pub fn store_matrix_element(
        &mut self,
        matrix_vars: &[Variable],
        row: usize,
        col: usize,
        rows: usize,
        value: cranelift_codegen::ir::Value,
    ) {
        let index = col * rows + row;
        self.builder.def_var(matrix_vars[index], value);
    }

    /// Load a matrix element at (row, col)
    pub fn load_matrix_element(
        &mut self,
        matrix_vars: &[Variable],
        row: usize,
        col: usize,
        rows: usize,
    ) -> cranelift_codegen::ir::Value {
        let index = col * rows + row;
        self.builder.use_var(matrix_vars[index])
    }

    /// Load an entire matrix column as a vector
    /// Returns the values for the column
    pub fn load_matrix_column(
        &mut self,
        matrix_vars: &[Variable],
        col: usize,
        rows: usize,
    ) -> Vec<cranelift_codegen::ir::Value> {
        let mut result = Vec::new();
        for row in 0..rows {
            let index = col * rows + row;
            result.push(self.builder.use_var(matrix_vars[index]));
        }
        result
    }
}
