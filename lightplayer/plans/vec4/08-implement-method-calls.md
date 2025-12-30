# Implement Method Calls (v.length())

## Problem

Method calls like `v.length()` are not supported. The error is: "complex function identifiers not yet supported".

**Current behavior:**
- `v.length()` fails with compilation error
- Expected: Should return the number of components (4 for vec4)

**Affected tests:**
- `vec4/builtins/length.glsl:14` - `test_vec4_length_method()` fails

## Root Cause

In `lightplayer/crates/lp-glsl/src/codegen/expr/function.rs` or similar, method calls (member function calls) are not implemented. The parser/type checker likely doesn't recognize `v.length()` as a valid expression.

GLSL spec allows:
- `v.length()` → returns number of components (compile-time constant)
- This is different from `length(v)` which computes the magnitude

## Fix Strategy

1. **Update parser/ast** to recognize method calls:
   - `Expr::MethodCall { object, method, args }` or similar
   - Parse `v.length()` as a method call

2. **Update type checker** to handle method calls:
   - Recognize `.length()` as a valid method
   - Return type is `int` (compile-time constant)

3. **Update codegen** to handle method calls:
   - For `.length()`, return the component count
   - This is a compile-time constant, so no runtime code needed

## Implementation Steps

1. **Check parser** (`parser/` or `syntax/`):
   - See if method calls are already parsed
   - If not, add parsing for `expr.method(args)`

2. **Check type checker** (`semantic/type_check/`):
   - Add handling for method calls
   - For `.length()`, verify it's called on a vector/matrix
   - Return type is `int`

3. **Check codegen** (`codegen/expr/function.rs` or similar):
   - Add handling for method calls
   - For `.length()`, return constant component count

4. **Test with different types**:
   - `vec2.length()` → 2
   - `vec3.length()` → 3
   - `vec4.length()` → 4
   - `mat2.length()` → 2 (if supported)

## Files to Modify

- Parser files (if method calls aren't parsed)
- `lightplayer/crates/lp-glsl/src/semantic/type_check/` - Type checking
- `lightplayer/crates/lp-glsl/src/codegen/expr/function.rs` - Codegen

## Test Cases

- `vec4/builtins/length.glsl:14` - `test_vec4_length_method()` should pass
- Verify `vec2.length()` returns 2
- Verify `vec3.length()` returns 3
- Verify `vec4.length()` returns 4

## Acceptance Criteria

- [ ] `test_vec4_length_method()` passes
- [ ] All tests in `vec4/builtins/length.glsl` pass
- [ ] Method calls work for vec2, vec3, vec4
- [ ] No regressions in other tests
- [ ] Code compiles without warnings

## Verification

Run the test:
```bash
scripts/glsl-filetests.sh vec4/builtins/length.glsl:14
```

Expected result: Test passes, `v.length()` returns 4 for vec4.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: implement method calls (v.length())"
```



