# Phase 4b: Refactor Context Module Structure

## Goal

Break up `context.rs` into a module structure with separate files for different categories of functionality. Use short, verb-based file names for clarity and maintainability. This prepares the codebase for future additions.

## Current Architecture

**Current State**: `cranelift/codegen/src/context.rs` is a single ~394 line file containing:

- Context struct definition
- All implementation methods in one large `impl` block
- Methods organized by functionality but all in one place

**Problem**:

- Hard to navigate as the file grows
- Difficult to find specific functionality
- Merge conflicts more likely
- Doesn't match patterns used in `ir/` and `isa/` modules

## Target Architecture

### Module Structure

```
cranelift/codegen/src/context/
├── mod.rs          # Context struct definition, re-exports
├── construct.rs    # Construction: new, for_function, clear
├── compile.rs      # Compilation pipeline: compile, compile_stencil, optimize
├── verify.rs       # Verification: verify, verify_if
├── optimize.rs     # Optimization passes: legalize, canonicalize_nans, remove_constant_phis, etc.
├── analyze.rs      # Analysis computation: compute_cfg, compute_domtree, compute_loop_analysis
└── access.rs       # Accessors/utilities: compiled_code, take_compiled_code, set_disasm, etc.
```

### File Responsibilities

**`mod.rs`**:

- Context struct definition
- Module documentation
- Re-exports from submodules (if needed)
- Public API surface

**`construct.rs`**:

- `new()` - Allocate new compilation context
- `for_function(func)` - Allocate context with existing Function
- `clear()` - Clear all data structures

**`compile.rs`**:

- `compile(isa, ctrl_plane)` - Main compilation entry point
- `compile_stencil(isa, ctrl_plane)` - Internal compilation to stencil
- `compile_and_emit(isa, mem, ctrl_plane)` - Deprecated wrapper
- `optimize(isa, ctrl_plane)` - Run optimization pipeline

**`verify.rs`**:

- `verify(fisa)` - Run verifier on function
- `verify_if(fisa)` - Run verifier only if enabled

**`optimize.rs`**:

- `legalize(isa)` - Run legalizer
- `canonicalize_nans(isa)` - Perform NaN canonicalization
- `remove_constant_phis(fisa)` - Remove constant phis
- `eliminate_unreachable_code(fisa)` - Eliminate unreachable code
- `replace_redundant_loads()` - Replace redundant loads
- `egraph_pass(fisa, ctrl_plane)` - Run egraph optimizations
- `inline(inliner)` - Perform function inlining
- `souper_harvest(out)` - Harvest for Souper (feature-gated)

**`analyze.rs`**:

- `compute_cfg()` - Compute control flow graph
- `compute_domtree()` - Compute dominator tree
- `compute_loop_analysis()` - Compute loop analysis
- `flowgraph()` - Compute CFG and dominator tree together

**`access.rs`**:

- `compiled_code()` - Get reference to compiled code
- `take_compiled_code()` - Take ownership of compiled code
- `set_disasm(val)` - Set disassembly flag
- `get_code_bb_layout()` - Get code layout (deprecated)
- `create_unwind_info(isa)` - Create unwind info (deprecated, feature-gated)

## Implementation Steps

### Step 1: Create Module Directory Structure

**File**: `cranelift/codegen/src/context/mod.rs` (new)

1. Create `context/` directory
2. Create `mod.rs` with:
   - Module documentation
   - Context struct definition (move from `context.rs`)
   - All necessary imports
   - Re-export Context struct: `pub use super::context::Context;` (temporary, until we move everything)

### Step 2: Extract Construction Methods

**File**: `cranelift/codegen/src/context/construct.rs` (new)

1. Move `new()`, `for_function()`, `clear()` methods
2. Add module documentation
3. Import necessary types
4. Keep method signatures and documentation unchanged

### Step 3: Extract Compilation Methods

**File**: `cranelift/codegen/src/context/compile.rs` (new)

1. Move `compile()`, `compile_stencil()`, `compile_and_emit()`, `optimize()` methods
2. Add module documentation
3. Import necessary types
4. Keep method signatures and documentation unchanged

### Step 4: Extract Verification Methods

**File**: `cranelift/codegen/src/context/verify.rs` (new)

1. Move `verify()`, `verify_if()` methods
2. Add module documentation
3. Import necessary types
4. Keep method signatures and documentation unchanged

### Step 5: Extract Optimization Pass Methods

**File**: `cranelift/codegen/src/context/optimize.rs` (new)

1. Move all optimization pass methods:
   - `legalize()`
   - `canonicalize_nans()`
   - `remove_constant_phis()`
   - `eliminate_unreachable_code()`
   - `replace_redundant_loads()`
   - `egraph_pass()`
   - `inline()`
   - `souper_harvest()` (feature-gated)
2. Add module documentation
3. Import necessary types
4. Keep method signatures and documentation unchanged

