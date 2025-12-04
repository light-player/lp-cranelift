# Add Span Information to glsl-parser

Add source location/span tracking to the glsl-parser fork at `/Users/yona/dev/photomancer/glsl-parser` to enable precise error reporting.

## Current State

The glsl-parser currently:

- Uses `nom` for parsing
- Has no span information in AST nodes
- Only provides basic `ParseError` with a String message
- `nom`'s `convert_error` gives location but it's lost after parsing

## Goal

Add span information to all AST nodes so the GLSL compiler can report errors with:

- Line and column numbers
- Source spans for highlighting
- Multi-line span support

## Approach

Follow **nom's `nom_locate`** pattern used in other parsers, similar to how glslang tracks `TSourceLoc`.

## Implementation Plan

### 1. Add Span Type

In `glsl/src/syntax.rs`, add:

```rust
/// Source location information
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    /// Byte offset from start of input
    pub offset: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Length of the span in bytes
    pub len: usize,
}

impl SourceSpan {
    pub fn new(offset: usize, line: usize, column: usize, len: usize) -> Self {
        Self { offset, line, column, len }
    }

    pub fn unknown() -> Self {
        Self { offset: 0, line: 0, column: 0, len: 0 }
    }

    pub fn is_unknown(&self) -> bool {
        self.line == 0 && self.column == 0
    }
}
```

### 2. Add nom_locate Dependency

In `glsl/Cargo.toml`:

```toml
[dependencies]
nom = "7.1"
nom_locate = "4.2"  # Add this
```

### 3. Create Span-Aware Input Type

In `glsl/src/parsers.rs`, add:

```rust
use nom_locate::LocatedSpan;

/// Input type with location tracking
pub type Span<'a> = LocatedSpan<&'a str>;

/// Parser result with span tracking
pub type SpanResult<'a, T> = IResult<Span<'a>, T, VerboseError<Span<'a>>>;

/// Helper to extract SourceSpan from nom's LocatedSpan
fn to_source_span(span: Span, len: usize) -> SourceSpan {
    SourceSpan::new(
        span.location_offset(),
        span.location_line() as usize,
        span.get_column(),
        len,
    )
}

/// Helper to get span between two positions
fn span_between(start: Span, end: Span) -> SourceSpan {
    let len = end.location_offset() - start.location_offset();
    to_source_span(start, len)
}
```

### 4. Update AST Nodes to Include Spans

**Option A: Add span field to key nodes** (lightweight, recommended)

Add `span` field to important AST nodes:

```rust
// In syntax.rs
pub struct Identifier {
    pub name: String,
    pub span: SourceSpan,  // Add this
}

pub enum Expr {
    Variable(Identifier, SourceSpan),  // Add span
    IntConst(i32, SourceSpan),
    FloatConst(f32, SourceSpan),
    BoolConst(bool, SourceSpan),
    Binary(BinaryOp, Box<Expr>, Box<Expr>, SourceSpan),
    // ... etc
}

pub struct FunctionDefinition {
    pub prototype: FunctionPrototype,
    pub statement: CompoundStatement,
    pub span: SourceSpan,  // Add this
}
```

**Option B: Wrap all nodes** (more invasive, more complete)

Create a generic wrapper:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: SourceSpan,
}
```

### 5. Update Parsers to Track Spans

Update all parsers in `glsl/src/parsers.rs` to use `Span` input and track locations:

```rust
// Before:
pub fn identifier(i: &str) -> ParserResult<'_, syntax::Identifier> {
    // ...
}

