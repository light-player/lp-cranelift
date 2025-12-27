# GLSL Intrinsics Architecture Plan

## Overview

This plan establishes the architecture for implementing GLSL built-in functions as intrinsic implementations written in GLSL itself. The goal is to avoid linking to native math libraries while maintaining clean separation between intrinsic implementations and user code.

## References

- **GLSL Spec**: `/Users/yona/dev/photomancer/glsl-spec/chapters/builtinfunctions.adoc` - Complete specification of all built-in functions
- **Questions Document**: `00-questions.md` - All architectural questions and decisions
- **Phase Plans**: `01-source-map.md` through `06-testing.md` - Detailed implementation phases
- **Acceptance Tests**: `lightplayer/crates/lp-glsl-filetests/filetests/builtins/phase/` - Phase-specific acceptance tests
- **Test Script**: `scripts/glsl-filetests.sh` - Run tests with `scripts/glsl-filetests.sh builtins/phase/XX-phase-name.glsl`

## Current State

- **Existing infrastructure**: Intrinsic system already exists behind `intrinsic-math` feature flag
- **Current approach**: Lazy loading during codegen when built-in functions are called
- **Implementation**: Intrinsics written in GLSL files (e.g., `trig.glsl`) compiled separately and linked into the module
- **Cross-function calls**: Already supported within intrinsic files (e.g., `__lp_cos` calls `__lp_sin`)
- **Multi-file source support**: The codebase now has `GlSourceMap` for managing multiple source files, but intrinsics currently use separate source maps
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

- `frontend/intrinsics/compiler.rs`: Compiles GLSL source to `HashMap<String, Function>`
- `frontend/intrinsics/loader.rs`: Manages loading, caching, and linking
- `IntrinsicCache`: Caches compiled functions per module

**Note on multi-file source support:**

- Currently, `compile_intrinsic_functions()` creates its own `GlSourceMap` for intrinsic files
- User code compilation uses a separate `GlSourceMap`
- **Future consideration**: Integrate intrinsic files into the main `GlSourceMap` for unified source location tracking and error reporting

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

## Implementation Phases

See detailed phase plans:

- **Phase 1**: `01-source-map.md` - Integrate GlSourceMap for unified error reporting
- **Phase 2**: `02-basic-trig.md` - Implement sin, cos, tan with dependency tracking
- **Phase 3**: `03-inverse-trig.md` - Implement asin, acos, atan, atan2
- **Phase 4**: `04-hyperbolic-trig.md` - Implement sinh, cosh, tanh, asinh, acosh, atanh
- **Phase 5**: `05-exponential.md` - Create exponential.glsl and implement exp, log, exp2, log2, pow
- **Phase 6**: `06-testing.md` - Testing and validation

Each phase has:

- Detailed task breakdown
- Files to modify/create
- Implementation notes
- Acceptance test file in `lightplayer/crates/lp-glsl-filetests/filetests/builtins/phases/`

## File Structure

```
lightplayer/crates/lp-glsl/src/
├── frontend/
│   ├── intrinsics/
│   │   ├── mod.rs                    # Module exports
│   │   ├── compiler.rs              # Compiles GLSL intrinsic files
│   │   ├── loader.rs                # Loads and caches intrinsics
│   │   ├── trig.glsl                # Trigonometric functions
│   │   ├── exponential.glsl         # Exponential/logarithmic functions (future)
│   │   └── common.glsl              # Common helper functions (future)
│   ├── codegen/
│   │   └── builtins/
│   │       ├── mod.rs               # Routes to specific built-in handlers
│   │       ├── helpers.rs           # get_math_libcall() - entry point
│   │       ├── trigonometric.rs     # Calls get_math_libcall() for trig functions
│   │       └── ...
│   ├── src_loc.rs                  # GlSourceMap for multi-file source tracking
│   └── glsl_compiler.rs             # Main compiler (uses codegen)
└── ...
```

## Key Design Principles

1. **Separation of concerns**: Intrinsic implementations are separate GLSL files
2. **Lazy loading**: Only load intrinsics that are actually used (per file)
3. **Dependency tracking**: Only compile functions that are directly called or transitively needed
4. **Caching**: Avoid recompiling the same intrinsic file multiple times
5. **Pure functions first**: Start with algorithmic implementations, add lookup tables only if needed
6. **Feature gated**: Intrinsics behind `intrinsic-math` feature flag
7. **No AST changes**: Keep intrinsic handling at codegen level
8. **Self-contained files**: Each intrinsic file must include all dependencies (no cross-file support yet)
9. **Unified source tracking**: Intrinsic files added to main `GlSourceMap` for error reporting

## Testing Strategy

1. **Acceptance tests**: Phase-specific tests in `lightplayer/crates/lp-glsl-filetests/filetests/builtins/phases/` - Simple smoke tests for each phase
2. **Correctness tests**: Comprehensive tests in `lightplayer/crates/lp-glsl-filetests/filetests/builtins/` - Full mathematical correctness tests
3. **Edge case tests**: Tests for domain errors, NaN/infinity propagation, precision in `edge-*.glsl` files
4. **Filetests infrastructure**: Use existing `lp-glsl-filetests` infrastructure for all tests

## Open Questions

See `00-questions.md` for a comprehensive list of questions that need to be answered before full implementation.

Key questions include:

1. **Multi-file integration**: How should intrinsic files integrate with `GlSourceMap`?
2. **Cross-file dependencies**: How should intrinsics in different files call each other?
3. **Function coverage**: Which functions need intrinsics vs. can use direct codegen?
4. **Performance vs accuracy**: What are the performance and accuracy requirements?
5. **Vectorization**: Should intrinsics handle vector types directly or rely on component-wise calls?

## Success Criteria

- [ ] `sin()` function works with intrinsic implementation
- [ ] All trigonometric functions implemented
- [ ] No external math library dependencies when `intrinsic-math` feature is enabled
- [ ] Intrinsics can call helper functions within the same file
- [ ] Architecture supports adding globals/lookup tables in the future (even if not implemented now)
