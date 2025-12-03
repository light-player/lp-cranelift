# Fork Cranelift Analysis Plan

## Overview

This plan analyzes the effort to fork wasmtime, extract cranelift, gate unnecessary components behind features, convert to no_std, and integrate your existing GLSL frontend and RISC-V32 backend.

**Note**: This plan uses manual lowering for the RISC-V32 backend (from `lp-glsl-vm`). ISLE migration is handled separately in **Plan 01** (`01-riscv32-isle.md`).

## Plan Status & Review

**Last Updated**: Plan updated for Option 1 approach (keep wasmtime, gate behind features)
**Current Workspace**: `/Users/yona/dev/photomancer/lp-cranelift`
**Status**: Plan ready for implementation

### Key Observations

- ✅ **Repository confirmed**: This is the correct fork (`lp-cranelift`, renamed from `wasmtime`)
- ✅ **Approach**: Keep wasmtime structure, gate behind features (minimal changes)
- **ISLE migration**: Separated into Plan 01 - will use manual lowering initially (from `lp-glsl-vm`)

## Working Rules

**Commit Guidelines**:

- All commits must start with `lpc: ` prefix (for Light Player Compiler)
- Commit frequently - make small, incremental changes
- Keep commit messages short - one line when possible
- Strive to keep code compiling and tests passing between commits (when possible, exceptions for refactoring)

**Rationale**:

- `lpc: ` prefix keeps commits clearly separated from upstream wasmtime history
- Frequent commits make it easier to track progress and revert if needed
- Short messages keep git log clean and readable
- Compiling/passing tests between commits reduces debugging complexity

## Repository References

**Source Repository** (existing code to copy from):

- Path: `/Users/yona/dev/photomancer/lp-glsl-vm`
- Contains: GLSL frontend, RISC-V32 backend, emulator, filetests, toy language, runtime

**Target Repository** (wasmtime fork - work happens here):

- Path: `/Users/yona/dev/photomancer/lp-cranelift` (current workspace)
- **Note**: Plan originally referenced `/Users/yona/dev/photomancer/wasmtime`, but current workspace is `lp-cranelift`
- Will contain: Forked cranelift + LP-specific crates/apps + wasmtime (gated)

**Note**: When implementing, copy files FROM `lp-glsl-vm` TO `lp-cranelift` as specified in each task.

## Current State

### Your Codebase (`lp-glsl-vm` - Source: `/Users/yona/dev/photomancer/lp-glsl-vm`)

- **GLSL Frontend**: `crates/lpc-glsl/` - GLSL parser → LPIR compiler
- **IR**: `crates/lpc-lpir/` - SSA IR (similar to CLIF)
- **Codegen**: `crates/lpc-codegen/` - RISC-V32 backend with emulator, assembler
- **Runtime**: `crates/runtime-embive/`, `apps/embive-program/` - no_std runtime
- **Filetests**: `crates/lpc-filetests/` - test infrastructure

### Cranelift Structure (`/Users/yona/dev/photomancer/lp-cranelift/cranelift/`)

- **Core crates**: `codegen/`, `frontend/`, `entity/`, `control/`, `bitset/`, `bforest/`, `module/`
- **Backends**: `x64/`, `aarch64/`, `riscv64/`, `s390x/`, `pulley32/`, `pulley64/`
- **Tools**: `filetests/`, `reader/`, `interpreter/`, `jit/`, `object/`, `native/`
- **Build system**: ISLE codegen, meta crates
- **~419 Rust files** in cranelift directory
- **Current State**: Repository appears to already be a fork (workspace is `lp-cranelift`), some modifications detected (e.g., `cranelift/codegen/src/machinst/mod.rs` modified, `test_block_params.rs` untracked)

## Effort Analysis

### 1. Gate Unnecessary Components Behind Features

**Approach**: Keep all wasmtime components but gate them behind features to exclude from LP builds.

**Gate Behind Features** (not needed for GLSL → CLIF → RISC-V32 no_std builds):

