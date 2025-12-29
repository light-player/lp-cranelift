# Builtin System Architecture - Questions and Options

This document captures architectural questions and decisions that need to be made before implementing the Rust-based builtin system.

## Binary Serialization Format

### Q1: Which binary serialization format should we use for `.bclif` files?

**Context**: We need to serialize `Function` structs to binary format. Cranelift supports serde serialization with the `enable-serde` feature.

**Options**:

- a) `postcard` - Used in wasmtime serialization and cranelift incremental cache (recommended)
- b) `bincode` - Mentioned in some codebase areas but less common
- c) Custom format - More control but more work

**Considerations**:

- `postcard` is `no_std` compatible (important for embedded targets)
- `postcard` is used in cranelift's own caching system
- `bincode` may have better performance but requires `std`

**Suggested Answer**: Use `postcard` for consistency with cranelift and `no_std` compatibility.

**DECISION**: ✅ **postcard** - Use `postcard` for binary serialization.

---

## Rust Compilation Target

### Q2: What target triple should we use when compiling builtins with Cranelift backend?

**Context**: `rustc +nightly -Zcodegen-backend=cranelift` needs a target triple. The builtins will eventually run on riscv32imac.

**Options**:

- a) Native target (current host) - CLIF is mostly target-independent, easier to develop
- b) `riscv32imac-unknown-none-elf` - Matches final target, but may be harder to test locally
- c) Both - Generate for native during development, riscv32 for final artifacts

**Considerations**:

- CLIF IR is mostly target-independent for our use case
- Native target allows easier local testing
- riscv32 target ensures compatibility with final platform
- Some instructions may differ between targets

**Suggested Answer**: Start with native target for development, add riscv32 option later if needed.

**DECISION**: ✅ **riscv32imac-unknown-none-elf** - Use riscv32 target. Can test with interpreter and emulator.

---

## Function Discovery

### Q3: How should we discover builtin functions from source code?

**Context**: We need to find all `#[no_mangle] pub extern "C"` functions in the builtins directory.

**Options**:

- a) Parse Rust source files (using `syn` crate) - Most accurate, finds exact signatures
- b) Walk directory and look for function names in comments/doc - Simple but fragile
- c) Compile to CLIF first, then discover functions from CLIF - Extract `__lp_*` functions from generated CLIF
- d) Use a manifest file (e.g., `builtins.toml`) - Explicit but requires maintenance

**Considerations**:

- Parsing with `syn` gives us exact signatures for validation
- Compiling to CLIF first means we discover what actually gets compiled (handles conditional compilation, etc.)
- CLIF already has function signatures, so we get them for free
- Directory walking is simpler but may miss edge cases
- Manifest file is explicit but adds maintenance burden

**Suggested Answer**: Compile to CLIF first, then discover functions from CLIF by looking for `__lp_*` function names. This ensures we only discover functions that actually compile and gives us signatures directly from CLIF.

**DECISION**: ✅ **Compile to CLIF first, then discover from CLIF** - Compile `lp-glsl-builtins-src` to CLIF, then extract all functions matching `__lp_*` pattern from the generated CLIF files.

---

## CLIF Function Matching

### Q4: How do we match discovered builtins to CLIF functions?

**Context**: After compiling, we get CLIF files. We need to match them to discovered builtins.

**Options**:

- a) Exact name match - `__lp_fixed32_sqrt_recip` in Rust → same name in CLIF
- b) Pattern matching - Match by symbol name patterns
- c) Use function signatures - Match by parameter/return types
- d) Use a mapping file - Explicit mapping between names

**Considerations**:

- `#[no_mangle]` should preserve exact names
- CLIF function names may have mangling or prefixes
- Since we're discovering from CLIF (Q3), matching is automatic - we extract `__lp_*` functions directly
- Need to handle cases where function isn't found

**Suggested Answer**: Since we're extracting `__lp_*` functions from CLIF, matching is automatic. Use exact name match.

