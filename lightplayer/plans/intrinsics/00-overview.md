# GLSL Intrinsics Architecture Plan

## Overview

This plan establishes the architecture for implementing GLSL built-in functions as intrinsic implementations written in GLSL itself. The goal is to avoid linking to native math libraries while maintaining clean separation between intrinsic implementations and user code.

## Current State

- **Existing infrastructure**: Intrinsic system already exists behind `intrinsic-math` feature flag
- **Current approach**: Lazy loading during codegen when built-in functions are called
- **Implementation**: Intrinsics written in GLSL files (e.g., `trig.glsl`) compiled separately and linked into the module
- **Cross-function calls**: Already supported within intrinsic files (e.g., `__lp_cos` calls `__lp_sin`)
- **Current limitation**: No support for global variables/constants in intrinsic files

## Architecture Decisions

### 1. When and How Intrinsics Are Included

**Decision: Lazy loading during codegen (current approach)**

- Intrinsics are loaded when a built-in function is first encountered during codegen
- Location: `CodegenContext::translate_builtin_call()` → `builtin_*()` → `get_math_libcall()` → `get_or_create_intrinsic()`
- Benefits:
  - Only includes intrinsics that are actually used
  - No upfront analysis needed
  - Simple to implement

**Implementation flow:**

```
User GLSL: sin(x)
  ↓
AST: FunctionCall("sin", ...)
  ↓
Codegen: translate_builtin_call("sin")
  ↓
builtin_sin() → get_math_libcall("sinf")
  ↓
get_or_create_intrinsic("sinf") → maps to "__lp_sin"
  ↓
Load & compile trig.glsl → declare/define in module
  ↓
Return FuncRef for __lp_sin
```

### 2. AST Level vs Codegen Level

**Decision: Codegen level (current approach)**

- Intrinsics are handled at codegen time, not AST level
- Rationale:
  - AST doesn't need to know about intrinsic implementations
  - Keeps semantic analysis separate from implementation details
  - Allows switching between intrinsic and external implementations via feature flags

### 3. Separate Program Compilation and Linking

**Decision: Compile intrinsic GLSL files separately, link into main module**

- Each intrinsic file (e.g., `trig.glsl`) is compiled independently using `compile_intrinsic_functions()`
- All functions from the file are declared and defined in the main module
- Functions can call each other within the same file (already supported)
- Functions are linked via `Module::declare_func_in_func()` to create `FuncRef`s

**Current implementation:**

- `intrinsics/compiler.rs`: Compiles GLSL source to `HashMap<String, Function>`
- `intrinsics/loader.rs`: Manages loading, caching, and linking
- `IntrinsicCache`: Caches compiled functions per module

### 4. Cross-Function Calls Between Intrinsics

**Decision: Already supported - functions in same file can call each other**

- When an intrinsic file is loaded, all functions are compiled together
- Functions can reference each other (e.g., `__lp_cos` calls `__lp_sin`)
- The `compile_intrinsic_functions()` function handles this by:
  1. Parsing the entire GLSL file
  2. Declaring all functions first
  3. Compiling each function with access to all declared functions

**Future consideration**: If intrinsics need to call functions from different files:
- Option A: Load dependent files automatically (e.g., loading `trig.glsl` also loads `common.glsl` if needed)
- Option B: Explicit dependency declaration in intrinsic files
- **Recommendation**: Start with same-file calls only, add cross-file support later if needed

### 5. Global Variables and Lookup Tables

**Decision: Not required for initial implementation, but architecture should support them**

**Analysis from GLSL spec:**
- Built-in functions are mathematical operations that can be implemented algorithmically
- No lookup tables are required by the spec
- Current implementations (CORDIC for sin/cos) are pure algorithmic

**Future support (if needed):**
- **Global constants**: Can be declared as `const` in GLSL intrinsic files (already supported by GLSL)
- **Global arrays (lookup tables)**: Would require:
  1. Extending `compile_intrinsic_functions()` to extract and compile global data
  2. Using Cranelift's `Module::declare_data()` and `Module::define_data()` APIs
  3. Storing `DataId`s in `IntrinsicCache` similar to how `FuncId`s are stored
  4. Making data accessible to intrinsic functions via `GlobalValue` or similar

**Recommendation**: 
- Phase 1: Implement intrinsics without globals (pure functions)
- Phase 2: Add global constant support if needed (simple - already works in GLSL)
- Phase 3: Add lookup table support only if performance requires it

## Implementation Plan

### Phase 1: Establish Pattern with `sin` (Starting Point)

**Goal**: Verify the architecture works correctly for a single function

**Tasks**:
1. ✅ Review existing `sin` implementation in `trig.glsl`
2. ✅ Verify `get_or_create_intrinsic()` flow works correctly
3. ✅ Test that `__lp_sin` is properly linked and callable
4. Document the pattern for future functions

**Files involved**:
- `lightplayer/crates/lp-glsl/src/intrinsics/trig.glsl` - Implementation
- `lightplayer/crates/lp-glsl/src/intrinsics/loader.rs` - Loading logic
- `lightplayer/crates/lp-glsl/src/codegen/builtins/trigonometric.rs` - Built-in handler

