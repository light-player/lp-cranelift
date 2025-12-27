# Ternary Operator Support

## Overview

Add support for GLSL ternary operator (`condition ? true_expr : false_expr`) by implementing type inference, validation, and codegen. Per GLSL spec (operators.adoc:926-945):

- Condition must be **scalar Boolean** (not vector bool)
- The two branch expressions can be any type (including void)
- Their types must match exactly OR there must be an implicit conversion that can make them match
- The result type is the matching type (after implicit conversion if needed)
- Implicit conversions follow variables.adoc:1182-1229 (int→uint/float/double, uint→float/double, float→double, and similar for vectors/matrices)

## Implementation Steps

### 1. Type Inference (`lightplayer/crates/lp-glsl/src/frontend/semantic/type_check/inference.rs`)

Add a case for `Expr::Ternary` in `infer_expr_type_with_registry`:

- Extract condition, true_expr, and false_expr from the ternary
- Validate condition type is scalar `Bool` (error if not scalar bool)
- Infer types of true_expr and false_expr recursively
- Determine result type:
  - If types match exactly, use that type
  - Else if `can_implicitly_convert(true_ty, false_ty)`, use `false_ty` as result type
  - Else if `can_implicitly_convert(false_ty, true_ty)`, use `true_ty` as result type
  - Else error: types don't match and no implicit conversion available
- Return the result type

Note: Unlike binary operators which use `promote_numeric`, ternary uses `can_implicitly_convert` which follows the GLSL spec implicit conversion table (one-way conversions, not bidirectional promotion).

### 2. Codegen Module (`lightplayer/crates/lp-glsl/src/frontend/codegen/expr/ternary.rs`)

Create new module with `emit_ternary_rvalue` function:

- Extract condition, true_expr, false_expr from `Expr::Ternary`
- Emit condition expression and validate it's scalar bool (must be single value, not vector)
- Emit true_expr and false_expr as RValues
- Get types of both branches
- Determine result type using same logic as type inference:
  - If types match exactly, use that type
  - Else determine which branch needs conversion using `can_implicitly_convert`
- Coerce branches to result type:
  - If true_ty != result_ty, coerce true_expr to result_ty
  - If false_ty != result_ty, coerce false_expr to result_ty
- Handle scalar case: use `builder.ins().select(cond_val, true_val, false_val)`
- Handle vector/matrix cases: select component-wise (similar to how vector binary ops work)
- Return RValue with result type

### 3. Expression Dispatch (`lightplayer/crates/lp-glsl/src/frontend/codegen/expr/mod.rs`)

- Add `ternary` module to imports
- Add `Expr::Ternary(..) => ternary::emit_ternary_rvalue(self, expr)` case in `emit_rvalue_impl`
- Remove ternary from the catch-all error case

### 4. Module Registration (`lightplayer/crates/lp-glsl/src/frontend/codegen/expr/mod.rs`)

Add `pub mod ternary;` to the module declarations.

## Key Implementation Details

- **Type Matching**: Use `can_implicitly_convert` from `conversion.rs` to check if one type can convert to another (one-way, not bidirectional like `promote_numeric`)
- **Condition Validation**: Condition must be **scalar** `Bool` type (not vector bool), same validation pattern as `if` statements but must ensure scalar
- **Codegen Pattern**: Use Cranelift's `select` instruction (`builder.ins().select(cond, true_val, false_val)`)
- **Type Coercion**: Coerce branches to result type before select (only coerce the branch that needs conversion)
- **Vector/Matrix/Struct Support**: 
  - For vectors: select component-wise (similar to how vector binary ops work)
  - For matrices: select component-wise (each component selected independently)
  - For structures: select member-wise (each struct member selected independently)
  - For arrays: select element-wise (each array element selected independently)
- **Implicit Conversions**: Follow GLSL spec implicit conversion table (int→uint/float/double, uint→float/double, float→double, and similar for vectors/matrices)
- **Short-Circuit Evaluation**: Note: Using Cranelift's `select` instruction means both branches are evaluated at codegen time. The spec says only one branch should be evaluated, but this is acceptable for initial implementation. Future optimization could use control flow (branches) for true short-circuit evaluation.

## Files to Modify

1. `lightplayer/crates/lp-glsl/src/frontend/semantic/type_check/inference.rs` - Add ternary type inference
2. `lightplayer/crates/lp-glsl/src/frontend/codegen/expr/mod.rs` - Add ternary module and dispatch
3. `lightplayer/crates/lp-glsl/src/frontend/codegen/expr/ternary.rs` - New file for ternary codegen

## Testing

Comprehensive test suite in `lightplayer/crates/lp-glsl-filetests/filetests/control/ternary/`:

- **basic.glsl**: Basic ternary operations with bool conditions and int results
- **types.glsl**: Support for vectors (vec2/vec3/vec4, ivec2, bvec2), matrices (mat2/mat3), and structures
- **type_conversions.glsl**: Implicit type conversions (int↔float, int↔bool, etc.)
- **nested.glsl**: Nested ternary operators with right-to-left associativity
- **precedence.glsl**: Precedence and associativity rules
- **edge_cases.glsl**: Edge cases (function calls, arithmetic operations, complex expressions)
- **short_circuit.glsl**: Short-circuit evaluation behavior (note: both branches evaluated with `select`)

