# Vec4 Implementation Fixes - Overview

This directory contains plans for fixing vec4-related issues in the GLSL compiler.

## NOTES

- GLSL Spec is here: `/Users/yona/dev/photomancer/glsl-spec/chapters`
- Run tests with: `scripts/glsl-filetests.sh [filename]` or `--list` to see all tests, `--help` for help
- To run a specific test: `scripts/glsl-filetests.sh vec4/assignment/element-assignment.glsl`
- To run all vec4 tests: `scripts/glsl-filetests.sh vec4/`
- Target: `riscv32.fixed32`

## Current Test Status

**Before fixes:** 87 passed, 21 failed

**Failed tests:**
1. `vec4/assignment/element-assignment.glsl` - vector indexing not supported as LValue
2. `vec4/builtins/distance.glsl` - i64 sdiv not supported on riscv32
3. `vec4/builtins/faceforward.glsl` - undefined function `faceforward`
4. `vec4/builtins/length.glsl` - complex function identifiers not yet supported (v.length())
5. `vec4/builtins/normalize.glsl` - i64 sdiv not supported on riscv32
6. `vec4/builtins/reflect.glsl` - undefined function `reflect`
7. `vec4/builtins/refract.glsl` - undefined function `refract`
8. `vec4/constructors/shortening.glsl` - cannot construct vec2 from Vec4
9. `vec4/edge-cases/unit-vectors.glsl` - i64 sdiv not supported on riscv32
10. `vec4/edge-cases/zero-vector.glsl` - i64 sdiv not supported on riscv32
11. `vec4/indexing/array-indexing.glsl` - verifier error (non-dominating value)
12. `vec4/indexing/component-access.glsl` - verifier error (non-dominating value)
13. `vec4/relational/all.glsl` - undefined function `all`
14. `vec4/relational/any.glsl` - undefined function `any`
15. `vec4/relational/equal.glsl` - operator Equal not supported on vectors yet
16. `vec4/relational/greater-than-equal.glsl` - undefined function `greaterThanEqual`
17. `vec4/relational/greater-than.glsl` - undefined function `greaterThan`
18. `vec4/relational/less-than-equal.glsl` - undefined function `lessThanEqual`
19. `vec4/relational/less-than.glsl` - undefined function `lessThan`
20. `vec4/relational/not-equal.glsl` - operator NonEqual not supported on vectors yet
21. `vec4/relational/not.glsl` - undefined function `not`

## Issues Identified

1. **01-fix-vector-indexing-lvalue.md** - `v[0] = float` not supported as LValue
2. **02-fix-verifier-errors.md** - Codegen issues causing non-dominating value errors
3. **03-fix-i64-division.md** - `length` and `distance` generating i64 division on riscv32
4. **04-implement-vector-shortening.md** - `vec2(vec4)` constructor not supported
5. **05-implement-relational-builtins.md** - `all`, `any`, `greaterThan`, `lessThan`, etc. not implemented
6. **06-implement-geometric-builtins.md** - `faceforward`, `reflect`, `refract` not implemented
7. **07-implement-vector-comparison-operators.md** - `==` and `!=` operators not supported on vectors
8. **08-implement-method-calls.md** - `v.length()` method calls not supported

## Priority Order

**Critical (affects correctness):**

1. Fix vector indexing LValue (01) - blocks assignment via array indexing
2. Fix verifier errors (02) - compilation failures in indexing tests
3. Fix i64 division (03) - blocks `length` and `distance` functions

**Missing Features:**

4. Implement vector shortening (04) - compilation error
5. Implement relational builtins (05) - missing functions
6. Implement geometric builtins (06) - missing functions
7. Implement vector comparison operators (07) - operator support
8. Implement method calls (08) - method call syntax

## Dependencies

- Plan 01 (vector indexing LValue) should be done first as it affects assignment
- Plan 02 (verifier errors) should be done early as it blocks tests
- Plan 03 (i64 division) blocks geometric functions that use `length`
- Plan 05 (relational builtins) depends on Plan 07 (vector comparison) for some operations
- Plan 08 (method calls) is independent but lower priority

## Test Commands

### Run all vec4 tests:
```bash
scripts/glsl-filetests.sh vec4/
```

### Run specific test file:
```bash
scripts/glsl-filetests.sh vec4/assignment/element-assignment.glsl
```

### Run specific test case:
```bash
scripts/glsl-filetests.sh vec4/assignment/element-assignment.glsl:58
```

### List all tests:
```bash
scripts/glsl-filetests.sh --list
```

### Get help:
```bash
scripts/glsl-filetests.sh --help
```

## Commit Instructions

After completing each phase:

1. **Verify all tests pass:**
   ```bash
   scripts/glsl-filetests.sh vec4/
   ```

2. **Commit with appropriate message:**
   ```bash
   git add -A
   git commit -m "lpc: [phase description]"
   ```

   Example:
   ```bash
   git commit -m "lpc: implement vector indexing as LValue"
   ```

3. **Keep commits small and focused** - one logical change per commit




