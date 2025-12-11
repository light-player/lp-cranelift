# Phase 1: Basic Run Test Infrastructure

## Goal

Establish basic infrastructure for running GLSL filetests with function calls, focusing on a single test case that validates float and int addition operations.

**Requires compiler changes in `lp-glsl` crate** (see "Compiler Changes Required" section below) to support:

- Parsing and type-checking programs to get function registries
- Type inference for expressions within a program context
- Parsing literal values into `GlslValue` enum

**Workflow:**

1. Parse program → Get function registry
2. For each `// run:` line: Get expression type, parse expected value
3. Generate bootstrap: Original functions + `main()` returning expression
4. For each target: Compile bootstrap → Execute `main()` → Compare result

## Test Case

**File:** `crates/lp-glsl-filetests/filetests/math/add.glsl`

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
- Convert fixed-point return values back to floats for comparison
- Verify results match expectations (`==` for exact, `~=` for float tolerance)
- Basic error handling and reporting

**Out of scope (future phases):**

- Multiple targets (host, riscv32 without fixed32, etc.)
- Compile tests (CLIF expectations)
- Error tests
- ~~Bless mode~~ (Actually IN SCOPE - matches Cranelift semantics)
- ~~Test discovery~~ (Actually IN SCOPE - matches Cranelift semantics)
- Multiple test files (Phase 1 focuses on one test)
- Complex types (vectors, matrices)
- Other fixed-point formats (fixed64)

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

## Compiler Changes Required (lp-glsl crate)

**Location:** `crates/lp-glsl/src/`

### A. Add `GlslValue::parse()` method

**File:** `crates/lp-glsl/src/backend/executable.rs`

**Add method:**

```rust
impl GlslValue {
    /// Parse a literal value string into GlslValue using GLSL parser
    /// Only supports literals: integers, floats, booleans
    /// Uses type checking to ensure valid literal syntax
    pub fn parse(literal_str: &str) -> Result<Self, GlslError> {
        // Parse as expression: "42", "3.14", "true", etc.
        // Type check to ensure it's a literal
        // Convert to GlslValue
    }
}
```

**Requirements:**

- Use GLSL parser to parse the literal string as an expression
- Type check to ensure it's a literal (IntConst, FloatConst, BoolConst)
- Convert to appropriate `GlslValue` variant
- Support: `"0"`, `"42"`, `"-1"` → `GlslValue::I32`
- Support: `"0.0"`, `"1.5"`, `"-3.14"` → `GlslValue::F32`
- Support: `"true"`, `"false"` → `GlslValue::Bool`
- Error if not a literal or unsupported type

**Unit tests:** `crates/lp-glsl/src/backend/executable.rs` or separate test file

### B. Add program parsing with function registry extraction

**File:** `crates/lp-glsl/src/compiler/pipeline.rs` or new helper module

**Add function:**

```rust
/// Parse and type-check a GLSL program, returning function registry
pub fn parse_program_with_registry(source: &str) -> Result<FunctionRegistry, GlslError> {
    // Parse program
    // Run semantic analysis
    // Extract function registry
    // Return registry for use in expression type inference
}
```

**Requirements:**

- Parse GLSL source code
- Run semantic analysis (type checking)
- Extract `FunctionRegistry` from semantic result
- Return registry that can be used for expression type inference
- Handle parse errors, type errors gracefully

**Unit tests:** Test with simple programs containing functions

### C. Add expression type inference in context

**File:** `crates/lp-glsl/src/semantic/type_check/inference.rs` (may already exist)

**Verify/Enhance:**

```rust
/// Infer type of expression within a program context
pub fn infer_expr_type_in_context(
    expr_str: &str,
    function_registry: &FunctionRegistry,
) -> Result<Type, GlslError> {
    // Parse expression string
    // Use infer_expr_type_with_registry() with the provided registry
    // Return inferred type
}
```

**Requirements:**

- Parse expression string (e.g., `"add_float(0.0, 0.0)"`)
- Use existing `infer_expr_type_with_registry()` with provided function registry
- Return `Type` (Int, Float, Bool, etc.)
- Handle parse errors, unknown functions, type errors