- `crates/wasmtime/` - Wasmtime runtime (gate behind `feature = "wasmtime"`, requires std)
- `crates/wasi*/` - WASI implementations (gate behind `feature = "wasi"`, requires std)
- `crates/c-api/` - C API (gate behind `feature = "c-api"`, requires std)
- `crates/winch/` - Winch compiler (gate behind `feature = "winch"`, requires std)
- `crates/wizer/` - Wizer tool (gate behind `feature = "wizer"`, requires std)
- `cranelift/jit/` - JIT compilation (gate behind `feature = "jit"`, requires std)
- `cranelift/native/` - Native codegen helpers (gate behind `feature = "native"`, requires std)
- `cranelift/object/` - Object file generation (gate behind `feature = "object"`, requires std)
- `cranelift/interpreter/` - Interpreter (gate behind `feature = "interpreter"`, requires std)
- `cranelift/assembler-x64/` - x64 assembler (gate behind `feature = "assembler-x64"`, requires std)
- `cranelift/src/` - CLI tools (`clif-util`, etc.) (gate behind `feature = "cli"`, requires std)
- `cranelift/tests/` - Integration tests (gate behind `feature = "test-integration"`, requires std)
- `pulley/` - Pulley interpreter (gate behind `feature = "pulley"`, requires std)

**Gate Backends** (for testing only, require std):

- `cranelift/codegen/src/isa/x64/` - x64 backend (gate behind `feature = "x86"` or `test-x64`)
- `cranelift/codegen/src/isa/aarch64/` - ARM64 backend (gate behind `feature = "arm64"` or `test-arm64`)
- `cranelift/codegen/src/isa/s390x/` - s390x backend (gate behind `feature = "s390x"` or `test-s390x`)
- `cranelift/codegen/src/isa/riscv64/` - riscv64 backend (gate behind `feature = "riscv64"` or `test-riscv64`)
- `cranelift/codegen/src/isa/pulley*/` - Pulley backends (gate behind `feature = "pulley"`)

**Keep Always** (core functionality, no_std compatible):

- `cranelift/codegen/` - Core codegen (already `#![no_std]`)
- `cranelift/frontend/` - Frontend builder (already `#![no_std]`)
- `cranelift/entity/`, `control/`, `bitset/`, `bforest/` - Core data structures
- `cranelift/module/` - Module management
- `cranelift/filetests/` - Test infrastructure (can be gated if needed)
- `cranelift/reader/` - CLIF parser
- `cranelift/isle/` - ISLE DSL compiler
- `cranelift/serde/` - Serialization (optional, gate behind feature)

**Default Features**:

- `default = ["riscv32"]` - LP-focused defaults (no wasmtime, no std)
- `std` - Enables std-dependent features
- `riscv32` - Your RISC-V32 backend (always available)
- `wasmtime` - Enables wasmtime runtime and related crates
- `pulley` - Enables pulley interpreter (for filetests)
- `test-all-backends` - Enables all backends for testing (requires std)

**Effort**: ~4-6 hours (update Cargo.toml files, add feature gates, minimal code changes)

### 2. Convert to no_std

**Current State**: Most cranelift crates already have `#![no_std]`:

- `cranelift-codegen`: `#![no_std]` with conditional `std` feature
- `cranelift-frontend`: `#![no_std]`
- `cranelift-entity`: `#![no_std]`
- `cranelift-control`: `#![no_std]`
- `cranelift-module`: `#![no_std]`

**Files Requiring Changes** (~15-20 files):

- `cranelift/codegen/src/machinst/mod.rs` - Uses `std::string::String`, `std::fmt::Write`
- `cranelift/codegen/src/timing.rs` - Uses `std::time::*`, `std::any::*`
- `cranelift/codegen/src/souper_harvest.rs` - Uses `std::collections::*`, `std::sync::mpsc`
- `cranelift/codegen/src/result.rs` - Uses `std::string::String`
- `cranelift/codegen/src/isa/mod.rs` - Uses `std::string::String`
- Various test files (can be `#[cfg(feature = "std")]`)

**Changes Needed**:

- Replace `std::string::String` → `alloc::string::String` (requires `extern crate alloc;`)
- Replace `std::collections::*` → `hashbrown::*` (already used conditionally via `#[cfg(not(feature = "std"))]`)
- Replace `std::time::*` → feature-gated or remove timing
- Replace `std::fmt::Write` → `alloc::string::String` or feature-gate (for `Display` implementations)
- Make `souper_harvest` feature-gated (requires `std`, already behind `#[cfg(feature = "souper-harvest")]`)
- Make `timing` feature-gated (requires `std`)