### Phase 2: Expand to Other Trigonometric Functions

**Goal**: Complete all trigonometric built-ins using the established pattern

**Tasks**:
1. Implement remaining functions in `trig.glsl`:
   - `cos` (already calls `sin`, verify it works)
   - `tan` (already implemented)
   - `asin`, `acos`, `atan` (currently placeholders)
   - `sinh`, `cosh`, `tanh` (currently placeholders)
   - `asinh`, `acosh`, `atanh` (currently placeholders)
2. Update `map_to_intrinsic_name()` to include all trigonometric functions
3. Test each function individually

**Files involved**:
- `lightplayer/crates/lp-glsl/src/intrinsics/trig.glsl`
- `lightplayer/crates/lp-glsl/src/intrinsics/loader.rs`

### Phase 3: Add Other Built-in Categories

**Goal**: Extend intrinsics to other built-in function categories

**Categories to add** (in priority order):
1. **Exponential functions**: `exp`, `log`, `pow`, `sqrt`, `exp2`, `log2`, `inversesqrt`
   - Create `intrinsics/exponential.glsl`
   - Implement using polynomial approximations or iterative methods
2. **Common functions**: `abs`, `sign`, `floor`, `ceil`, `fract`, `mod`, `min`, `max`, `clamp`
   - Most are simple operations, may not need intrinsics
   - `pow` might benefit from intrinsic implementation
3. **Geometric functions**: `dot`, `cross`, `length`, `normalize`, `distance`
   - Simple arithmetic, may not need intrinsics
4. **Matrix functions**: `transpose`, `determinant`, `inverse`
   - Linear algebra operations, may benefit from intrinsics

**Files to create**:
- `lightplayer/crates/lp-glsl/src/intrinsics/exponential.glsl`
- Update `get_intrinsic_file()` to map functions to files
- Update `map_to_intrinsic_name()` for new functions

### Phase 4: Global Constants Support (If Needed)

**Goal**: Support `const` declarations in intrinsic files

**Tasks**:
1. Extend `compile_intrinsic_functions()` to extract global constants
2. Store constants in `IntrinsicCache`
3. Make constants accessible during intrinsic compilation
4. Test with a simple constant (e.g., PI)

**Note**: This may not be needed if constants can be inlined directly in function bodies.

### Phase 5: Lookup Tables Support (Only If Needed)

**Goal**: Support global arrays for lookup tables

**Tasks**:
1. Extend GLSL parser/semantic analysis to extract global array declarations
2. Use `Module::declare_data()` and `Module::define_data()` APIs
3. Store `DataId`s in `IntrinsicCache`
4. Generate code to access lookup tables in intrinsic functions
5. Test with a simple lookup table

**Note**: This is optional and should only be implemented if performance analysis shows it's needed.

## File Structure

```
lightplayer/crates/lp-glsl/src/
├── intrinsics/
│   ├── mod.rs                    # Module exports
│   ├── compiler.rs              # Compiles GLSL intrinsic files
│   ├── loader.rs                # Loads and caches intrinsics
│   ├── trig.glsl                # Trigonometric functions
│   ├── exponential.glsl         # Exponential/logarithmic functions (future)
│   └── common.glsl              # Common helper functions (future)
├── codegen/
│   └── builtins/
│       ├── mod.rs               # Routes to specific built-in handlers
│       ├── helpers.rs           # get_math_libcall() - entry point
│       ├── trigonometric.rs     # Calls get_math_libcall() for trig functions
│       └── ...
└── compiler/
    └── glsl_compiler.rs         # Main compiler (uses codegen)
```

## Key Design Principles

1. **Separation of concerns**: Intrinsic implementations are separate GLSL files
2. **Lazy loading**: Only load intrinsics that are actually used
3. **Caching**: Avoid recompiling the same intrinsic file multiple times
4. **Pure functions first**: Start with algorithmic implementations, add lookup tables only if needed
5. **Feature gated**: Intrinsics behind `intrinsic-math` feature flag
6. **No AST changes**: Keep intrinsic handling at codegen level

## Testing Strategy

1. **Unit tests**: Test each intrinsic function individually
2. **Integration tests**: Test intrinsic functions called from user GLSL code
3. **Filetests**: Use existing `lp-glsl-filetests` infrastructure
4. **Accuracy tests**: Verify mathematical correctness (e.g., `sin(0) = 0`, `sin(π/2) = 1`)

## Open Questions

1. **Cross-file dependencies**: How should intrinsics in different files call each other?
   - **Answer**: Start with same-file only, add cross-file support later if needed
2. **Performance vs accuracy**: Should we provide multiple implementations (fast vs accurate)?
   - **Answer**: Start with one implementation, add variants later if needed
3. **Vectorization**: Should intrinsics handle vector types directly or rely on component-wise calls?
   - **Answer**: Current approach (component-wise) is fine, can optimize later

## Success Criteria

- [ ] `sin()` function works with intrinsic implementation
- [ ] All trigonometric functions implemented
- [ ] No external math library dependencies when `intrinsic-math` feature is enabled
- [ ] Intrinsics can call helper functions within the same file
- [ ] Architecture supports adding globals/lookup tables in the future (even if not implemented now)