**Note:** May already exist or be straightforward wrapper around existing function

**Unit tests:** Test with various expressions referencing functions in registry

### D. Unit Test Requirements

Each new function needs unit tests:

1. **`GlslValue::parse()` tests:**

   - Valid integers: `"0"`, `"42"`, `"-1"`
   - Valid floats: `"0.0"`, `"1.5"`, `"-3.14"`
   - Valid booleans: `"true"`, `"false"`
   - Invalid: `"not_a_literal"`, `"x + y"`, `"add(1, 2)"`

2. **`parse_program_with_registry()` tests:**

   - Simple program with one function
   - Program with multiple functions
   - Program with no functions (empty registry)
   - Invalid programs (parse errors, type errors)

3. **`infer_expr_type_in_context()` tests:**
   - Function call: `"add_float(0.0, 0.0)"` → `Float`
   - Function call: `"add_int(1, 2)"` → `Int`
   - Literal: `"42"` → `Int`
   - Literal: `"3.14"` → `Float`
   - Unknown function (should error)
   - Invalid expression (should error)

## Infrastructure Components Needed

### 1. Run Directive Parser

**Location:** `src/filetest.rs` or new `src/run_parser.rs`

**Responsibilities:**

- Parse `// test run` directive
- Parse `// target riscv32.fixed32` directive (format: `<arch>.<format>`)
- Extract all `// run:` lines from test file
- Parse run line format: `// run: <function_name>(<args>) == <expected>` or `~= <expected>`
- Support both integer (`==`) and float (`~=`) comparisons
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

**Implementation notes:**

- Use regex or manual parsing to extract function name, arguments, comparison operator, and expected value
- Arguments can be integers or floats (no vectors/matrices yet)
- Default tolerance for floats: `1e-4` if not specified

### 2. Function Extraction

**Location:** `src/filetest.rs` (existing `extract_glsl_source` function)

**Responsibilities:**

- Extract GLSL source code from test file
- Filter out directive comments (`// test ...`, `// target ...`, `// run: ...`)
- Preserve function definitions
- Return clean GLSL code ready for compilation

**Current implementation:** Already exists in `extract_glsl_source()`, may need enhancement to handle function extraction separately if needed.

### 3. GLSL Compilation Pipeline with Fixed32 Transform

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

**Current implementation:** Check `lp-glsl` crate for fixed-point transformation support. Likely in `lp-glsl/src/transform/fixed32/` or similar.

### 4. Run Expression Parsing and Bootstrap Generation

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

**Responsibilities:**

1. **Parse run directive:**

   - Extract LHS expression: `add_float(0.0, 0.0)`
   - Extract comparison operator: `==` or `~=`
   - Extract expected value: `0.0`
   - Extract optional tolerance: `(tolerance: 1e-4)`

2. **Parse GLSL expression:**

   - Use GLSL parser to parse the LHS expression string
   - Build an AST node for the expression
   - This allows complex expressions, not just function calls

3. **Type inference:**

   - Use `lp_glsl::semantic::type_check::infer_expr_type_with_registry()`
   - Requires:
     - Expression AST
     - Symbol table (for variables, if any)
     - Function registry (for function calls)
   - Returns: `Type` (Int, Float, Bool, Vec2, etc.)

4. **Bootstrap generation:**

   - Generate `main()` function with inferred return type
   - Include all original functions from test file
   - Return the parsed expression

5. **Expected value parsing:**
   - Parse expected value string based on inferred type
   - Create `GlslValue` enum:
     ```rust
     pub enum GlslValue {
         Int(i32),
         Float(f32),
         Bool(bool),
         Vec2([f32; 2]),
         Vec3([f32; 3]),
         Vec4([f32; 4]),
         // etc.
     }
     ```

**Key advantages:**

- Handles any expression, not just function calls
- Type-safe: compiler tells us the return type
- Matches Cranelift's approach (bootstrap pattern)
- Easier to extend to complex expressions later

