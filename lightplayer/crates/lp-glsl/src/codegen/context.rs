use cranelift_codegen::ir::{Block, InstBuilder, Value};
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::{FuncId, Module};
use hashbrown::HashMap;

use crate::codegen::sourceloc::SourceLocManager;
use crate::error::{ErrorCode, GlslError};
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

    // Intrinsic function cache (for math function implementations)
    #[cfg(feature = "intrinsic-math")]
    pub intrinsic_cache: Option<crate::intrinsics::loader::IntrinsicCache>,

    // Source location manager for mapping SourceLoc to GLSL source positions
    pub source_loc_manager: SourceLocManager,
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
            #[cfg(feature = "intrinsic-math")]
            intrinsic_cache: None,
            source_loc_manager: SourceLocManager::new(),
        }
    }

    /// Get mutable reference to the source location manager.
    pub fn source_loc_manager(&mut self) -> &mut SourceLocManager {
        &mut self.source_loc_manager
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

    pub fn declare_variable(&mut self, name: String, glsl_ty: GlslType) -> Result<Vec<Variable>, crate::error::GlslError> {
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

        let cranelift_ty = base_ty.to_cranelift_type()
            .map_err(|e| crate::error::GlslError::new(
                crate::error::ErrorCode::E0400,
                format!("Failed to convert type to Cranelift type: {}", e.message)
            ))?;

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

        Ok(vars)
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

    /// Store a value to a matrix element at m[col][row]
    ///
    /// Matrix is stored in column-major order per GLSL specification.
    /// Element m[col][row] is stored at index `col * rows + row` in the flat array.
    ///
    /// Example for mat2:
    /// - Element m[0][0] → index 0 * 2 + 0 = 0
    /// - Element m[0][1] → index 0 * 2 + 1 = 1 (column 0, row 1)
    /// - Element m[1][0] → index 1 * 2 + 0 = 2 (column 1, row 0)
    /// - Element m[1][1] → index 1 * 2 + 1 = 3
    pub fn store_matrix_element(
        &mut self,
        matrix_vars: &[Variable],
        col: usize,
        row: usize,
        rows: usize,
        value: cranelift_codegen::ir::Value,
    ) {
        let index = col * rows + row;
        self.builder.def_var(matrix_vars[index], value);
    }

    /// Load a matrix element at m[col][row]
    pub fn load_matrix_element(
        &mut self,
        matrix_vars: &[Variable],
        col: usize,
        row: usize,
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

    // ============================================================================
    // Block Management API (following Clang's pattern)
    // ============================================================================

    /// Get the current block (insertion point)
    pub fn current_block(&self) -> Option<Block> {
        self.builder.current_block()
    }

    /// Ensure we're in a block before evaluating expressions
    /// Returns error if not in a block (for production code)
    pub fn ensure_block(&self) -> Result<Block, GlslError> {
        self.builder.current_block().ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                "not in a block - cannot evaluate expressions",
            )
        })
    }

    /// Switch to block and seal it (following Clang's EmitBlock pattern)
    /// This is the primary method for switching to a new block.
    pub fn emit_block(&mut self, block: Block) {
        self.builder.switch_to_block(block);
        self.builder.seal_block(block);
    }

    /// Create a new block, switch to it, and seal it
    /// Convenience method for common pattern.
    pub fn create_and_emit_block(&mut self) -> Block {
        let block = self.builder.create_block();
        self.emit_block(block);
        block
    }

    /// Branch to target block (following Clang's EmitBranch pattern)
    /// Ensures we're in a block before branching.
    pub fn emit_branch(&mut self, target: Block) -> Result<(), GlslError> {
        self.ensure_block()?;
        self.builder.ins().jump(target, &[]);
        Ok(())
    }

    /// Conditional branch (following Clang's EmitBranchOnBoolExpr pattern)
    /// Evaluates condition in current block, then branches.
    pub fn emit_cond_branch(
        &mut self,
        cond: Value,
        then_block: Block,
        else_block: Block,
    ) -> Result<(), GlslError> {
        self.ensure_block()?;
        self.builder.ins().brif(cond, then_block, &[], else_block, &[]);
        Ok(())
    }
}