**Dependencies**:

- `hashbrown` - Already in workspace dependencies, used conditionally
- `alloc` crate - Standard library, always available in no_std (via `extern crate alloc;`)
- Ensure `#![no_std]` and `extern crate alloc;` are present in all modified files

**Effort**: ~6-8 hours (systematic find/replace + testing)

### 3. Backend Feature Gating

**Strategy**: Use Cargo features to exclude other backends from no_std builds:

```toml
# In cranelift/codegen/Cargo.toml
[features]
default = ["riscv32"]  # LP-focused defaults
riscv32 = []  # Your backend
std = ["serde?/std"]

# Testing-only features (require std)
test-x64 = ["x86"]
test-arm64 = ["arm64"]
test-riscv64 = ["riscv64"]
test-s390x = ["s390x"]
all-backends = ["test-x64", "test-arm64", "test-riscv64", "test-s390x"]
```

**Files to Modify**:

- `cranelift/codegen/Cargo.toml` - Feature definitions
- `cranelift/codegen/src/isa/mod.rs` - Conditional compilation
- Each backend's `mod.rs` - Add `#[cfg(feature = "...")]`

**Effort**: ~2-3 hours

### 4. Add RISC-V32 Backend (ISLE-based)

**Current State**: Cranelift has `riscv64/` (ISLE-based), we'll port it to `riscv32/`

**Approach**: Port riscv64 ISLE backend to riscv32, removing RV64-specific instructions and adapting types

**Note**: Using ISLE directly (not manual lowering) - this aligns with cranelift's standard approach and Plan 01.

**Integration Steps**:

1. Copy riscv64 backend structure to riscv32:
   - Copy `cranelift/codegen/src/isa/riscv64/` → `cranelift/codegen/src/isa/riscv32/`
   - Copy ISLE files: `lower.isle`, `inst.isle`, `inst_vector.isle`
   - Copy Rust modules: `mod.rs`, `abi.rs`, `settings.rs`, `lower/`, `inst/`
2. Remove RV64-specific code:
   - Remove all `*w` instructions (addw, subw, mulw, divw, remw, sllw, srlw, sraw, addiw, slliw, srliw, sraiw)
   - Remove `fits_in_32` type checks (not needed on RV32)
   - Remove `ty_int_ref_scalar_64` checks
3. Adapt types for 32-bit:
   - Change `$I64` → `$I32` in lowering rules
   - Change `u64_from_imm64` → `u32_from_imm32`
   - Update immediate handling for 32-bit values
   - Update ABI for 32-bit registers
4. Update settings for imac variant:
   - Default to I, M, A, C enabled
   - F, D disabled by default (can be enabled if needed)
   - Remove G-extension requirement (unlike riscv64)
5. Register in build system:
   - Already registered in `cranelift/codegen/src/isa/mod.rs`
   - Already added to `cranelift/codegen/meta/src/isa/mod.rs`
   - Update ISLE build script to include riscv32
6. Reference lp-glsl-vm code as needed:
   - Check instruction encoding patterns if needed
   - Verify ABI matches expectations
   - Use as reference for 32-bit specific behavior

**Files to Create/Adapt**:

- `cranelift/codegen/src/isa/riscv32/` - New directory (port from riscv64)
- `cranelift/codegen/src/isa/riscv32/mod.rs` - Main module (port from riscv64/mod.rs)
- `cranelift/codegen/src/isa/riscv32/lower.isle` - ISLE lowering rules (port from riscv64/lower.isle)
- `cranelift/codegen/src/isa/riscv32/inst.isle` - ISLE instruction definitions (port from riscv64/inst.isle)
- `cranelift/codegen/src/isa/riscv32/inst_vector.isle` - Vector instructions (if needed)
- `cranelift/codegen/src/isa/riscv32/lower/` - ISLE lowering glue code
- `cranelift/codegen/src/isa/riscv32/inst/` - Instruction encoding/emitting (port from riscv64/inst/)
- `cranelift/codegen/src/isa/riscv32/abi.rs` - 32-bit ABI (port from riscv64/abi.rs)
- `cranelift/codegen/src/isa/riscv32/settings.rs` - ISA-specific settings

