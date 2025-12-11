# Compile and Transform Tests for lp-glsl-filetests

**Status:** Planning  
**Date:** 2024

## Overview

This plan extends lp-glsl-filetests to support compile tests (CLIF IR verification) and transform tests (fixed32 transformation verification). This enables testing the compiler pipeline at different stages: pre-transform CLIF and post-transform fixed32 CLIF.

## Goals

1. **Compile tests:** Verify CLIF IR output before any transformations (arch-agnostic)
2. **Transform tests:** Verify CLIF IR output after fixed32 transformation
3. **Clean test organization:** All tests for a feature (e.g., `add`) in one file, with multiple test types
4. **Bless mode support:** Auto-update CLIF expectations when codegen changes

## Reference: Cranelift Patterns

Cranelift uses:

- `test compile` - Full compilation pipeline, checks final CLIF
- `test legalizer` - Checks CLIF after legalization pass
- `test optimize` - Checks CLIF after optimization passes

Each test type can have multiple functions in one file, with expectations following each function as comments.

## Directive Scheme

### Compile Test: `test compile`

Tests CLIF IR **before** any fixed-point transformations. This is arch-agnostic and shows the "pure" GLSL → CLIF translation.

**Syntax:**

```glsl
// test compile
```

**Example:**

```glsl
// test compile

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

### Transform Test: `test transform.fixed32`

Tests CLIF IR **after** fixed32 transformation. Shows how floats are converted to fixed-point integers.

**Syntax:**

```glsl
// test transform.fixed32
```

**Example:**

```glsl
// test transform.fixed32

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

### Combined Tests in One File

A single test file can contain multiple test types. All tests share the same GLSL source, but have separate CLIF expectations.

**Example: `filetests/float/add.glsl`:**

```glsl
// test compile
// test transform.fixed32
// test run host
// test run host.fixed32
// test run riscv32.fixed32

float main() {
    return 1.5 + 2.0;
}

// Pre-transform CLIF (test compile)
// function u0:0() -> f32 fast {
// block0:
//     v0 = f32const 0x1.8p0  ; 1.5
//     v1 = f32const 0x1.0p1  ; 2.0
//     v2 = fadd v0, v1
//     return v2
// }
//
// Post-transform CLIF (test transform.fixed32)
// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 0x18000  ; 1.5 in fixed16.16
//     v1 = iconst.i32 0x20000  ; 2.0 in fixed16.16
//     v2 = iadd v0, v1
//     return v2
// }
//
// Run expectations
// run: ~= 3.5 (tolerance: 1e-4)
```

**Key points:**

- Multiple `// test` directives at the top
- CLIF expectations follow the GLSL code, separated by blank comment lines (`//`)
- Sections can be labeled with comments for clarity
- Run expectations come after all CLIF expectations

## Expectation Format

### CLIF Expectations

