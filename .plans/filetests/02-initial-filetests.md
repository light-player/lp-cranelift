# Initial Filetests Infrastructure (lp-glsl-filetests crate)

**Location:** `lightplayer/crates/lp-glsl-filetests/`

## Overview

Implement the filetests infrastructure that discovers, parses, compiles, executes, and verifies GLSL test files. This matches Cranelift's filetests semantics.

## Test Case

**File:** `lightplayer/crates/lp-glsl-filetests/filetests/math/add.glsl`

**Based on:** `cranelift/filetests/filetests/runtests/arithmetic.clif` (lines 14-27)

**Initial content:**

```glsl
// test run
// target riscv32.fixed32

float add_float(float a, float b) {
    return a + b;
}

// run: add_float(0.0, 0.0) ~= 0.0
// run: add_float(1.5, 2.5) ~= 4.0

int add_int(int a, int b) {
    return a + b;
}

// run: add_int(0, 0) == 0
// run: add_int(1, 2) == 3
```

## Matching Cranelift Filetests Semantics

**Goal:** Ensure our filetests match Cranelift's semantics for parsing, discovery, blessing, and execution.

### Test Discovery (matches `cranelift/filetests/src/runner.rs`)

**Cranelift:**

- `cranelift_filetests::run()` takes directory paths
- Recursively scans for `.clif` or `.wat` files
- Files with extensions are test cases, directories are scanned recursively
- Uses `TestRunner` with `scan_dirs()` method

**Our approach:**

- `lp_glsl_filetests::run()` takes directory paths (or can be called per-file)
- Recursively scans for `.glsl` files
- Same pattern: extension-based matching, recursive directory traversal
- Use `walkdir` crate (like Cranelift uses directory reading)

### Test Parsing (matches `cranelift/filetests/src/runone.rs`)

**Cranelift:**

- `parse_test()` parses entire `.clif` file
- Extracts: functions, test commands (`test run`, `test compile`), ISA specs, settings
- Run commands parsed from function comments using `parse_run_command()`
- Returns `TestFile` with functions and commands

**Our approach:**

- Parse entire `.glsl` file
- Extract: functions, test commands (`// test run`), target directives (`// target riscv32.fixed32`)
- Run commands parsed from `// run:` lines (standalone, not in function comments)
- Return structured data with functions and run directives

**Difference:** Our run commands are standalone lines, not function comments. This is acceptable GLSL adaptation.

### Run Command Parsing (matches `cranelift/reader/src/run_command.rs`)

**Cranelift:**

- `parse_run_command()` parses `; run: %function_name(args) == expected`
- Returns `RunCommand` enum with `Invocation` and `Comparison`
- Supports `==` and `!=` comparisons
- Arguments parsed as `DataValue` enum

**Our approach:**

- Parse `// run: function_name(args) ==/~= expected`
- Return `RunDirective` with expression, comparison, expected value
- Support `==` (exact) and `~=` (approximate with tolerance)
- Expected values parsed as `GlslValue` enum

**Difference:** We use `~=` for float tolerance (GLSL-specific), Cranelift uses exact equality.

### Bless Mode (matches `cranelift/filetests/src/subtest.rs`)

**Cranelift:**

- Checks `CRANELIFT_TEST_BLESS` environment variable (must be `"1"`)
- Uses `FileUpdate` helper to track line changes for multiple edits
- Updates expectations in-place at specific line locations
- Error messages suggest using bless mode

**Our approach:**

- Check `CRANELIFT_TEST_BLESS` environment variable (must be `"1"`)
- Use `FileUpdate` helper (same pattern as Cranelift)
- Update `// run:` expectations in-place
- Error messages suggest using bless mode (same wording)

**Match:** Exact same semantics.

### Execution (matches `cranelift/filetests/src/test_run.rs`)

**Cranelift:**

- For each function in test file
- For each run command in function's comments
- Call function directly with parsed arguments
- Compare result with expected value
- Handle bless mode on mismatch

**Our approach:**

- For each `// run:` directive
- Generate bootstrap with `main()` returning expression
- Call `main()` (equivalent to calling function)
- Compare result with expected value
- Handle bless mode on mismatch

**Difference:** We use bootstrap `main()` instead of direct function calls. This is implementation detail; semantics match (call function, compare result).

### Error Messages (matches Cranelift style)

**Cranelift pattern:**

```
compilation of function on line {} does not match
the text expectation

{}

This test assertion can be automatically updated by setting the
CRANELIFT_TEST_BLESS=1 environment variable when running this test.
```