**DECISION**: ✅ **Exact name match** - Matching is automatic since we discover functions from CLIF by extracting `__lp_*` pattern.

---

## Validation Rules

### Q5: What validation rules should we enforce on builtin functions?

**Context**: Builtins should be self-contained and not depend on external functions.

**Required checks**:

- No external function calls (`Call`, `CallIndirect`)
- No panics/unreachable (or they're handled gracefully)
- Signature matches expected (parameters and return types)

**Optional checks**:

- No global values or memory accesses?
- No unsupported instructions?
- Size/complexity limits?
- Performance characteristics?

**Questions**:

- Should builtins be allowed to call other builtins? (probably yes, but need to handle dependencies)
- Should we allow `trap` instructions for error handling?
- Should we validate instruction count or function size?

**Suggested Answer**:

- Required: No external calls, no panics, signature match
- Optional: Allow calls to other builtins (handle in dependency order), allow `trap` for error handling
- Don't enforce size/complexity limits initially

**DECISION**: ✅ **All suggested validations** - Enforce: no external calls, no panics, signature match. Allow: calls to other builtins, `trap` instructions. Keep validation clean with one Rust file per validation rule.

---

## Transform Rules

### Q6: What transformations should we apply to CLIF functions?

**Context**: We may need to modify CLIF to ensure compatibility or optimize.

**Options**:

- a) No transformations - Use CLIF as-is from compiler
- b) Convert panics to returns - Replace panic paths with error returns
- c) Remove debug assertions - Strip debug-only code
- d) Optimize for size/speed - Apply optimizations
- e) Ensure ABI compatibility - Verify calling convention

**Considerations**:

- Cranelift backend should already generate valid CLIF
- Panics may need conversion for `no_std` compatibility
- Debug assertions may not be present if compiled with `-C debuginfo=0`
- Optimizations may already be applied by rustc

**Suggested Answer**: Start with minimal transformations (maybe panic conversion if needed). Add more as requirements emerge.

**DECISION**: ✅ **Minimal transformations** - Convert panics to traps (matching existing system). Debug assertions shouldn't exist (compile with `-C debuginfo=0`). Keep transformations to minimum to start. Verify ABI compatibility but don't transform.

---

## Error Handling

### Q7: How should the codegen tool handle errors?

**Context**: Many things can go wrong: compilation failures, parsing errors, validation failures, etc.

**Options**:

- a) Fail fast - Stop on first error
- b) Collect all errors - Continue processing and report all errors at end
- c) Partial success - Generate what we can, report errors for failures
- d) Filetest-style - Summary mode by default, detail mode on demand

**Considerations**:

- Fail fast is simpler to implement
- Collecting all errors is better UX for developers
- Partial success may be useful for incremental development
- Filetest-style matches existing tooling patterns

**Suggested Answer**: Use filetest-style error handling:

- **Summary mode (default)**: Process all builtins, collect errors, show summary at end with commands to run in detail mode
- **Detail mode**: Show full error details as we go, summarize at end (so `tail` catches them)
- Use error codes (like compiler) for easy grepping

**DECISION**: ✅ **Filetest-style error handling** - Summary mode by default (process all, show summary with detail commands), detail mode shows full errors. Use error codes for grepping.

---

## Multiple Functions Per File

### Q8: How should we handle multiple `#[no_mangle]` functions in one file?

**Context**: A single Rust file might export multiple builtin functions.

**Options**:

- a) One function per file - Enforce convention, simpler discovery
- b) Multiple functions per file - More flexible, need to extract all
- c) Support both - Allow either pattern

**Considerations**:

- One per file is simpler and clearer
- Multiple per file allows grouping related functions
- Current structure (`sqrt_recip.rs`) suggests one per file
- Since we extract from CLIF (Q3), it doesn't matter - we'll find all `__lp_*` functions regardless of source file
- Multiple functions per file allows shared utilities/helpers

