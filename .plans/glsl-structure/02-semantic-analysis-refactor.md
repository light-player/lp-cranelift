# Phase 2: Semantic Analysis Refactoring

## Current State

`semantic/mod.rs` contains a monolithic `analyze_with_source` function that mixes:

1. Function signature extraction (first pass)
2. Function body extraction (second pass)
3. Validation (third pass)

Helper functions are mixed with orchestration code.

## Target Architecture

### Proposed Structure

```
semantic/
├── mod.rs              - Public API, orchestrates passes
├── passes/              - Individual semantic passes
│   ├── mod.rs          - Pass trait and registry
│   ├── function_registry.rs  - Function signature collection
│   ├── function_extraction.rs - Function body extraction
│   └── validation.rs   - Statement/expression validation
├── ast_visitor.rs      - AST traversal utilities
├── type_resolver.rs    - Type parsing from AST nodes
├── types.rs            - (existing, keep as-is)
├── type_check.rs       - (existing, will be refactored separately)
├── validator.rs        - (existing, may move to passes/)
├── functions.rs        - (existing, keep as-is)
├── scope.rs            - (existing, keep as-is)
└── builtins.rs         - (existing, keep as-is)
```

## Refactoring Plan

### Step 1: Create Pass Trait

**File**: `semantic/passes/mod.rs`

```rust
/// A semantic analysis pass that processes the AST
pub trait SemanticPass {
    /// Execute the pass on a translation unit
    fn run(&mut self, shader: &TranslationUnit, source: &str) -> Result<(), GlslError>;

    /// Pass name for debugging
    fn name(&self) -> &str;
}

/// Registry for semantic passes
pub struct SemanticPassRegistry {
    passes: Vec<Box<dyn SemanticPass>>,
}
```

**Benefits**:

- Extensible: easy to add new passes (constant folding, dead code elimination)
- Testable: each pass can be tested independently
- Composable: passes can be run in different orders

### Step 2: Extract Function Registry Pass

**File**: `semantic/passes/function_registry.rs`

Extract the first pass from `analyze_with_source`:

```rust
pub struct FunctionRegistryPass {
    registry: FunctionRegistry,
}

impl SemanticPass for FunctionRegistryPass {
    fn run(&mut self, shader: &TranslationUnit, source: &str) -> Result<(), GlslError> {
        // Extract function signatures (current first pass logic)
        for decl in &shader.0 {
            if let ExternalDeclaration::FunctionDefinition(func) = decl {
                let sig = extract_function_signature(&func.prototype)?;
                self.registry.register_function(sig)?;
            }
        }
        Ok(())
    }
}
```

**Move from `mod.rs`**:

- `extract_function_signature` function
- `extract_parameter` function
- `extract_param_qualifier` function

### Step 3: Extract Function Body Extraction Pass

**File**: `semantic/passes/function_extraction.rs`

Extract the second pass:

```rust
pub struct FunctionExtractionPass {
    main_func: Option<TypedFunction>,
    user_functions: Vec<TypedFunction>,
}

impl SemanticPass for FunctionExtractionPass {
    fn run(&mut self, shader: &TranslationUnit, source: &str) -> Result<(), GlslError> {
        // Extract function bodies (current second pass logic)
        for decl in &shader.0 {
            if let ExternalDeclaration::FunctionDefinition(func) = decl {
                let typed_func = extract_function_body(func)?;
                if func.prototype.name.name == "main" {
                    self.main_func = Some(typed_func);
                } else {
                    self.user_functions.push(typed_func);
                }
            }
        }
        Ok(())
    }
}
```

**Move from `mod.rs`**:

- `extract_function_body` function

### Step 4: Refactor Validation Pass

**File**: `semantic/passes/validation.rs`

Extract the third pass (currently in `semantic/mod.rs`):

```rust
pub struct ValidationPass {
    function_registry: FunctionRegistry,
}

impl SemanticPass for ValidationPass {
    fn run(&mut self, shader: &TypedShader, source: &str) -> Result<(), GlslError> {
        // Validate all functions (current third pass logic)
        // Use existing validator::validate_function
    }
}
```

**Note**: This pass operates on `TypedShader`, not `TranslationUnit`, so we need a different trait or phase separation.

### Step 5: Create Type Resolver Module

**File**: `semantic/type_resolver.rs`

Extract type parsing utilities:

```rust
/// Parse GLSL type specifier into our Type enum
pub fn parse_type_specifier(
    ty: &glsl::syntax::TypeSpecifier,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<types::Type, GlslError> {
    // Move from mod.rs
}

/// Parse return type from fully specified type
pub fn parse_return_type(
    ty: &glsl::syntax::FullySpecifiedType,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<types::Type, GlslError> {
    // Move from mod.rs
}
```

**Benefits**:

- Reusable across passes
- Single responsibility: AST → Type conversion
- Easy to extend for new type forms

### Step 6: Refactor Main Entry Point

**File**: `semantic/mod.rs`

New structure:

```rust
use passes::{SemanticPass, SemanticPassRegistry};

pub struct SemanticAnalyzer {
    registry: FunctionRegistry,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            registry: FunctionRegistry::new(),
        }
    }

    pub fn analyze(&mut self, shader: &TranslationUnit, source: &str) -> Result<TypedShader, GlslError> {
        // Pass 1: Collect function signatures
        let mut registry_pass = passes::FunctionRegistryPass::new();
        registry_pass.run(shader, source)?;
        let registry = registry_pass.into_registry();

        // Pass 2: Extract function bodies
        let mut extraction_pass = passes::FunctionExtractionPass::new();
        extraction_pass.run(shader, source)?;
        let (main_func, user_functions) = extraction_pass.into_results();

        // Pass 3: Validate
        let typed_shader = TypedShader {
            main_function: main_func.ok_or_else(|| GlslError::no_main_function())?,
            user_functions,
            function_registry: registry.clone(),
        };

        let mut validation_pass = passes::ValidationPass::new(registry);
        validation_pass.run(&typed_shader, source)?;

        Ok(typed_shader)
    }
}

// Keep backward compatibility
pub fn analyze_with_source(shader: &TranslationUnit, source: &str) -> Result<TypedShader, GlslError> {
    SemanticAnalyzer::new().analyze(shader, source)
}
```

## Migration Strategy

1. **Create new structure** alongside existing code
2. **Move functions gradually** with tests to verify
3. **Update callers** to use new API
4. **Remove old code** once migration complete

## Testing Strategy

- Unit tests for each pass independently
- Integration tests for full analysis pipeline
- Ensure existing tests still pass
- Add tests for pass composition

## Benefits

1. **Clarity**: Each pass has single responsibility
2. **Extensibility**: Easy to add new passes (constant folding, optimization)
3. **Testability**: Passes can be tested in isolation
4. **Reusability**: Type resolver can be used by codegen
5. **Maintainability**: Smaller, focused modules