CLIF expectations use `//` prefix (matching Cranelift's `;` prefix for `.clif` files):

```glsl
// function u0:0() -> f32 fast {
// block0:
//     v0 = f32const 0x1.8p0
//     v1 = f32const 0x1.0p1
//     v2 = fadd v0, v1
//     return v2
// }
```

**Rules:**

- Each CLIF line is prefixed with `//`
- Comments in CLIF (after `;`) are preserved
- Empty lines in CLIF become `//` (empty comment lines)
- Multiple CLIF sections separated by blank comment lines

### Section Separation

When multiple test types are in one file, separate CLIF sections with blank comment lines:

```glsl
// Pre-transform CLIF (test compile)
// function u0:0() -> f32 fast {
//     ...
// }
//
// Post-transform CLIF (test transform.fixed32)
// function u0:0() -> i32 fast {
//     ...
// }
```

The blank `//` line separates sections and helps the parser identify boundaries.

## Implementation Plan

### Phase 1: Directive Parser Enhancement

**File:** `lightplayer/crates/lp-glsl-filetests/src/filetest.rs`

**Changes:**

1. Parse `// test compile` directive
2. Parse `// test transform.fixed32` directive
3. Support multiple test directives per file
4. Extract CLIF expectations from comments

**Data structures:**

```rust
pub enum TestType {
    Compile,              // test compile
    TransformFixed32,     // test transform.fixed32
    Run { target: String, format: Option<String> },  // test run host[.fixed32]
    Error,                // test error
}

pub struct TestFile {
    pub glsl_source: String,
    pub test_types: Vec<TestType>,
    pub clif_expectations: ClifExpectations,
    pub run_directives: Vec<RunDirective>,
    // ...
}

pub struct ClifExpectations {
    pub pre_transform: Option<String>,      // For test compile
    pub post_transform_fixed32: Option<String>,  // For test transform.fixed32
}
```

**Parser logic:**

1. Collect all `// test` directives
2. Extract GLSL source (filter out directives and expectations)
3. Extract CLIF expectations:
   - Find section between GLSL code and first `// run:` or `// EXPECT_ERROR:`
   - Split by blank comment lines (`//`)
   - First section = pre-transform (if `test compile` present)
   - Second section = post-transform (if `test transform.fixed32` present)

### Phase 2: CLIF Extraction from Compiler

**File:** `lightplayer/crates/lp-glsl-filetests/src/test_compile.rs` (new)

**Responsibilities:**

1. Compile GLSL to CLIF module (without transformations)
2. Extract CLIF text for each function
3. Format CLIF text consistently
4. Compare with expectations

**Implementation:**

```rust
pub fn run_compile_test(
    glsl_source: &str,
    expected_clif: &str,
    path: &Path,
) -> Result<()> {
    // Compile to CLIF (no transformations)
    let mut compiler = GlslCompiler::new();
    let isa = create_riscv32_isa()?;  // Arch-agnostic, but need ISA for compilation
    let module = compiler.compile_to_clif_module(glsl_source, isa)?;

    // Extract CLIF text
    let mut actual_clif = String::new();

    // Add user functions
    for (name, func) in module.user_functions() {
        actual_clif.push_str(&format!("// function {}:\n", name));
        actual_clif.push_str(&format_clif_function(func));
        actual_clif.push_str("\n");
    }

    // Add main function
    actual_clif.push_str("// function main:\n");
    actual_clif.push_str(&format_clif_function(module.main_function()));

    // Compare with expectations
    compare_clif(&actual_clif, expected_clif, path)?;

    Ok(())
}

fn format_clif_function(func: &Function) -> String {
    // Use Function::display() to get CLIF text
    // Prefix each line with "// "
    func.display()
        .lines()
        .map(|line| format!("// {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Phase 3: Transform Test Implementation

**File:** `lightplayer/crates/lp-glsl-filetests/src/test_transform.rs` (new)

**Responsibilities:**

1. Compile GLSL to CLIF module
2. Apply fixed32 transformation
3. Extract CLIF text from transformed module
4. Compare with expectations

**Implementation:**

```rust
pub fn run_transform_fixed32_test(
    glsl_source: &str,
    expected_clif: &str,
    path: &Path,
) -> Result<()> {
    // Compile to CLIF
    let mut compiler = GlslCompiler::new();
    let isa = create_riscv32_isa()?;
    let module = compiler.compile_to_clif_module(glsl_source, isa)?;

    // Apply fixed32 transformation
    use lp_glsl::transform::fixed32::{FixedPointFormat, transform_module};
    let transformed_module = transform_module(&module, FixedPointFormat::Fixed16x16)?;

    // Extract CLIF text
    let mut actual_clif = String::new();

    // Add user functions
    for (name, func) in transformed_module.user_functions() {
        actual_clif.push_str(&format!("// function {}:\n", name));
        actual_clif.push_str(&format_clif_function(func));
        actual_clif.push_str("\n");
    }

    // Add main function
    actual_clif.push_str("// function main:\n");
    actual_clif.push_str(&format_clif_function(transformed_module.main_function()));

    // Compare with expectations
    compare_clif(&actual_clif, expected_clif, path)?;

    Ok(())
}
```

### Phase 4: CLIF Comparison and Bless Mode

**File:** `lightplayer/crates/lp-glsl-filetests/src/test_compile.rs`

**CLIF comparison:**

- Normalize whitespace (trim lines, collapse multiple blank lines)
- Use filecheck-style matching (like Cranelift) for flexibility
- Or use precise matching for exact CLIF verification

**Bless mode:**

- When `CRANELIFT_TEST_BLESS=1` is set, update CLIF expectations in-place
- Replace the appropriate section (pre-transform or post-transform)
- Preserve file structure and other expectations

**Implementation:**

```rust
fn compare_clif(actual: &str, expected: &str, path: &Path) -> Result<()> {
    let bless_enabled = env::var("CRANELIFT_TEST_BLESS").unwrap_or_default() == "1";

    // Normalize both strings
    let actual_normalized = normalize_clif(actual);
    let expected_normalized = normalize_clif(expected);

    if actual_normalized != expected_normalized {
        if bless_enabled {
            // Update expectations in file
            update_clif_expectations(path, actual)?;
        } else {
            // Show diff and suggest bless mode
            anyhow::bail!(
                "CLIF mismatch:\n\n{}\n\n\
                 This test assertion can be automatically updated by setting the\n\
                 CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
                format_diff(&expected_normalized, &actual_normalized)
            );
        }
    }

    Ok(())
}

fn normalize_clif(clif: &str) -> String {
    // Trim lines, collapse blank lines, normalize whitespace
    clif.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() || line.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Phase 5: Integration with Test Runner

**File:** `lightplayer/crates/lp-glsl-filetests/src/lib.rs`

**Update `run_filetest()`:**

```rust
pub fn run_filetest(path: &Path) -> Result<()> {
    let test_file = filetest::parse_test_file(path)?;

    // Run compile test if requested
    if test_file.test_types.contains(&TestType::Compile) {
        test_compile::run_compile_test(
            &test_file.glsl_source,
            &test_file.clif_expectations.pre_transform.as_deref().unwrap_or(""),
            path,
        )?;
    }

    // Run transform test if requested
    if test_file.test_types.contains(&TestType::TransformFixed32) {
        test_transform::run_transform_fixed32_test(
            &test_file.glsl_source,
            &test_file.clif_expectations.post_transform_fixed32.as_deref().unwrap_or(""),
            path,
        )?;
    }

    // Run execution tests if requested
    if test_file.test_types.iter().any(|t| matches!(t, TestType::Run { .. })) {
        test_run::run_test_file(&test_file, path)?;
    }

    Ok(())
}
```

## Test File Examples

### Example 1: Simple Compile Test

**File:** `filetests/float/add-compile.glsl`

```glsl
// test compile

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

### Example 2: Transform Test Only

**File:** `filetests/float/add-transform.glsl`

```glsl
// test transform.fixed32

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

### Example 3: Combined Tests (Recommended Pattern)

**File:** `filetests/float/add.glsl`

```glsl
// test compile
// test transform.fixed32
// test run host
// test run host.fixed32
// test run riscv32.fixed32

float main() {
    return 1.5 + 2.0;
}

// Pre-transform CLIF
// function u0:0() -> f32 fast {
// block0:
//     v0 = f32const 0x1.8p0  ; 1.5
//     v1 = f32const 0x1.0p1  ; 2.0
//     v2 = fadd v0, v1
//     return v2
// }
//
// Post-transform CLIF (fixed32)
// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 0x18000  ; 1.5 in fixed16.16
//     v1 = iconst.i32 0x20000  ; 2.0 in fixed16.16
//     v2 = iadd v0, v1
//     return v2
// }
//
// Run expectations
// run: ~= 3.5 (tolerance: 1e-4)
```

**Benefits:**

- All tests for `add` in one place
- Easy to see pre/post transform differences
- Run tests verify correctness
- Single file to maintain

### Example 4: Function with Multiple Test Cases

**File:** `filetests/float/add-multi.glsl`

```glsl
// test compile
// test transform.fixed32
// test run host

float add_float(float a, float b) {
    return a + b;
}

float main() {
    return add_float(1.5, 2.5);
}

// Pre-transform CLIF
// function u0:0(f32, f32) -> f32 fast {
// block0(v0: f32, v1: f32):
//     v2 = fadd v0, v1
//     return v2
// }
// function u0:1() -> f32 fast {
// block0:
//     v0 = f32const 0x1.8p0  ; 1.5
//     v1 = f32const 0x1.4p1  ; 2.5
//     v2 = call u0:0(v0, v1)
//     return v2
// }
//
// Post-transform CLIF (fixed32)
// function u0:0(i32, i32) -> i32 fast {
// block0(v0: i32, v1: i32):
//     v2 = iadd v0, v1
//     return v2
// }
// function u0:1() -> i32 fast {
// block0:
//     v0 = iconst.i32 0x18000  ; 1.5 in fixed16.16
//     v1 = iconst.i32 0x28000  ; 2.5 in fixed16.16
//     v2 = call u0:0(v0, v1)
//     return v2
// }
//
// Run expectations
// run: ~= 4.0 (tolerance: 1e-4)
// run: %add_float(0.0, 0.0) ~= 0.0 (tolerance: 1e-4)
// run: %add_float(10.5, 20.5) ~= 31.0 (tolerance: 1e-4)
```

## CLIF Expectation Extraction Algorithm

**Input:** Test file content with GLSL code and expectations

**Algorithm:**

1. Find the end of GLSL code (last non-comment, non-directive line)
2. Find the start of run expectations (first `// run:` line) or error expectations (`// EXPECT_ERROR:`)
3. Extract all lines between (2) and (3)
4. Split by blank comment lines (`//` alone on a line)
5. First section = pre-transform CLIF (if `test compile` directive present)
6. Second section = post-transform CLIF (if `test transform.fixed32` directive present)

**Edge cases:**

- No CLIF expectations → empty string
- Only one section → assign based on which directive is present
- Multiple blank lines → collapse to single separator

## Bless Mode Updates

**File:** `lightplayer/crates/lp-glsl-filetests/src/file_update.rs`

**Enhance `FileUpdate` to support CLIF expectations:**

```rust
impl FileUpdate {
    pub fn update_clif_pre_expectations(
        &self,
        new_clif: &str,
    ) -> Result<()> {
        // Find the pre-transform CLIF section
        // Replace with new_clif (with // prefix on each line)
        // Preserve other sections
    }

    pub fn update_clif_post_fixed32_expectations(
        &self,
        new_clif: &str,
    ) -> Result<()> {
        // Find the post-transform CLIF section
        // Replace with new_clif (with // prefix on each line)
        // Preserve other sections
    }
}
```

**Update logic:**

1. Read file
2. Identify section boundaries (blank comment lines)
3. Replace appropriate section
4. Write file back

## File Structure

```
lightplayer/crates/lp-glsl-filetests/
├── src/
│   ├── lib.rs                    # Main entry point
│   ├── filetest.rs               # Test file parsing (enhance)
│   ├── test_compile.rs           # NEW: Compile test implementation
│   ├── test_transform.rs         # NEW: Transform test implementation
│   ├── test_run.rs               # Existing: Run test implementation
│   └── file_update.rs            # Bless mode (enhance for CLIF)
├── tests/
│   └── filetests.rs              # Test discovery (no changes)
└── filetests/
    └── float/
        └── add.glsl               # Example: Combined tests
```

## Testing Strategy

1. **Unit tests:** Test CLIF extraction and comparison logic
2. **Integration tests:** Test full compile/transform pipeline
3. **Bless mode:** Verify expectations update correctly
4. **Seed tests:** Create `add.glsl` with all test types as example

## Migration Path

1. **Phase 1:** Implement compile test (`test compile`)
2. **Phase 2:** Implement transform test (`test transform.fixed32`)
3. **Phase 3:** Add bless mode support for CLIF expectations
4. **Phase 4:** Create seed test files with combined tests
5. **Phase 5:** Migrate existing tests to new format (if applicable)

## Open Questions

1. **CLIF matching:** Use filecheck (flexible) or precise matching (exact)?

   - **Recommendation:** Start with precise matching, add filecheck later if needed

2. **Function ordering:** Should CLIF functions appear in a specific order?

   - **Recommendation:** Main function last (matches execution order)

3. **Comments in CLIF:** Preserve or strip?

   - **Recommendation:** Preserve (helps readability)

4. **Multiple user functions:** How to organize CLIF expectations?

   - **Recommendation:** One function per section, or all functions in one section with labels

5. **ISA selection:** For compile tests, which ISA to use?
   - **Recommendation:** riscv32 (arch-agnostic CLIF, but need ISA for compilation)

## Success Criteria

- [ ] `test compile` directive parses correctly
- [ ] `test transform.fixed32` directive parses correctly
- [ ] CLIF expectations extracted correctly from test files
- [ ] Compile test generates and compares CLIF correctly
- [ ] Transform test applies fixed32 and compares CLIF correctly
- [ ] Combined tests (compile + transform + run) work in one file
- [ ] Bless mode updates CLIF expectations correctly
- [ ] Seed test `filetests/float/add.glsl` passes with all test types
- [ ] Clear error messages when CLIF mismatches
- [ ] Documentation updated with examples

## References

- `cranelift/filetests/src/test_compile.rs` - Cranelift compile test implementation
- `cranelift/filetests/src/test_legalizer.rs` - Cranelift transform test pattern
- `lightplayer/plans/new-filetests.md` - Overall filetests plan
- `lightplayer/crates/lp-glsl/src/transform/fixed32/` - Fixed32 transformation code