**Suggested Answer**: Since we extract from CLIF, it doesn't matter - we'll discover all `__lp_*` functions regardless of which file they're in. Support multiple functions per file naturally (useful for shared utilities).

**DECISION**: ✅ **Doesn't matter** - Since we extract from CLIF, we'll find all `__lp_*` functions regardless of source file organization. Multiple functions per file is fine (useful for shared utilities).

---

## Registry Code Generation

### Q9: How should the registry enum be generated?

**Context**: We need to generate `BuiltinId` enum variants from file paths.

**Naming convention**: Function name `__lp_fixed32_sqrt_recip` → Enum variant `Fixed32SqrtRecip`

**Algorithm**:

1. Strip `__lp_` prefix from function name → `fixed32_sqrt_recip`
2. Split on `_` → `["fixed32", "sqrt", "recip"]`
3. Convert each segment to PascalCase → `["Fixed32", "Sqrt", "Recip"]`
4. Join together → `Fixed32SqrtRecip`

**Questions**:

- How to handle special characters? (hyphens, underscores)
- How to handle nested directories? (Doesn't matter - we use function name, not path)

**Options**:

- a) PascalCase - `Fixed32SqrtRecip` (matches Rust conventions)
- b) SCREAMING_SNAKE_CASE - `FIXED32_SQRT_RECIP` (more explicit)
- c) Configurable - Allow both styles

**Suggested Answer**: Use PascalCase (standard Rust enum convention). Derive from function name: strip `__lp_` prefix, split on `_`, convert segments to PascalCase, join.

**DECISION**: ✅ **PascalCase from function name** - `__lp_fixed32_sqrt_recip` → strip `__lp_` → split on `_` → PascalCase each segment → `Fixed32SqrtRecip`.

---

## Generated Code Location

### Q10: Where should generated code for `lp-glsl` be placed?

**Context**: We need to generate integration code that `lp-glsl` uses. Also considering generating filetests.

**Proposed Structure**:

- `lp-glsl-builtins-src` - Source implementations (no_std)
- `lp-glsl-builtins-tool` - CLI tool (can use std, separate crate)
- Generated code goes directly into `lp-glsl/src/backend/builtins/`
- Also generate filetests into `cranelift/filetests/filetests/32bit/builtins/`

**Options**:

- a) `lp-glsl/src/backend/builtins/` - Separate module (recommended)
- b) `lp-glsl/src/backend/transform/fixed32/builtins.rs` - In transform module
- c) Generated at build time into `OUT_DIR` - Standard Rust approach

**Considerations**:

- Separate module keeps concerns separated
- In transform module is simpler but mixes concerns
- `OUT_DIR` is standard but files aren't committed
- Generating filetests allows testing the generated CLIF directly
- Separate tool crate can use std (easier for CLI)

**Suggested Answer**:

- Generate integration code to `lp-glsl/src/backend/builtins/` (committed to git)
- Generate filetests to `cranelift/filetests/filetests/32bit/builtins/` (for testing generated CLIF)
- Use separate `lp-glsl-builtins-tool` crate for CLI (can use std)

**DECISION**: ✅ **Generate directly into `lp-glsl`** - Use separate `lp-glsl-builtins-tool` crate (can use std). Generate integration code to `lp-glsl/src/backend/builtins/`. Also generate filetests to `cranelift/filetests/filetests/32bit/builtins/` for testing generated CLIF.

---

## Dependency Management

### Q11: How should builtin dependencies be handled?

**Context**: One builtin might call another (e.g., `cos` calls `sin`).

**Options**:

- a) No dependencies - Each builtin is independent
- b) Explicit dependencies - Declare in manifest or source
- c) Automatic detection - Parse CLIF to find calls to other `__lp_*` functions
- d) Dependency graph - Build graph and load in order

**Considerations**:

