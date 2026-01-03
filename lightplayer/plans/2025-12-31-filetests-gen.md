# Plan: Generate Vector and Matrix Test Files

## Goal
Create a Rust script (`lp-filetests-gen` crate) to generate vector and matrix test files, reducing duplication while maintaining readable test files.

## Current State Analysis

### Test File Structure
- **Vector types**: `vec2`, `vec3`, `vec4`, `ivec2`, `ivec3`, `ivec4`, `uvec2`, `uvec3`, `uvec4`, `bvec2`, `bvec3`, `bvec4`
- **Test categories**: 
  - Function tests: `fn-equal.glsl`, `fn-greater-than.glsl`, `fn-min.glsl`, etc.
  - Operator tests: `op-equal.glsl`, `op-add.glsl`, `op-multiply.glsl`, etc.
  - Constructor tests: `from-scalar.glsl`, `from-scalars.glsl`, `from-vectors.glsl`, etc.

### Duplication Patterns Observed

1. **Same test structure across dimensions** (vec2, vec3, vec4):
   - Same test cases (mixed, all_true, all_false, zero, negative, variables, expressions)
   - Only difference: vector dimension and component count

2. **Same test structure across base types** (vec, ivec, uvec):
   - Same test logic
   - Only difference: type name and literal syntax (5.0 vs 5 vs 5u)

3. **Same test cases across functions/operators**:
   - Most functions have similar test patterns (mixed, all_true, all_false, zero, negative, variables, expressions, in_expression)

4. **Constructor tests are highly repetitive**:
   - Same patterns across all vector types

## Requirements

### CLI Interface
- Accept test file specifiers: `vec/vec3/fn-equal` or `vec/vec3/fn-equal.glsl` (optional extension)
- Support folder targets: `vec/vec3` or `vec` (generate all matching tests)
- Start with function tests (`fn-*`), but build cleanly to support other test types

### Code Structure
- File structure should represent how tests are structured
- Clean separation between test generators for different categories
- Extensible design for adding new test types

## Questions to Answer

1. **Scope**: ✅ Function tests first (equal, greaterThanEqual, min, max, etc.)
   - [x] Function tests (equal, greaterThanEqual, min, max, etc.) - START HERE
   - [ ] Operator tests (==, !=, +, -, *, /, etc.) - FUTURE
   - [ ] Constructor tests (from-scalar, from-scalars, etc.) - FUTURE

2. **Generation Strategy**: ✅ Code-based
   - Main generation function takes: `(VecType, Dimension)` where:
     - `VecType`: `Vec`, `IVec`, `UVec`, `BVec`
     - `Dimension`: `2`, `3`, `4` (width/dimension)
   - Each test category (e.g., `fn-equal`) has a generator module
   - Generator handles all variants: `[u,i,b]vec[2,3,4]`
   - Clean, type-safe code structure

3. **Output Format**: ✅ Default dry-run, require `--write` flag
   - Default: Print generated content to stdout (dry-run mode)
   - `--write`: Actually write files to filetests directory
   - At end of dry-run, output copy-paste command with `--write` flag for convenience
   - Safety-first: no accidental overwrites

4. **Maintenance**: ✅ Generated files use `.gen.glsl` extension
   - Generated files: `vec/vec4/fn-equal.gen.glsl`
   - Manual files: `vec/vec4/fn-equal.glsl` (if needed for special cases)
   - Clear separation: manual tests never added to `.gen.glsl` files
   - Each generated file has header:
     - Clear indication it's generated
     - Command to regenerate: `lp-filetests-gen vec/vec4/fn-equal --write`
   - Manual files take precedence (if both exist, manual is used)

5. **Validation**: ✅ Generator just generates, testing is separate
   - Generator only generates files, no testing
   - Testing done via separate command (existing test infrastructure)
   - Future: wrapper script can combine both if needed
   - Keep it simple: one tool, one job

6. **Crate Structure**: ✅ Binary-only crate in `lightplayer/crates/lp-filetests-gen/`
   - Binary-only crate (main.rs)
   - Independent of `lp-glsl-filetests` (no circular dependencies)
   - Structure:
     ```
     src/
     ├── main.rs              # CLI entry point
     ├── cli.rs               # CLI argument parsing
     ├── types.rs             # VecType, Dimension enums
     ├── generator.rs         # Main dispatch (parses path, routes to generators)
     ├── util.rs              # General utilities (file headers, path handling)
     └── vec/
         ├── mod.rs           # Vector test generators module
         ├── util.rs          # Vector-specific utilities (literal formatting, type names)
         ├── fn_equal.rs      # fn-equal generator (handles all types/dims)
         ├── fn_greater_equal.rs
         ├── fn_greater_than.rs
         ├── fn_less_equal.rs
         ├── fn_less_than.rs
         ├── fn_min.rs
         └── fn_max.rs
     ```