**Our approach:** Use same error message pattern and bless suggestion.

## Architecture Overview: Bootstrap Approach

**Key insight:** We generate a bootstrap `main()` function that returns the expression from the `// run:` directive. This matches Cranelift's approach where run commands are associated with functions.

**Why bootstrap instead of direct calls:**

1. **Type inference:** The compiler tells us the return type automatically
2. **Expression flexibility:** Works for any expression, not just function calls
3. **Simpler execution:** Just call `main()`, no need to map function names to addresses
4. **Matches pattern:** Similar to how Cranelift handles run commands (they're associated with functions)

**Flow:**

```
Test file: add.glsl
  ↓
Parse GLSL → Extract functions + run directives
  ↓
For each // run: add_float(0.0, 0.0) == 0.0
  ↓
1. Parse expression "add_float(0.0, 0.0)" → Expr AST
2. Infer type using function registry → Type::Float
3. Parse expected value "0.0" → GlslValue::Float(0.0)
  ↓
Generate bootstrap:
  float add_float(float a, float b) { return a + b; }
  float main() { return add_float(0.0, 0.0); }
  ↓
Compile bootstrap → Apply fixed32 → Lower to riscv32
  ↓
Execute main() → Get i32 result → Convert to float → Compare
```

## Scope

**In scope:**

- Parse `// test run` directive
- Parse `// target riscv32.fixed32` directive
- Parse `// run:` lines with function calls and arguments
- Extract GLSL functions from test file
- Compile GLSL to CLIF
- Apply fixed32 transform (convert floats to 16.16 fixed-point integers)
- Lower transformed CLIF to riscv32 machine code
- Execute on riscv32 emulator
- Call functions with arguments from `// run:` lines
- Verify results match expectations (`==` for exact, `~=` for float tolerance)
- Test discovery (recursive directory scanning)
- Bless mode (`CRANELIFT_TEST_BLESS=1`)
- Basic error handling and reporting

**Out of scope (future phases):**

- Multiple targets (host, riscv32 without fixed32, etc.)
- Compile tests (CLIF expectations)
- Error tests
- Multiple test files (Phase 1 focuses on one test)
- Complex types (vectors, matrices)
- Other fixed-point formats (fixed64)

## Infrastructure Components Needed

### 1. Test Discovery

**Location:** `tests/filetests.rs`

**Responsibilities:**

- Recursively scan `filetests/` directory for `.glsl` files
- Match Cranelift's `cranelift_filetests::run()` pattern
- Use `walkdir` crate for directory traversal

**Implementation:**

```rust
#[test]
fn filetests() -> anyhow::Result<()> {
    let filetests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("filetests");

    for entry in WalkDir::new(&filetests_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("glsl") {
            lp_glsl_filetests::run_filetest(path)?;
        }
    }
    Ok(())
}
```

**Dependencies:** Add `walkdir` to `Cargo.toml`

### 2. Run Directive Parser

**Location:** `src/filetest.rs` or new `src/run_parser.rs`

**Responsibilities:**

- Parse `// test run` directive
- Parse `// target riscv32.fixed32` directive (format: `<arch>.<format>`)
- Extract all `// run:` lines from test file
- Parse run line format: `// run: <function_name>(<args>) == <expected>` or `~= <expected>`
- Support both integer (`==`) and float (`~=` with tolerance) comparisons
- Recognize that `riscv32.fixed32` means riscv32 target with Fixed16x16 format

**Run line format:**

```
// run: <function_name>(<arg1>, <arg2>, ...) == <expected_value>
// run: <function_name>(<arg1>, <arg2>, ...) ~= <expected_value> (tolerance: <tol>)
```

**Examples:**

- `// run: add_float(0.0, 0.0) ~= 0.0` (float comparison with tolerance)
- `// run: add_float(1.5, 2.5) ~= 4.0` (float comparison with tolerance)
- `// run: add_int(0, 0) == 0` (integer exact equality)
- `// run: add_int(1, 2) == 3` (integer exact equality)

**Data structures:**

```rust
pub struct RunDirective {
    pub expression: Expr,           // Parsed GLSL expression AST
    pub expression_str: String,     // Original string for errors
    pub return_type: Type,          // Inferred return type
    pub comparison: ComparisonOp,   // == or ~=
    pub expected_value: GlslValue,  // Parsed expected value
    pub tolerance: Option<f32>,     // For float comparisons
    pub location: Location,         // Line number for bless mode
}

pub enum ComparisonOp {
    Exact,      // ==
    Approx,     // ~=
}
```

**Implementation:**

```rust
pub fn parse_run_directives(
    source: &str,
    function_registry: &FunctionRegistry,  // From parsed test file
) -> Result<Vec<RunDirective>> {
    // Extract all lines starting with "// run:"
    // For each line:
    //   1. Extract LHS expression string (e.g., "add_float(0.0, 0.0)")
    //   2. Extract comparison operator ("==" or "~=")
    //   3. Extract expected value string (e.g., "0.0")
    //   4. Infer type using lp_glsl::infer_expr_type_in_context(expr_str, function_registry)
    //   5. Parse expected value using GlslValue::parse(expected_str)
    //   6. Build RunDirective with location
}
```

### 3. Function Extraction

**Location:** `src/filetest.rs` (existing `extract_glsl_source` function)

**Responsibilities:**

- Extract GLSL source code from test file
- Filter out directive comments (`// test ...`, `// target ...`, `// run: ...`)
- Preserve function definitions
- Return clean GLSL code ready for compilation

**Current implementation:** Already exists in `extract_glsl_source()`, may need enhancement to handle function extraction separately if needed.

### 4. Bootstrap Generation

**Location:** `src/test_run.rs` or new `src/run_bootstrap.rs`

**Approach:** Bootstrap method (cleaner than direct function calls)

Instead of calling functions directly, we:

1. Extract the LHS expression from `// run:` lines
2. Generate a `main()` function that returns that expression
3. Compile and execute the bootstrap
4. Compare the result

**Example:**

```glsl
// Original test file:
float add_float(float a, float b) {
    return a + b;
}

// run: add_float(0.0, 0.0) == 0.0

// Generated bootstrap:
float add_float(float a, float b) {
    return a + b;
}

float main() {
    return add_float(0.0, 0.0);
}
```

**Implementation:**

```rust
pub fn generate_bootstrap(
    original_source: &str,
    expression_str: &str,
    return_type: Type,
) -> String {
    // Generate GLSL code:
    // - All original function definitions (from original_source)
    // - main() function: "return_type main() { return <expression>; }"
}
```

### 5. GLSL Compilation Pipeline with Fixed32 Transform

**Location:** Reuse existing compilation infrastructure

**Responsibilities:**

- Compile GLSL to CLIF IR
- Apply fixed32 transform (Fixed16x16):
  - Convert `float` types to `i32` (16.16 fixed-point)
  - Transform float operations to fixed-point integer operations
  - Convert float literals to fixed-point integer constants
- Lower transformed CLIF to riscv32 machine code
- Generate executable binary or in-memory code

**Fixed32 transform details:**

- Format: 16.16 fixed-point (16 bits integer, 16 bits fractional)
- Range: approximately -32768.0 to 32767.9999847412109375
- Conversion: `float_value * 65536.0` → `i32`
- Reverse conversion: `i32_value / 65536.0` → `float`

**Current implementation:** Check `lp-glsl` crate for fixed-point transformation support. Likely in `lightplayer/crates/lp-glsl/src/transform/fixed32/` or similar.

**Usage:**

```rust
// Use lp_glsl::JIT with fixed_point_format set to Fixed16x16
let jit = JIT::new()
    .with_fixed_point_format(FixedPointFormat::Fixed16x16)
    .build()?;
```

### 6. Bootstrap Execution and Result Comparison

**Location:** `src/test_run.rs`

**Responsibilities:**

- Execute compiled bootstrap's `main()` function
- Retrieve return value (i32 from riscv32 a0 register, after fixed32 transform)
- Convert return value based on original return type
- Compare actual result with expected result
- Handle bless mode on mismatch

**Execution:**

```rust
// Execute main() via riscv32 emulator or JIT
let result_i32 = execute_main(&executable)?;

// Convert based on return type
let actual_value = match return_type {
    Type::Float => {
        // Convert fixed32 to float
        GlslValue::F32(fixed32_to_float(result_i32))
    }
    Type::Int => GlslValue::I32(result_i32),
    Type::Bool => GlslValue::Bool(result_i32 != 0),
    // ...
};
```

**Fixed-point conversion helpers:**

```rust
fn float_to_fixed32(f: f32) -> i32 {
    let clamped = f.clamp(-32768.0, 32767.9999847412109375);
    let scaled = clamped * 65536.0;
    if scaled >= 0.0 {
        (scaled + 0.5) as i32
    } else {
        (scaled - 0.5) as i32
    }
}

fn fixed32_to_float(fixed: i32) -> f32 {
    fixed as f32 / 65536.0
}
```

**Comparison logic:**

```rust
fn compare_results(
    actual: GlslValue,
    expected: GlslValue,
    comparison: ComparisonOp,
    tolerance: Option<f32>,
) -> Result<(), String> {
    match (actual, expected) {
        (GlslValue::I32(a), GlslValue::I32(e)) => {
            if a == e { Ok(()) } else { Err(format!("expected {}, got {}", e, a)) }
        }
        (GlslValue::F32(a), GlslValue::F32(e)) => {
            let tol = tolerance.unwrap_or(1e-4);
            if (a - e).abs() <= tol { Ok(()) } else {
                Err(format!("expected {} (tolerance: {}), got {}", e, tol, a))
            }
        }
        (GlslValue::Bool(a), GlslValue::Bool(e)) => {
            if a == e { Ok(()) } else { Err(format!("expected {}, got {}", e, a)) }
        }
        _ => Err(format!("type mismatch: {:?} vs {:?}", actual, expected))
    }
}
```

### 7. Bless Mode Implementation

**Location:** `src/file_update.rs` (may already exist)

**Cranelift approach:**

- Check `CRANELIFT_TEST_BLESS` environment variable
- If set to `"1"`, update expectations in-place instead of failing
- Use `FileUpdate` helper to track line changes for multiple edits

**Our approach:** Match this exactly

**FileUpdate helper:**

```rust
pub struct FileUpdate {
    path: PathBuf,
    line_diff: Cell<isize>,  // Track line count changes
    last_update: Cell<usize>,
}

impl FileUpdate {
    pub fn new(path: &Path) -> Self {
        FileUpdate {
            path: path.to_path_buf(),
            line_diff: Cell::new(0),
            last_update: Cell::new(0),
        }
    }

    pub fn update_at(
        &self,
        location: &Location,
        f: impl FnOnce(&mut String, &mut Lines<'_>),
    ) -> Result<()> {
        // Update file at specific location
        // Track line differences for subsequent updates
        // Write file back to filesystem
    }

    pub fn update_run_expectation(
        &self,
        location: &Location,
        new_value: &GlslValue,
    ) -> Result<()> {
        // Update // run: line with new expected value
        // Preserve rest of file structure
    }
}
```

**Usage:**

```rust
let bless_enabled = std::env::var("CRANELIFT_TEST_BLESS")
    .unwrap_or_default() == "1";

let mut file_update = FileUpdate::new(path);

// When expectation mismatch occurs:
if actual != expected {
    if bless_enabled {
        file_update.update_run_expectation(&directive.location, &actual)?;
    } else {
        anyhow::bail!(
            "run test failed: expected {:?}, got {:?}\n\
             This test assertion can be automatically updated by setting the\n\
             CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
            expected,
            actual
        );
    }
}
```

### 8. Riscv32 Emulator Integration

**Location:** `src/execution/emulator.rs` (may already exist)

**Responsibilities:**

- Execute riscv32 machine code
- Provide function calling interface for `main()`
- Handle memory management
- Return execution results (i32 from a0 register)

**Current implementation:** Check if `lp-riscv-tools` or similar provides emulator functionality.

**Calling convention:**

- riscv32: `main()` takes no arguments
- Return value in a0 register (always i32 after fixed32 transform)
- Follow standard riscv32 ABI

## Implementation Steps

### Step 1: Test discovery (matching Cranelift)

1. Add `walkdir` dependency to `Cargo.toml`
2. Implement recursive directory scanning in `tests/filetests.rs`
3. Create directory: `lightplayer/crates/lp-glsl-filetests/filetests/math/`
4. Create `add.glsl` with initial content (functions + run directives)

### Step 2: Parse test file and extract function registry

1. Parse entire test file using `lp_glsl::parse_program_with_registry()`
2. Extract GLSL source (filter out directives)
3. Get `FunctionRegistry` from parsed program

### Step 3: Parse run directives

1. Extract all `// run:` lines from test file
2. For each line:
   - Extract LHS expression string
   - Extract comparison operator (`==` or `~=`)
   - Extract expected value string
   - Infer type using `lp_glsl::infer_expr_type_in_context()`
   - Parse expected value using `GlslValue::parse()`
   - Build `RunDirective` with location

### Step 4: Generate bootstrap for each run directive

1. For each `RunDirective`:
   - Generate bootstrap GLSL code (original functions + `main()`)
   - `main()` returns the expression from the run directive

### Step 5: Compile and execute bootstrap

1. Compile bootstrap GLSL to CLIF IR
2. Apply fixed32 transform (if target is `riscv32.fixed32`)
3. Lower transformed CLIF to riscv32 machine code
4. Execute `main()` via riscv32 emulator
5. Retrieve return value (i32 from a0 register)

### Step 6: Convert result and compare

1. Convert return value based on original return type:
   - Float: fixed32 → float
   - Int: use i32 directly
   - Bool: i32 → bool
2. Compare actual with expected using appropriate comparison
3. Handle bless mode on mismatch

### Step 7: Implement bless mode

1. Check `CRANELIFT_TEST_BLESS` environment variable
2. Implement `FileUpdate` helper
3. Update run expectations in-place when bless enabled
4. Add error messages suggesting bless mode

### Step 8: Integration and testing

1. Run test: `cargo test -p lp-glsl-filetests` (discovers all `.glsl` files)
2. Verify all `// run:` lines execute correctly
3. Test bless mode: `CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests`
4. Test error cases (wrong arguments, wrong types, etc.)
5. Add error messages matching Cranelift style

## Success Criteria

**Filetest infrastructure (matching Cranelift semantics):**

- [ ] **Test discovery:** Recursive directory scanning for `.glsl` files (matches Cranelift's `.clif` scanning)
- [ ] **Test parsing:** Parse entire file, extract functions and run commands (matches `parse_test()`)
- [ ] `filetests/math/add.glsl` exists with `add_float` and `add_int` functions
- [ ] `// test run` and `// target riscv32.fixed32` directives are parsed correctly
- [ ] `// run:` lines are parsed and extracted (matches `parse_run_command()` semantics)
- [ ] Expression types are inferred correctly using function registry
- [ ] Expected values are parsed into `GlslValue` correctly (matches `DataValue` parsing)
- [ ] Bootstrap generation works (combines original source + generated `main()`)
- [ ] Fixed32 transform is applied correctly (floats → fixed-point integers)
- [ ] Bootstrap can be compiled to riscv32 machine code (after transform)
- [ ] `main()` can be executed and return value retrieved
- [ ] Fixed32 return values are converted back to floats for comparison
- [ ] Results are verified correctly (int `==`, float `~=`)
- [ ] **Bless mode:** `CRANELIFT_TEST_BLESS=1` updates expectations in-place (matches Cranelift exactly)
- [ ] **FileUpdate helper:** Tracks line changes for multiple edits (matches Cranelift's `FileUpdate`)
- [ ] **Error messages:** Suggest using `CRANELIFT_TEST_BLESS=1` (matches Cranelift wording)
- [ ] Test passes: `cargo test -p lp-glsl-filetests` (discovers and runs all tests)
- [ ] Clear error messages matching Cranelift style

## Questions to Resolve

1. **Expression parsing:** How to parse just an expression (not a full GLSL program)?

   - Option A: Parse as `return <expr>;` in a function body context
   - Option B: Use a dedicated expression parser if available
   - Option C: Parse as a statement and extract the expression

2. **Function registry:** How to build a function registry from the test file's functions?

   - Use `lp_glsl::parse_program_with_registry()` (from Part 1)

3. **Symbol table:** For Phase 1, expressions are self-contained (no variables), so empty symbol table is fine. Future phases may need variable support.

4. **Bootstrap main():** Should we always generate `main()`, or support explicit `main()` in test files?

   - Phase 1: Always generate `main()` from expression
   - Future: Support both patterns

5. **Emulator interface:** What's the interface for executing compiled code?

   - Does it support calling `main()` directly?
   - How to retrieve return value?
   - Check existing `lp-glsl` execution backends

6. **Error handling:** How should we handle:

   - Parse errors in `// run:` expressions?
   - Type inference failures?
   - Compilation errors in bootstrap?
   - Execution errors?
   - Type mismatches between expected and actual?

7. **Expected value parsing:** How to parse expected values?

   - Use `GlslValue::parse()` (from Part 1)
   - Integers: `"0"`, `"42"`, `"-1"`
   - Floats: `"0.0"`, `"1.5"`, `"-3.14"`
   - Booleans: `"true"`, `"false"`
   - Future: Vectors `"[1.0, 2.0, 3.0]"`

8. **Multiple run directives:** Should we compile a new bootstrap for each `// run:` line, or try to combine them?
   - Phase 1: One bootstrap per `// run:` line (simpler)
   - Future: Could optimize by compiling once and calling multiple times

## Next Phase Preview

Phase 2 will add:

- Multiple targets (host, riscv32 without fixed32, etc.)
- More test cases (subtract, multiply, etc.)
- Better error handling
- Compile tests (CLIF expectations)
