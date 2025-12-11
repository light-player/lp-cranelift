# Math Function Intrinsics in GLSL Compiler

## Overview

Instead of handling math function replacement in the fixed-point transform, we'll generate math function implementations directly in the GLSL compiler. This provides cleaner separation of concerns:

- **Compiler**: Handles math function implementations (sin, cos, etc.) - generates them as regular GLSL functions
- **Fixed-point transform**: Only handles type conversions (float -> fixed), doesn't need to know about math functions

## Acceptance Criteria

**PRIMARY GOAL**: `filetests/math/sine.glsl` must pass with validation.

The sine function test establishes the pattern for all other math functions:

- ✅ Compile test: CLIF shows `__lp_sin` function (not external `%sinf` call)
- ✅ Run test: Correct values at key points (sin(0)=0, sin(π/2)=1, sin(π)=0, sin(3π/2)=-1)
- ✅ Pattern: Test structure can be copied for cos, tan, etc.

**Implementation order**: Remove unneeded call conversion code FIRST, then add intrinsics.

## Goals

- Write intrinsic implementations in GLSL using **float types** (not fixed-point)
- Use existing float->fixed conversion to handle fixed-point support
- Avoid multiple implementations (one GLSL implementation works for both float and fixed-point)
- Make fixed64 support easier later (same GLSL code, different type conversion)
- Generate intrinsics behind a default-true feature flag
- Simplify fixed-point transform (no call replacement needed)

## Current State

- `get_math_libcall()` in `crates/lp-glsl/src/codegen/builtins/helpers.rs` creates external function calls
- Math functions (sin, cos, etc.) in `crates/lp-glsl/src/codegen/builtins/trigonometric.rs` call `get_math_libcall()`
- Fixed-point transform currently needs to detect and replace these external calls
- Intrinsic implementations would be written in GLSL using floats

## Implementation Plan

**Priority Order**: Remove unneeded code FIRST, then add intrinsics. This ensures we don't build on top of code that will be removed.

### Phase 1: Clean up fixed-point transform (FIRST)

#### 1.1 Remove math function call conversion code from fixed-point transform

**File**: `crates/lp-glsl/src/transform/fixed_point/converters/calls.rs`

- **Remove** `get_function_name()` helper (no longer needed)
- **Remove** math function detection logic (the `if name == "sinf"` block)
- **Remove** `get_or_create_sin_fixed()` call and related code
- Keep only the generic external function conversion logic
- Fixed-point transform should now only handle:
  - Type conversions (float -> fixed)
  - Signature conversions for external functions
  - No special handling for math functions

**File**: `crates/lp-glsl/src/transform/fixed_point/rewrite.rs`

- **Remove** `created_functions` and `created_bodies` fields from `RewriteContext`
- **Remove** initialization of these maps
- **Remove** passing these maps to `convert_call()`
- Simplify `convert_instruction()` - no need to pass intrinsic-related maps

**File**: `crates/lp-glsl/src/transform/fixed_point/functions.rs`

- **Remove** entire file (or keep as placeholder for future if needed)
- All math function handling moves to compiler

**File**: `crates/lp-glsl/src/transform/fixed_point/fixed/`

- **Remove** entire directory (CLIF files no longer needed)
- Intrinsic implementations will be in GLSL format

