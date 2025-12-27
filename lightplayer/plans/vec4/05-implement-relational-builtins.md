# Implement Relational Builtin Functions

## Problem

Relational builtin functions for vectors are not implemented. These functions operate component-wise on vectors and return boolean vectors or scalars.

**Missing functions:**
- `all(bvec4)` → `bool` - returns true if all components are true
- `any(bvec4)` → `bool` - returns true if any component is true
- `greaterThan(vec4, vec4)` → `bvec4` - component-wise greater than
- `lessThan(vec4, vec4)` → `bvec4` - component-wise less than
- `greaterThanEqual(vec4, vec4)` → `bvec4` - component-wise greater than or equal
- `lessThanEqual(vec4, vec4)` → `bvec4` - component-wise less than or equal
- `not(bvec4)` → `bvec4` - component-wise logical NOT

**Affected tests:**
- `vec4/relational/all.glsl:14` - `test_vec4_all_all_true()` fails
- `vec4/relational/any.glsl:14` - `test_vec4_any_all_true()` fails
- `vec4/relational/greater-than.glsl:18` - `test_vec4_greater_than()` fails
- `vec4/relational/less-than.glsl:18` - `test_vec4_less_than()` fails
- `vec4/relational/greater-than-equal.glsl:18` - `test_vec4_greater_than_equal()` fails
- `vec4/relational/less-than-equal.glsl:18` - `test_vec4_less_than_equal()` fails
- `vec4/relational/not.glsl:14` - `test_vec4_not_all_true()` fails

## Root Cause

These functions are not registered in the builtin function lookup and don't have codegen implementations.

## Fix Strategy

1. **Add function signatures** to `semantic/builtins.rs`:
   - Add signatures for `all`, `any`, `greaterThan`, `lessThan`, `greaterThanEqual`, `lessThanEqual`, `not`
   - Define parameter types and return types

2. **Implement codegen** in `codegen/builtins/`:
   - Create new module `relational.rs` or add to existing module
   - Implement each function:
     - `all`: Check if all components are true (AND reduction)
     - `any`: Check if any component is true (OR reduction)
     - `greaterThan`: Component-wise `>` comparison
     - `lessThan`: Component-wise `<` comparison
     - `greaterThanEqual`: Component-wise `>=` comparison
     - `lessThanEqual`: Component-wise `<=` comparison
     - `not`: Component-wise logical NOT

3. **Register functions** in `codegen/builtins/mod.rs`:
   - Add cases to `translate_builtin_call` match statement

## Implementation Steps

1. **Add signatures** in `semantic/builtins.rs`:
   ```rust
   "all" => Some(vec![BuiltinSignature {
       name: "all",
       param_types: vec![BuiltinParamType::GenBType], // bvec2, bvec3, bvec4
       return_type: BuiltinReturnType::AlwaysBool,
   }]),
   // ... similar for other functions
   ```

2. **Create `codegen/builtins/relational.rs`**:
   - Implement `builtin_all`: AND all components together
   - Implement `builtin_any`: OR all components together
   - Implement `builtin_greater_than`: Component-wise `>` using `fcmp gt`
   - Implement `builtin_less_than`: Component-wise `<` using `fcmp lt`
   - Implement `builtin_greater_than_equal`: Component-wise `>=` using `fcmp ge`
   - Implement `builtin_less_than_equal`: Component-wise `<=` using `fcmp le`
   - Implement `builtin_not`: Component-wise NOT (XOR with true)

3. **Register in `codegen/builtins/mod.rs`**:
   ```rust
   "all" => self.builtin_all(args),
   "any" => self.builtin_any(args),
   "greaterThan" => self.builtin_greater_than(args),
   // ... etc
   ```

4. **Handle boolean vectors**:
   - Ensure boolean vectors are properly represented
   - Use i8 or i32 for boolean values (check existing code)

## Files to Modify

- `lightplayer/crates/lp-glsl/src/semantic/builtins.rs` - Add function signatures
- `lightplayer/crates/lp-glsl/src/codegen/builtins/mod.rs` - Register functions
- `lightplayer/crates/lp-glsl/src/codegen/builtins/relational.rs` - New file with implementations

## Test Cases

- `vec4/relational/all.glsl` - All tests should pass
- `vec4/relational/any.glsl` - All tests should pass
- `vec4/relational/greater-than.glsl` - All tests should pass
- `vec4/relational/less-than.glsl` - All tests should pass
- `vec4/relational/greater-than-equal.glsl` - All tests should pass
- `vec4/relational/less-than-equal.glsl` - All tests should pass
- `vec4/relational/not.glsl` - All tests should pass

## Acceptance Criteria

- [ ] All relational builtin function tests pass
- [ ] Functions work for vec2, vec3, vec4 (if tests exist)
- [ ] Functions work for ivec2, ivec3, ivec4 (if tests exist)
- [ ] No regressions in other tests
- [ ] Code compiles without warnings

## Verification

Run the tests:
```bash
scripts/glsl-filetests.sh vec4/relational/all.glsl
scripts/glsl-filetests.sh vec4/relational/any.glsl
scripts/glsl-filetests.sh vec4/relational/greater-than.glsl
scripts/glsl-filetests.sh vec4/relational/less-than.glsl
scripts/glsl-filetests.sh vec4/relational/greater-than-equal.glsl
scripts/glsl-filetests.sh vec4/relational/less-than-equal.glsl
scripts/glsl-filetests.sh vec4/relational/not.glsl
```

Expected result: All tests pass.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: implement relational builtin functions for vectors"
```