**Reference**:

- Primary: `cranelift/codegen/src/isa/riscv64/` (port from this)
- Secondary: `lp-glsl-vm/crates/lpc-codegen/src/isa/riscv32/` (reference for 32-bit specifics)

**Effort**: ~6-8 hours (porting, removing RV64 code, adapting types, testing)

**Note**: Using ISLE directly - no manual lowering needed. This is cleaner and aligns with cranelift's architecture.

### 5. Copy RISC-V Testing Infrastructure

**Goal**: Bring over RISC-V emulator, toy language, and filetests to validate the riscv32 backend before integrating other components.

**Focus**: Validate riscv32 backend works correctly with minimal dependencies.

**Code to Copy** (FROM `/Users/yona/dev/photomancer/lp-glsl-vm` TO `/Users/yona/dev/photomancer/lp-cranelift`):

- `lp-glsl-vm/crates/lpc-codegen/src/emu/` → `crates/lp-riscv-tools/src/emu/` (RISC-V emulator)
- `lp-glsl-vm/crates/lpc-codegen/src/isa/riscv32/asm_parser.rs` → `crates/lp-riscv-tools/src/asm_parser.rs`
- `lp-glsl-vm/crates/lpc-codegen/src/isa/riscv32/decode.rs` → `crates/lp-riscv-tools/src/decode.rs`
- `lp-glsl-vm/crates/lpc-codegen/src/isa/riscv32/disasm.rs` → `crates/lp-riscv-tools/src/disasm.rs`
- `lp-glsl-vm/crates/lpc-codegen/src/isa/riscv32/encode.rs` → `crates/lp-riscv-tools/src/encode.rs`
- **Note**: ELF generation not needed - emulator takes raw bytes. Use `cranelift-object` if object files needed.
- `lp-glsl-vm/crates/lpc-filetests/` → `crates/lp-filetests/` (LP-specific filetests infrastructure)
- `lp-glsl-vm/crates/lpc-toy-lang/` → `crates/lp-toy-lang/` (Toy language for backend validation)

**Deferred** (will be brought over later after backend validation):

- `lp-glsl-vm/crates/lpc-glsl/` → `crates/lp-glsl/` (GLSL frontend - defer until backend validated)
- `lp-glsl-vm/apps/embive-program/` → `apps/embive-program/` (defer)
- `lp-glsl-vm/apps/esp32c3-jit-test/` → `apps/esp32c3-jit-test/` (defer)
- `lp-glsl-vm/crates/runtime-embive/` → `crates/lp-runtime-embive/` (defer)
- Structural files (justfile, scripts, etc.) - defer

**New Crate Structure** (minimal, focused on testing):

```
crates/
  lp-riscv-tools/     # RISC-V utilities (emulator, assembler, decoder, disassembler)
  lp-filetests/       # LP-specific filetests (riscv backend tests, toy language tests)
  lp-toy-lang/        # Toy language (for architecture validation)
```

**Note on ELF**: The emulator takes raw bytes directly (`Vec<u8>`), so no ELF generation needed. If object files are required for linking, use `cranelift-object` (already in cranelift).

**Tasks**:

1. Create `crates/lp-riscv-tools/` crate:
   - Copy emulator code
   - Copy RISC-V instruction utilities (asm_parser, decode, disasm, encode)
   - Update imports and paths
   - Ensure no_std compatibility
   - **Skip ELF utilities** - emulator uses raw bytes, use `cranelift-object` if needed
2. Copy `lp-filetests` infrastructure:
   - Copy filetest framework
   - Adapt to work with cranelift's CLIF format
   - Set up riscv32-specific test cases
3. Copy `lp-toy-lang`:
   - Toy language compiler/frontend
   - Adapt to generate CLIF (instead of LPIR)
   - Use for backend validation
4. Update workspace Cargo.toml:
   - Add lp-riscv-tools, lp-filetests, lp-toy-lang to workspace
   - Set up dependencies correctly
5. Create initial riscv32 filetests:
   - Basic instruction tests
   - ABI tests
   - Integration tests using toy language

**Effort**: ~4-5 hours (copy + adapt + integrate + create initial tests)

