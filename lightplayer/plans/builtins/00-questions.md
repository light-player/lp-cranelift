# GLSL Intrinsics Implementation - Questions

This document captures questions that need to be answered before implementing the full math intrinsics system. After reviewing the current codebase, GLSL spec, and the overview document, these questions need clarification.

## Multi-File Source Support

### Q1: How should intrinsic files be integrated with multi-file source support?

**Context**: The codebase now has `GlSourceMap` and multi-file support (`add_file()`). Intrinsic files are currently loaded via `include_str!()` in `loader.rs`.

**Questions**:

- Should intrinsic files be added to the main `GlSourceMap` used for user code compilation?
- Or should intrinsics maintain their own separate source map (as currently done in `compile_intrinsic_functions()`)?
- How should intrinsic file IDs relate to user file IDs? Should they share the same namespace or be separate?

**Current State**:

- `compile_intrinsic_functions()` creates its own `GlSourceMap` with a synthetic file ID
- User code compilation creates a separate `GlSourceMap` for the main source
- Intrinsics are compiled separately and linked into the module

**Suggested Answer**:

- Intrinsics should be added to the main `GlSourceMap` when loaded
- Use `GlFileSource::Intrinsic(name)` to mark them as intrinsic files
- This allows unified error reporting and source location tracking
- Intrinsic file IDs should be in the same namespace as user files

**DECISION**: ✅ **Confirmed** - Intrinsics will be added to the main `GlSourceMap`. The `GlSourceMap` should be passed in when compiling intrinsics.

---

### Q2: Should intrinsic files support cross-file dependencies?

**Context**: The overview mentions that functions in the same intrinsic file can call each other (e.g., `__lp_cos` calls `__lp_sin`). But what if `exponential.glsl` needs to call `sqrt()` from a `common.glsl` file?

**Questions**:

- Should we support intrinsic files calling functions from other intrinsic files?
- If yes, should dependencies be explicit (declared in the file) or automatic (loaded when needed)?
- How should we handle circular dependencies?

**Current State**:

- Only same-file function calls are supported
- Each intrinsic file is compiled independently

**Suggested Answer**:

- Phase 1: Start with same-file only (current approach)
- Phase 2: Add explicit dependency support if needed
- Use a dependency graph to detect cycles and load files in order

**DECISION**: ✅ **Confirmed** - Start with same-file only. Cross-file dependencies can be added later if needed.

---

## Function Coverage and Implementation

### Q3: Which math functions need intrinsic implementations vs. can use simple operations?

**Context**: The GLSL spec defines many built-in functions. Some are simple operations (e.g., `abs`, `min`, `max`) while others require complex algorithms (e.g., `sin`, `exp`, `log`).

**Questions**:

- Which functions MUST have intrinsic implementations (cannot be implemented with simple Cranelift instructions)?
- Which functions can be implemented directly in codegen without intrinsics?
- Should we have a clear categorization?

**Current State**:

- Trigonometric functions (`sin`, `cos`, `tan`, etc.) use intrinsics when `intrinsic-math` feature is enabled
- Simple functions (`abs`, `min`, `max`, `sqrt`, `floor`, `ceil`) are implemented directly in codegen
- `pow` is implemented directly but might benefit from intrinsic

**Suggested Answer**:

- **Must have intrinsics**: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`, `exp`, `log`, `exp2`, `log2`, `pow` (complex cases)
- **Can use direct codegen**: `abs`, `sign`, `floor`, `ceil`, `fract`, `mod`, `min`, `max`, `clamp`, `radians`, `degrees`, `sqrt` (if hardware instruction available), `inversesqrt`
- **Decision criteria**: If it requires iterative algorithms, lookup tables, or complex approximations → intrinsic. If it's a simple operation → direct codegen.

**DECISION**: ✅ **Confirmed** - This categorization is correct. `sqrt` and `inversesqrt` can use direct codegen if hardware instructions are available.

---

### Q4: How should we handle exponential and logarithmic functions?

**Context**: The spec defines `exp`, `log`, `exp2`, `log2`, `pow`, `sqrt`, `inversesqrt`. These are mathematically complex.

**Questions**:

- Should all exponential functions go in one `exponential.glsl` file?
- What algorithms should we use? (Taylor series? CORDIC? Lookup tables?)
- Should `pow` be implemented using `exp(log(x) * y)` or have its own algorithm?
- What precision/accuracy requirements do we have?

**Current State**:

- No exponential functions implemented as intrinsics yet
- `pow` is implemented directly in codegen (likely using external library calls)

**Suggested Answer**:

- Create `exponential.glsl` with: `exp`, `log`, `exp2`, `log2`, `pow`, `sqrt`, `inversesqrt`
- Use polynomial approximations or iterative methods (similar to CORDIC for trig)
- `pow(x, y)` can use `exp(log(x) * y)` for most cases, with special handling for integer powers
- Target IEEE 754 single-precision accuracy (±1 ULP where possible)

**DECISION**: ✅ **Confirmed** - Use `exponential.glsl` for all exponential functions. Use polynomial/iterative approximations. For `pow`, use `exp(log(x) * y)` for now (a dedicated algorithm could be faster but can be added later if needed). Target ±2-3 ULP initially, optimize to ±1 ULP later.

---

### Q5: Should we implement vectorized versions or rely on component-wise calls?

**Context**: GLSL built-ins work component-wise on vectors. Currently, codegen calls intrinsics for each component separately.

**Questions**:

- Should intrinsic functions handle vector types directly (e.g., `vec3 __lp_sin(vec3 x)`)?
- Or continue with component-wise calls from codegen (current approach)?
- What are the performance implications?

**Current State**:

- Intrinsics are scalar (`float __lp_sin(float x)`)
- Codegen calls them for each component of vectors

**Suggested Answer**:

- **Phase 1**: Keep component-wise approach (simpler, works correctly)
- **Phase 2**: Consider vectorized intrinsics if performance analysis shows benefit
- Component-wise is easier to implement and test, and GLSL semantics are component-wise anyway

**DECISION**: ✅ **Confirmed** - Keep component-wise approach for now. Vectorized optimizations can come later if profiling shows benefit.

---

## Implementation Details

### Q6: How should we handle `atan2` (two-argument atan)?

**Context**: GLSL has both `atan(x)` and `atan(y, x)`. The two-arg version is currently handled separately with `get_atan2_libcall()`.

**Questions**:

- Should `atan2` be an intrinsic function in `trig.glsl`?
- Or should it remain as an external library call?
- How should it relate to single-arg `atan`?

**Current State**:

- `atan2` uses `get_atan2_libcall()` which creates an external function call
- Single-arg `atan` uses `get_math_libcall("atanf")` → intrinsic

**Suggested Answer**:

- Implement `__lp_atan2(float y, float x)` in `trig.glsl`
- Update `get_atan2_libcall()` to use intrinsic when `intrinsic-math` feature is enabled
- Can implement using `atan(y/x)` with quadrant handling, or use a dedicated algorithm

**DECISION**: ✅ **Confirmed** - `atan2` should be an intrinsic in `trig.glsl`, as should all trig functions. Use `atan(y/x)` with quadrant handling for now.

---

### Q7: How should we handle global constants in intrinsic files?

**Context**: The overview mentions that global constants might be needed (e.g., PI). Currently, constants are inlined in function bodies.

**Questions**:

- Should we extract `const` declarations from intrinsic GLSL files?
- Or continue inlining constants in function bodies?
- If we extract them, how should they be made available to intrinsic functions?

**Current State**:

- Constants are inlined (e.g., `float pi = 3.14159265358979323846;` in `__lp_reduce_angle`)
- No global constant extraction

**Suggested Answer**:

- **Phase 1**: Continue inlining constants (simpler, works fine)
- **Phase 2**: Extract constants if we need to share them across files or if they're large
- Constants are small and inlining is more efficient anyway

**DECISION**: ✅ **Confirmed** - Continue inlining constants in function bodies. Global variable support hasn't been added yet, so constants must stay in functions.

---

### Q8: How should we test intrinsic implementations?

**Context**: We need to verify mathematical correctness and performance.

**Questions**:

- What accuracy requirements should we test against?
- Should we have unit tests for each intrinsic function?
- Should we use filetests or separate test infrastructure?
- How do we test edge cases (NaN, infinity, very large/small values)?

**Current State**:

- Filetests infrastructure already exists at `lightplayer/crates/lp-glsl-filetests/filetests/builtins/`
- Tests exist for trig functions (e.g., `trig-sin.glsl`), exponential functions (e.g., `exp-exp.glsl`), and edge cases (e.g., `edge-trig-domain.glsl`, `edge-nan-inf-propagation.glsl`)
- Tests use `// run: function_name() ~= expected_value` format
- Tests cover basic cases, vector types, and edge cases