- Builtins should be allowed to call other builtins (e.g., `cos` calls `sin`, or shared utilities like `__lp_util_cordic`)
- Validator should ensure no calls to non-builtin functions (ensures things were inlined correctly)
- Can detect dependencies automatically by parsing CLIF for calls to `__lp_*` functions
- Need to load builtins in dependency order

**Suggested Answer**: Allow builtins to call each other. Automatically detect dependencies by parsing CLIF for calls to `__lp_*` functions. Build dependency graph and load in order. Validator ensures no calls to non-`__lp_*` functions.

**DECISION**: ✅ **Allow builtin-to-builtin calls** - Builtins can call other `__lp_*` functions (including shared utilities like `__lp_util_cordic`). Automatically detect dependencies from CLIF. Build dependency graph and load in order. Validator ensures no calls to non-builtin functions.

---

## Testing Strategy

### Q12: How should we test the builtin system?

**Context**: Need to verify codegen, validation, and integration work correctly.

**Test areas**:

- Builtin correctness (unit tests in source)
- Codegen tool correctness
- CLIF extraction and parsing
- Validation rules
- Registry loading
- End-to-end integration

**Testing Layers**:

1. **Builtin correctness**:

   - Unit tests in builtin source code for edge cases
   - Formal expectations (call args, expected result) in special format
   - Used for: unit tests for correctness AND transformed into CLIF runtests

2. **Generated CLIF filetests** (`cranelift/filetests/filetests/32bit/builtins/`):

   - Use interpreter and emulator
   - Help distinguish CLIF compile/transform bugs vs lowering bugs

3. **Tool unit tests**:

   - Normal unit tests for codegen tool components

4. **Existing GLSL filetests**:
   - Final layer of sanity testing

**Options**:

- a) Unit tests for each component
- b) Integration tests with real builtins
- c) Filetests (like existing GLSL filetests)
- d) All of the above

**Suggested Answer**: Multi-layer testing:

- Builtin unit tests + formal expectations → CLIF runtests
- Generated CLIF filetests (interpreter/emulator)
- Tool unit tests
- Existing GLSL filetests (final sanity check)

**DECISION**: ✅ **Multi-layer testing** - Builtin unit tests + formal expectations (transformed to CLIF runtests), generated CLIF filetests (interpreter/emulator), tool unit tests, existing GLSL filetests. Need to define format for formal expectations.

---

## Version Compatibility

### Q13: How should we handle CLIF version compatibility?

**Context**: CLIF format may change between Cranelift versions. Generated `.bclif` files may become incompatible.

**Options**:

- a) Regenerate on version mismatch - Detect and regenerate
- b) Version check in binary format - Embed version, fail if mismatch
- c) Always regenerate - Don't commit `.bclif`, always generate fresh
- d) Version migration - Support multiple versions

**Considerations**:

- Committing `.bclif` means they can become stale
- Version checks add complexity
- Regenerating is simplest but requires codegen tool
- If we invoke tool as part of `lp-glsl` build, files are always fresh

**Options for build integration**:

- a) Invoke `lp-glsl-builtins-tool` in `lp-glsl/build.rs` - Ensures files are always up-to-date
- b) Manual invocation - Developer runs tool when needed
- c) CI-only - Regenerate in CI, commit results

**Suggested Answer**: Invoke `lp-glsl-builtins-tool` as part of `lp-glsl` build process (via `build.rs`). This ensures generated files are always fresh. Embed version marker in `.bclif` files for safety. Commit both `.clif` (text) and `.bclif` (binary) - text is human-readable, binary is for runtime.

**DECISION**: ✅ **Invoke tool in build.rs** - Run `lp-glsl-builtins-tool` as part of `lp-glsl` build process. Ensures generated files are always fresh. Embed version marker in `.bclif` for safety. Commit both `.clif` and `.bclif`.

---

## Rust Toolchain Requirements

### Q14: What Rust toolchain requirements should we document?

**Context**: Codegen tool needs nightly Rust with Cranelift codegen backend.