**Rationale**: Validate riscv32 backend works correctly before bringing over GLSL frontend, runtime, and apps. This incremental approach reduces risk and makes debugging easier.

### 6. Update Build System

**Changes Needed**:

- Update root `Cargo.toml` workspace (add LP testing crates: lp-riscv-tools, lp-filetests, lp-toy-lang)
- Ensure ISLE build scripts include riscv32
- Update `cranelift/Cargo.toml` if needed (umbrella crate)
- Set up filetest infrastructure to work with cranelift

**Effort**: ~2-3 hours

### 7. Testing & Verification

**Tasks**:

- Run existing cranelift filetests with riscv32 backend
- Set up lp-filetests infrastructure:
  - RISC-V backend tests (instruction encoding, ABI, etc.)
  - Toy language → CLIF → riscv32 compilation tests
  - Emulator-based execution tests
- Verify no_std builds work (riscv32 backend, lp-riscv-tools)
- Verify std builds work (for testing, filetests)
- Test RISC-V32 backend compilation pipeline end-to-end:
  - CLIF → riscv32 codegen
  - Binary emission
  - Emulator execution
- Validate riscv32 backend correctness before proceeding

**Effort**: ~5-7 hours

**Success Criteria**:

- riscv32 backend compiles CLIF correctly
- Generated code executes correctly in emulator
- Filetests pass
- Ready to integrate GLSL frontend and other components

## Total Effort Estimate

| Task                        | Hours           | Complexity |
| --------------------------- | --------------- | ---------- |
| Gate unnecessary components | 4-6             | Medium     |
| Convert to no_std           | 6-8             | Medium     |
| Backend feature gating      | 2-3             | Low        |
| Add RISC-V32 backend (ISLE) | 6-8             | Medium     |
| Copy RISC-V testing infra   | 4-5             | Medium     |
| Update build system         | 2-3             | Low        |
| Testing & verification      | 5-7             | Medium     |
| **Total**                   | **28-40 hours** | **High**   |

**Note**: Using ISLE directly for riscv32 - no manual lowering needed. Plan 01 can focus on optimizations rather than initial migration.

## File Change Estimates

- **Files to modify**: ~40-60 (no_std conversion, feature gating)
- **Files to add**: ~50-70 (RISC-V32 backend with manual lowering, GLSL frontend, LP crates)
- **Cargo.toml changes**: ~25-30 files (cranelift + LP crates + root workspace)

**Note**: ISLE migration (Plan 01) will add/modify additional files later.

## Risks & Considerations

1. **API Compatibility**: Cranelift's `MachInst` trait may differ from your `LowerBackend` trait. Need to adapt manual lowering to match.
2. **Upstream Merges**: Keeping wasmtime structure makes merges easier, but feature gating adds complexity.
3. **Testing**: Other backends useful for regression testing, but add complexity.
4. **Size**: Even with feature gating, unused backend code increases compile time.
5. **ISLE Migration**: Manual lowering will need to be migrated to ISLE later (Plan 01) for better maintainability.
6. **Feature Management**: Need to carefully manage feature flags to ensure LP builds work without wasmtime.

## Decisions Summary

1. ✅ **Git history**: Skip history rewrite for now - just delete unneeded files directly
2. ✅ **Cranelift components**: Keep most, gate behind features (not remove)
3. ✅ **Wasmtime components**: Keep all, gate behind features (Option 1 approach)
4. ✅ **Pulley**: Keep and gate behind feature (for filetests)
5. ✅ **Lowering approach**: Use manual lowering initially (from `lp-glsl-vm`), migrate to ISLE later (Plan 01)
6. ✅ **Code organization**: Keep LP-specific code separate in `crates/lp-*` and `apps/lp-*`
7. ✅ **Filetests**: Keep both cranelift-filetests (in cranelift) and LP-filetests (in crates/)
8. ✅ **GLSL frontend**: Separate `lp-glsl` crate (not integrated into cranelift)
9. ✅ **RISC-V tools**: Separate `lp-riscv-tools` crate (emulator, assembler, etc.)
10. ✅ **Toy language**: Bring over for architecture validation
11. ✅ **Repository**: Confirmed - `lp-cranelift` is the correct fork (renamed from `wasmtime`)

## Recommendations

