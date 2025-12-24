# Phase 5: Extract Expression Helpers (Idiomatic Rust Refactoring)

## Goal

Refactor `emit_rvalue` to use idiomatic Rust patterns by extracting helper functions for complex and repeated logic. This improves maintainability and readability while keeping the code idiomatic (using match statements rather than visitor patterns).

## Current Architecture

**Current State**: `emit_rvalue` contains a large match statement with some complex nested logic:

- Nested match for `Unary` expressions (handles Inc/Dec specially)
- Repeated "resolve LValue then load" pattern (Variable, Dot, Bracket)
- Some arms are simple delegations, others have inline logic

**Problem**:

- Nested match makes `Unary` handling harder to read
- Repeated patterns reduce maintainability
- Some complex logic is inline rather than extracted

## Target Architecture (Idiomatic Rust)

### Principles

1. **Keep match statements** - They're exhaustive, type-safe, and idiomatic Rust
2. **Extract complex logic** - Move nested matches and repeated patterns to helpers
3. **Public helpers for testing** - Make helpers public so they can be tested independently
4. **Documentation** - Add doc comments to all helpers
5. **Common patterns** - Extract repeated patterns like "resolve LValue then load"

### Structure

```rust
impl<'a> CodegenContext<'a> {
    /// Public API - main entry point for RValue evaluation
    pub fn emit_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        self.ensure_block()?;
        self.emit_rvalue_impl(expr)
    }

    /// Internal dispatch - clean match statement delegating to helpers
    fn emit_rvalue_impl(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        match expr {
            // Simple delegations stay inline
            Expr::IntConst(..) | Expr::FloatConst(..) | Expr::BoolConst(..) => {
                literal::emit_literal_rvalue(self, expr)
            }
            Expr::Binary(..) => binary::emit_binary_rvalue(self, expr),
            Expr::FunCall(..) => function::emit_function_call_rvalue(self, expr),

            // Complex cases delegate to helpers
            Expr::Variable(..) => self.emit_variable_rvalue(expr),
            Expr::Unary(..) => self.emit_unary_rvalue(expr),
            Expr::Dot(..) => self.emit_component_access_rvalue(expr),
            Expr::Bracket(..) => self.emit_matrix_indexing_rvalue(expr),
            Expr::Assignment(..) => self.emit_assignment_rvalue(expr),
            Expr::PostInc(..) => self.emit_postinc_rvalue(expr),
            Expr::PostDec(..) => self.emit_postdec_rvalue(expr),

            _ => Err(GlslError::new(ErrorCode::E0400, format!("expression not supported yet: {:?}", expr))),
        }
    }

    // Public helpers (for testing)

    /// Emit variable expression as RValue
    ///
    /// Reads a variable by resolving it as an LValue, then loading its value.
    pub fn emit_variable_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        self.emit_lvalue_as_rvalue(expr)
    }

    /// Emit unary expression as RValue
    ///
    /// Handles pre-increment/decrement specially, delegates other unary operations
    /// to the unary module.
    pub fn emit_unary_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        let Expr::Unary(op, operand, span) = expr else {
            unreachable!("emit_unary_rvalue called on non-unary expr");
        };

        use glsl::syntax::UnaryOp::*;
        match op {
            Inc => {
                let (vals, ty) = incdec::translate_preinc(self, operand, span.clone())?;
                Ok(RValue::from_aggregate(vals, ty))
            }
            Dec => {
                let (vals, ty) = incdec::translate_predec(self, operand, span.clone())?;
                Ok(RValue::from_aggregate(vals, ty))
            }
            _ => unary::emit_unary_rvalue(self, expr),
        }
    }

    /// Emit component access expression as RValue
    ///
    /// Handles dot notation (e.g., `vec.x`, `vec.xy`) by resolving as LValue then loading.
    pub fn emit_component_access_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        self.emit_lvalue_as_rvalue(expr)
    }

    /// Emit matrix/vector indexing expression as RValue
    ///
    /// Handles bracket notation (e.g., `vec[0]`, `mat[0][1]`) by resolving as LValue then loading.
    pub fn emit_matrix_indexing_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        self.emit_lvalue_as_rvalue(expr)
    }

    /// Emit assignment expression as RValue
    ///
    /// Evaluates an assignment expression and returns the assigned value(s) as RValue.
    pub fn emit_assignment_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        let Expr::Assignment(lhs, op, rhs, _span) = expr else {
            unreachable!("emit_assignment_rvalue called on non-assignment expr");
        };
        let (vals, ty) = self.translate_assignment_typed(lhs, op, rhs)?;
        Ok(RValue::from_aggregate(vals, ty))
    }

    /// Emit post-increment expression as RValue
    ///
    /// Returns the original value before incrementing (post-increment semantics).
    pub fn emit_postinc_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        let Expr::PostInc(operand, span) = expr else {
            unreachable!("emit_postinc_rvalue called on non-postinc expr");
        };
        let (vals, ty) = incdec::translate_postinc(self, operand, span.clone())?;
        Ok(RValue::from_aggregate(vals, ty))
    }

    /// Emit post-decrement expression as RValue
    ///
    /// Returns the original value before decrementing (post-decrement semantics).
    pub fn emit_postdec_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        let Expr::PostDec(operand, span) = expr else {
            unreachable!("emit_postdec_rvalue called on non-postdec expr");
        };
        let (vals, ty) = incdec::translate_postdec(self, operand, span.clone())?;
        Ok(RValue::from_aggregate(vals, ty))
    }

    /// Common pattern: resolve expression as LValue, then load it as RValue
    ///
    /// This pattern is used for Variable, Dot, and Bracket expressions.
    /// First resolves the expression to a modifiable location (LValue),
    /// then reads the current value(s) from that location.
    pub fn emit_lvalue_as_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        let lvalue = self.emit_lvalue(expr)?;
        self.load_lvalue(lvalue)
    }
}
```

