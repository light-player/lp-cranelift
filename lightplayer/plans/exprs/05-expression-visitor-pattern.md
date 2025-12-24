# Phase 5: Expression Visitor Pattern (Optional)

## Goal

Refactor expression evaluation to use a visitor pattern, improving maintainability and extensibility. This is optional but recommended for long-term maintainability.

## Current Architecture

**Problem**: Large match statement in `emit_rvalue`:
- All expression types handled in one place
- Hard to extend
- Hard to test individual expression types

## Target Architecture (Visitor Pattern)

### Expression Visitor Trait

```rust
/// Trait for visiting expressions and generating code
pub trait ExprVisitor {
    type Result;
    
    fn visit_literal(&mut self, expr: &Expr) -> Result<Self::Result, GlslError>;
    fn visit_variable(&mut self, expr: &Expr) -> Result<Self::Result, GlslError>;
    fn visit_binary(&mut self, expr: &Expr) -> Result<Self::Result, GlslError>;
    fn visit_unary(&mut self, expr: &Expr) -> Result<Self::Result, GlslError>;
    fn visit_function_call(&mut self, expr: &Expr) -> Result<Self::Result, GlslError>;
    fn visit_component_access(&mut self, expr: &Expr) -> Result<Self::Result, GlslError>;
    fn visit_matrix_indexing(&mut self, expr: &Expr) -> Result<Self::Result, GlslError>;
    // ... other expression types
}

/// RValue visitor - evaluates expressions as RValues
pub struct RValueVisitor<'a> {
    ctx: &'a mut CodegenContext<'a>,
}

impl ExprVisitor for RValueVisitor {
    type Result = RValue;
    
    fn visit_literal(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        literal::emit_literal_rvalue(self.ctx, expr)
    }
    
    fn visit_variable(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        // Variable as RValue: resolve as LValue, then load
        let lvalue = lvalue::resolve_lvalue(self.ctx, expr)?;
        lvalue::read_lvalue(self.ctx, &lvalue)
    }
    
    // ... other methods
}

/// LValue visitor - evaluates expressions as LValues
pub struct LValueVisitor<'a> {
    ctx: &'a mut CodegenContext<'a>,
}

impl ExprVisitor for LValueVisitor {
    type Result = LValue;
    
    fn visit_variable(&mut self, expr: &Expr) -> Result<LValue, GlslError> {
        lvalue::resolve_lvalue(self.ctx, expr)
    }
    
    fn visit_component_access(&mut self, expr: &Expr) -> Result<LValue, GlslError> {
        lvalue::resolve_lvalue(self.ctx, expr)
    }
    
    // ... other methods
}
```

### Expression Dispatch

```rust
impl CodegenContext {
    /// Visit expression with RValue visitor
    pub fn visit_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        let mut visitor = RValueVisitor { ctx: self };
        match expr {
            Expr::IntConst(..) | Expr::FloatConst(..) | Expr::BoolConst(..) => {
                visitor.visit_literal(expr)
            }
            Expr::Variable(..) => {
                visitor.visit_variable(expr)
            }
            Expr::Binary(..) => {
                visitor.visit_binary(expr)
            }
            // ... other cases dispatch to visitor
        }
    }
    
    /// Visit expression with LValue visitor
    pub fn visit_lvalue(&mut self, expr: &Expr) -> Result<LValue, GlslError> {
        let mut visitor = LValueVisitor { ctx: self };
        match expr {
            Expr::Variable(..) => {
                visitor.visit_variable(expr)
            }
            Expr::Dot(..) => {
                visitor.visit_component_access(expr)
            }
            // ... other cases dispatch to visitor
        }
    }
}
```

## Implementation Steps

### Step 1: Create Visitor Trait

1. Create `lightplayer/crates/lp-glsl/src/codegen/expr/visitor.rs`:
   - Define `ExprVisitor` trait
   - Define methods for each expression type
   - Document visitor pattern

### Step 2: Implement RValue Visitor

1. Create `RValueVisitor`:
   - Implement `ExprVisitor` for `RValueVisitor`
   - Delegate to existing expression handlers
   - Return `RValue` for all cases

2. Update expression handlers:
   - Keep existing handlers
   - Visitor calls handlers
   - No changes to handler logic

### Step 3: Implement LValue Visitor

1. Create `LValueVisitor`:
   - Implement `ExprVisitor` for `LValueVisitor`
   - Delegate to `lvalue::resolve_lvalue`
   - Return `LValue` for all cases

### Step 4: Refactor Expression Dispatch

1. Update `emit_rvalue`:
   - Use `visit_rvalue` instead of match statement
   - Dispatch to visitor

2. Update `emit_lvalue`:
   - Use `visit_lvalue` instead of match statement
   - Dispatch to visitor

### Step 5: Test and Verify

1. Run all tests:
   - Verify no regressions
   - Verify visitor pattern works correctly

2. Add tests for visitor:
   - Test individual expression types
   - Test visitor dispatch

## Files to Create

- `lightplayer/crates/lp-glsl/src/codegen/expr/visitor.rs` - Visitor trait and implementations

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs` - Use visitor pattern
- `lightplayer/crates/lp-glsl/src/codegen/mod.rs` - Export visitor module

## Benefits

1. **Maintainability**: Each expression type handled separately
2. **Extensibility**: Easy to add new expression types
3. **Testability**: Can test individual expression types
4. **Clarity**: Clear separation of concerns

## Test Cases

- All existing tests should continue to pass
- Test visitor dispatch
- Test individual expression types

## Acceptance Criteria

- [ ] Visitor trait created
- [ ] RValue visitor implemented
- [ ] LValue visitor implemented
- [ ] Expression dispatch uses visitor pattern
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] Code is more maintainable

## Verification

Run all tests:
```bash
scripts/glsl-filetests.sh vec4/
```

Expected result: All tests pass, code is more maintainable.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: refactor expression evaluation to visitor pattern"
```

## Notes

This phase is optional but recommended for long-term maintainability. It can be done independently of other phases.