**Implementation details:**

```rust
pub struct RunDirective {
    pub expression: Expr,           // Parsed GLSL expression AST
    pub expression_str: String,      // Original string for error messages
    pub comparison: ComparisonOp,    // == or ~=
    pub expected_value: GlslValue,   // Parsed expected value
    pub tolerance: Option<f32>,      // For float comparisons
}

pub fn parse_run_directives(
    source: &str,
    functions: &[Function],  // From test file
) -> Result<Vec<RunDirective>> {
    // 1. Extract all "// run:" lines
    // 2. For each line:
    //    a. Parse LHS expression using GLSL parser
    //    b. Infer type using infer_expr_type_with_registry
    //    c. Parse expected value based on type
    //    d. Build RunDirective
}

pub fn generate_bootstrap(
    original_functions: &[Function],
    expression: &Expr,
    return_type: Type,
) -> String {
    // Generate GLSL code:
    // - All original functions
    // - main() function returning the expression
}
```

### 5. Result Verification

**Location:** `src/test_run.rs`

**Responsibilities:**

- Compare actual result with expected result
- For integers: exact equality (`==`)
- For floats: approximate equality (`~=`) within tolerance
- Report failures with clear error messages
- Support multiple `// run:` lines per test file

**Comparison logic:**

```rust
match comparison_op {
    "==" => {
        // Exact equality for int/bool
        actual == expected
    }
    "~=" => {
        // Approximate equality for float
        (actual - expected).abs() <= tolerance
    }
}
```

### 6. Riscv32 Emulator Integration

**Location:** `src/execution/emulator.rs` (may already exist)

**Responsibilities:**

- Execute riscv32 machine code
- Provide function calling interface
- Handle memory management
- Return execution results

**Current implementation:** Check if `lp-riscv-tools` or similar provides emulator functionality.

## Implementation Steps

### Step 1: Test discovery (matching Cranelift)

**Cranelift approach:** `cranelift_filetests::run()` takes directories, recursively scans for `.clif` files

**Our approach:** Match this exactly

1. Add `walkdir` dependency to `Cargo.toml`
2. Implement recursive directory scanning in `tests/filetests.rs`:
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
3. Create directory: `crates/lp-glsl-filetests/filetests/math/`
4. Create `add.glsl` with initial content (functions + run directives)

**Note:** Phase 1 can start with manual test entry, but plan should reflect automatic discovery goal

### Step 2: Enhance directive parsing and expression parsing

1. Update `parse_target_directives()` to handle `riscv32.fixed32` target format (`<arch>.<format>`)
2. Parse format component: `fixed32` → `FixedPointFormat::Fixed16x16`

