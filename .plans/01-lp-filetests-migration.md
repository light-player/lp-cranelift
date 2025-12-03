# RISC-V32 Lowering Tests with Cranelift ISLE Backend

## Overview

Create filetests to verify the Cranelift RISC-V32 ISLE backend generates correct machine code that can be emulated.

**Goal**: CLIF → Cranelift RISC-V32 ISLE → Machine Code → Emulator verification
**Scope**: **Only** compilation/lowering tests - no IR analysis tests
**Dependencies**: Requires completed RISC-V32 ISLE cleanup (see `riscv32-isle-cleanup-guide.md`)

## Background

### What We're Building

**Location**: `crates/lp-filetests/`

**Single Test Type**: **compile** - Lowering tests (CLIF → RISC-V32 assembly/machine code)

**Purpose**:

1. Verify Cranelift RISC-V32 backend generates correct instructions
2. Test instruction selection and lowering rules
3. Validate machine code with emulator
4. Catch regressions in code generation

**Out of Scope** (NOT migrating these):

- ❌ cat, cfg, domtree - IR analysis (use Cranelift's own tests)
- ❌ toy language - Not needed for backend verification
- ❌ transform, verifier - IR passes (use Cranelift's own tests)
- ❌ instruction tests - Low-level asm tests (keep if useful, but separate)

**Required Dependencies**:

```toml
cranelift-codegen = { workspace = true, features = ["riscv32", "std"] }
cranelift-reader = { workspace = true }  # For parsing .clif files
lp-riscv-tools = { path = "../lp-riscv-tools" }  # For disasm and emulation
filecheck = { workspace = true }  # For pattern matching
```

**Remove**:

- ❌ `cranelift-frontend` - Not needed (tests use pre-written CLIF)
- ❌ `cranelift-filetests` - Not integrating with it (too complex)
- ❌ `lp-toy-lang` - Not testing toy language

### What Needs to Happen

1. **Use Cranelift IR parser** (`cranelift-reader`) to parse `.clif` files
2. **Use Cranelift compilation** with RISC-V32 ISLE backend
3. **Create new test files** in `.clif` format (simple, focused tests)
4. **Keep disassembly/emulation** utilities from `lp-riscv-tools`
5. **Verify correctness** by running generated code in emulator

## Implementation Strategy

### Phase 1: Create Minimal Test Runner

**Goal**: Compile CLIF → RISC-V32 machine code and verify with emulator

**Tasks**:

1. Create simple test runner: CLIF → compile → disassemble → verify
2. Use `cranelift-reader` to parse `.clif` files
3. Use `cranelift-codegen` with `riscv32` ISA for compilation
4. Use `lp-riscv-tools` for disassembly
5. **Optional**: Use `lp-riscv-tools::emu` to verify by execution

**Files to Create/Modify**:

- `src/lib.rs` - Simplify, remove unused test modules
- `src/compile.rs` - NEW: Simple Cranelift compile + verify
- `Cargo.toml` - Remove unused dependencies
- `filetests/riscv32/` - NEW: Directory for lowering tests

### Phase 2: Create New Test Files

**Goal**: Write focused RISC-V32 lowering tests in CLIF format

**Tasks**:

1. Write simple CLIF test files (don't convert old tests - start fresh)
2. Focus on testing specific lowering patterns
3. Verify assembly output with filecheck
4. **Optional**: Verify execution with emulator

**CLIF Test Format**:

```clif
test compile
target riscv32

function %add(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}

; check: add {{[ast][0-9]+}}, {{[ast][0-9]+}}, {{[ast][0-9]+}}
; check: ret
```

**Test Categories to Create**:

1. **Basic arithmetic** - iadd, isub, imul, udiv, sdiv, etc.
2. **Shifts and rotates** - ishl, ushr, sshr, rotl, rotr
3. **Bit operations** - band, bor, bxor, bnot
4. **Comparisons** - icmp (eq, ne, slt, sle, sgt, sge, ult, ule, ugt, uge)
5. **Loads and stores** - load, store (various sizes)
6. **Branches** - br, brif, brz, brnz
7. **Calls** - call, return (simple tests only)
8. **Constants** - iconst, immediate generation
9. **Extensions** - Zba, Zbb, Zbs, Zbc tests (if implemented)

**Start Simple**:

- 5-10 lines per test
- One concept per test
- Clear filecheck patterns
- Focus on instruction selection correctness

### Phase 3: Emulator Verification (Optional)

**Goal**: Run generated machine code in emulator to verify correctness

**Tasks**:

1. Extract machine code from compilation result
2. Load into `lp-riscv-tools::emu::Emulator`
3. Set up registers/memory
4. Execute and verify results
5. Add execution tests alongside assembly tests

**Benefit**: Catches bugs that static assembly checks miss

## Detailed Task Breakdown

### Task 1: Simplify lp-filetests Structure

**File**: `crates/lp-filetests/Cargo.toml`

**Remove**:

```toml
cranelift-frontend = { workspace = true }
cranelift-filetests = { workspace = true }
lp-toy-lang = { path = "../lp-toy-lang" }
```

**Keep**:

```toml
cranelift-codegen = { workspace = true, features = ["riscv32", "std"] }
cranelift-reader = { workspace = true }
lp-riscv-tools = { path = "../lp-riscv-tools" }
filecheck = { workspace = true }
```

### Task 2: Create Simple Compile Module

**File**: `crates/lp-filetests/src/compile.rs` (NEW)

**Purpose**: Core compilation and verification logic

**Key Functions**:

```rust
/// Compile CLIF text to RISC-V32 machine code
pub fn compile_clif(clif_text: &str) -> Result<CompiledCode, CompileError>

/// CompiledCode contains machine code + metadata
pub struct CompiledCode {
    pub code: Vec<u8>,           // Machine code bytes
    pub disassembly: String,      // Disassembled text
    pub func_name: String,        // Function name
}

/// Run a compile test with filecheck verification
pub fn run_compile_test(clif_text: &str, filecheck_patterns: &str) -> Result<(), String>
```

**Implementation**:

```rust
use cranelift_codegen::{Context, settings, isa};
use cranelift_reader::parse_functions;
use lp_riscv_tools::disasm;

pub fn compile_clif(clif_text: &str) -> Result<CompiledCode, CompileError> {
    // 1. Parse CLIF
    let funcs = parse_functions(clif_text)?;

    // 2. Create RISC-V32 ISA
    let isa = create_riscv32_isa()?;

    // 3. Compile
    let mut ctx = Context::new();
    ctx.func = funcs[0].clone();
    let compiled = ctx.compile(&*isa, &mut Default::default())?;

    // 4. Extract and disassemble
    let code = compiled.buffer.data().to_vec();
    let disasm = disasm::disassemble_code(&code);

    Ok(CompiledCode { code, disassembly: disasm, func_name: "..." })
}
```

### Task 3: Create Test Files

**Directory**: `crates/lp-filetests/filetests/riscv32/` (NEW)

**DO NOT convert old tests** - Write new, focused tests from scratch

**Test Examples**:

**1. Basic Arithmetic** (`iadd.clif`):

```clif
test compile
target riscv32

function %iadd(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}

; check: add {{[ast][0-9]+}}, {{[ast][0-9]+}}, {{[ast][0-9]+}}
```

**2. Immediate Values** (`iconst.clif`):

```clif
test compile
target riscv32

function %iconst() -> i32 {
block0:
    v0 = iconst.i32 42
    return v0
}

; check: li {{[ast][0-9]+}}, 42
```

**3. Branches** (`brif.clif`):

```clif
test compile
target riscv32

function %brif(i32) -> i32 {
block0(v0: i32):
    v1 = iconst.i32 0
    v2 = icmp eq v0, v1
    brif v2, block1, block2

block1:
    v3 = iconst.i32 1
    return v3

block2:
    v4 = iconst.i32 2
    return v4
}

; check: beqz
; check-OR: beq
```

**Start with ~10-15 simple tests**, covering core instruction types

## File Organization

### Minimal Structure

```
crates/lp-filetests/
├── Cargo.toml                    # Simplified dependencies
├── src/
│   ├── lib.rs                    # Minimal exports
│   ├── compile.rs                # NEW: Cranelift compile + verify
│   └── filecheck.rs              # Keep: Pattern matching (or use external crate)
└── filetests/
    └── riscv32/                  # NEW: RISC-V32 lowering tests
        ├── iadd.clif             # Integer add
        ├── isub.clif             # Integer subtract
        ├── imul.clif             # Integer multiply
        ├── udiv.clif             # Unsigned divide
        ├── sdiv.clif             # Signed divide
        ├── shifts.clif           # Shift operations
        ├── branches.clif         # Branch instructions
        ├── loads.clif            # Load instructions
        ├── stores.clif           # Store instructions
        ├── iconst.clif           # Constant generation
        ├── comparisons.clif      # Comparison operations
        └── ...more as needed...
```

**What Gets Deleted**:

- `src/test_cat.rs`, `test_cfg.rs`, `test_domtree.rs`, `test_transform.rs`, `test_verifier.rs`
- `src/test_toy.rs` (unless we want toy language later)
- `src/test_instruction.rs` (or keep separate for low-level asm tests)
- `src/parser.rs` (use cranelift-reader instead)
- `filetests/backend3/`, `filetests/cat/`, `filetests/cfg/`, etc.

## Implementation Steps

### Step 1: Clean Up lp-filetests

**Estimated Time**: 30 minutes

1. Update `Cargo.toml` - Remove unused dependencies
2. Delete old test modules and files
3. Create minimal `src/lib.rs` with just compile test support

**Files to Delete**:

```bash
rm -rf filetests/backend3/ filetests/cat/ filetests/cfg/
rm -rf filetests/domtree/ filetests/toy/ filetests/transform/ filetests/verifier/
rm src/test_cat.rs src/test_cfg.rs src/test_domtree.rs
rm src/test_toy.rs src/test_transform.rs src/test_verifier.rs
rm src/parser.rs  # Use cranelift-reader instead
```

### Step 2: Create Compile Module

**Estimated Time**: 1-2 hours

**File**: `src/compile.rs`

```rust
//! Compile CLIF to RISC-V32 and verify output

use cranelift_codegen::{Context, settings, isa};
use cranelift_reader::parse_functions;
use lp_riscv_tools::disasm::disassemble_code;

pub fn run_test(clif_text: &str) -> Result<(), String> {
    // Parse CLIF
    let funcs = parse_functions(clif_text)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Create RISC-V32 ISA
    let mut builder = settings::builder();
    builder.set("opt_level", "speed").unwrap();
    let isa_builder = isa::lookup_by_name("riscv32").unwrap();
    let isa = isa_builder.finish(settings::Flags::new(builder)).unwrap();

    // Compile
    let mut ctx = Context::new();
    ctx.func = funcs[0].clone();
    let compiled = ctx.compile(&*isa, &mut Default::default())
        .map_err(|e| format!("Compile error: {}", e))?;

    // Disassemble
    let code = compiled.buffer.data();
    let disasm = disassemble_code(code);

    // Run filecheck if patterns present
    if let Some(patterns) = extract_filecheck_patterns(clif_text) {
        crate::filecheck::match_filecheck(&disasm, &patterns)?;
    }

    Ok(())
}
```

### Step 3: Write First Test

**Estimated Time**: 30 minutes

**File**: `filetests/riscv32/iadd.clif`

```clif
test compile
target riscv32

function %iadd_i32(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}

; Verify we use ADD not ADDW (opcode 0x33 not 0x3b)
; check: add {{[ast][0-9]+}}, {{[ast][0-9]+}}, {{[ast][0-9]+}}
; check-NOT: addw
```

**Test it**:

```rust
#[test]
fn test_iadd() {
    let content = include_str!("../filetests/riscv32/iadd.clif");
    crate::compile::run_test(content).unwrap();
}
```

### Step 4: Create Core Test Suite

**Estimated Time**: 4-6 hours

Write 10-15 focused tests covering:

1. `iadd.clif`, `isub.clif`, `imul.clif` - Basic arithmetic
2. `udiv.clif`, `sdiv.clif`, `urem.clif`, `srem.clif` - Division
3. `ishl.clif`, `ushr.clif`, `sshr.clif` - Shifts
4. `band.clif`, `bor.clif`, `bxor.clif` - Bitwise ops
5. `icmp.clif` - Comparisons (all conditions)
6. `iconst.clif` - Constant generation
7. `load.clif`, `store.clif` - Memory ops
8. `brif.clif`, `br.clif` - Branches
9. `call.clif` - Simple function calls
10. `select.clif` - Select instruction

**Each test**: 10-20 lines, one concept, clear verification

### Step 5: Add Emulator Verification (Optional)

**Estimated Time**: 2-3 hours

Add execution tests that run code in emulator:

```rust
pub fn run_execution_test(
    clif_text: &str,
    setup_fn: impl FnOnce(&mut Emulator),
    verify_fn: impl FnOnce(&Emulator) -> bool,
) -> Result<(), String>
```

**Example**:

```rust
#[test]
fn test_iadd_executes() {
    let clif = "...";
    run_execution_test(clif,
        |emu| {
            emu.set_reg(10, 5);   // a0 = 5
            emu.set_reg(11, 10);  // a1 = 10
        },
        |emu| {
            emu.get_reg(10) == 15  // a0 should be 15
        }
    ).unwrap();
}
```

### Step 6: Document and Verify

**Estimated Time**: 30 minutes

1. Create `crates/lp-filetests/README.md`
2. Document test format and how to add tests
3. Run full test suite: `cargo test --package lp-filetests`
4. Verify all tests pass

## Expected Challenges

### 1. Learning Cranelift Compilation API

**Challenge**: Need to understand Cranelift's compilation pipeline

**Solution**:

- Study examples in `cranelift/filetests/src/`
- Reference Cranelift documentation
- Start with simplest possible test

### 2. Verifying Correctness

**Challenge**: How do we know the generated code is correct?

**Solution**:

- **Assembly inspection**: Use filecheck to verify instruction selection
- **Emulator execution**: Run code in `lp-riscv-tools::emu` and check results
- **Reference**: Compare with known-good RISC-V assemblers
- Start with trivial cases where correctness is obvious

### 3. Unsupported Instructions

**Challenge**: Some CLIF instructions may not lower to RISC-V32

**Solution**:

- Check `cranelift/codegen/src/isa/riscv32/lower.isle` before writing tests
- Focus on well-supported instructions (iadd, isub, imul, loads, stores, branches)
- Document gaps, file TODOs for future work
- Use `#[ignore]` for tests that can't work yet

### 4. Function Calling Convention

**Challenge**: Cranelift may use different calling convention than expected

**Solution**:

- Study RISC-V32 calling convention in Cranelift
- Adapt tests to match (e.g., arguments in a0, a1, return in a0)
- Or configure custom calling convention if needed

## Quick Reference: Cranelift Pipeline

### Basic Compilation Flow

```rust
use cranelift_codegen::{Context, settings};
use cranelift_codegen::isa;
use cranelift_reader::parse_functions;

// 1. Create ISA target
let mut flag_builder = settings::builder();
flag_builder.set("opt_level", "speed").unwrap();
let isa_builder = isa::lookup_by_name("riscv32").unwrap();
let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();

// 2. Parse CLIF
let clif_text = "function %add(i32, i32) -> i32 { ... }";
let funcs = parse_functions(clif_text).unwrap();

// 3. Compile
let mut ctx = Context::new();
ctx.func = funcs[0].clone();
let code = ctx.compile(&*isa, &mut Default::default()).unwrap();

// 4. Get machine code
let code_bytes = code.buffer.data();
```

### Integration with lp-riscv-tools

```rust
use lp_riscv_tools::disasm::disassemble_code;

// Disassemble compiled code
let disasm = disassemble_code(code_bytes);
println!("{}", disasm);

// Or run in emulator
use lp_riscv_tools::emu::Emulator;
let mut emu = Emulator::new();
emu.load_program(0x1000, code_bytes);
emu.run();
```

## Test File Format

### CLIF Format (Cranelift IR)

```clif
test compile
target riscv32

function %add(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}

; check: function %add
; check: add {{[ast][0-9]+}}, {{[ast][0-9]+}}, {{[ast][0-9]+}}
; check: ret
```

**Key Elements**:

- `test compile` - Test command
- `target riscv32` - Target architecture (optional, can be set in code)
- Function definition in CLIF syntax
- `;` comments for filecheck directives

### Filecheck Patterns

**Exact match**:

```clif
; check: add a0, a0, a1
```

**Regex match**:

```clif
; check: add {{[ast][0-9]+}}, {{[ast][0-9]+}}, {{[ast][0-9]+}}
```

**Negative match**:

```clif
; check-NOT: addw
```

**Multiple patterns**:

```clif
; check: addi sp, sp, -16
; nextln: sw ra, 12(sp)
; nextln: sw s0, 8(sp)
```

## Success Criteria

### Must Have

- ✅ Cranelift RISC-V32 ISLE backend compiles (DONE - see previous cleanup)
- ✅ Can compile simple CLIF function to RISC-V32 machine code
- ✅ Can disassemble and verify output
- ✅ At least 5 basic compile tests passing

### Should Have

- ✅ All basic compile tests converted and passing
- ✅ Block argument tests working
- ✅ Multi-function tests working
- ✅ Integration with lp-riscv-tools emulator

### Nice to Have

- ✅ Toy language tests working (toy → CLIF → RISC-V32)
- ✅ All test categories migrated
- ✅ Integration with cranelift-filetests framework
- ✅ Automated test discovery

## Testing Strategy

### Unit Testing

Each phase should have passing tests:

**Phase 1**: Basic compilation

```bash
cargo test --package lp-filetests test_basic_compile
```

**Phase 2**: Converted tests

```bash
cargo test --package lp-filetests test_branch_lowering
cargo test --package lp-filetests test_call_emission
```

**Phase 3**: Full suite

```bash
cargo test --package lp-filetests
```

### Integration Testing

Verify end-to-end pipeline:

1. Write CLIF function
2. Compile with Cranelift
3. Disassemble with lp-riscv-tools
4. Run in emulator
5. Verify results

## Timeline Estimate

**Total**: ~6-10 hours

- Step 1 (Cleanup): 30 minutes
- Step 2 (Compile module): 1-2 hours
- Step 3 (First test): 30 minutes
- Step 4 (Test suite): 4-6 hours
- Step 5 (Emulator - optional): 2-3 hours
- Step 6 (Documentation): 30 minutes

**Incremental Progress**:

- Session 1: Steps 1-3 complete (basic infrastructure working)
- Session 2: 10 tests written and passing
- Session 3: Emulator verification (optional)
- Session 4: Documentation and polish

## Success Criteria

### Minimum Viable Product

- ✅ Can compile CLIF to RISC-V32 machine code via Cranelift
- ✅ Can disassemble and verify with filecheck
- ✅ 5 basic tests passing (iadd, isub, imul, load, store)
- ✅ Verifies correct opcode usage (0x33 not 0x3b)

### Complete Implementation

- ✅ 10-15 tests covering all major instruction types
- ✅ Filecheck patterns verify correct lowering
- ✅ Tests run in CI automatically
- ✅ Documentation for adding new tests

### Stretch Goals

- ✅ Emulator execution verification
- ✅ Extension tests (Zba, Zbb, etc.)
- ✅ Performance comparison vs old backend3
- ✅ Integration with upstream Cranelift filetests

## Decisions

1. **Keep it simple** - Just compile tests, no IR analysis
2. **Start fresh** - Write new tests, don't convert old ones
3. **Focus on verification** - Use emulator to prove correctness
4. **Minimal dependencies** - Only what's needed for compile+verify

## Key Points

### From Previous Work

**RISC-V32 ISLE Backend** (Reference: `riscv32-isle-cleanup-guide.md`):

- ✅ RV64-specific instructions removed
- ✅ Generates opcode 0x33 (OP) not 0x3b (OP-32)
- ✅ Atomic operations correct (amoadd.w, etc.)
- ✅ Compiles successfully

### Commit Guidelines (Reference: `00-initial.md`, `.cursorrules`)

- All commits start with `lpc: ` prefix
- Commit frequently with small incremental changes
- Keep commit messages short (one line when possible)
- Keep code compiling between commits

### What Makes This Simple

1. **No format conversion** - CLIF syntax already defined
2. **No complex features** - Just basic lowering tests
3. **Small scope** - 10-15 tests is enough
4. **Existing tools** - Emulator and disassembler already work
5. **Clear verification** - Filecheck + emulator = high confidence

## Example: Complete First Test

**File**: `filetests/riscv32/iadd.clif`

```clif
test compile
target riscv32

function %iadd_i32(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}

; Verify correct RV32 instruction (ADD not ADDW)
; check: add {{[ast][0-9]+}}, {{[ast][0-9]+}}, {{[ast][0-9]+}}
; check-NOT: addw
```

**Test code**: `src/compile.rs`

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_iadd() {
        let content = include_str!("../filetests/riscv32/iadd.clif");
        super::run_test(content).unwrap();
    }
}
```

**Verify**:

```bash
cargo test --package lp-filetests test_iadd
```

## Next Steps

1. **Implement Step 1** - Clean up lp-filetests directory
2. **Implement Step 2** - Create compile.rs module
3. **Implement Step 3** - Write and test iadd.clif
4. **Implement Step 4** - Write remaining core tests
5. **Commit frequently** - One commit per test file or small group

## References

- **Cranelift IR**: `cranelift/docs/ir.md`
- **Existing .clif examples**: `cranelift/filetests/filetests/isa/riscv64/*.clif`
- **RISC-V32 Backend**: `cranelift/codegen/src/isa/riscv32/lower.isle`
- **Previous Plan**: `.plans/00-initial.md`
- **ISLE Cleanup**: `.plans/riscv32-isle-cleanup-guide.md`
- **Emulator**: `crates/lp-riscv-tools/src/emu/`