1. **Gradual migration**:
   - Phase 1: Gate components (task 1) - add feature flags, keep everything compiling ✅
   - Phase 2: Convert to no_std (task 2) - fix std imports ✅
   - Phase 3: Backend feature gating (task 3) - gate other backends ✅
   - Phase 4: Add RISC-V32 backend with ISLE (task 4) ✅
   - Phase 5: Copy RISC-V testing infrastructure (task 5) - emulator, toy lang, filetests
   - Phase 6: Update build system (task 6) - integrate testing crates
   - Phase 7: Testing & validation (task 7) - validate riscv32 backend works
   - **Future phases** (after backend validation):
     - Phase 8: Integrate GLSL frontend
     - Phase 9: Bring over runtime and apps
     - Phase 10: Structural files (justfile, scripts, CI)
2. **Validation-first approach**: Validate riscv32 backend before integrating other components
3. **Structure**: Clear separation between cranelift (upstream-ready), wasmtime (gated), and LP-specific code
4. **Testing focus**: Use emulator and toy language to validate backend correctness
5. **Feature Strategy**: Default to LP-focused features (`riscv32`), enable wasmtime features only when needed

## Questions to Resolve

1. ✅ **What to keep/remove from cranelift?**

   - **Answer**: Keep most of it, gate behind features (updated in section 1)

2. ✅ **Do you want to use ISLE for RISC-V32 lowering, or keep manual lowering?**

   - **Decision**: Use ISLE directly (port from riscv64)
   - **Rationale**:
     - Faster to port riscv64 ISLE code than adapt manual lowering from lp-glsl-vm
     - Aligns with cranelift's standard architecture
     - Better maintainability and easier to extend
     - Settings system already gates float instructions correctly
   - **Approach**: Port riscv64 → riscv32, remove RV64-specific code, adapt types
   - **Reference**: Use lp-glsl-vm code as reference for 32-bit specifics, but base on riscv64

3. ✅ **Should `cranelift-riscv32-emu` be separate crate or part of codegen?**

   - **Decision**: Keep LP-specific code separate from cranelift
   - **Structure**:
     - `cranelift/` - Pure cranelift fork (keep clean for upstream merges)
     - `crates/` - LP-specific crates:
       - `lp-glsl/` - GLSL frontend (generates CLIF)
       - `lp-riscv-tools/` or `lp-riscv-util/` - RISC-V utilities (emulator, assembler, disassembler, decoder)
       - Other LP-specific crates as needed
     - `apps/` - LP-specific applications:
       - `embive-program/` - Embedded program
       - `esp32c3-jit-test/` - ESP32 test app
       - Other LP apps
   - **Rationale**: Keeps cranelift clean for upstream merges, LP code clearly separated

4. ✅ **Do you want to keep `cranelift-filetests` or port to your test system?**

   - **Decision**: Keep both
   - **Structure**:
     - `cranelift/filetests/` - Keep in cranelift (for testing cranelift backends/components)
     - `crates/lp-filetests/` - LP-specific filetests for:
       - RISC-V backend validation (riscv32 codegen, ABI, instruction encoding)
       - Toy language → CLIF → riscv32 compilation tests
       - Emulator-based execution tests
   - **Rationale**: Cranelift filetests test cranelift itself; LP filetests validate riscv32 backend correctness
   - **Deferred**: GLSL frontend filetests will be added after backend validation

5. ✅ **Should GLSL frontend be `cranelift-glsl` or separate `lp-glsl` crate?**

   - **Decision**: Separate `lp-glsl` crate in `crates/` directory
   - **Rationale**: Keep LP-specific code separate from cranelift for cleaner upstream merges

6. ✅ **Should we keep wasmtime structure or delete it?**
   - **Decision**: Keep wasmtime structure, gate behind features (Option 1)
   - **Rationale**: Minimal changes, easier upstream merges, everything compiles
   - **Approach**: Gate wasmtime crates behind `wasmtime` feature, keep pulley for filetests

## Clarifications & Open Questions

### Repository Status

✅ **Confirmed**: `/Users/yona/dev/photomancer/lp-cranelift` is the correct repository (renamed from `wasmtime`)

- **Current observation**: Workspace is `lp-cranelift`, git status shows modifications to `cranelift/codegen/src/machinst/mod.rs` and untracked `test_block_params.rs`
- **Status**: This is the correct fork to work in

