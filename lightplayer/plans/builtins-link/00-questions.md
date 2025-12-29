# Builtin System Architecture - Questions and Options (Linking Approach)

This document captures architectural questions and decisions for the new linking-based builtin system. This approach avoids CLIF generation entirely by compiling Rust functions to ELF binaries and linking them directly.

## Overview

**New Architecture**:

- `lp-builtins`: `no_std` crate with `__lp_xyz` functions (mostly delegating to Rust stdlib)
- **JIT**: `lp-glsl` includes `lp-builtins` as dependency, calls functions directly from host
- **Emulator**: Compile `lp-builtins` to ELF binary, link into emulator test binaries
- **No CLIF**: Rust compiler handles all lowering, no CLIF extraction/transformation needed

---

## Crate Structure

### Q1: What should the crate be named and where should it live?

**Context**: We need a `no_std` crate containing builtin implementations.

**Options**:

- a) `lp-builtins` in `lightplayer/crates/lp-builtins/` - Simple, clear name
- b) `lp-glsl-builtins` in `lightplayer/crates/lp-glsl-builtins/` - More specific, matches existing naming
- c) Keep `lp-glsl-builtins-src` but repurpose it - Reuse existing crate

**Considerations**:

- Should be clear it's for GLSL builtins
- Should indicate it's a library (not a tool)
- Existing `lp-glsl-builtins-src` already exists but was for CLIF generation

**Suggested Answer**: Use `lp-builtins` - simpler name, clear purpose.

**DECISION**: ✅ **lp-builtins** in `lightplayer/crates/lp-builtins/` - Create new crate, keep `lp-glsl-builtins-src` for now (can remove later)

---

## Function Implementation Strategy

### Q2: How should builtin functions be implemented?

**Context**: Functions like `__lp_sqrt_u64` need implementations. We want to use Rust stdlib where possible.

**Options**:

- a) Direct delegation: `pub extern "C" fn __lp_sqrt_u64(n: u64) -> u64 { n.isqrt() }`
- b) Wrapper with validation: Add bounds checking, error handling before delegating
- c) Custom implementation: Reimplement algorithms when stdlib isn't suitable
- d) Hybrid: Use stdlib for simple cases, custom for complex ones

**Considerations**:

- Rust stdlib functions are well-tested and optimized
- Some functions may need custom implementations (e.g., fixed-point math)
- Wrappers add overhead but provide safety
- Custom implementations give more control but more maintenance

**Suggested Answer**: Direct delegation for stdlib-compatible functions, custom for fixed-point and other special cases.

**DECISION**: ✅ **Hybrid with manual definitions** - Manually define each function (`pub extern "C" fn __lp_xyz`), delegate to Rust stdlib where possible (e.g., `n.isqrt()`), custom implementations for fixed-point and special cases. One file per function. Our own tests for custom implementations (fixed32, etc.).

---

## ELF Compilation Target

### Q3: What target triple should we use for ELF compilation?

**Context**: We need to compile `lp-builtins` to ELF for emulator linking. The emulator runs riscv32imac.

**Options**:

- a) `riscv32imac-unknown-none-elf` - Matches emulator exactly
- b) `riscv32-unknown-none-elf` - More generic, may work
- c) Native target for development, riscv32 for final - Easier local testing

**Considerations**:

- Emulator expects RISC-V 32-bit ELF (see `elf_loader.rs`)
- `riscv32imac-unknown-none-elf` matches the emulator's expectations
- Native target won't work for emulator (different ISA)
- Can test riscv32 ELF with emulator locally

**Suggested Answer**: Use `riscv32imac-unknown-none-elf` - matches emulator exactly.

**DECISION**: ✅ **riscv32imac-unknown-none-elf** - Use directly for now. May support multiple targets in the future.

---

## ELF Generation Method

### Q4: How should we generate the ELF binary?

**Context**: We need to compile `lp-builtins` to an ELF file that can be linked into emulator tests.

**Options**:

