# Phase 6: Pipeline Extensibility

## Current State

`pipeline.rs` provides a simple pipeline:

1. Parse → `ParseResult`
2. Analyze → `SemanticResult`
3. `parse_and_analyze` convenience method

**Issues**:

- No hooks for transformations
- No intermediate result types for debugging
- No support for multiple backends
- Transformations (like fixed-point) are applied in JIT, not pipeline

## Target Architecture

### Proposed Structure

```
pipeline.rs (enhanced)
├── Pipeline stages (trait-based)
├── Transformation passes (pluggable)
├── Backend abstraction
└── Result types for each stage
```

## Refactoring Plan

### Step 1: Create Pipeline Stage Trait

**File**: `pipeline/stage.rs` (or in `pipeline.rs`)

```rust
/// A stage in the compilation pipeline
pub trait PipelineStage<Input, Output> {
    /// Execute the stage
    fn execute(&mut self, input: Input) -> Result<Output, GlslError>;

    /// Stage name for debugging
    fn name(&self) -> &str;
}

/// A transformation stage that can be inserted into the pipeline
pub trait Transformation<Input>: PipelineStage<Input, Input> {
    /// Whether this transformation can be enabled/disabled
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}
```

### Step 2: Define Pipeline Stages

**Enhance `pipeline.rs`**:

```rust
/// Parsing stage
pub struct ParseStage;

impl PipelineStage<&str, ParseResult> for ParseStage {
    fn execute(&mut self, source: &str) -> Result<ParseResult, GlslError> {
        CompilationPipeline::parse(source)
    }

    fn name(&self) -> &str {
        "parse"
    }
}

/// Semantic analysis stage
pub struct AnalyzeStage;

impl PipelineStage<ParseResult, SemanticResult> for AnalyzeStage {
    fn execute(&mut self, input: ParseResult) -> Result<SemanticResult, GlslError> {
        CompilationPipeline::analyze(input)
    }

    fn name(&self) -> &str {
        "analyze"
    }
}

/// Transformation stage (abstract)
pub struct FixedPointTransform {
    enabled: bool,
    format: Option<FixedPointFormat>,
}

impl Transformation<SemanticResult> for FixedPointTransform {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl PipelineStage<SemanticResult, SemanticResult> for FixedPointTransform {
    fn execute(&mut self, mut input: SemanticResult) -> Result<SemanticResult, GlslError> {
        if !self.enabled {
            return Ok(input);
        }

        // Apply fixed-point transformation
        // This needs to work on TypedShader, which requires different approach
        // For now, transformation is applied during codegen
        Ok(input)
    }

    fn name(&self) -> &str {
        "fixed_point"
    }
}
```

### Step 3: Create Pipeline Builder

**Add to `pipeline.rs`**:

```rust
/// Configurable compilation pipeline
pub struct Pipeline {
    stages: Vec<Box<dyn PipelineStage<dyn Any, dyn Any>>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
        }
    }

    /// Add a parsing stage
    pub fn with_parse(mut self) -> Self {
        // Add parse stage
        self
    }

    /// Add a semantic analysis stage
    pub fn with_analyze(mut self) -> Self {
        // Add analyze stage
        self
    }

    /// Add a transformation
    pub fn with_transform<T: Transformation<SemanticResult>>(mut self, transform: T) -> Self {
        // Add transformation
        self
    }

    /// Execute the pipeline
    pub fn execute(&mut self, source: &str) -> Result<SemanticResult, GlslError> {
        // Execute stages in order
        // Type checking will be challenging without proper type erasure
        // May need different approach
        todo!()
    }
}
```

**Note**: The type erasure approach is complex in Rust. Consider simpler alternatives.

### Step 4: Alternative: Simple Pipeline with Callbacks

**Simpler approach** for `pipeline.rs`:

```rust
/// Compilation pipeline with transformation hooks
pub struct CompilationPipeline;

impl CompilationPipeline {
    /// Parse GLSL source into an AST
    pub fn parse<'a>(source: &'a str) -> Result<ParseResult<'a>, GlslError> {
        // Existing implementation
    }

    /// Perform semantic analysis on parsed shader
    pub fn analyze<'a>(parse_result: ParseResult<'a>) -> Result<SemanticResult<'a>, GlslError> {
        // Existing implementation
    }

    /// Apply transformations to semantic result
    pub fn transform<'a>(
        result: SemanticResult<'a>,
        transforms: &[Box<dyn TransformationPass>],
    ) -> Result<SemanticResult<'a>, GlslError> {
        let mut current = result;
        for transform in transforms {
            if transform.is_enabled() {
                current = transform.apply(current)?;
            }
        }
        Ok(current)
    }

    /// Parse, analyze, and transform in one step
    pub fn compile<'a>(
        source: &'a str,
        transforms: &[Box<dyn TransformationPass>],
    ) -> Result<SemanticResult<'a>, GlslError> {
        let parse_result = Self::parse(source)?;
        let semantic_result = Self::analyze(parse_result)?;
        Self::transform(semantic_result, transforms)
    }
}

/// A transformation pass that operates on semantic results
pub trait TransformationPass {
    /// Apply the transformation
    fn apply<'a>(&self, result: SemanticResult<'a>) -> Result<SemanticResult<'a>, GlslError>;

    /// Whether this transformation is enabled
    fn is_enabled(&self) -> bool;

    /// Name of the transformation
    fn name(&self) -> &str;
}
```