// After:
pub fn identifier(i: Span) -> SpanResult<'_, syntax::Identifier> {
    let start = i;
    let (i, name) = /* parse name */;
    let span = span_between(start, i);

    Ok((i, syntax::Identifier {
        name: name.to_string(),
        span,
    }))
}
```

Key parsers to update:

- `identifier` - track identifier locations
- `literal` parsers - track literal locations
- `expr` parsers - track expression locations
- `statement` parsers - track statement locations
- `type_specifier` - track type locations
- `function_definition` - track function locations

### 6. Update Parser Entry Point

In `glsl/src/parser.rs`, update `run_parser`:

```rust
pub(crate) fn run_parser<P, T>(source: &str, parser: P) -> Result<T, ParseError>
where
  P: FnOnce(Span) -> SpanResult<T>,
{
    let span = Span::new(source);
    match parser(span) {
        Ok((_, x)) => Ok(x),
        Err(e) => {
            // nom_locate provides better error messages with line/col
            match e {
                NomErr::Incomplete(_) => Err(ParseError {
                    info: String::from("incomplete parser"),
                    span: None,
                }),
                NomErr::Error(err) | NomErr::Failure(err) => {
                    let info = convert_error(source, err);
                    // Extract span from error if available
                    let span = err.errors.first().map(|(span, _)| {
                        SourceSpan::new(
                            span.location_offset(),
                            span.location_line() as usize,
                            span.get_column(),
                            1,
                        )
                    });
                    Err(ParseError { info, span })
                }
            }
        }
    }
}
```

### 7. Update ParseError

In `glsl/src/parser.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub info: String,
    pub span: Option<SourceSpan>,  // Add this
}
```

### 8. Update Tests

- Update tests to account for new span fields
- Add tests specifically for span correctness
- Create helper functions for building test AST nodes with dummy spans

In `glsl/src/parse_tests.rs`:

```rust
// Helper for tests that don't care about spans
impl SourceSpan {
    pub fn dummy() -> Self {
        Self::unknown()
    }
}

// Helper to compare AST ignoring spans
pub fn ast_eq_ignore_spans<T: PartialEq>(a: &T, b: &T) -> bool {
    // Compare with spans set to dummy
    // ...
}
```

### 9. Maintain Backward Compatibility

Add feature flag for spans (optional):

In `glsl/Cargo.toml`:

```toml
[features]
default = ["std"]
std = []
spans = ["nom_locate"]  # Make spans optional
```

When `spans` feature is disabled, use dummy/zero spans:

```rust
#[cfg(feature = "spans")]
pub type ParserInput<'a> = Span<'a>;

#[cfg(not(feature = "spans"))]
pub type ParserInput<'a> = &'a str;
```

## Files to Modify

**In `/Users/yona/dev/photomancer/glsl-parser/glsl/`:**

- `Cargo.toml` - Add nom_locate dependency
- `src/syntax.rs` - Add SourceSpan type, update AST nodes
- `src/parser.rs` - Update run_parser, ParseError
- `src/parsers.rs` - Update all parsers to use Span and track locations
- `src/parsers/nom_helpers.rs` - Add span helper functions
- `src/parse_tests.rs` - Update tests
- `tests/*.rs` - Update integration tests

## Migration Strategy

1. **Phase 1**: Add SourceSpan type and nom_locate dependency
2. **Phase 2**: Update core parsers (identifier, literals) to track spans
3. **Phase 3**: Update expression parsers
4. **Phase 4**: Update statement and declaration parsers
5. **Phase 5**: Update tests
6. **Phase 6**: Verify all parsers produce correct spans

## Testing

Create comprehensive span tests:

```rust
#[test]
fn test_identifier_span() {
    let source = "foo";
    let ident = Identifier::parse(source).unwrap();
    assert_eq!(ident.span.line, 1);
    assert_eq!(ident.span.column, 1);
    assert_eq!(ident.span.len, 3);
}

#[test]
fn test_multiline_span() {
    let source = "int x = 5;\nfloat y = 3.14;";
    let tu = TranslationUnit::parse(source).unwrap();
    // Verify second declaration is on line 2
    // ...
}
```

## Dependencies

This plan should be implemented **before** or **in parallel** with the glsl-errors.md plan. The error handling can use placeholder locations initially and be enhanced once spans are available.

## References

- nom_locate docs: https://docs.rs/nom_locate/
- Similar implementation: https://github.com/rust-bakery/nom/blob/main/examples/s_expression.rs
- glslang TSourceLoc: `/Users/yona/dev/photomancer/glslang/glslang/Include/Common.h:235`