### File Structure Details

- **Dispatch**: `generator.rs` parses path (`vec/vec4/fn-equal` → `VecType::Vec, Dimension::D4, "fn-equal"`) and routes to appropriate generator
- **Utilities**: 
  - `util.rs`: General utilities (file header generation with regeneration command)
  - `vec/util.rs`: Vector-specific (literal formatting `5.0` vs `5` vs `5u`, type name formatting)
- **Return Types**: Each generator function knows its return type:
  - Comparison functions (`equal`, `greaterThanEqual`, etc.) → always `bvec4`
  - Math functions (`min`, `max`) → same type as input

## Plan Phases

1. **Create Crate Structure and CLI Skeleton**
   - Create `lp-filetests-gen` crate with basic Cargo.toml
   - Implement CLI argument parsing (`clap` or similar)
   - Support file specifiers: `vec/vec3/fn-equal` (with optional `.glsl` extension)
   - Support folder targets: `vec/vec3` or `vec`
   - Implement `--write` flag (default: dry-run)
   - Basic path resolution and file discovery
   - **Success criteria**: CLI accepts arguments, can parse paths, dry-run mode works

2. **Implement Core Types and Infrastructure**
   - Define `VecType` enum (Vec, IVec, UVec, BVec) in `types.rs`
   - Define `Dimension` enum (D2, D3, D4) in `types.rs`
   - Create `util.rs` with file header generation (includes regeneration command)
   - Create `vec/util.rs` with vector-specific utilities:
     - Literal formatting (`format_literal()`: `5.0` vs `5` vs `5u`)
     - Type name formatting (`format_type_name()`: `vec4` vs `ivec4` vs `uvec4`)
     - Vector constructor generation
   - Create `generator.rs` with path parsing and dispatch logic
   - **Success criteria**: Types defined, utilities work, path parsing works

3. **Implement fn-equal Generator (Proof of Concept)**
   - Create `vec/fn_equal.rs` module
   - Implement `generate()` function taking `(VecType, Dimension)`
   - Generate all test cases: mixed, all_true, all_false, zero, negative, variables, expressions, in_expression
   - Handle return type: always `bvec4` for comparison functions
   - Generate complete `.gen.glsl` file with header
   - Wire up dispatch in `vec/mod.rs` and `generator.rs`
   - **Success criteria**: Can generate `vec/vec4/fn-equal.gen.glsl` matching existing structure

4. **Implement Remaining Function Test Generators**
   - `fn_greater_equal.rs` (same pattern as fn-equal)
   - `fn_greater_than.rs`
   - `fn_less_equal.rs`
   - `fn_less_than.rs`
   - `fn_min.rs` (returns same type as input)
   - `fn_max.rs` (returns same type as input)
   - Ensure consistent structure across all generators
   - **Success criteria**: All function test generators implemented and working

5. **Test and Verify**
   - Generate sample files for all vector types (vec2/3/4, ivec2/3/4, uvec2/3/4)
   - Verify generated files match expected structure
   - Test CLI with various specifiers (single file, folder, wildcard)
   - Verify dry-run mode works correctly
   - Verify `--write` flag works correctly
   - Test that generated files compile and tests pass (manual verification)
   - **Success criteria**: Generated files are correct, CLI works for all cases

6. **Cleanup and Documentation**
   - Remove any temporary code/debug prints
   - Fix all warnings
   - Add README.md with usage examples
   - Document generator structure for future extensions
   - Ensure generated files have proper headers with regeneration commands
   - Move plan file to `lightplayer/plans/_done/`
   - Run `cargo +nightly fmt` on `lightplayer/` directory
   - **Success criteria**: Code is clean, documented, formatted

## Acceptance Criteria

- [ ] Generator can produce test files matching current structure
- [ ] Generated tests compile and pass
- [ ] Manual overrides work for type-specific cases
- [ ] Generated files are readable and maintainable
- [ ] Process is documented

