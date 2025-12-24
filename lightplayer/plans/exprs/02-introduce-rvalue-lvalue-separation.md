# Phase 2: Introduce RValue/LValue Separation

## Goal

Separate LValue and RValue evaluation paths, following Clang's proven architecture. This provides clear separation of concerns and makes the code more maintainable.

## Current Architecture

**Problem**: Single `translate_expr_typed` method handles both LValues and RValues:
- Returns `(Vec<Value>, Type)` for both cases
- No clear distinction between "location" and "value"
- Assignment handling mixed with expression evaluation

## Target Architecture (Clang Pattern)

### RValue Type

```rust
/// Represents an RValue (right-hand value) - a computed value
pub enum RValue {
    /// Scalar value (single Value)
    Scalar(Value),
    /// Complex value (pair of Values for complex numbers)
    Complex(Value, Value),
    /// Aggregate value (vector, matrix, struct)
    Aggregate(Vec<Value>),
}

impl RValue {
    pub fn as_scalar(&self) -> Option<Value> { ... }
    pub fn as_aggregate(&self) -> Option<&[Value]> { ... }
    pub fn into_values(self) -> Vec<Value> { ... }
}
```

### LValue Type

```rust
/// Represents an LValue (left-hand value) - a modifiable location
/// Already exists in lvalue.rs, but needs to be integrated properly
pub enum LValue {
    Variable { vars: Vec<Variable>, ty: GlslType },
    Component { base_vars: Vec<Variable>, indices: Vec<usize>, ... },
    // ... existing variants
}
```

### Expression Evaluation Methods

```rust
impl CodegenContext {
    /// Emit code to compute an RValue (right-hand value)
    pub fn emit_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        match expr {
            Expr::Variable(..) => {
                // Read variable as RValue
                let lvalue = self.emit_lvalue(expr)?;
                self.load_lvalue(lvalue)
            }
            Expr::Binary(..) => {
                // Evaluate binary expression as RValue
                binary::emit_binary_rvalue(self, expr)
            }
            // ... other cases
        }
    }
    
    /// Emit code to compute an LValue (left-hand value - modifiable location)
    pub fn emit_lvalue(&mut self, expr: &Expr) -> Result<LValue, GlslError> {
        match expr {
            Expr::Variable(..) => {
                // Variable is an LValue
                lvalue::resolve_lvalue(self, expr)
            }
            Expr::Dot(..) => {
                // Component access is an LValue
                lvalue::resolve_lvalue(self, expr)
            }
            // ... other cases
        }
    }
    
    /// Load an LValue to get its RValue
    pub fn load_lvalue(&mut self, lvalue: LValue) -> Result<RValue, GlslError> {
        lvalue::read_lvalue(self, &lvalue)
    }
}
```

## Implementation Steps

### Step 1: Create RValue Type

1. Create `lightplayer/crates/lp-glsl/src/codegen/rvalue.rs`:
   - Define `RValue` enum (Scalar, Complex, Aggregate)
   - Implement conversion methods
   - Implement helper methods

2. Update `lightplayer/crates/lp-glsl/src/codegen/mod.rs`:
   - Export `rvalue` module

### Step 2: Refactor Expression Evaluation

1. Create `emit_rvalue` method:
   - Replace `translate_expr_typed` calls with `emit_rvalue`
   - Handle scalar/complex/aggregate cases
   - Return `RValue` instead of `(Vec<Value>, Type)`

2. Update expression handlers:
   - `binary::translate_binary` → `binary::emit_binary_rvalue`
   - `unary::translate_unary` → `unary::emit_unary_rvalue`
   - `literal::translate_literal` → `literal::emit_literal_rvalue`
   - etc.

3. Update variable reading:
   - `variable::translate_variable` → `variable::emit_variable_rvalue`
   - Or keep as LValue resolution + load

### Step 3: Integrate with LValue

1. Update `lvalue.rs`:
   - Ensure `read_lvalue` returns `RValue`
   - Ensure `write_lvalue` takes `RValue`

2. Update assignment handling:
   - `emit_assignment` uses `emit_lvalue` for LHS
   - `emit_assignment` uses `emit_rvalue` for RHS
   - Clear separation of concerns

### Step 4: Update Call Sites

1. Update statement translation:
   - `translate_selection` uses `emit_rvalue` for conditions
   - `translate_return` uses `emit_rvalue` for return values

2. Update expression contexts:
   - All places that call `translate_expr_typed` → `emit_rvalue`
   - Assignment contexts use `emit_lvalue` + `emit_rvalue`

## Files to Create

- `lightplayer/crates/lp-glsl/src/codegen/rvalue.rs` - RValue type and methods

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/mod.rs` - Export rvalue module
- `lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs` - Add `emit_rvalue` method
- `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs` - Refactor to `emit_binary_rvalue`
- `lightplayer/crates/lp-glsl/src/codegen/expr/unary.rs` - Refactor to `emit_unary_rvalue`
- `lightplayer/crates/lp-glsl/src/codegen/expr/literal.rs` - Refactor to `emit_literal_rvalue`
- `lightplayer/crates/lp-glsl/src/codegen/expr/variable.rs` - Refactor to use LValue + load
- `lightplayer/crates/lp-glsl/src/codegen/expr/component.rs` - Refactor to use LValue + load
- `lightplayer/crates/lp-glsl/src/codegen/expr/function.rs` - Update to return RValue
- `lightplayer/crates/lp-glsl/src/codegen/expr/constructor.rs` - Update to return RValue
- `lightplayer/crates/lp-glsl/src/codegen/lvalue.rs` - Update `read_lvalue` to return RValue
- `lightplayer/crates/lp-glsl/src/codegen/stmt.rs` - Update to use `emit_rvalue`
- `lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs` - Update assignment to use `emit_lvalue` + `emit_rvalue`

## Test Cases

- All existing tests should continue to pass
- Verify no regressions
- Test both LValue and RValue paths

## Acceptance Criteria

- [ ] `RValue` type created and integrated
- [ ] `emit_rvalue` method implemented
- [ ] `emit_lvalue` method properly separated
- [ ] All expression handlers updated
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] Clear separation between LValue and RValue evaluation

## Verification

Run all tests:
```bash
scripts/glsl-filetests.sh vec4/
```

Expected result: All tests pass, code is cleaner and more maintainable.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: introduce RValue/LValue separation following Clang patterns"
```

