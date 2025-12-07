# GLSL Parser Code Review - Architecture Overview

## Current Structure Analysis

### Module Organization

```
crates/lp-glsl/src/
├── lib.rs           - Entry point, minimal coordination
├── frontend.rs      - Parser wrapper (empty, just re-exports)
├── pipeline.rs      - Compilation pipeline (parse → analyze)
├── semantic/        - Semantic analysis
│   ├── mod.rs       - Main entry (multi-pass analysis in single function)
│   ├── types.rs     - Type system (well-structured)
│   ├── type_check.rs - Type inference (878 lines, too many responsibilities)
│   ├── validator.rs - Statement/expression validation (239 lines)
│   ├── functions.rs - Function registry (120 lines)
│   ├── scope.rs     - Symbol table (89 lines)
│   └── builtins.rs  - Built-in functions
├── codegen/         - Cranelift IR generation
│   ├── context.rs   - Codegen context (179 lines)
│   ├── expr.rs      - Expression codegen (1339 lines, needs splitting)
│   ├── stmt.rs      - Statement codegen (567 lines)
│   ├── builtins.rs  - Built-in codegen
│   └── signature.rs - Function signatures
├── jit.rs           - JIT compilation (474 lines, duplicate logic)
├── compiler.rs      - Compiler orchestration
└── error.rs         - Error handling (483 lines, well-structured)
```

### Reference: DirectXShaderCompiler Structure

DXC uses a clear separation:

- **Parse/** - Parsing logic (ParseHLSL.cpp, ParseExpr.cpp, ParseStmt.cpp)
- **Sema/** - Semantic analysis (SemaHLSL.cpp, SemaExpr.cpp, SemaStmt.cpp)
- **CodeGen/** - Code generation
- **HLSL/** - HLSL-specific passes

Each major construct (expr, stmt, decl) has dedicated files.

## Key Issues Identified

### 1. Semantic Analysis Module (`semantic/mod.rs`)

**Problem**: Monolithic `analyze_with_source` function (82 lines) mixes:

- Function signature extraction (first pass)
- Function body extraction (second pass)
- Validation (third pass)
- Type parsing utilities mixed with orchestration

**Issues**:

- No clear separation between passes
- Helper functions (`extract_function_signature`, `extract_function_body`, `parse_type_specifier`) are module-private but large
- Hard to test individual passes
- Difficult to add new passes (e.g., constant folding, dead code elimination)

**Reference Pattern** (DXC):

- Separate `SemaExpr.cpp`, `SemaStmt.cpp`, `SemaDecl.cpp`
- Clear visitor pattern for AST traversal
- Pass-based architecture (each pass is a distinct function/module)

### 2. Type Checking (`semantic/type_check.rs`)

**Problem**: 878 lines with mixed responsibilities:

- Type inference (`infer_expr_type`, `infer_binary_result_type`, `infer_unary_result_type`)
- Type conversion rules (`promote_numeric`, `can_implicitly_convert`)
- Constructor validation (`check_vector_constructor`, `check_matrix_constructor`)
- Swizzle parsing (`parse_swizzle_length`)
- Matrix operation type inference (`infer_matrix_binary_result_type`)

**Issues**:

- Too many concerns in one file
- Matrix operations are complex (270+ lines) and deserve separate module
- Constructor logic (vector/matrix) could be separate
- Swizzle parsing is implementation detail that could be abstracted

### 3. Expression Codegen (`codegen/expr.rs`)

**Problem**: 1339 lines handling everything:

- Literal translation
- Variable access
- Binary/unary operators
- Function calls (built-in, user, constructors)
- Vector/matrix operations
- Component access/swizzling

**Issues**:

- Matrix operations dominate (300+ lines)
- Vector operations are complex (200+ lines)
- Function call logic is spread throughout
- Hard to find specific operation implementations

**Reference Pattern** (DXC CodeGen):

- Separate files: `CodeGenExpr.cpp`, `CodeGenFunction.cpp`, `CodeGenStmt.cpp`
- Visitor pattern for AST traversal
- Helper classes for complex operations (e.g., `ScalarExprEmitter`)

### 4. JIT Module (`jit.rs`)

**Problem**: Duplicate logic between `compile_function` (223 lines) and `compile_main_function` (71 lines):

- Both create entry blocks
- Both set up function builder context
- Both handle default returns (duplicated 70+ lines)
- Main function is special case, not abstraction

**Issues**:

- Violates DRY principle
- Default return generation duplicated
- Main function handling could be cleaner (it's just a function with no parameters)

### 5. Pipeline Structure (`pipeline.rs`)

**Current**: Simple wrapper around parse → analyze

- Good start but could be more extensible
- No hooks for transformations
- No intermediate result types for debugging

**Opportunities**:

- Add transformation passes as separate steps
- Support multiple backends (JIT, static compilation, CLIF output)
- Better error propagation with context

## Comparison to DXC Architecture

### DXC Strengths

1. **Clear separation of concerns**: Parse → Sema → CodeGen
2. **Visitor patterns**: Easy to extend and test
3. **Pass-based**: Each transformation is a distinct pass
4. **Modular files**: One file per major construct type
5. **Diagnostic infrastructure**: Centralized error reporting

### Our Strengths

1. **Rust idioms**: Type safety, pattern matching
2. **Error types**: Structured errors with spans
3. **Pipeline abstraction**: Clean separation of parse/analyze/codegen
4. **no_std support**: Well-gated features

### Areas for Improvement

1. **Semantic analysis**: Needs pass-based architecture
2. **Codegen organization**: Split by construct type (expr, stmt, function)
3. **Type checking**: Separate concerns into modules
4. **JIT module**: Eliminate duplication

## Refactoring Priorities

1. **Phase 1**: Semantic analysis pass separation
2. **Phase 2**: Type checking module reorganization
3. **Phase 3**: Expression codegen splitting
4. **Phase 4**: JIT module cleanup
5. **Phase 5**: Pipeline extensibility
