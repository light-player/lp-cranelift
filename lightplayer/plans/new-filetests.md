# New lp-glsl-filetests plan

## Current status

**Implemented:**

- `lightplayer/crates/lp-glsl-filetests/filetests/` directory exists with ~274 test files organized by feature
- Bless mode (`CRANELIFT_TEST_BLESS=1`) is implemented in `src/file_update.rs`
- Test execution harness (`run_filetest`) exists in `src/filetest.rs`
- Support for compile, run, and error tests
- Support for fixed32/fixed64 transforms and riscv32 emulator execution

**Needs implementation:**

- `tests/filetests.rs` still lists ~1000 manual `#[test]` entries from the old suite
- README describes the old directive scheme; not updated for the new directives or globbing
- Cargo.toml lacks `walkdir` (needed for automatic test discovery)
- Test discovery: currently manual; needs automatic globbing like Cranelift
- Seed tests for new directive scheme (`float/add.glsl`, `float/mul.glsl`, `functions/user_return_float.glsl`) don't exist yet

## Goals

- Rebuild lp-glsl-filetests from scratch with clear, Cranelift-style organization.
- Feature-based file layout; directives define subtests (compile/run/transform/targets).
- Seed with a small set of representative tests (float math + user function returning float).

## Constraints / principles

- Keep Cranelift familiarity: one file = feature scenario; multiple directives per file.
- Support pre-transform CLIF, post-transform CLIF (fixed32, later fixed64), run on host JIT, run on riscv32 emulator.
- Bless-first workflow (`CRANELIFT_TEST_BLESS=1`) updates expectations and run directives.
- Fixed64 compile allowed even if execution unavailable; skip runs gracefully.

## Directory layout (feature-first)

- `lightplayer/crates/lp-glsl-filetests/filetests/`
  - `float/` (e.g., `add.glsl`, `mul.glsl`, `mix.glsl`)
  - `functions/` (user-defined functions, returns float/int/bool)
  - `control/` (loops/if; keep small starter set)
  - `builtins/` (subdirs per builtin family, optional initially)
  - Future: `fixed32/`, `fixed64/`, `vectors/`, `matrices/`, `errors/`
- Naming: operation-focused (`add.glsl`, `user_function_return_float.glsl`); avoid target in names.

## Directive scheme (per test file)

- Compile checks:
  - `// test clif.pre` — compare CLIF before fixed-point transform (arch-agnostic).
  - `// test clif.post.fixed32` — compare CLIF after fixed32 transform.
  - `// test clif.post.fixed64` — allow compile-only; skip execution if backend missing.
- Run checks:
  - `// test run host` — native JIT, no transform unless format given.
  - `// test run host.fixed32` — host JIT with fixed32 transform.
  - `// test run host.fixed64` — allowed; may skip until support lands.
  - `// test run riscv32.fixed32` — emulator path.
  - Future: `// test run riscv32.fixed64` once available.
- Errors:
  - `// test error` + `// EXPECT_ERROR: <pattern>`.
- Targets:
  - Optional `// target <arch>[.<fmt>]` retained for explicit coverage; otherwise derived from `test run` directives.
- Expectations:
  - CLIF expectations in trailing `//` (like Cranelift).
  - Run expectations via `// run: == <int|bool|float>` or `// run: ~= <float> (tolerance: <tol>)`.

## Test discovery (like Cranelift)

**Current state:** Manual test entries in `tests/filetests.rs` (~1000 `#[test]` functions)

**Target state:** Automatic discovery via recursive directory scanning

**Implementation approach (mirroring `cranelift/filetests/src/runner.rs`):**