**Suggested Answer**:

- Use existing filetests infrastructure
- Ensure intrinsic implementations pass existing tests
- Add additional tests if needed for accuracy validation
- Target: ±2-3 ULP accuracy initially, optimize to ±1 ULP later

**DECISION**: ✅ **Confirmed** - Use existing filetests infrastructure. Ensure intrinsic implementations pass existing tests. Add additional accuracy tests if needed.

---

## Architecture and Design

### Q9: Should intrinsic files be compiled eagerly or lazily?

**Context**: Currently, intrinsics are loaded lazily when first needed. But if we have cross-file dependencies, we might need to load multiple files at once.

**Questions**:

- Should we continue with lazy loading per function?
- Or load entire intrinsic files when any function from that file is needed?
- How should we handle the case where loading one function requires loading multiple files?

**Current State**:

- Lazy loading: when `sin()` is called, `trig.glsl` is loaded and all functions in it are compiled
- All functions from the file are declared/defined even if only one is needed

**Suggested Answer**:

- Continue lazy loading per file (current approach is good)
- When a function is needed, load its file and compile all functions in that file
- If cross-file dependencies are added later, load dependent files automatically
- This balances code size (only load what's used) with simplicity (load entire file at once)

**DECISION**: ✅ **Confirmed** - Lazy loading per file is good. However, we should only include functions that are actually needed (directly called or transitively called), not all functions from the file. This requires tracking function dependencies within a file to determine which functions to compile. Pruning unneeded functions is important to avoid very large code that's hard to debug.

---

### Q10: How should we organize intrinsic files?

**Context**: We have `trig.glsl` now. We'll need `exponential.glsl` and possibly others.

**Questions**:

- What's the best way to organize functions into files?
- Should we have one file per category (trig, exponential, common)?
- Or group by algorithm (cordic.glsl, polynomial.glsl)?
- How should `get_intrinsic_file()` map functions to files?

**Current State**:

- `trig.glsl` contains all trigonometric functions
- `get_intrinsic_file()` maps intrinsic names to file names

**Suggested Answer**:

- Organize by mathematical category (matches GLSL spec organization):
  - `trig.glsl`: All trigonometric functions
  - `exponential.glsl`: `exp`, `log`, `exp2`, `log2`, `pow`, `sqrt`, `inversesqrt`
  - `common.glsl`: If we need shared helper functions (e.g., `__lp_reduce_angle` could be shared)
- Keep `get_intrinsic_file()` simple: map function name → file name

**DECISION**: ✅ **Confirmed** - Organize by mathematical category (trig, exponential). Since we're not doing cross-file support, each file must be self-contained and include all helper functions it needs. No shared `common.glsl` file - each file is independent.

---

### Q11: How should error handling work for intrinsic compilation?

**Context**: If an intrinsic file fails to compile, we need to report errors clearly.

**Questions**:

- Should intrinsic compilation errors be reported with source locations?
- How should we handle errors in intrinsic files vs. user code?
- Should we cache compilation failures or retry?

**Current State**:

- Intrinsic compilation errors propagate as `GlslError`
- Source locations might not be properly tracked for intrinsic files

**Suggested Answer**:

- Use `GlSourceMap` to track intrinsic file locations
- Report errors with proper file context (e.g., "error in trig.glsl: line 42")
- Cache compilation failures (don't retry if a file failed to compile)
- Make sure error messages clearly distinguish intrinsic errors from user code errors

**DECISION**: ✅ **Confirmed** - Use `GlSourceMap` for error reporting (this is why it was added). Cache compilation failures. Error messages should clearly distinguish intrinsic errors from user code errors.

---

## Feature Flags and Compatibility

### Q12: How should the `intrinsic-math` feature interact with other features?

**Context**: The feature flag gates intrinsic implementations. But there might be interactions with other features.

**Questions**:

- Should intrinsics work with fixed-point conversion?
- Should intrinsics work with emulator mode?
- Are there any feature combinations that don't make sense?

**Current State**:

- `intrinsic-math` feature gates intrinsic loading
- Intrinsics are compiled with float types (no fixed-point conversion mentioned)

**Suggested Answer**:

- Intrinsics should work with fixed-point conversion (conversion happens after intrinsic compilation)
- Intrinsics should work in both JIT and emulator modes
- The feature flag is independent of other features
- Document any known limitations or interactions

**DECISION**: ✅ **Confirmed** - Intrinsics should work with fixed-point conversion and in both JIT and emulator modes. The feature flag allows using native calls when disabled and exists because the feature wasn't finished. We can revisit the feature flag design later if needed.

---

### Q13: What's the migration path from external calls to intrinsics?

**Context**: Currently, when `intrinsic-math` is disabled, functions use external library calls. When enabled, they use intrinsics.

**Questions**:

- Should we support mixing external calls and intrinsics?
- How should we handle the transition period?
- Should there be a way to force external calls for specific functions?

**Current State**:

- Feature flag switches between external calls and intrinsics globally
- No per-function control

**Suggested Answer**:

- Keep global feature flag (simple and clear)
- No mixing: either all math functions use intrinsics or all use external calls
- If needed later, can add per-function overrides, but start simple
- Document the trade-offs: intrinsics = no external dependencies, external = potentially faster/more accurate

**DECISION**: ✅ **Confirmed** - Keep global feature flag approach. No mixing for now. Can add per-function overrides later if needed. Document trade-offs clearly.

---

## Performance and Optimization

### Q14: What performance characteristics should we target?

**Context**: Intrinsics are meant to avoid external library dependencies, but we should still care about performance.

**Questions**:

- What's acceptable performance vs. accuracy trade-off?
- Should we have multiple implementations (fast vs. accurate)?
- How do we measure and optimize intrinsic performance?

**Current State**:

- CORDIC implementation for sin/cos (good accuracy, reasonable performance)
- No performance benchmarks mentioned

**Suggested Answer**:

- **Phase 1**: Focus on correctness first, reasonable performance second
- Single implementation per function (start simple)
- **Phase 2**: Add performance benchmarks and optimize if needed
- **Phase 3**: Consider multiple implementations only if profiling shows it's needed
- Target: Comparable to libm performance for common cases, acceptable for all cases

**DECISION**: ✅ **Confirmed** - Make it work, make it right, make it fast. Focus on correctness first, then reasonable performance. Single implementation per function to start. Trig functions will likely need speed optimization later, but not now. Add benchmarks and optimize based on profiling.

---

### Q15: Should we optimize intrinsic function calls?

**Context**: Some optimizations might be possible (e.g., `sin(x)` and `cos(x)` called together could share computation).

**Questions**:

- Should the compiler optimize multiple calls to related functions?
- Or should intrinsic functions be optimized internally (e.g., `cos` calls `sin`)?
- Should we rely on Cranelift optimizations or do GLSL-level optimizations?

**Current State**:

- `cos` calls `sin` internally (GLSL-level optimization)
- No cross-call optimizations mentioned

**Suggested Answer**:

- **Phase 1**: Rely on intrinsic function internal optimizations (like `cos` calling `sin`)
- **Phase 2**: Let Cranelift handle optimizations (it should handle common subexpression elimination)
- **Phase 3**: Consider GLSL-level optimizations only if profiling shows significant benefit
- Keep intrinsics simple and let the compiler optimize

**DECISION**: ✅ **Confirmed** - Rely on internal optimizations (like `cos` calling `sin`) and Cranelift optimizations. Keep intrinsics simple. We're in "make it work" phase - optimizations can come later if profiling shows they're needed.

---

## Summary

These questions cover:

1. **Multi-file integration** (Q1-Q2): How intrinsics fit into the multi-file source system
2. **Function coverage** (Q3-Q5): What to implement and how
3. **Implementation details** (Q6-Q8): Specific technical decisions
4. **Architecture** (Q9-Q11): System design and organization
5. **Features and compatibility** (Q12-Q13): Feature interactions
6. **Performance** (Q14-Q15): Optimization strategy

After answering these questions, we can create a concrete implementation plan with phases and tasks.