- a) Static library (`.a`): `rustc --crate-type staticlib` - Traditional linking approach
- b) Object file (`.o`): `rustc --crate-type lib --emit obj` - Single object file
- c) Executable ELF: `rustc --crate-type bin` - Full executable (overkill?)
- d) Use `cargo build --target riscv32imac-unknown-none-elf` - Standard build tool

**Considerations**:

- Static library (`.a`) is standard for linking
- Object file (`.o`) is simpler but may have issues with multiple modules
- Cargo build is more standard and handles dependencies better
- Emulator loader can handle relocations (see `elf_loader.rs`)

**Suggested Answer**: Use `cargo build --target riscv32imac-unknown-none-elf --crate-type staticlib` to generate a `.a` file, or use `--emit obj` for a single `.o` file.

**DECISION**: ✅ **cargo build --target riscv32imac-unknown-none-elf** - Generate static library (`.a` file) using Cargo. Can revisit if needed.

---

## Linking Strategy for Emulator

### Q5: How should we link the builtins ELF into emulator test binaries?

**Context**: Emulator tests need to call builtin functions. We need to link the compiled builtins.

**Options**:

- a) Static linking: Link `.a` or `.o` into test binary at compile time
- b) Dynamic loading: Load builtins ELF separately, resolve symbols at runtime
- c) Embed in test binary: Include builtins code directly in test ELF
- d) Separate ELF + symbol resolution: Load builtins ELF, resolve function addresses

**Considerations**:

- Static linking is simplest - functions are directly callable
- Dynamic loading is more flexible but requires symbol resolution
- Emulator already has ELF loading infrastructure (`elf_loader.rs`)
- `find_symbol_address` can find symbols by name
- Tests might need to load multiple ELF files (test code + builtins)

**Suggested Answer**: Start with static linking (simplest), consider dynamic loading if we need flexibility later.

**DECISION**: ✅ **Static linking** - Link `.a` file into test binary at compile time. Functions are directly callable.

---

## Symbol Visibility and Naming

### Q6: How should builtin functions be exported for linking?

**Context**: Functions need to be callable from both JIT (host) and emulator (ELF).

**Options**:

- a) `#[no_mangle] pub extern "C"` - Standard FFI, works for both
- b) `#[export_name = "__lp_xyz"]` - Explicit export name
- c) Use `#[link_section]` for specific sections - More control over placement
- d) Different names for JIT vs emulator - Separate concerns

**Considerations**:

- `#[no_mangle] pub extern "C"` is standard and works everywhere
- `extern "C"` ensures C ABI compatibility
- `#[no_mangle]` prevents name mangling
- Same functions work for both JIT and emulator

**Suggested Answer**: Use `#[no_mangle] pub extern "C"` - standard, works for both use cases.

**DECISION**: ✅ **#[no_mangle] pub extern "C"** - Standard FFI, works for both JIT and emulator. Already in use.

---

## JIT Integration

### Q7: How should `lp-glsl` call builtin functions in JIT mode?

**Context**: In JIT execution, we can call Rust functions directly from host code.

**Options**:

- a) Direct function calls: Include `lp-builtins` as dependency, call functions directly
- b) Function pointers: Get function pointers, call through pointers
- c) Trait objects: Define a trait, use dynamic dispatch
- d) Separate FFI layer: Create a wrapper layer for JIT calls

**Considerations**:

- Direct calls are simplest and most efficient
- Function pointers allow runtime selection but add indirection
- Trait objects add overhead (vtable lookup)
- Since `lp-glsl` includes `lp-builtins` as dependency, direct calls work fine

**Suggested Answer**: Direct function calls - `lp-glsl` includes `lp-builtins` as dependency, calls functions directly.

**DECISION**: ✅ **Direct function calls** - `lp-glsl` includes `lp-builtins` as dependency, references functions directly. JIT code calls through function pointers set up during module linking.

---

## Build Integration

### Q8: How should the build system generate the ELF binary?

**Context**: We need to compile `lp-builtins` to ELF during the build process.

**Options**:

- a) `build.rs` script in `lp-glsl` or test crate - Runs during build
- b) Separate tool (`lp-builtins-tool`) - Dedicated build tool
- c) Cargo build script - Use `cargo build` directly
- d) Makefile/script wrapper - External build script