### Step 6: Extract Analysis Methods

**File**: `cranelift/codegen/src/context/analyze.rs` (new)

1. Move analysis computation methods:
   - `compute_cfg()`
   - `compute_domtree()`
   - `compute_loop_analysis()`
   - `flowgraph()`
2. Add module documentation
3. Import necessary types
4. Keep method signatures and documentation unchanged

### Step 7: Extract Accessor Methods

**File**: `cranelift/codegen/src/context/access.rs` (new)

1. Move accessor/utility methods:
   - `compiled_code()`
   - `take_compiled_code()`
   - `set_disasm()`
   - `get_code_bb_layout()` (deprecated)
   - `create_unwind_info()` (deprecated, feature-gated)
2. Add module documentation
3. Import necessary types
4. Keep method signatures and documentation unchanged

### Step 8: Update Main Module File

**File**: `cranelift/codegen/src/context/mod.rs`

1. Add all submodule declarations:
   ```rust
   mod construct;
   mod compile;
   mod verify;
   mod optimize;
   mod analyze;
   mod access;
   ```
2. Move Context struct definition here
3. Add all necessary imports
4. Ensure all methods are accessible (they're all in the same `impl Context` block across files)

### Step 9: Update lib.rs

**File**: `cranelift/codegen/src/lib.rs`

1. Change `mod context;` to `mod context;` (should work as-is since we're creating `context/mod.rs`)
2. Ensure `pub use crate::context::Context;` still works

### Step 10: Delete Old File

**File**: `cranelift/codegen/src/context.rs` (delete)

1. Remove the old monolithic file after all methods are moved

## Implementation Notes

### Rust Module System

Rust allows splitting `impl` blocks across multiple files. Each file will have:

```rust
use super::Context;  // or crate::context::Context

impl Context {
    // Methods for this category
}
```

### Import Organization

Each submodule should import:

- `super::Context` or `crate::context::Context`
- All necessary types from other modules
- Keep imports minimal and organized

### Documentation

Each submodule should have:

- Module-level documentation explaining its purpose
- Method documentation preserved from original

### Public API

The public API remains unchanged:

- All methods keep their visibility (`pub`, `pub(crate)`, etc.)
- Method signatures remain identical
- Only internal organization changes

## Files to Create

- `cranelift/codegen/src/context/mod.rs` - Main module file with struct definition
- `cranelift/codegen/src/context/construct.rs` - Construction methods
- `cranelift/codegen/src/context/compile.rs` - Compilation pipeline
- `cranelift/codegen/src/context/verify.rs` - Verification methods
- `cranelift/codegen/src/context/optimize.rs` - Optimization passes
- `cranelift/codegen/src/context/analyze.rs` - Analysis computation
- `cranelift/codegen/src/context/access.rs` - Accessor methods

## Files to Modify

- `cranelift/codegen/src/lib.rs` - Ensure context module is properly declared (should work as-is)

## Files to Delete

- `cranelift/codegen/src/context.rs` - After migration is complete

## Testing Strategy

1. **No functional changes**: This is a pure refactoring - all existing tests should pass
2. **Run test suite**: Execute Cranelift tests to verify no regressions
3. **Verify compilation**: Ensure all code compiles without warnings

## Benefits

1. **Better organization**: Each file has a clear, focused purpose
2. **Easier navigation**: Find methods by category quickly
3. **Reduced merge conflicts**: Changes in different areas are in separate files
4. **Consistency**: Matches patterns used in `ir/` and `isa/` modules
5. **Maintainability**: Smaller files are easier to understand and modify
6. **Extensibility**: Easy to add new methods in appropriate files

## Notes

- This is a pure refactoring - no changes to code generation logic
- All method signatures and documentation remain unchanged
- Public API surface remains identical
- Rust's module system allows splitting `impl` blocks across files
- Future additions can be made to appropriate files based on functionality

## Acceptance Criteria

- [ ] `context/` module directory created
- [ ] `mod.rs` contains Context struct definition
- [ ] `construct.rs` contains construction methods
- [ ] `compile.rs` contains compilation pipeline methods
- [ ] `verify.rs` contains verification methods
- [ ] `optimize.rs` contains optimization pass methods
- [ ] `analyze.rs` contains analysis computation methods
- [ ] `access.rs` contains accessor methods
- [ ] All methods properly accessible
- [ ] All tests pass (no regressions)
- [ ] Code compiles without warnings
- [ ] Old `context.rs` file deleted

## Verification

Run matix tests to ensure no regressions:

```bash
scripts/glsl-filetests.sh matrix
```

Expected result: All tests pass, code is better organized.

## Commit Instructions

After completing the refactoring:

```bash
git add -A
git commit -m "lpc: refactor context.rs into module structure with verb-based names"
```

Keep the commit focused on the refactoring - no functional changes.