**Verification**: After this phase, fixed-point transform should compile and work for non-math functions. Math functions will temporarily fail (expected - we'll fix in Phase 2).

### Phase 2: Add intrinsic infrastructure

### 2. Add feature flag for intrinsic implementations

**File**: `crates/lp-glsl/Cargo.toml`

- Add feature flag: `intrinsic-math` (default = true)
- This allows disabling intrinsics to use external calls if needed

```toml
[features]
default = ["std", "intrinsic-math"]
intrinsic-math = []  # Enable intrinsic math function implementations
```

### 2. Create GLSL intrinsic implementations

**New directory structure**:

```
crates/lp-glsl/src/intrinsics/
  trig.glsl              # Trigonometric functions: __lp_sin, __lp_cos, __lp_tan, etc.
  common.glsl            # Common functions: __lp_sqrt, __lp_pow, etc. (future)
  hyperbolic.glsl        # Hyperbolic functions: __lp_sinh, __lp_cosh, etc. (future)
```

**File**: `crates/lp-glsl/src/intrinsics/trig.glsl`

- Write implementations using **float types** (not fixed-point)
- Functions use `__lp_` prefix for internal/compiler-generated functions
- Example for sine:

  ```glsl
  // Helper: Reduce angle to [0, π/2] range
  vec2 __lp_reduce_angle(float angle) {
      // ... angle reduction logic using floats ...
      return vec2(reduced_angle, quadrant);
  }

  // Helper: CORDIC rotation
  vec2 __lp_cordic_rotation(float angle) {
      // ... CORDIC algorithm using floats ...
      return vec2(sin_val, cos_val);
  }

  // Main sine function
  float __lp_sin(float angle) {
      vec2 reduced = __lp_reduce_angle(angle);
      vec2 result = __lp_cordic_rotation(reduced.x);
      // ... apply quadrant transformations ...
      return sin_result;
  }
  ```

**Key points**:

- All functions use `float` types (not `int` or fixed-point)
- Functions can call each other naturally within the same file
- Single implementation works for both float and fixed-point (via type conversion)

### 3. Add intrinsic function compiler infrastructure

**File**: `crates/lp-glsl/src/intrinsics/mod.rs` (new)

- Module to handle intrinsic function compilation
- Function: `compile_intrinsic_functions(glsl_source: &str, isa: &dyn TargetIsa) -> Result<HashMap<String, Function>, GlslError>`
  - Parses GLSL source containing intrinsic functions
  - Compiles each function into a `Function` object
  - Returns map of function name -> `Function` object
  - Does NOT add functions to module (that happens lazily)

**File**: `crates/lp-glsl/src/intrinsics/loader.rs` (new)

- Function: `get_or_create_intrinsic(func_name: &str, ctx: &mut CodegenContext, cache: &mut IntrinsicCache) -> Result<FuncRef, GlslError>`
  - Checks cache for already-compiled function
  - If not found:
    - Loads appropriate GLSL file (e.g., `trig.glsl`)
    - Compiles all functions in that file
    - Caches compiled functions
    - Extracts requested function
    - Declares function in module
    - Returns `FuncRef`
  - Uses `IntrinsicCache` to track compiled functions per module

### 4. Modify `get_math_libcall()` to use intrinsics

**File**: `crates/lp-glsl/src/codegen/builtins/helpers.rs`

- Modify `get_math_libcall()` to check feature flag
- If `intrinsic-math` feature enabled:
  - Call `get_or_create_intrinsic()` instead of creating external call
  - Map function names: `"sinf"` -> `"__lp_sin"`, `"cosf"` -> `"__lp_cos"`, etc.
- If feature disabled:
  - Use current behavior (create external call)

**Function signature change**:

```rust
pub fn get_math_libcall(&mut self, func_name: &str) -> Result<FuncRef, GlslError> {
    #[cfg(feature = "intrinsic-math")]
    {
        // Use intrinsic implementation
        let intrinsic_name = map_to_intrinsic_name(func_name)?;
        get_or_create_intrinsic(&intrinsic_name, self, &mut self.intrinsic_cache)
    }

    #[cfg(not(feature = "intrinsic-math"))]
    {
        // Use external call (current behavior)
        // ... existing code ...
    }
}
```

### 5. Add intrinsic cache to CodegenContext

**File**: `crates/lp-glsl/src/codegen/context.rs`

- Add `intrinsic_cache: Option<IntrinsicCache>` field to `CodegenContext`
- `IntrinsicCache` tracks compiled intrinsic functions per module
- Structure:
  ```rust
  pub struct IntrinsicCache {
      compiled_functions: HashMap<String, Function>,  // Function name -> Function object
      module_func_refs: HashMap<String, FuncRef>,    // Function name -> FuncRef in module
  }
  ```

### 6. Update fixed-point transform

**File**: `crates/lp-glsl/src/transform/fixed_point/converters/calls.rs`

- **Remove** math function detection and replacement logic
- Fixed-point transform now only handles:
  - Type conversions (float -> fixed)
  - Signature conversions for external functions
  - No special handling for math functions needed

**File**: `crates/lp-glsl/src/transform/fixed_point/rewrite.rs`

- Remove `created_functions` and `created_bodies` maps (no longer needed)
- Simplify `convert_instruction()` - no need to pass intrinsic-related maps

### 7. Add filetests with validation (ACCEPTANCE CRITERIA)

**File**: `lightplayer/crates/lp-glsl-filetests/filetests/math/sine.glsl` (NEW - establishes pattern)

This is the **acceptance criteria** - sine function must work in a filetest with validation.

**Test structure** (establishes pattern for other math functions):

```glsl
// test compile
// test run

float main() {
    // Test key sine values
    float result = 0.0;

    // sin(0) = 0
    result += sin(0.0);

    // sin(π/2) = 1
    float pi_2 = 1.570796327;
    result += sin(pi_2);

    // sin(π) = 0
    float pi = 3.141592654;
    result += sin(pi);

    // sin(3π/2) = -1
    float pi_3_2 = 4.712388981;
    result += sin(pi_3_2);

    return result;  // Should be 0.0
}

// Expected CLIF should show:
// - Function __lp_sin is present (not external call)
// - Helper functions __lp_reduce_angle, __lp_cordic_rotation may be present
// - No external calls to "sinf"

// run: ~= 0.0  (tolerance: 0.001)
```

**Key validation points**:

1. **Compile test**: CLIF output shows `__lp_sin` function (not external `%sinf` call)
2. **Run test**: All sine values are correct within tolerance
3. **Pattern**: This structure can be copied for other math functions (cos, tan, etc.)

**File**: `lightplayer/crates/lp-glsl-filetests/filetests/math/sine.glsl` (NEW)

Test that fixed-point conversion works with intrinsic functions:

```glsl
// test compile
// test fixed32

float main() {
    float pi_2 = 1.570796327;  // π/2
    return sin(pi_2);  // Should be ~1.0
}

// Expected CLIF should show:
// - Function __lp_sin with i32 types (fixed-point)
// - Fixed-point operations (imul, ishr, etc.) instead of float operations
// - No external calls

// run: ~= 1.0  (tolerance: 0.01 for fixed-point)
```

**Test organization**:

- `filetests/math/` - New directory for math function tests
- `sine.glsl` - Float version (establishes pattern)
- `sine_fixed32.glsl` - Fixed-point version
- Future: `cos.glsl`, `tan.glsl`, etc. following same pattern

**Test Pattern** (established by `sine.glsl`):

1. **File structure**:

   ```glsl
   // test compile
   // test run

   float main() {
       // Test multiple key values
       // Return aggregated result or individual test
   }

   // Expected CLIF output in comments (use BLESS to generate)

   // run: ~= expected_value  (with tolerance)
   ```

2. **Validation points**:

   - Compile test verifies intrinsic function is used (not external call)
   - Run test verifies correctness at key mathematical points
   - Tolerance specified for floating-point precision

3. **Reusable pattern**: Copy this structure for `cos.glsl`, `tan.glsl`, etc.

## Key Implementation Details

### Function Name Mapping

```rust
fn map_to_intrinsic_name(libcall_name: &str) -> Result<&str, GlslError> {
    match libcall_name {
        "sinf" => Ok("__lp_sin"),
        "cosf" => Ok("__lp_cos"),
        "tanf" => Ok("__lp_tan"),
        "asinf" => Ok("__lp_asin"),
        "acosf" => Ok("__lp_acos"),
        "atanf" => Ok("__lp_atan"),
        "sinhf" => Ok("__lp_sinh"),
        "coshf" => Ok("__lp_cosh"),
        "tanhf" => Ok("__lp_tanh"),
        // ... etc
        _ => Err(GlslError::new(ErrorCode::E0400, format!("Unknown math function: {}", libcall_name)))
    }
}
```

### Intrinsic Compilation Pattern

```rust
fn compile_intrinsic_functions(
    glsl_source: &str,
    isa: &dyn TargetIsa,
) -> Result<HashMap<String, Function>, GlslError> {
    // 1. Parse and analyze GLSL
    let semantic_result = CompilationPipeline::parse_and_analyze(glsl_source)?;

    // 2. For each function in the GLSL source:
    let mut functions = HashMap::new();
    for func in &semantic_result.typed_ast.user_functions {
        // Compile function into Function object
        let compiled_func = compile_single_function(func, isa)?;
        functions.insert(func.name.clone(), compiled_func);
    }

    // Also compile main function if it exists (for standalone testing)
    // ...

    Ok(functions)
}
```

### Lazy Function Loading

- Intrinsic functions are compiled on-demand when first needed
- All functions in a GLSL file are compiled together (for inter-function calls)
- Compiled functions are cached per module to avoid recompilation

### Fixed-Point Compatibility

- Intrinsic functions use `float` types
- When fixed-point transform runs:
  - Function signatures are converted (float -> fixed)
  - Function bodies are converted (float operations -> fixed operations)
  - No special handling needed - works automatically!

## Files to Create/Modify

### Phase 1: Cleanup (FIRST)

**Files to Remove/Modify**:

1. `crates/lp-glsl/src/transform/fixed_point/converters/calls.rs` - Remove math function detection/replacement
2. `crates/lp-glsl/src/transform/fixed_point/rewrite.rs` - Remove `created_functions`/`created_bodies` maps
3. `crates/lp-glsl/src/transform/fixed_point/functions.rs` - Remove entire file
4. `crates/lp-glsl/src/transform/fixed_point/fixed/` - Remove entire directory (CLIF files)

### Phase 2: Implementation

**New Files**:

1. `crates/lp-glsl/src/intrinsics/mod.rs` - Module exports
2. `crates/lp-glsl/src/intrinsics/loader.rs` - Intrinsic function loader
3. `crates/lp-glsl/src/intrinsics/compiler.rs` - GLSL to Function compiler
4. `crates/lp-glsl/src/intrinsics/trig.glsl` - Trigonometric function implementations (float-based)
5. `lightplayer/crates/lp-glsl-filetests/filetests/math/sine.glsl` - **ACCEPTANCE CRITERIA**: Sine test with validation
6. `lightplayer/crates/lp-glsl-filetests/filetests/math/sine_fixed32.glsl` - Fixed-point sine test

**Modified Files**:

1. `crates/lp-glsl/Cargo.toml` - Add `intrinsic-math` feature flag
2. `crates/lp-glsl/src/codegen/builtins/helpers.rs` - Modify `get_math_libcall()` to use intrinsics
3. `crates/lp-glsl/src/codegen/context.rs` - Add `intrinsic_cache` field

## Success Criteria

### Phase 1: Cleanup

- [ ] **Fixed-point transform simplified**: All math function call conversion code removed
- [ ] **No compilation errors**: Fixed-point transform compiles without removed code
- [ ] **Non-math functions work**: Fixed-point transform still works for non-math external functions

### Phase 2: Implementation

- [ ] **Feature flag works**: Can disable intrinsics to use external calls
- [ ] **Intrinsic compilation**: GLSL intrinsic functions compile correctly
- [ ] **Function calls**: Math function calls use intrinsic implementations when enabled
- [ ] **ACCEPTANCE CRITERIA**: `filetests/math/sine.glsl` passes both compile and run tests
  - Compile test: CLIF shows `__lp_sin` function (not external `%sinf` call)
  - Run test: All sine values correct (sin(0)=0, sin(π/2)=1, sin(π)=0, sin(3π/2)=-1)
- [ ] **Fixed-point compatibility**: `filetests/math/sine_fixed32.glsl` passes
  - Compile test: Shows fixed-point types (i32) in `__lp_sin`
  - Run test: Correct values with fixed-point precision
- [ ] **Pattern established**: Test structure in `sine.glsl` can be copied for other functions
- [ ] **No call replacement**: Fixed-point transform doesn't detect/replace math calls (verified by absence of code)

## Benefits

1. **Cleaner separation**: Compiler handles math functions, transform handles types
2. **Single implementation**: One GLSL implementation works for both float and fixed-point
3. **Easier fixed64**: Same GLSL code, different type conversion
4. **Simpler transform**: Fixed-point transform is much simpler
5. **Feature flag**: Can disable intrinsics if needed for compatibility

## Future Work

- Add more intrinsic implementations (sqrt, pow, exp, log, etc.)
- Optimize intrinsic implementations
- Consider inlining small intrinsics
- Add vectorized versions for better performance