**Considerations**:

- `build.rs` integrates well with Cargo
- Separate tool gives more control but adds complexity
- Cargo build script is standard Rust approach
- Need to ensure ELF is regenerated when source changes

**Suggested Answer**: Use `build.rs` in a test utility crate or `lp-glsl` to compile `lp-builtins` to ELF.

**DECISION**: ✅ **build.rs** - Use `build.rs` in test crate (or `lp-riscv-tools`) to run `cargo build --target riscv32imac-unknown-none-elf` for `lp-builtins` and copy the `.a` file. Integrates with Cargo's build system.

---

## Testing Strategy

### Q9: How should we test builtin functions?

**Context**: We need to verify builtins work correctly in both JIT and emulator contexts.

**Options**:

- a) Unit tests in `lp-builtins` - Test functions directly in Rust
- b) Integration tests in emulator - Test via ELF loading
- c) Both - Unit tests for correctness, integration tests for linking
- d) Use existing `// run:` format from old plan - Keep test expectations

**Considerations**:

- Unit tests are fast and easy to write
- Integration tests verify ELF linking works
- Both approaches complement each other
- `// run:` format was for CLIF filetests, may not apply here

**Suggested Answer**: Both unit tests (in `lp-builtins`) and integration tests (in emulator) - unit tests for correctness, integration tests for linking.

**DECISION**: ✅ **Selective unit tests + filetests** - Unit tests in Rust for functions with custom implementations (e.g., fixed32). No unit tests for pure wrappers. Initially rely on GLSL filetests for correctness. May add one or two sanity unit tests for linking verification.

---

## Function Discovery

### Q10: How should we discover which builtin functions exist?

**Context**: We need to know which `__lp_*` functions are available for code generation and linking.

**Options**:

- a) Parse Rust source (using `syn`) - Extract function signatures
- b) Compile and extract symbols from ELF - Get actual exported symbols
- c) Manifest file (e.g., `builtins.toml`) - Explicit list
- d) Use `nm` or `objdump` on compiled ELF - Extract symbol table

**Considerations**:

- Parsing source gives signatures but may miss conditional compilation
- Extracting from ELF gives actual exported symbols
- Manifest file is explicit but requires maintenance
- Symbol extraction is reliable and matches what's actually available

**Suggested Answer**: Extract symbols from compiled ELF using `object` crate or `nm` - gives actual exported symbols.

**DECISION**: ✅ **Extract from ELF** - Use `object` crate or `nm`/`objdump` to extract symbols from compiled `.a` file. All exported functions will be `__lp*` prefixed, making discovery straightforward.

---

## Registry Generation

### Q11: Do we need a registry system for builtins?

**Context**: In the old plan, we generated a `BuiltinId` enum and `BuiltinRegistry`. Do we still need this?

**Options**:

- a) Yes, generate registry - Maps IDs to function pointers/addresses
- b) No, use direct function calls - Simpler, no registry needed
- c) Minimal registry - Just function name → address mapping
- d) Runtime discovery - Query ELF for available functions

**Considerations**:

- JIT can use direct function calls (no registry needed)
- Emulator might need symbol resolution (but can use `find_symbol_address`)
- Registry adds indirection but provides abstraction
- For simple case, direct calls are sufficient

**Suggested Answer**: No registry for JIT (direct calls), use symbol resolution for emulator (via `find_symbol_address`).

**DECISION**: ✅ **Yes, registry needed** - Need to iterate through all builtins to link them for JIT, and need a type-safe way to reference them in the compiler. Registry maps IDs to function pointers/addresses.

---

## Fixed-Point Functions

### Q12: How should fixed-point functions (like `fixed32_sqrt`) be implemented?

**Context**: Fixed-point math doesn't have direct stdlib equivalents.

**Options**:

- a) Custom implementation in `lp-builtins` - Full control
- b) Use existing `fixed32` transform code - Reuse existing logic
- c) Delegate to floating-point then convert - Use stdlib `sqrt` + conversion
- d) Hybrid - Use stdlib where possible, custom for fixed-point