3. Add function to parse `// run:` lines using bootstrap approach:

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
       //   6. Build RunDirective
   }
   ```

4. Define `RunDirective` struct:

   ```rust
   pub struct RunDirective {
       pub expression: Expr,           // Parsed GLSL expression AST
       pub expression_str: String,      // Original string for errors
       pub return_type: Type,          // Inferred return type
       pub comparison: ComparisonOp,   // == or ~=
       pub expected_value: GlslValue,   // Parsed expected value
       pub tolerance: Option<f32>,     // For float comparisons
   }

   pub enum GlslValue {
       Int(i32),
       Float(f32),
       Bool(bool),
       // Vec2, Vec3, Vec4 for future phases
   }

   pub enum ComparisonOp {
       Exact,      // ==
       Approx,     // ~=
   }
   ```

5. Add bootstrap generation function:
   ```rust
   pub fn generate_bootstrap(
       original_functions: &[Function],
       expression: &Expr,
       return_type: Type,
   ) -> String {
       // Generate GLSL code with:
       // - All original function definitions
       // - main() function: "return_type main() { return <expression>; }"
   }
   ```

### Step 3: GLSL parsing and type inference integration

1. Parse the test file's GLSL source to get:

   - Function definitions (for function registry)
   - Symbol table (for variables, if any in expressions)

2. For each `// run:` directive:

   - Parse the LHS expression string using GLSL parser
   - Build expression AST (`Expr` node)
   - Use `infer_expr_type_with_registry()` with:
     - Expression AST
     - Symbol table (empty for Phase 1, since expressions are self-contained)
     - Function registry (built from test file's functions)
   - Get inferred return type

3. Parse expected value:
   - Based on inferred type, parse the expected value string
   - Convert to `GlslValue` enum
   - Handle tolerance parsing for float comparisons

### Step 4: Bootstrap generation and compilation

1. For each `RunDirective`:

   - Generate bootstrap GLSL code:

     ```glsl
     // All original functions from test file
     float add_float(float a, float b) { ... }

     // Generated main() function
     float main() {
         return add_float(0.0, 0.0);  // The expression from run directive
     }
     ```

2. Compile bootstrap GLSL to CLIF IR:

   - Use `lp_glsl::JIT` or `lp_glsl::Pipeline::parse_and_analyze()`
   - Get semantic analysis result with function registry

3. Apply fixed32 transform:

   - Set `fixed_point_format = Some(FixedPointFormat::Fixed16x16)` on JIT
   - Transform CLIF: `float` → `i32`, float ops → fixed-point ops
   - Note: After transform, `main()` returning `float` actually returns `i32` (fixed-point)

4. Lower transformed CLIF to riscv32 machine code:
   - Use Cranelift's module/function compilation with riscv32 ISA
   - Generate executable binary or in-memory code

**Implementation:**

- Use `lp_glsl::JIT` with `fixed_point_format` set to `Fixed16x16`
- Compile the bootstrap code (not the original test file)
- Execute `main()` function (not individual test functions)

### Step 5: Bootstrap execution and result comparison

1. For each `RunDirective` and each target:

   a. Execute the compiled bootstrap's `main()` function:

   - Call `main()` via riscv32 emulator or JIT
   - Retrieve return value (i32 from riscv32 a0 register, after fixed32 transform)

   b. Convert return value based on original return type:

   - If `main()` returns `float` (transformed to i32): convert fixed32 to float (`i32_value / 65536.0`)
   - If `main()` returns `int`: use i32 value directly
   - If `main()` returns `bool`: convert i32 to bool (0 = false, non-zero = true)
   - Convert to `GlslValue` for comparison

   c. Compare actual `GlslValue` with expected `GlslValue`:

   - For `GlslValue::I32`: exact equality (`==`)
   - For `GlslValue::F32`: approximate equality (`~=`) within tolerance
   - For `GlslValue::Bool`: exact equality (`==`)
   - Report failure with clear error message if mismatch

**Calling convention:**

- riscv32: `main()` takes no arguments
- Return value in a0 register (always i32 after fixed32 transform)
- Follow standard riscv32 ABI

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
        (GlslValue::Int(a), GlslValue::Int(e)) => {
            if a == e { Ok(()) } else { Err(format!("expected {}, got {}", e, a)) }
        }
        (GlslValue::Float(a), GlslValue::Float(e)) => {
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

### Step 6: Result verification

1. Compare actual vs expected using appropriate comparison
2. Report success/failure
3. Continue to next `// run:` line even if one fails (collect all failures)

### Step 7: Bless mode implementation (matching Cranelift)

**Cranelift approach:**
- Check `CRANELIFT_TEST_BLESS` environment variable
- If set to `"1"`, update expectations in-place instead of failing
- Use `FileUpdate` helper to track line changes for multiple edits

**Our approach:** Match this exactly

1. Check `CRANELIFT_TEST_BLESS` environment variable:
   ```rust
   let bless_enabled = std::env::var("CRANELIFT_TEST_BLESS")
       .unwrap_or_default() == "1";
   ```

2. When expectation mismatch occurs:
   - If bless enabled: Update test file with actual result
   - If bless disabled: Fail with error message suggesting bless mode

3. Implement `FileUpdate` helper (like Cranelift's):
   ```rust
   pub struct FileUpdate {
       path: PathBuf,
       line_diff: Cell<isize>,  // Track line count changes
       last_update: Cell<usize>,
   }
   
   impl FileUpdate {
       pub fn update_at(&self, location: &Location, f: impl FnOnce(&mut String, &mut Lines)) -> Result<()> {
           // Update file at specific location
           // Track line differences for subsequent updates
       }
   }
   ```

4. Update run expectations:
   - Parse expected value location from `// run:` line
   - Replace expected value with actual result
   - Preserve rest of file structure

### Step 8: Integration and testing

1. Run test: `cargo test -p lp-glsl-filetests` (discovers all `.glsl` files)
2. Run specific test: `cargo test -p lp-glsl-filetests test_math_add` (if manual entry exists)
3. Verify all `// run:` lines execute correctly
4. Test bless mode: `CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests`
5. Test error cases (wrong arguments, wrong types, etc.)
6. Add error messages matching Cranelift style

## Questions to Resolve

1. **Expression parsing:** How to parse just an expression (not a full GLSL program)?

   - Option A: Parse as `return <expr>;` in a function body context
   - Option B: Use a dedicated expression parser if available
   - Option C: Parse as a statement and extract the expression

2. **Function registry:** How to build a function registry from the test file's functions?

   - Need to extract function signatures (name, param types, return type)
   - Use semantic analysis result from parsing the test file

3. **Symbol table:** For Phase 1, expressions are self-contained (no variables), so empty symbol table is fine. Future phases may need variable support.

4. **Bootstrap main():** Should we always generate `main()`, or support explicit `main()` in test files?

   - Phase 1: Always generate `main()` from expression
   - Future: Support both patterns

5. **Emulator interface:** What's the interface for executing compiled code?

   - Does it support calling `main()` directly?
   - How to retrieve return value?

6. **Error handling:** How should we handle:

   - Parse errors in `// run:` expressions?
   - Type inference failures?
   - Compilation errors in bootstrap?
   - Execution errors?
   - Type mismatches between expected and actual?

7. **Expected value parsing:** How to parse expected values?

   - Integers: `"0"`, `"42"`, `"-1"`
   - Floats: `"0.0"`, `"1.5"`, `"-3.14"`
   - Booleans: `"true"`, `"false"`
   - Future: Vectors `"[1.0, 2.0, 3.0]"`

8. **Multiple run directives:** Should we compile a new bootstrap for each `// run:` line, or try to combine them?
   - Phase 1: One bootstrap per `// run:` line (simpler)
   - Future: Could optimize by compiling once and calling multiple times

## Success Criteria

Phase 1 is complete when:

**Compiler changes (lp-glsl crate):**

- [ ] `GlslValue::parse()` implemented with unit tests
- [ ] `parse_program_with_registry()` implemented with unit tests
- [ ] `infer_expr_type_in_context()` implemented/verified with unit tests
- [ ] All unit tests pass

**Filetest infrastructure:**

- [ ] `filetests/math/add.glsl` exists with `add_float` and `add_int` functions
- [ ] `// test run` and `// target riscv32.fixed32` directives are parsed correctly
- [ ] `// run:` lines are parsed and extracted using new compiler functions
- [ ] Expression types are inferred correctly using function registry
- [ ] Expected values are parsed into `GlslValue` correctly
- [ ] Bootstrap generation works (combines original source + generated `main()`)
- [ ] Fixed32 transform is applied correctly (floats → fixed-point integers)
- [ ] Bootstrap can be compiled to riscv32 machine code (after transform)
- [ ] `main()` can be executed and return value retrieved
- [ ] Fixed32 return values are converted back to floats for comparison
- [ ] Results are verified correctly (int `==`, float `~=`)
- [ ] Test passes: `cargo test -p lp-glsl-filetests test_math_add`
- [ ] Clear error messages for failures

## Next Phase Preview

Phase 2 will add:

- Multiple targets (host, riscv32 without fixed32, etc.)
- More test cases (subtract, multiply, etc.)
- Better error handling
- Bless mode support
- Compile tests (CLIF expectations)