### Step 5: Move Fixed-Point Transform to Pipeline

**Current**: Fixed-point transformation is applied in `jit.rs` during codegen.

**Problem**: Transformations should be applied to semantic representation, not during codegen.

**Solution**: Create transformation that operates on `TypedShader`:

```rust
// In transform/fixed_point.rs (enhance existing)

/// Fixed-point transformation pass
pub struct FixedPointTransformation {
    enabled: bool,
    format: Option<FixedPointFormat>,
}

impl TransformationPass for FixedPointTransformation {
    fn apply<'a>(&self, mut result: SemanticResult<'a>) -> Result<SemanticResult<'a>, GlslError> {
        if !self.enabled || self.format.is_none() {
            return Ok(result);
        }

        // Apply transformation to TypedShader
        // This requires walking the AST and converting float types/operations
        // For now, keep existing approach but mark for future work

        // Note: Current implementation works on Cranelift IR, not AST
        // To move to pipeline, need to transform at AST level
        // This is a larger refactoring

        Ok(result)
    }

    fn is_enabled(&self) -> bool {
        self.enabled && self.format.is_some()
    }

    fn name(&self) -> &str {
        "fixed_point"
    }
}
```

**Note**: Moving fixed-point to pipeline requires AST-level transformation, which is more complex. Consider this a future enhancement.

### Step 6: Add Backend Abstraction

**Add to `pipeline.rs`**:

```rust
/// Result of compilation, ready for backend
pub struct CompiledShader {
    pub typed_shader: TypedShader,
    pub source: String,
}

/// Backend that generates code from compiled shader
pub trait Backend {
    type Output;
    type Error;

    fn compile(&mut self, shader: CompiledShader) -> Result<Self::Output, Self::Error>;
}

/// JIT backend
pub struct JITBackend {
    jit: JIT,
}

impl Backend for JITBackend {
    type Output = *const u8;
    type Error = GlslError;

    fn compile(&mut self, shader: CompiledShader) -> Result<Self::Output, Self::Error> {
        self.jit.compile_detailed(&shader.source)
    }
}

/// CLIF output backend
pub struct CLIFBackend {
    jit: JIT,
}

impl Backend for CLIFBackend {
    type Output = String;
    type Error = GlslError;

    fn compile(&mut self, shader: CompiledShader) -> Result<Self::Output, Self::Error> {
        self.jit.compile_to_clif_detailed(&shader.source)
    }
}
```

### Step 7: Complete Pipeline API

**Final `pipeline.rs` structure**:

```rust
/// High-level compilation API
pub struct Compiler {
    pipeline: CompilationPipeline,
    transforms: Vec<Box<dyn TransformationPass>>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            pipeline: CompilationPipeline,
            transforms: Vec::new(),
        }
    }

    /// Add a transformation pass
    pub fn add_transform(&mut self, transform: Box<dyn TransformationPass>) {
        self.transforms.push(transform);
    }

    /// Compile GLSL source
    pub fn compile<'a>(
        &mut self,
        source: &'a str,
    ) -> Result<CompiledShader, GlslError> {
        let semantic_result = CompilationPipeline::compile(source, &self.transforms)?;
        Ok(CompiledShader {
            typed_shader: semantic_result.typed_ast,
            source: semantic_result.source.to_string(),
        })
    }

    /// Compile with backend
    pub fn compile_with<B: Backend>(
        &mut self,
        source: &str,
        backend: &mut B,
    ) -> Result<B::Output, B::Error> {
        let compiled = self.compile(source)?;
        backend.compile(compiled)
    }
}
```

## Migration Strategy

1. **Add new API alongside existing** (don't break existing code)
2. **Migrate JIT to use new pipeline** gradually
3. **Add transformations** one at a time
4. **Remove old API** once migration complete

## Testing Strategy

- Unit tests for each pipeline stage
- Integration tests for full pipeline
- Test transformation passes independently
- Test backend abstraction

## Benefits

1. **Extensibility**: Easy to add new transformations
2. **Testability**: Each stage can be tested independently
3. **Flexibility**: Support multiple backends (JIT, static, CLIF)
4. **Composability**: Transformations can be combined
5. **Debugging**: Intermediate results available at each stage

## Future Enhancements

1. **AST-level transformations**: Move fixed-point to pipeline
2. **Optimization passes**: Constant folding, dead code elimination
3. **Multiple backends**: Static compilation, WASM output
4. **Pipeline visualization**: Debug output showing stages
5. **Conditional transformations**: Enable/disable based on target