### RISC-V32 Backend Approach

✅ **Decision**: Use manual lowering from `lp-glsl-vm` initially, migrate to ISLE later (Plan 01)

- **Approach**: Port your existing `riscv32` manual lowering backend from `lp-glsl-vm` to cranelift
  - Pros: Leverages your existing working code, faster initial integration
  - Cons: Will need ISLE migration later for maintainability
- **ISLE Migration**: Separated into Plan 01 - can be done after initial integration is working
- **Rationale**: Get backend working first with known-good code, then optimize/maintain with ISLE

### ISLE no_std Compatibility Details

**Note**: This will be addressed in Plan 01 (ISLE migration)

- **For Plan 00**: Manual lowering should already be no_std compatible (from `lp-glsl-vm`)
- **For Plan 01**: ISLE no_std compatibility will be handled during ISLE migration
  - ISLE compiler (`cranelift/isle/islec`) is build-time only (uses `std`, fine)
  - Generated ISLE code (`lower/isle/generated_code.rs`) - needs verification
  - ISLE integration glue needs fixes: replace `std::boxed::Box`/`std::vec::Vec` with `alloc::boxed::Box`/`alloc::vec::Vec`

### Feature Naming Conventions

**Question**: Should feature names match cranelift's existing conventions?

- **Current cranelift features**: `x86`, `arm64`, `riscv64`, `s390x`, `pulley`
- **Proposed**: `riscv32` (matches pattern), `test-x64`, `test-arm64`, etc.
- **Clarification needed**: Confirm feature naming strategy - should `riscv32` be the primary feature or should it be gated behind `test-riscv32` for consistency?

### Dependencies for no_std

**Question**: What dependencies are required for no_std builds?

- **Known dependencies**:
  - `hashbrown` (already used conditionally in cranelift-codegen)
  - `alloc` crate (standard library, always available)
  - `core` crate (standard library, always available)
- **Clarification needed**:
  - Are there any other dependencies needed?
  - Should we document minimum `alloc` crate requirements?
  - Are there any optional dependencies that should be feature-gated?

### Testing Strategy

**Question**: How will no_std builds be tested?

- **Proposed approach**:
  - Separate test suites: `#[cfg(feature = "std")]` for std tests, `#[cfg(not(feature = "std"))]` for no_std tests
  - Use `cargo test --no-default-features --features riscv32` for no_std testing
  - Keep std tests for comprehensive coverage
- **Clarification needed**:
  - Should we have separate CI jobs for no_std builds?
  - How to test LP-specific crates in no_std mode?

### Migration Timeline

**Question**: Is this a phased migration or all-at-once?

- **Current plan**: Phased approach (8 phases recommended)
- **Clarification needed**:
  - Timeline expectations?
  - Can phases be done incrementally with working checkpoints?
  - Should each phase be independently testable?

### Upstream Synchronization

**Question**: How will upstream changes be handled after forking?

- **Current plan**: Keep wasmtime structure to enable merges
- **Clarification needed**:
  - How often should upstream be synced?
  - What's the strategy for handling conflicts in modified files?
  - Should we maintain a separate branch for upstream tracking?

### Build System Details

**For Plan 00 (Manual Lowering)**:

- Add riscv32 feature to `cranelift/codegen/Cargo.toml`
- Register riscv32 in `cranelift/codegen/src/isa/mod.rs`
- Add riscv32 to `cranelift/codegen/meta/src/isa/mod.rs` (for settings, if needed)

**For Plan 01 (ISLE Migration)**:

- ISLE compilation happens in `cranelift/codegen/build.rs`
- Need to add `riscv32` to ISLE compilation units in `cranelift/codegen/meta/src/isle.rs`
- Need to register riscv32 in `cranelift/codegen/meta/src/isa/mod.rs` ISA definitions

### Current Work Status

**Question**: What work has already been started?

- **Observed**:
  - Modified: `cranelift/codegen/src/machinst/mod.rs`
  - Untracked: `cranelift/codegen/src/machinst/test_block_params.rs`
- **Clarification needed**:
  - What changes were made to `machinst/mod.rs`?
  - Is `test_block_params.rs` related to this migration?
  - Should we incorporate existing work or start fresh?
