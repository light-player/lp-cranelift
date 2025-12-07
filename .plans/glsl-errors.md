# GLSL Frontend Error Handling

Implement structured error handling for the GLSL frontend with error codes, clear descriptions, and source location/span information. Based on glslang's simple approach but with Rust-inspired error codes and formatting.

## Approach

Following **glslang's** lightweight approach (TSourceLoc + InfoSink) rather than DXC's heavyweight DiagnosticsEngine, but adding:

- Rust-style error codes (E0001, E0002, etc.)
- Structured error types with source locations
- Clear span/line highlighting
- Optional spec references (future expansion)

## Prerequisites

The glsl-parser fork already has span support implemented in the `feature/spans` branch. We need to:

1. **Update dependency** to use `branch = "feature/spans"` in `Cargo.toml`
2. **Extract spans** from AST nodes (Expr, Identifier, Statement, etc.) which already have `span: SourceSpan` fields
3. **Convert spans** from `glsl::syntax::SourceSpan` to our `SourceLocation` type
4. **Store source text** during compilation to enable span text extraction for error display

## Implementation Plan

### 0. Update Dependency to feature/spans Branch

**First step**: Update `crates/lp-glsl/Cargo.toml`:

```toml
glsl = { git = "https://github.com/Yona-Appletree/glsl-parser.git", branch = "feature/spans", default-features = false }
```

### 1. Create Error Infrastructure Module

Create `crates/lp-glsl/src/error.rs`:

- **SourceLocation** struct (inspired by glslang's TSourceLoc):

  - `line: usize`
  - `column: usize`
  - `filename: Option<String>` (for multi-file support later)

- **ErrorCode** enum with variants for each error category:

  ```rust
  pub enum ErrorCode {
      // Parse errors (E0001-E0099) - passed through from parser
      E0001, // Parse error

      // Semantic errors (E0100-E0299)
      E0100, // Undefined variable
      E0101, // Undefined function
      E0102, // Type mismatch
      E0103, // Cannot implicitly convert
      E0104, // Wrong argument count
      E0105, // Wrong argument type
      E0106, // Incompatible types for operator
      E0107, // Condition must be bool
      E0108, // No main function
      E0109, // Unsupported type
      E0110, // Invalid vector constructor
      E0111, // Component out of range
      E0112, // Invalid component access
      E0113, // Invalid swizzle
      E0114, // No matching function
      E0115, // Cannot assign
      E0116, // Return type mismatch
      // ... more as needed

      // Transform errors (E0300-E0399)
      E0300, // Fixed-point transformation error
      E0301, // Verification failed after transformation

      // Codegen errors (E0400-E0499)
      E0400, // Codegen error
      E0401, // Verification error
  }
  ```

- **GlslError** struct:

  - `code: ErrorCode`
  - `message: String` (primary error description)
  - `location: Option<SourceLocation>` (optional for non-source errors)
  - `span_text: Option<String>` (the actual source line, if available)
  - `notes: Vec<String>` (additional context/hints)
  - `spec_ref: Option<String>` (for future: link to GLSL spec section)

- **Span extraction helpers**:

  ```rust
  /// Convert glsl::syntax::SourceSpan to our SourceLocation
  pub fn source_span_to_location(span: &glsl::syntax::SourceSpan) -> SourceLocation {
      SourceLocation::new(span.line, span.column)
  }

  /// Extract span from an expression
  pub fn extract_span_from_expr(expr: &glsl::syntax::Expr) -> glsl::syntax::SourceSpan {
      match expr {
          glsl::syntax::Expr::Variable(_, span) => *span,
          glsl::syntax::Expr::IntConst(_, span) => *span,
          glsl::syntax::Expr::FloatConst(_, span) => *span,
          glsl::syntax::Expr::BoolConst(_, span) => *span,
          glsl::syntax::Expr::Binary(_, _, _, span) => *span,
          glsl::syntax::Expr::Unary(_, _, span) => *span,
          glsl::syntax::Expr::Assignment(_, _, _, span) => *span,
          glsl::syntax::Expr::FunCall(_, _, span) => *span,
          // ... etc for all variants
      }
  }

  /// Extract span from an identifier
  pub fn extract_span_from_identifier(ident: &glsl::syntax::Identifier) -> glsl::syntax::SourceSpan {
      ident.span
  }

  /// Extract source line text from a span
  pub fn extract_source_line(source: &str, span: &glsl::syntax::SourceSpan) -> Option<String> {
      if span.is_unknown() {
          return None;
      }
      source.lines().nth(span.line.saturating_sub(1)).map(|s| s.to_string())
  }
  ```

- **Display impl** for GlslError (Rust-style formatting):
  ```
  error[E0100]: undefined variable `foo`
   --> shader.glsl:5:10
    |
  5 |     int x = foo + 1;
    |             ^^^ undefined variable
    |
   note: did you mean `bar`?
  ```

### 2. Update Semantic Analysis

Modify `crates/lp-glsl/src/semantic/mod.rs` and related files:

- **Store source text**: Add a parameter or field to track the original source string
- Change `Result<T, String>` to `Result<T, GlslError>`
- **Extract spans from AST nodes**: Use helper functions to get `SourceSpan` from:
  - `Identifier.span` for variable references
  - `Expr` variants (each has a `SourceSpan` field)
  - `Statement` variants
  - `FunctionDefinition.span`
- Convert `glsl::syntax::SourceSpan` to `SourceLocation` using helper
- Update all error sites to use structured errors:

  ```rust
  // Old:
  Err(format!("Undefined variable: {}", name))

  // New:
  let span = extract_span_from_identifier(ident);
  let loc = source_span_to_location(&span);
  Err(GlslError::undefined_variable(name)
      .with_location(loc)
      .with_span_text(extract_source_line(&source_text, &span))
      .with_note(format!("variable `{}` is not defined in this scope", name)))
  ```

### 3. Update Type Checker

Modify `crates/lp-glsl/src/semantic/type_check.rs`:

- Use `GlslError` for all type errors
- Extract spans from expressions being type-checked
- Include both operand types and expected types in error messages
- Add helpful notes for common mistakes (e.g., "help: use explicit cast: `float(x)`")

### 4. Update Transform Errors

Modify `crates/lp-glsl/src/transform/fixed_point.rs`:

- Replace `TransformError` with `GlslError`
- Use structured error codes (E0300, E0301)
- Note: Transform errors may not have source spans (they operate on Cranelift IR)

### 5. Update Codegen

Modify `crates/lp-glsl/src/codegen/*.rs`:

- Change `Result<T, String>` to `Result<T, GlslError>`
- Use appropriate error codes (E0400, E0401)
- Note: Codegen errors may not have source spans (they operate on Cranelift IR)

### 6. Update Public APIs

Modify `crates/lp-glsl/src/compiler.rs` and `crates/lp-glsl/src/jit.rs`:

- **Store source text**: Pass source string through compilation pipeline
- For `std` feature: convert `GlslError` to `String` at the boundary (backward compatibility)
- Consider adding a `compile_with_diagnostics()` method that returns full `GlslError`

### 7. Parser Integration

The glsl-parser `feature/spans` branch already provides `SourceSpan` in AST nodes:

- **Parse errors**: Wrap external parser errors in `GlslError` with `E0001`
- Extract span from parse error if available (check if `ParseError` has span info)
- If parser error doesn't have span, use `SourceLocation::unknown()`

### 8. Update Tests

Modify `crates/lp-glsl-filetests/src/test_error.rs`:

- Update to check for error codes in addition to message patterns
- Add directive support for error codes:
  ```glsl
  // EXPECT_ERROR: E0100
  // EXPECT_ERROR_MSG: undefined variable
  ```

### 9. Documentation

- Add error code documentation in `crates/lp-glsl/src/error.rs`
- Document each error code with explanation and examples

## Files to Modify

- **New**: `crates/lp-glsl/src/error.rs` - Core error infrastructure + span extraction helpers
- `crates/lp-glsl/Cargo.toml` - **Update dependency to `branch = "feature/spans"`**
- `crates/lp-glsl/src/lib.rs` - Export error types
- `crates/lp-glsl/src/semantic/mod.rs` - Use GlslError, store source text, extract spans
- `crates/lp-glsl/src/semantic/type_check.rs` - Structured type errors with spans
- `crates/lp-glsl/src/semantic/scope.rs` - Variable lookup errors with spans
- `crates/lp-glsl/src/semantic/functions.rs` - Function errors with spans
- `crates/lp-glsl/src/codegen/*.rs` - Codegen errors (may not have spans)
- `crates/lp-glsl/src/transform/fixed_point.rs` - Replace TransformError
- `crates/lp-glsl/src/compiler.rs` - Update API, pass source text through
- `crates/lp-glsl/src/jit.rs` - Update API, pass source text through
- `crates/lp-glsl-filetests/src/test_error.rs` - Update test infrastructure

## Design Decisions

1. **Error code style**: `E0001` format (like Rust) - familiar to developers
2. **Location tracking**: Use spans from glsl-parser when available, gracefully degrade otherwise
3. **Output format**: Plain text by default, no color dependency (can add feature-gated later)
4. **Backward compatibility**: Keep `String` API for existing callers, add new `*_with_diagnostics()` methods
5. **No external dependencies**: Use simple custom implementation, avoid `codespan-reporting` to keep it lightweight
6. **Source text storage**: Store original source string during compilation to enable span text extraction

## Dependencies

The glsl-parser `feature/spans` branch already has full span support implemented. We need to:

1. **Update dependency** to use `branch = "feature/spans"` in `Cargo.toml`
2. **Extract spans** from AST nodes (Expr, Identifier, Statement, etc.) which already have `span: SourceSpan` fields
3. **Convert spans** from `glsl::syntax::SourceSpan` to our `SourceLocation` type
4. **Store source text** during compilation to enable span text extraction for error display

The glsl-parser implementation is complete - we just need to integrate it into our error handling system.
