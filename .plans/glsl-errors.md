# GLSL Frontend Error Handling

Implement Rust-style error handling for the GLSL frontend with error codes, clear descriptions, and source location/span information. Based on glslang's simple approach but with Rust-inspired error codes and formatting.

## Approach

Following **glslang's** lightweight approach (TSourceLoc + InfoSink) rather than DXC's heavyweight DiagnosticsEngine, but adding:

- Rust-style error codes (E0001, E0002, etc.)
- Structured error types with source locations
- Clear span/line highlighting
- Optional spec references (future expansion)

## Implementation Plan

### 1. Create Error Infrastructure Module

Create `crates/lp-glsl/src/error.rs`:

- **SourceLocation** struct (inspired by glslang's TSourceLoc):

  - `line: usize`
  - `column: usize`
  - `filename: Option<String>` (for multi-file support later)
  - Best-effort: gracefully degrade if parser doesn't provide span info

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
      // ... more as needed

      // Transform errors (E0300-E0399)
      E0300, // Fixed-point transformation error

      // Codegen errors (E0400-E0499)
      E0400, // Codegen error
  }
  ```

- **GlslError** struct:

  - `code: ErrorCode`
  - `message: String` (primary error description)
  - `location: Option<SourceLocation>` (optional for non-source errors)
  - `span_text: Option<String>` (the actual source line, if available)
  - `notes: Vec<String>` (additional context/hints)
  - `spec_ref: Option<String>` (for future: link to GLSL spec section)

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

- Change `Result<T, String>` to `Result<T, GlslError>`
- Extract location from GLSL AST nodes where possible (check if glsl-parser provides this)
- Update all error sites to use structured errors:

  ```rust
  // Old:
  Err(format!("Undefined variable: {}", name))

  // New:
  Err(GlslError::new(ErrorCode::E0100, format!("undefined variable `{}`", name))
      .with_location(loc)
      .with_note(format!("variable `{}` is not defined in this scope", name)))
  ```

### 3. Update Type Checker

Modify `crates/lp-glsl/src/semantic/type_check.rs`:

- Use `GlslError` for all type errors
- Include both operand types and expected types in error messages
- Add helpful notes for common mistakes (e.g., "help: use explicit cast: `float(x)`")

### 4. Update Transform Errors

Modify `crates/lp-glsl/src/transform/fixed_point.rs`:

- Replace `TransformError` with `GlslError`
- Use structured error codes

### 5. Update Codegen

Modify `crates/lp-glsl/src/codegen/*.rs`:

- Change `Result<T, String>` to `Result<T, GlslError>`
- Use appropriate error codes

### 6. Update Public APIs

Modify `crates/lp-glsl/src/compiler.rs` and `crates/lp-glsl/src/jit.rs`:

- For `std` feature: convert `GlslError` to `String` at the boundary (backward compatibility)
- Consider adding a `compile_with_diagnostics()` method that returns full `GlslError`

### 7. Parser Integration

Check glsl-parser for span information:

- If available: extract and use it
- If not: use best-effort approach (track line numbers during semantic analysis)
- Parse errors: wrap external parser errors in `GlslError` with `E0001`

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

- **New**: `crates/lp-glsl/src/error.rs` - Core error infrastructure
- `crates/lp-glsl/src/lib.rs` - Export error types
- `crates/lp-glsl/src/semantic/mod.rs` - Use GlslError
- `crates/lp-glsl/src/semantic/type_check.rs` - Structured type errors
- `crates/lp-glsl/src/semantic/scope.rs` - Variable lookup errors
- `crates/lp-glsl/src/semantic/functions.rs` - Function errors
- `crates/lp-glsl/src/codegen/*.rs` - Codegen errors
- `crates/lp-glsl/src/transform/fixed_point.rs` - Replace TransformError
- `crates/lp-glsl/src/compiler.rs` - Update API
- `crates/lp-glsl/src/jit.rs` - Update API
- `crates/lp-glsl-filetests/src/test_error.rs` - Update test infrastructure

## Design Decisions

1. **Error code style**: `E0001` format (like Rust) - familiar to developers
2. **Location tracking**: Line/column when available, gracefully degrade otherwise
3. **Output format**: Plain text by default, no color dependency (can add feature-gated later)
4. **Backward compatibility**: Keep `String` API for existing callers, add new `*_with_diagnostics()` methods
5. **No external dependencies**: Use simple custom implementation, avoid `codespan-reporting` to keep it lightweight

## Dependencies

This plan depends on **glsl-spans.md** for full span information from the parser. The error handling can be implemented with best-effort location tracking initially and enhanced once span support is added to the parser.