## Implementation Steps

### Step 1: Extract Common Pattern Helper

**File**: [`lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs`](lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs)

1. Add `emit_lvalue_as_rvalue` helper method:

   - Public for testing
   - Documented with doc comment
   - Extracts the repeated "resolve LValue then load" pattern

2. Update `emit_rvalue` to use this helper for Variable, Dot, and Bracket cases

### Step 2: Extract Unary Expression Helper

**File**: [`lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs`](lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs)

1. Add `emit_unary_rvalue` helper method:

   - Public for testing
   - Documented with doc comment
   - Extracts the nested match logic for Inc/Dec handling
   - Delegates to `unary::emit_unary_rvalue` for other unary ops

2. Update `emit_rvalue` to delegate Unary case to this helper

### Step 3: Extract Assignment Helper

**File**: [`lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs`](lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs)

1. Add `emit_assignment_rvalue` helper method:

   - Public for testing
   - Documented with doc comment
   - Wraps `translate_assignment_typed` call

2. Update `emit_rvalue` to delegate Assignment case to this helper

### Step 4: Extract Post-Increment/Decrement Helpers

**File**: [`lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs`](lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs)

1. Add `emit_postinc_rvalue` helper method:

   - Public for testing
   - Documented with doc comment
   - Wraps `incdec::translate_postinc` call

2. Add `emit_postdec_rvalue` helper method:

   - Public for testing
   - Documented with doc comment
   - Wraps `incdec::translate_postdec` call

3. Update `emit_rvalue` to delegate PostInc/PostDec cases to these helpers

### Step 5: Extract Component Access and Matrix Indexing Helpers

**File**: [`lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs`](lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs)

1. Add `emit_component_access_rvalue` helper:

   - Public for testing
   - Documented with doc comment
   - Uses `emit_lvalue_as_rvalue` helper

2. Add `emit_matrix_indexing_rvalue` helper:

   - Public for testing
   - Documented with doc comment
   - Uses `emit_lvalue_as_rvalue` helper

3. Update `emit_rvalue` to delegate Dot/Bracket cases to these helpers

### Step 6: Extract Variable Helper

**File**: [`lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs`](lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs)

1. Add `emit_variable_rvalue` helper:

   - Public for testing
   - Documented with doc comment
   - Uses `emit_lvalue_as_rvalue` helper

2. Update `emit_rvalue` to delegate Variable case to this helper

### Step 7: Refactor Main Match Statement

**File**: [`lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs`](lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs)

1. Split `emit_rvalue` into:

   - Public `emit_rvalue` - calls `ensure_block()` then delegates
   - Private `emit_rvalue_impl` - contains the match statement

2. Clean up match statement:
   - Keep simple delegations inline (Literal, Binary, FunCall)
   - Delegate complex cases to helpers
   - Ensure all cases are covered

## Files to Modify

- [`lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs`](lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs) - Extract helpers and refactor match statement

## Benefits

1. **Idiomatic Rust**: Uses match statements (exhaustive, type-safe) rather than visitor pattern
2. **Maintainability**: Complex logic extracted to named helpers
3. **Readability**: Match statement becomes cleaner and easier to scan
4. **Testability**: Helpers are public and can be tested independently
5. **DRY**: Common "resolve LValue then load" pattern extracted
6. **Documentation**: Each helper has clear documentation

## Testing Strategy

1. **No functional changes**: This is a pure refactoring - all existing tests should pass
2. **Run test suite**: Execute `scripts/glsl-filetests.sh vec4/` to verify no regressions
3. **Optional unit tests**: Can add unit tests for helpers if complex logic warrants it (filetests cover correctness)

## Notes

- This is a pure refactoring - no changes to code generation logic
- Existing expression handler modules (literal, binary, unary, etc.) remain unchanged
- Helpers are public for testing but could be made private if preferred
- Match statement remains the primary dispatch mechanism (idiomatic Rust)
- Future expression types can be easily added by adding a match arm

## Acceptance Criteria

- [ ] `emit_lvalue_as_rvalue` helper extracted and used in 3 places
- [ ] `emit_unary_rvalue` helper extracted with nested match logic
- [ ] `emit_assignment_rvalue` helper extracted
- [ ] `emit_postinc_rvalue` and `emit_postdec_rvalue` helpers extracted
- [ ] `emit_component_access_rvalue` and `emit_matrix_indexing_rvalue` helpers extracted
- [ ] `emit_variable_rvalue` helper extracted
- [ ] `emit_rvalue` split into public API and private implementation
- [ ] Match statement cleaned up and all cases covered
- [ ] All helpers have doc comments
- [ ] All tests pass (no regressions)
- [ ] Code compiles without warnings

## Verification

Run all tests to ensure no regressions:

```bash
scripts/glsl-filetests.sh vec4/
scripts/glsl-filetests.sh matrix/
```

Expected result: All tests pass, code is more maintainable and idiomatic.

## Commit Instructions

After completing the refactoring:

```bash
git add -A
git commit -m "lpc: extract expression helpers for better maintainability"
```

Keep the commit focused on the refactoring - no functional changes.