**Questions**:

- Should we require specific nightly version?
- How to handle toolchain installation?
- Should we provide installation script?
- Should we have separate commands for CLIF generation vs binary generation?

**Two-Command Approach**:

- **Generate CLIF** (needs nightly rust): Compile source to CLIF, validate, transform
- **Generate binaries** (no nightly needed): Parse existing CLIF, serialize to binary, generate registry code

**Options**:

- a) Document requirement - Let users install manually
- b) Auto-install in build script - Use `rustup component add`
- c) Check and fail gracefully - Verify toolchain, provide helpful error

**Suggested Answer**:

- Two commands: `generate-clif` (needs nightly) and `generate-binaries` (no nightly)
- Tool should be friendly - check requirements and give helpful errors
- No specific nightly version requirement - deal with it as it comes up
- Generated code should include rust version for reproducibility

**DECISION**: ✅ **Two commands + friendly errors** - `generate-clif` (needs nightly) and `generate-binaries` (no nightly). Tool checks requirements and gives helpful errors. No specific nightly version. Include rust version in generated code for reproducibility.

---

## CLI Tool Arguments

### Q15: What arguments should the CLI tool accept?

**Context**: Need to specify input/output directories.

**Required arguments**:

- Builtins source directory
- Output directory for generated files
- Codegen directory for `lp-glsl` integration

**Optional arguments**:

- Target triple
- Verbose/debug flags
- Force regeneration
- Validation level

**Wrapper script**:

- Create `scripts/build-builtins.sh` (like `glsl-filetests.sh`)
- Uses workspace defaults for paths
- Makes it easy to invoke the tool

**Suggested Answer**:

- Use same args where possible between `generate-clif` and `generate-binaries`
- Required: `--builtins-src`, `--output-dir`, `--codegen-dir`
- Optional: `--target`, `--verbose`, `--force`, `--validate-level`
- Create wrapper script `scripts/build-builtins.sh` with workspace defaults

**DECISION**: ✅ **Shared args + wrapper script** - Use same args where possible. Required: `--builtins-src`, `--output-dir`, `--codegen-dir`. Optional: `--target`, `--verbose`, `--force`, `--validate-level`. Create `scripts/build-builtins.sh` wrapper with workspace defaults (like `glsl-filetests.sh`).

---

## Integration with Existing System

### Q16: How should this integrate with existing `Fixed32Builtin` enum?

**Context**: Currently `Fixed32Builtin::SqrtRecip` exists in `instructions.rs`.

**Options**:

- a) Replace entirely - Remove `Fixed32Builtin`, use generated `BuiltinId`
- b) Keep both - `Fixed32Builtin` wraps `BuiltinId`
- c) Migrate gradually - Support both during transition

**Considerations**:

- Clean break is simpler
- Gradual migration is safer
- Need to update all call sites
- This system will eventually replace ALL builtins (not just fixed32)
- Current GLSL builtins are buggy, this system will replace them too

**Suggested Answer**: Replace entirely - remove `Fixed32Builtin`, use generated `BuiltinId`. Update all call sites in one go. This is Phase 1 - eventually this system will replace all GLSL builtins (current GLSL intrinsics are buggy).

**DECISION**: ✅ **Replace entirely** - Remove `Fixed32Builtin`, use generated `BuiltinId`. Update all call sites. This is Phase 1 - eventually replace ALL builtins (including current GLSL intrinsics which are buggy).

---

## Summary of Decisions