**Considerations**:

- Fixed-point needs custom algorithms (no stdlib equivalent)
- Existing `fixed32` transform has implementations we could reuse
- Floating-point conversion may lose precision
- Custom implementation gives best control

**Suggested Answer**: Custom implementation in `lp-builtins` - fixed-point needs specialized algorithms.

**DECISION**: ✅ **Custom implementation in `lp-builtins`** - Part of builtin-heavy translation strategy. Will have `fixed32/div.rs` with `__lp_fixed32_div`, implement trig functions and other builtins this way. Moving away from GLSL-based intrinsic/builtin system.

---

## Error Handling

### Q13: How should builtin functions handle errors (e.g., division by zero)?

**Context**: Builtins may encounter invalid inputs. How should they behave?

**Options**:

- a) Panic - Standard Rust behavior, but may not be suitable for emulator
- b) Return error codes - C-style error handling
- c) Trap/abort - Use RISC-V trap mechanism
- d) Return sentinel values - E.g., return 0 for invalid sqrt

**Considerations**:

- Panics in `no_std` may not work well in emulator
- Error codes require checking return values
- Traps are standard for invalid operations in RISC-V
- Sentinel values are simple but may hide bugs

**Suggested Answer**: Use traps for invalid operations (matches RISC-V semantics), return sentinel values for edge cases.

**DECISION**: ✅ **Feature-based system** - Use features to configure panic handling. For emulator: trap-based panic handler with `ebreak`. For JIT: TBD, but need a way to handle panics from JIT code. This allows different behavior for different contexts.

---

## Dependencies

### Q14: What dependencies can `lp-builtins` use?

**Context**: `lp-builtins` is `no_std`, so it can't use standard library.

**Options**:

- a) Only `core` - Minimal, most compatible
- b) `core` + `alloc` - Allows heap allocation
- c) `core` + specific crates (e.g., `compiler-builtins`) - Use specialized crates
- d) Full `std` (not `no_std`) - Simplest but may not work for emulator

**Considerations**:

- `no_std` is required for embedded/emulator targets
- `alloc` allows `Vec`, `String`, etc. but requires allocator
- `compiler-builtins` provides low-level functions
- `std` won't work for emulator (no OS)

**Suggested Answer**: Use `no_std` with `core` + `alloc` - allows heap allocation while remaining compatible with emulator.

**DECISION**: ✅ **core only** - No heap allocation. Use only `core`, no `alloc`. Keeps things minimal and avoids needing an allocator.

---

## Code Duplication

### Q15: How do we avoid duplicating builtin code between JIT and emulator?

**Context**: User mentioned "we don't duplicate the code (most of the functions are already in the compiler binary)".

**Options**:

- a) Single source: `lp-builtins` crate used by both - No duplication
- b) Shared library: Compile once, link into both - Single binary
- c) Accept duplication: Separate implementations - Simpler but duplicates code
- d) Code generation: Generate both from same source - More complex

**Considerations**:

- Single crate (`lp-builtins`) used by both eliminates duplication
- JIT includes crate as dependency (functions in compiler binary)
- Emulator links ELF (separate binary but same source)
- This is the key advantage of this approach

**Suggested Answer**: Single `lp-builtins` crate - JIT includes it as dependency, emulator links compiled ELF. Same source, no duplication.

**DECISION**: ✅ **Single source: `lp-builtins` crate used by both** - JIT includes crate as dependency (functions in compiler binary), emulator links compiled ELF (separate binary but same source). No duplication.

---

## Summary

Key advantages of this approach:

- ✅ No CLIF generation/transformation complexity
- ✅ Rust compiler handles all lowering
- ✅ Single source (`lp-builtins`) for both JIT and emulator
- ✅ Direct function calls in JIT (no indirection)
- ✅ Standard ELF linking for emulator

Key questions to resolve:

1. Crate naming and structure
2. ELF compilation and linking method
3. Symbol visibility and discovery
4. Testing strategy
5. Error handling approach