- Use `walkdir` crate to recursively scan `filetests/` directory
- Match files with `.glsl` extension (similar to Cranelift's `.clif` matching)
- Generate one `#[test]` function per discovered `.glsl` file
- Test function name derived from file path: `test_float_add` from `float/add.glsl`
- Path sanitization: replace `/` and `-` with `_`, remove `.glsl` extension

**Example implementation:**

```rust
// tests/filetests.rs
use walkdir::WalkDir;
use std::path::PathBuf;

#[test]
fn filetests() -> anyhow::Result<()> {
    let filetests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("filetests");

    for entry in WalkDir::new(&filetests_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("glsl") {
            let test_name = path
                .strip_prefix(&filetests_dir)?
                .with_extension("")
                .to_string_lossy()
                .replace('/', "_")
                .replace('-', "_");

            // Generate test function dynamically or use macro
            run_filetest(path)?;
        }
    }
    Ok(())
}
```

**Alternative (macro-based, like current manual approach but auto-generated):**

- Use `include_dir` or `walkdir` at build time to generate test functions
- Or use a single test that discovers and runs all files dynamically
- Prefer dynamic discovery for simplicity (matches Cranelift's `run()` function)

## Running tests

**Current implementation:** `run_filetest(path: &Path)` in `src/filetest.rs`

**Test execution flow:**

1. Read test file from disk
2. Parse directives (`test compile`, `test run`, `test error`, `test fixed32`, etc.)
3. Extract GLSL source (filter out directive comments)
4. Execute subtests based on directives:
   - **Compile tests:** Generate CLIF, optionally apply transform, compare against expectations
   - **Run tests:** Compile, execute on target (host JIT or riscv32 emulator), verify results
     - Multiple `// run:` lines are executed independently
     - Each line can test different function calls with different arguments
     - Pattern matches Cranelift's approach (e.g., `simd-icmp-ne.clif` with multiple `; run:` lines)
   - **Error tests:** Attempt compilation, verify error message matches pattern
5. On failure, check `CRANELIFT_TEST_BLESS`; if set, update expectations instead of failing

**Integration with Cargo test:**

- `tests/filetests.rs` is a standard Rust test file
- Each discovered `.glsl` file becomes one test case
- Run with: `cargo test -p lp-glsl-filetests`
- Run specific test: `cargo test -p lp-glsl-filetests test_float_add`
- Show output: `cargo test -p lp-glsl-filetests -- --nocapture`

**Note:** Currently, tests are manually listed in `tests/filetests.rs`. After implementing automatic discovery, all `.glsl` files will be automatically included without manual registration.

**Parallel execution:**

- Consider adding concurrent test execution (like Cranelift's `ConcurrentRunner`)
- For now, sequential execution is acceptable for initial implementation

## Bless mode

**Current implementation:** `src/file_update.rs` with `is_bless_enabled()` checking `CRANELIFT_TEST_BLESS=1`

**How it works:**

1. When a test expectation fails (CLIF mismatch, run result mismatch, error pattern mismatch)
2. Check `CRANELIFT_TEST_BLESS` environment variable
3. If set to `"1"`, update the test file with actual output instead of failing
4. If not set, fail the test with a message suggesting to use bless mode

**Bless operations:**

- **CLIF expectations:** `update_compile_expectations()` replaces trailing `//` CLIF comments
- **Run expectations:** `update_run_directive()` updates `// run:` lines
- **Error expectations:** `update_error_expectation()` updates `// EXPECT_ERROR:` lines
- **Riscv32 fixed32:** `update_riscv32_fixed32_expectations()` handles dual CLIF output

**Usage:**

```bash
# Bless all tests (update expectations)
CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests

# Bless specific test
CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests test_float_add

# Review changes before committing
git diff filetests/
```

**Best practices:**

- Always review blessed output before committing
- Use bless mode when:
  - Adding new tests (generate initial expectations)
  - After intentional codegen changes (update affected tests)
  - After fixing bugs that change expected output
- Don't use bless mode to hide real test failures

## Seed tests

**Purpose:** Establish baseline tests using the new directive scheme before migrating existing tests

**Seed test files to create:**

1. **`filetests/float/add.glsl`**

   - **Directives:** `// test clif.pre`, `// test clif.post.fixed32`, `// test run host`, `// test run host.fixed32`, `// test run riscv32.fixed32`
   - **Body:** Simple float addition

   ```glsl
   float main() {
       return 1.5 + 2.0;
   }
   ```

   - **Expectations:**
     - CLIF before transform (arch-agnostic)
     - CLIF after fixed32 transform
     - Run result: `// run: ~= 3.5 (tolerance: 1e-4)`
   - **Coverage:** Basic float arithmetic, pre/post transform CLIF, host + riscv32 execution

2. **`filetests/float/mul.glsl`**

   - **Directives:** Same as `add.glsl`
   - **Body:** Float multiplication

   ```glsl
   float main() {
       return 2.5 * 3.0;
   }
   ```

   - **Expectations:** Similar structure, verify multiply operation and tolerance handling
   - **Coverage:** Multiplication, tolerance verification

3. **`filetests/functions/user_return_float.glsl`**

   - **Directives:** `// test clif.pre`, `// test clif.post.fixed32`, `// test run host`, `// test run host.fixed32`, `// test run riscv32.fixed32`
   - **Body:** User-defined function returning float

   ```glsl
   float add_float(float a, float b) {
       return a + b;
   }

   float main() {
       return add_float(1.5, 2.5);
   }
   ```

   - **Expectations:** CLIF showing function call/return lowering, run result
   - **Coverage:** Function calls, return values, parameter passing

**Creating seed tests:**

1. Create test files with GLSL code and directives (no expectations initially)
2. Run with bless mode: `CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests test_float_add`
3. Review generated expectations
4. Commit seed tests with blessed expectations

**After seed tests:**

- Use seed tests as templates for migrating existing tests
- Gradually migrate tests from old directive scheme to new scheme
- Remove old manual test entries as tests are migrated

## Expectation format examples

**Pattern:** Expectations follow the GLSL code, using `//` comments (similar to Cranelift's `;` comments for CLIF files)

### Example 1: Compile test with pre-transform CLIF

```glsl
// test clif.pre

float main() {
    return 1.5 + 2.0;
}

// function u0:0() -> f32 fast {
// block0:
//     v0 = f32const 0x1.8p0  ; 1.5
//     v1 = f32const 0x1.0p1  ; 2.0
//     v2 = fadd v0, v1
//     return v2
// }
```

**Notes:**

- CLIF expectations appear after the GLSL code
- Each CLIF line is prefixed with `//`
- Comments in CLIF (after `;`) are preserved
- Empty lines in CLIF become `//` (empty comment lines)

### Example 2: Compile test with post-transform CLIF (fixed32)

```glsl
// test clif.post.fixed32

float main() {
    return 1.5 + 2.0;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 0x18000  ; 1.5 in fixed16.16
//     v1 = iconst.i32 0x20000  ; 2.0 in fixed16.16
//     v2 = iadd v0, v1
//     return v2
// }
```

**Notes:**

- Shows CLIF after fixed-point transformation
- Values are represented as fixed-point integers
- Same format as pre-transform, but different IR

### Example 3: Run test with multiple targets

```glsl
// test run host
// test run host.fixed32
// test run riscv32.fixed32

float main() {
    return 1.5 + 2.0;
}

// run: main() ~= 3.5 (tolerance: 1e-4)
```

**Notes:**

- Run expectations use `// run:` prefix
- Float comparisons use `~=` with tolerance
- Tolerance can be specified: `(tolerance: 1e-4)` or use default
- Integer/bool comparisons use `==`: `// run: == 42` or `// run: == true`

### Example 4: Combined compile and run test

```glsl
// test clif.pre
// test clif.post.fixed32
// test run host
// test run host.fixed32
// test run riscv32.fixed32

float main() {
    return 1.5 + 2.0;
}

// Pre-transform CLIF (arch-agnostic)
// function u0:0() -> f32 fast {
// block0:
//     v0 = f32const 0x1.8p0
//     v1 = f32const 0x1.0p1
//     v2 = fadd v0, v1
//     return v2
// }
//
// Post-transform CLIF (fixed32)
// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 0x18000
//     v1 = iconst.i32 0x20000
//     v2 = iadd v0, v1
//     return v2
// }
//
// run: ~= 3.5 (tolerance: 1e-4)
```

**Notes:**

- Multiple CLIF sections can be separated by blank comment lines (`//`)
- Sections can be labeled with comments: `// Pre-transform CLIF`, `// Post-transform CLIF`
- Run expectations come after all CLIF expectations
- Each test type (clif.pre, clif.post, run) checks its corresponding section

### Example 5: Error test

```glsl
// test error

int main() {
    float x = 1.5;  // Error: cannot assign float to int
    return x;
}

// EXPECT_ERROR: cannot assign float to int
```

**Notes:**

- Error tests don't have CLIF expectations
- Error pattern is specified with `// EXPECT_ERROR: <pattern>`
- Pattern can be a substring of the actual error message
- Multiple error patterns can be checked if multiple errors occur

### Example 6: Function with multiple run cases

```glsl
// test run host

float add_float(float a, float b) {
    return a + b;
}

float main() {
    return add_float(1.5, 2.5);
}

// Test main() implicitly
// run: ~= 4.0 (tolerance: 1e-4)

// Test function directly with different arguments
// run: %add_float(2.0, 3.0) ~= 5.0 (tolerance: 1e-4)
```

**Notes:**

- Multiple `// run:` lines can test different scenarios
- Can test `main()` implicitly or named functions explicitly
- Function calls use syntax: `%function_name(args)`
- Each run line is checked independently
- See Examples 8-10 for more comprehensive multi-case testing patterns

### Example 7: Integer and boolean run tests

```glsl
// test run host

int main() {
    return 10 + 20;
}

// run: == 30

bool main() {
    return true && false;
}

// run: == false
```

**Notes:**

- Integer and boolean results use exact equality `==`
- No tolerance needed for exact types
- Float results use approximate equality `~=` with tolerance

### Example 8: Function with multiple test cases (different argument values)

```glsl
// test run host

float add_float(float a, float b) {
    return a + b;
}

float main() {
    return add_float(1.5, 2.5);
}

// Test main() implicitly
// run: ~= 4.0 (tolerance: 1e-4)

// Test function directly with different arguments
// run: %add_float(0.0, 0.0) ~= 0.0 (tolerance: 1e-4)
// run: %add_float(1.0, 2.0) ~= 3.0 (tolerance: 1e-4)
// run: %add_float(10.5, 20.5) ~= 31.0 (tolerance: 1e-4)
// run: %add_float(-1.5, 2.5) ~= 1.0 (tolerance: 1e-4)
// run: %add_float(100.0, -50.0) ~= 50.0 (tolerance: 1e-4)
```

**Notes:**

- Multiple `// run:` lines test the same function with different argument values
- Each test case is independent and verified separately
- Function calls use syntax: `%function_name(arg1, arg2, ...)`
- Can test both `main()` implicitly and named functions explicitly
- Useful for edge cases: zero, negative, large values, etc.

### Example 9: Vector function with multiple test cases

```glsl
// test run host

vec3 add_vec3(vec3 a, vec3 b) {
    return a + b;
}

vec3 main() {
    return add_vec3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0));
}

// Test main() implicitly
// run: ~= [5.0, 7.0, 9.0] (tolerance: 1e-4)

// Test function with different vector arguments
// run: add_vec3([0.0, 0.0, 0.0], [1.0, 2.0, 3.0]) ~= [1.0, 2.0, 3.0] (tolerance: 1e-4)
// run: add_vec3([1.0, 2.0, 3.0], [-1.0, -2.0, -3.0]) ~= [0.0, 0.0, 0.0] (tolerance: 1e-4)
// run: add_vec3([10.5, 20.5, 30.5], [5.0, 10.0, 15.0]) ~= [15.5, 30.5, 45.5] (tolerance: 1e-4)
```

**Notes:**

- Vector arguments use array notation: `[x, y, z]` for `vec3`
- Vector results also use array notation: `[x, y, z]`
- Each component is compared within tolerance
- Pattern matches Cranelift's vector test format (e.g., `[10 0]` for i64x2)

### Example 10: Comparison function with multiple test cases (like simd-icmp-ne.clif)

```glsl
// test run host

bool compare_float(float a, float b) {
    return a != b;
}

int main() {
    return compare_float(1.0, 2.0) ? 1 : 0;
}

// Test various comparison scenarios
// run: %compare_float(1.0, 1.0) == false
// run: %compare_float(1.0, 2.0) == true
// run: %compare_float(0.0, 0.0) == false
// run: %compare_float(-1.0, 1.0) == true
// run: %compare_float(10.5, 10.5) == false
```

**Notes:**

- Boolean results use exact equality `== true` or `== false`
- Multiple test cases verify different input combinations
- Tests edge cases: equal values, zero, negative, etc.
- Pattern similar to Cranelift's `simd-icmp-ne.clif` with multiple `; run:` lines

### Bless mode behavior

When `CRANELIFT_TEST_BLESS=1` is set:

1. **CLIF expectations:** Entire CLIF section is replaced with actual output

   - Pre-transform: replaces section between code and first `// run:` or `// EXPECT_ERROR:`
   - Post-transform: replaces section marked by `// Post-transform CLIF` comment or after pre-transform section

2. **Run expectations:** Each `// run:` line is updated with actual result

   - Float results: `// run: ~= <actual> (tolerance: <tol>)`
   - Integer results: `// run: == <actual>`
   - Boolean results: `// run: == <actual>`

3. **Error expectations:** `// EXPECT_ERROR:` line is updated with actual error message (or substring)

**Formatting rules:**

- CLIF output is formatted exactly as generated (with `//` prefix)
- Run results use appropriate format based on return type
- Tolerance defaults are applied if not specified
- File structure is preserved (directives, code, expectations order)

## Harness work items

- **Directive parser:** Enhance current parser to support new directive scheme:

  - `// test clif.pre` — compare CLIF before fixed-point transform (arch-agnostic)
  - `// test clif.post.fixed32` — compare CLIF after fixed32 transform
  - `// test clif.post.fixed64` — allow compile-only; skip execution if backend missing
  - `// test run host[.<fmt>]` — native JIT execution
  - `// test run riscv32[.<fmt>]` — emulator execution
  - Produce a `TestPlan` with `Subtest` variants (`ClifPre`, `ClifPost(Fmt)`, `Run { target, fmt }`, `Error`)

- **Target handling:**

  - Preserve `TestTarget` enum but derive default targets from `test run` lines
  - Fall back to host if no targets specified
  - Support `// target <arch>[.<fmt>]` for explicit coverage (backward compatibility)

- **Transform stage plumbing:**

  - Replace current boolean `check_fixed_point_clif` with `TransformStage` enum
  - Choose pre/post in compile tests based on directive (`clif.pre` vs `clif.post.fixed32`)

- **Execution backend mapping:**

  - Host → native JIT; apply fixed-point if `fmt` present
  - Riscv32 → emulator; requires transform when fmt present
  - Skip with warning when fmt unsupported (e.g., fixed64) but still allow CLIF checks

- **Bless support:**

  - Already implemented in `file_update.rs`
  - Ensure all new directive types are supported:
    - `clif.pre` expectations
    - `clif.post.fixed32` expectations
    - `clif.post.fixed64` expectations (compile-only)
    - Multiple `run` directives (one per target/format combination)

- **Test discovery:**
  - Add `walkdir` dependency to `Cargo.toml`
  - Replace manual test list in `tests/filetests.rs` with automatic discovery
  - Use recursive directory scanning to find all `filetests/**/*.glsl` files

## Current directive scheme vs new scheme

**Current implementation (backward compatible):**

- `// test compile` — checks CLIF (before or after transform based on `test fixed32`)
- `// test run` — executes on targets specified by `// target` directives
- `// test error` — verifies compilation errors
- `// test fixed32` — legacy directive for fixed32 transform
- `// target host[.<fmt>]` — explicit target specification

**New directive scheme (to be added):**

- `// test clif.pre` — explicit pre-transform CLIF check
- `// test clif.post.fixed32` — explicit post-transform CLIF check
- `// test clif.post.fixed64` — post-transform CLIF check (compile-only)
- `// test run host[.<fmt>]` — explicit run target specification
- `// test run riscv32[.<fmt>]` — explicit run target specification

**Migration strategy:**

- Support both old and new directive schemes during transition
- New tests use new scheme
- Existing tests continue to work with old scheme
- Gradually migrate tests to new scheme

## Migration / bootstrap steps

**Phase 1: Test discovery infrastructure**

1. Add `walkdir` dependency to `Cargo.toml`
2. Implement automatic test discovery in `tests/filetests.rs`:
   - Use `walkdir::WalkDir` to recursively scan `filetests/` directory
   - Filter for `.glsl` files
   - Generate test cases dynamically (or use macro to generate `#[test]` functions)
   - Test discovery works with existing test files

**Phase 2: Directive parser enhancements** 3. Enhance directive parser in `src/filetest.rs` to support new scheme:

- Parse `// test clif.pre`, `// test clif.post.fixed32`, `// test clif.post.fixed64`
- Parse `// test run host[.<fmt>]`, `// test run riscv32[.<fmt>]`
- Derive targets from `test run` directives (backward compatible with `// target`)

4. Implement `TestPlan` with `Subtest` variants
5. Replace `check_fixed_point_clif` boolean with `TransformStage` enum

**Phase 3: Bless mode enhancements** 6. Enhance `src/file_update.rs` to support new directive types:

- `update_clif_pre_expectations()` for `clif.pre` tests
- `update_clif_post_expectations()` for `clif.post.fixed32/fixed64` tests
- Ensure multiple `run` directives are handled correctly

**Phase 4: Seed tests** 7. Create seed test directory structure: `filetests/float/`, `filetests/functions/` 8. Author three seed tests (`float/add.glsl`, `float/mul.glsl`, `functions/user_return_float.glsl`) 9. Run with bless mode: `CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests` 10. Review and commit seed tests with blessed expectations

**Phase 5: Documentation** 11. Update `README.md` with: - New directive scheme documentation - Test discovery explanation (automatic vs manual) - Bless mode usage examples - Seed test examples 12. Remove references to old manual test list approach

## Open questions / decisions

- Tolerance: keep per-target defaults, allow per-run override `// run: ~= 3.5 (tol=1e-3)`.
- Shorthand for “run on all supported targets”: add `// test run all` meaning host + riscv32 for supported formats; expands at parse time so adding new targets later is automatic. Explicit directives still allowed for selective coverage.
- Fixed64: omit fixed64 directives for now; add when backend exists.