1. **Binary format**: ✅ `postcard` - Use `postcard` for binary serialization
2. **Target triple**: ✅ `riscv32imac-unknown-none-elf` - Use riscv32 target, can test with interpreter/emulator
3. **Function discovery**: ✅ **Compile to CLIF first, then discover from CLIF** - Extract `__lp_*` functions from generated CLIF
4. **CLIF matching**: ✅ **Exact name match** - Automatic since we discover from CLIF
5. **Validation**: ✅ **All suggested validations** - No external calls, no panics, signature match. Allow calls to other builtins, `trap` instructions. One Rust file per validation rule.
6. **Transforms**: ✅ **Minimal** - Convert panics to traps. Debug assertions shouldn't exist (compile with `-C debuginfo=0`). Keep to minimum.
7. **Error handling**: ✅ **Filetest-style** - Summary mode (default), detail mode on demand. Use error codes for grepping.
8. **Multiple functions**: ✅ **Doesn't matter** - Extract from CLIF, so file organization doesn't matter. Multiple functions per file fine.
9. **Enum naming**: ✅ **PascalCase from function name** - `__lp_fixed32_sqrt_recip` → strip `__lp_` → split on `_` → PascalCase → `Fixed32SqrtRecip`
10. **Generated code location**: ✅ **Generate directly into `lp-glsl`** - Use separate `lp-glsl-builtins-tool` crate. Generate to `lp-glsl/src/backend/builtins/` and filetests to `cranelift/filetests/filetests/32bit/builtins/`
11. **Dependencies**: ✅ **Allow builtin-to-builtin calls** - Automatically detect dependencies from CLIF. Build dependency graph and load in order.
12. **Testing**: ✅ **Multi-layer** - Builtin unit tests + formal expectations → CLIF runtests, generated CLIF filetests (interpreter/emulator), tool unit tests, existing GLSL filetests
13. **Version compatibility**: ✅ **Invoke tool in build.rs** - Run tool as part of `lp-glsl` build. Embed version marker. Commit both `.clif` and `.bclif`
14. **Toolchain**: ✅ **Two commands + friendly errors** - `generate-clif` (needs nightly) and `generate-binaries` (no nightly). Check requirements, give helpful errors. Include rust version in generated code.
15. **CLI args**: ✅ **Shared args + wrapper script** - Same args where possible. Create `scripts/build-builtins.sh` wrapper with workspace defaults
16. **Integration**: ✅ **Replace entirely** - Remove `Fixed32Builtin`, use `BuiltinId`. Phase 1 - eventually replace ALL builtins
17. **Macro API**: ✅ **`include_bclif!()` with `.clif` path** - `include_bclif!("path/to/file.clif")` generates `fn load() -> FunctionStencil`. Files stored as `.clif`, macro converts to binary at compile time using same Cranelift version.
18. **Registry-macro integration**: ✅ **Macro invocations in generated code** - Generated registry code contains macro invocations for each builtin. Registry provides `load_builtin(id: BuiltinId) -> FunctionStencil` method.
19. **Macro error handling**: ✅ **Compile-time errors via `proc_macro::Error`** - Compile-time errors for file/parsing issues (via `proc_macro::Error`), runtime panic for deserialization (unlikely).
20. **Macro crate dependencies**: ✅ **All needed dependencies** - Macro crate depends on: `syn`, `quote`, `proc_macro2`, `cranelift-reader`, `cranelift-codegen` (with `enable-serde`), `postcard`. Macro parses CLIF and serializes at compile time.

---

## Binary CLIF Loader Macro API

### Q17: What should the macro API look like?

**Context**: We need a proc_macro in `lp-glsl-builtins-loader` that loads `.clif` files at compile time, parses them, and converts to binary format. The key insight is that files are stored as textual `.clif` on the filesystem, and the macro converts to binary using the same version of Cranelift.

**Questions**:

- What should the macro be called? `include_bclif!()`, `load_bclif!()`, `load_function_stencil!()`?
- What should it take as input? Path to `.clif` file (not `.bclif`)?
- What does it generate? A function? A constant? Both?
- What's the return type? `FunctionStencil` or `Result<FunctionStencil, postcard::Error>`?
- How are paths resolved? Relative to crate root? Relative to file location?

**Options**:

- a) `include_bclif!("path/to/file.clif")` → reads `.clif`, parses with `cranelift-reader`, serializes with postcard, generates `fn load() -> Result<FunctionStencil, postcard::Error>`
- b) `include_bclif!("path/to/file.clif")` → same as (a) but also generates `const BYTES: &[u8]`
- c) `include_bclif!("path/to/file.clif")` → reads `.clif`, parses, serializes, generates `fn load() -> FunctionStencil` (panics on error)
- d) `include_bclif!("path/to/file.clif")` → reads `.clif`, parses, generates `const STENCIL: FunctionStencil` (deserializes at compile time)

**Considerations**:

- Files stored as `.clif` (textual) on filesystem - human-readable, version-controlled
- Macro converts to binary at compile time using same Cranelift version as runtime
- Path resolution: `include_str!()` resolves relative to file location
- Error handling: Compile-time parsing errors are better caught early
- Runtime deserialization: Postcard deserialization happens at runtime (simpler than compile-time)
- Version checking: Macro uses same Cranelift version, so version marker should match

**Suggested Answer**:

- Macro name: `include_bclif!()` (takes `.clif` path, converts to binary)
- Input: String literal path to `.clif` file (resolved relative to file location, like `include_str!()`)
- Process: Macro reads `.clif` text, parses with `cranelift-reader` (proc_macro can use std), serializes `FunctionStencil` to binary with postcard, embeds binary bytes in generated code
- Generates: `fn load() -> FunctionStencil` that deserializes from embedded binary bytes (panics on error - compile-time parsing ensures validity)
- Version checking: Not needed - macro uses same Cranelift version as runtime

**DECISION**: ✅ **Option (c)** - `include_bclif!("path/to/file.clif")` generates `fn load() -> FunctionStencil`. Files stored as `.clif`, macro converts to binary at compile time using same Cranelift version.

---

## Registry and Macro Integration

### Q18: How should the registry use the macro?

**Context**: The generated registry code needs to load builtin functions. Should it use the macro directly?

**Options**:

- a) Registry code invokes macro for each builtin - `load_bclif!("builtins/fixed32/sqrt_recip.bclif")` in generated code
- b) Registry stores paths, macro invoked at call site - Registry returns path, caller uses macro
- c) Registry stores pre-loaded stencils - Macro invoked at registry generation time (not possible - macros run at compile time)
- d) Registry uses macro internally - Each builtin gets a macro invocation in the generated registry code

**Considerations**:

- Macros run at compile time, so they can't be invoked by the tool
- Generated code can contain macro invocations
- Registry needs to map `BuiltinId` to loaded `FunctionStencil`
- Paths need to be correct relative to where the generated code lives

**Suggested Answer**:

- Generated registry code contains macro invocations for each builtin
- Registry provides `load_builtin(id: BuiltinId) -> FunctionStencil` method
- Each builtin has a corresponding macro invocation in the generated code
- Paths are relative to the generated registry file location
- Example generated code:
  ```rust
  impl BuiltinRegistry {
      pub fn load_builtin(&self, id: BuiltinId) -> FunctionStencil {
          match id {
              BuiltinId::Fixed32SqrtRecip => include_bclif!("clif/__lp_fixed32_sqrt_recip.clif").load(),
              // ... other builtins
          }
      }
  }
  ```

**DECISION**: ✅ **Option (a)** - Generated registry code contains macro invocations for each builtin. Registry provides `load_builtin(id: BuiltinId) -> FunctionStencil` method. Paths relative to generated file location.

---

## Macro Error Handling

### Q19: How should the macro handle errors?

**Context**: The macro reads a `.clif` file, parses it, and serializes it. Things can go wrong.

**Error scenarios**:

- File not found (compile-time)
- Invalid CLIF format (compile-time - parsing fails)
- Invalid postcard format (runtime - deserialization fails)
- Version mismatch (runtime - though shouldn't happen since macro uses same version)

**Options**:

- a) Compile-time errors for file/parsing issues, runtime errors for deserialization - File not found = compile error (via `proc_macro::Error`), CLIF parse error = compile error (via `proc_macro::Error`), deserialization = runtime panic
- b) All runtime errors - Return `Result`, let caller handle (but file not found must be compile-time)
- c) Compile-time validation - Try to deserialize at compile time, fail if invalid

**Considerations**:

- `include_str!()` fails at compile time if file not found (good - catches missing files early)
- CLIF parsing with `cranelift-reader` can happen at compile time (proc_macro can use std)
- Postcard deserialization happens at runtime (generated code deserializes)
- Compile-time parsing ensures CLIF is valid before embedding binary
- Runtime deserialization errors are unlikely if compile-time parsing succeeded
- Macros can produce compiler errors using `proc_macro::Error` or by returning invalid `TokenStream`

**Suggested Answer**:

- File not found: Compile-time error (macro uses `proc_macro::Error` to produce compiler error)
- CLIF parse error: Compile-time error (macro uses `proc_macro::Error` to produce compiler error)
- Deserialization errors: Runtime panic (unlikely if compile-time parsing succeeded, but postcard deserialization happens at runtime)
- Generated function: `fn load() -> FunctionStencil` (panics on deserialization error - compile-time parsing ensures CLIF validity)

**DECISION**: ✅ **Option (a)** - Compile-time errors for file/parsing issues (via `proc_macro::Error`), runtime panic for deserialization (unlikely). Generated function returns `FunctionStencil` directly.

---

## Macro Crate Dependencies

### Q20: What dependencies should `lp-glsl-builtins-loader` have?

**Context**: The macro crate needs to generate code that uses postcard and FunctionStencil.

**Required dependencies**:

- `proc_macro` (built-in)
- `syn`, `quote`, `proc_macro2` (for macro implementation)
- `cranelift-codegen` with `enable-serde` feature? (for `FunctionStencil` type)
- `postcard`? (for serialization code)
- `cranelift-reader`? (for parsing CLIF at compile time)

**Questions**:

- Should the macro crate depend on `cranelift-codegen`? (needs `FunctionStencil` type)
- Should the macro crate depend on `postcard`? (needs serialization)
- Should the macro crate depend on `cranelift-reader`? (needs to parse CLIF at compile time)
- Or should generated code use these dependencies, not the macro crate itself?

**Options**:

- a) Macro crate depends on all - `cranelift-codegen` (with `enable-serde`), `postcard`, `cranelift-reader`, `syn`, `quote`, `proc_macro2`
- b) Macro crate minimal, generated code uses dependencies - Macro just generates code, caller provides dependencies (but macro needs `cranelift-reader` to parse CLIF)
- c) Macro crate provides re-exports - Macro crate re-exports what's needed

**Considerations**:

- Generated code needs `FunctionStencil` type (from `cranelift-codegen`)
- Generated code needs `postcard::from_bytes()` (from `postcard`)
- Macro itself needs `cranelift-reader` to parse CLIF at compile time
- Macro crate itself doesn't need to deserialize at compile time (just generates code)
- Caller (`lp-glsl`) already depends on `cranelift-codegen`

**Suggested Answer**:

- Macro crate needs: `syn`, `quote`, `proc_macro2` (for macro implementation), `cranelift-reader` (for parsing CLIF at compile time), `cranelift-codegen` with `enable-serde` (for `FunctionStencil` type and serialization), `postcard` (for serialization at compile time)
- Generated code: References `FunctionStencil` and `postcard::from_bytes()` from caller's context
- The macro crate needs these dependencies because it parses CLIF and serializes `FunctionStencil` at compile time

**DECISION**: ✅ **Option (a)** - Macro crate depends on all needed dependencies: `syn`, `quote`, `proc_macro2`, `cranelift-reader`, `cranelift-codegen` (with `enable-serde`), `postcard`. Macro parses CLIF and serializes at compile time.
