# Rust Style Guide

## Overview

This document outlines the coding style and organizational principles for Rust code in this project. The guide is based on patterns observed in the Cranelift codebase and emphasizes clarity, maintainability, and consistency with Rust best practices.

## File Naming

### General Rules

- **Use `snake_case`** for all Rust source files (`.rs`)
- **Be descriptive**: File names should clearly indicate their purpose
- **One concept per file**: Each file should focus on a single primary concept (type, function, or cohesive set of related functions)

### Examples

```
✅ Good:
- function.rs       (Function type and related functionality)
- types.rs          (Type definitions)
- instructions.rs   (Instruction formats and opcodes)
- legalizer.rs      (Legalization functionality)
- error.rs          (Error types)

❌ Avoid:
- utils.rs          (too generic)
- misc.rs           (unclear purpose)
- stuff.rs          (not descriptive)
```

### Special Files

- **`mod.rs`**: Used for module declarations and re-exports in subdirectories
- **`lib.rs`**: Root module file for libraries (contains public API)
- **`main.rs`**: Entry point for binaries

## Module Organization

### Module Declaration Pattern

Modules are declared explicitly in parent modules. The pattern follows:

1. **Public modules** (`pub mod`) for public APIs
2. **Private modules** (`mod`) for internal implementation details
3. **Re-exports** (`pub use`) to expose the public API

Example from `cranelift/codegen/src/lib.rs`:

```rust
//! Cranelift code generation library.

#![no_std]

#[macro_use]
extern crate alloc;

// Public modules
pub mod ir;
pub mod isa;
pub mod verifier;
pub mod write;

// Private modules
mod context;
mod legalizer;
mod opts;

// Re-exports
pub use crate::context::Context;
pub use crate::verifier::verify_function;
pub use crate::write::write_function;
```

### Module Structure Order

1. **Module doc comment** (`//!`) - explains the module's purpose
2. **Feature gates** (`#![cfg(...)]`) - for optional functionality
3. **External crate declarations** (`extern crate`)
4. **Standard library imports** (with `#[cfg]` for `no_std`)
5. **Public module declarations** (`pub mod`)
6. **Private module declarations** (`mod`)
7. **Public re-exports** (`pub use`)
8. **Type aliases and constants** (if any)

Example from `cranelift/codegen/src/ir/mod.rs`:

```rust
//! Representation of Cranelift IR functions.

mod atomic_rmw_op;
mod builder;
pub mod condcodes;
pub mod constant;
mod debug_tags;
pub mod dfg;
pub mod function;
pub mod types;

pub use crate::ir::builder::{InstBuilder, InstBuilderBase};
pub use crate::ir::function::Function;
pub use crate::ir::types::Type;
```

## Directory Structure

### Hierarchical Organization

Organize code hierarchically by domain and responsibility. Cranelift uses this pattern:

```
cranelift/codegen/src/
├── lib.rs                    # Root module, public API
├── context.rs                # Compilation context
├── ir/                       # IR representation
│   ├── mod.rs                # Module declarations
│   ├── function.rs           # Function type
│   ├── types.rs              # Type system
│   ├── instructions.rs       # Instruction formats
│   ├── dfg.rs                # Data flow graph
│   └── entities.rs           # Entity references
├── isa/                      # Instruction Set Architectures
│   ├── mod.rs
│   ├── riscv32/              # RISC-V 32-bit backend
│   │   ├── mod.rs
│   │   ├── abi.rs
│   │   ├── inst/             # Instruction definitions
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── emit.rs
│   │   │   ├── regs.rs
│   │   │   └── imms.rs
│   │   └── lower.rs
│   └── x64/                  # x86-64 backend
│       └── ...
└── legalizer/                # Legalization passes
    ├── mod.rs
    ├── branch_to_trap.rs
    └── globalvalue.rs
```

### Directory Naming

- **Use `snake_case`** for directory names
- **Group related functionality** in subdirectories
- **Use `mod.rs`** to declare and organize submodules

### When to Create Subdirectories

Create a subdirectory when:

- You have **multiple related files** that form a cohesive unit (e.g., `isa/riscv32/inst/` with `args.rs`, `emit.rs`, `regs.rs`)
- The functionality is **distinct enough** to warrant separation (e.g., ISA backends)
- You need **nested organization** for clarity (e.g., `isa/riscv32/inst/unwind/`)

Keep files flat when:

- You have **1-2 related files** that are tightly coupled
- The functionality is **simple and cohesive** (e.g., `context.rs`, `error.rs`)
- **No clear subdomain** exists

### Subdirectory Module Pattern

When creating subdirectories, use `mod.rs` to organize:

```rust
// isa/riscv32/inst/mod.rs
//! RISC-V 32-bit instruction definitions.

pub mod args;
pub mod emit;
pub mod regs;
pub mod imms;
pub mod encode;

pub use self::args::*;
pub use self::emit::*;
pub use self::regs::*;
```

## Code Organization Within Files

### One Concept Per File

**Each file should focus on a single primary concept**. Examples from Cranelift:

- `function.rs` - The `Function` type and its methods
- `types.rs` - The `Type` type and type-related utilities
- `instructions.rs` - Instruction formats, opcodes, and instruction data
- `legalizer.rs` - Legalization entry point and coordination

### Main Concept at Top

**Place the main concept (primary type, struct, or function) at the top of the file**, immediately after imports.

Example from `cranelift/codegen/src/ir/types.rs`:

```rust
//! Common types for the Cranelift code generator.

use core::fmt;

/// The type of an SSA value.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type(u16);

/// Not a valid type.
pub const INVALID: Type = Type(0);

impl Type {
    /// Get the lane type of this SIMD vector type.
    pub fn lane_type(self) -> Self {
        // Implementation...
    }

    // More methods...
}
```

Example from `cranelift/codegen/src/ir/function.rs`:

```rust
//! Intermediate representation of a function.

use crate::ir::{Block, DataFlowGraph, Function, Layout, /* ... */};

/// A version marker used to ensure serialization compatibility.
#[derive(Default, Copy, Clone, Debug, PartialEq, Hash)]
pub struct VersionMarker;

/// Function parameters used when creating this function.
#[derive(Clone, PartialEq)]
pub struct FunctionParameters {
    base_srcloc: Option<SourceLoc>,
    // ...
}

/// A function represents a sequence of instructions that execute sequentially
/// within a single entry point and a single exit point (which might be early).
pub struct Function {
    // Main struct fields at top...
}
```

### File Structure Order

1. **Module doc comment** (`//!`) - explains the file's purpose
2. **Imports** - organized by:
   - Standard library (`std`/`alloc`/`core`)
   - External crates
   - Internal modules (`crate::`)
   - Super/parent modules (`super::`)
3. **Main concept** - primary type, struct, enum, or function
4. **Supporting types** - related types used by main concept
5. **Implementation blocks** - `impl` blocks for types
6. **Helper functions** - private utilities
7. **Tests** - `#[cfg(test)]` modules at the end

Example from `cranelift/codegen/src/legalizer/mod.rs`:

```rust
//! Legalize instructions.

use crate::cursor::{Cursor, FuncCursor};
use crate::ir::{self, InstBuilder, InstructionData, Value};
use crate::isa::TargetIsa;

mod branch_to_trap;
mod globalvalue;

use self::branch_to_trap::BranchToTrap;
use self::globalvalue::expand_global_value;

/// A command describing how the walk over instructions should proceed.
enum WalkCommand {
    Continue,
    Revisit,
}

/// Perform a simple legalization by expansion of the function.
pub fn simple_legalize(func: &mut ir::Function, isa: &dyn TargetIsa) {
    // Main function implementation...
}

// Helper functions follow...
fn expand_binary(/* ... */) -> WalkCommand {
    // ...
}
```

## Import Organization

### Import Order

1. **Standard library** (`std`, `alloc`, `core`)
2. **External crates** (third-party dependencies)
3. **Internal modules** (`crate::`)
4. **Parent/super modules** (`super::`)
5. **Sibling modules** (`self::` or `crate::module::`)

### Import Grouping

Group imports logically and separate with blank lines. Example from Cranelift:

```rust
// Standard library
use alloc::{boxed::Box, string::String, vec::Vec};
use core::fmt;

// External crates
use cranelift_entity::PrimaryMap;
use regalloc2::RegClass;
use target_lexicon::Triple;

// Internal modules
use crate::ir::{Function, Type, Value};
use crate::isa::TargetIsa;
use crate::machinst::*;

// Super modules
use super::inst::EmitInfo;
```

### Import Style

- **Prefer explicit imports** over wildcards (except for trait imports like `use crate::machinst::*;`)
- **Use multi-line imports** for readability when importing many items
- **Group related items** on the same line when reasonable
- **Use `pub use`** for re-exports in `mod.rs` files

## Visibility and Access Control

### Visibility Levels

Use appropriate visibility modifiers:

- **`pub`**: Public API, intended for external use
- **`pub(crate)`**: Public within the crate, but not exported
- **`pub(super)`**: Public to parent module only
- **`pub(in path)`**: Public to specific module path
- **No modifier**: Private, only accessible within the module

### When to Make Public

- **Public API**: Types and functions intended for external use (e.g., `Function`, `Type`, `Context`)
- **Crate-internal**: Types used across multiple modules within the crate (e.g., `pub(crate) type`)
- **Module-internal**: Keep private unless needed elsewhere

Example from Cranelift:

```rust
// Public API
pub struct Function { /* ... */ }

// Crate-internal
pub(crate) type SourceLocs = SecondaryMap<Inst, RelSourceLoc>;

// Private to module
struct WalkCommand { /* ... */ }
```

## Documentation

### Module Documentation

Every module should have a doc comment explaining its purpose:

```rust
//! Legalize instructions.
//!
//! A legal instruction is one that can be mapped directly to a machine code
//! instruction for the target ISA. The `legalize_function()` function takes as
//! input any function and transforms it into an equivalent function using only
//! legal instructions.
```

### Type Documentation

Document public types, especially enums and structs:

```rust
/// The type of an SSA value.
///
/// The `INVALID` type isn't a real type, and is used as a placeholder in the
/// IR where a type field is present but no type is needed, such as the
/// controlling type variable for a non-polymorphic instruction.
///
/// Basic integer types: `I8`, `I16`, `I32`, `I64`, and `I128`.
/// Basic floating point types: `F16`, `F32`, `F64`, and `F128`.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type(u16);
```

### Function Documentation

Document public functions with purpose, parameters, returns, and errors:

```rust
/// Perform a simple legalization by expansion of the function, without
/// platform-specific transforms.
///
/// This function walks through all instructions and expands illegal instructions
/// into legal equivalents.
pub fn simple_legalize(func: &mut ir::Function, isa: &dyn TargetIsa) {
    // Implementation...
}
```

## Best Practices Summary

### ✅ Do

- **One concept per file** - Keep files focused on a single primary concept
- **Main concept at top** - Place primary type/function immediately after imports
- **Descriptive names** - File names should clearly indicate purpose
- **Logical grouping** - Group related files in subdirectories when appropriate
- **Clear module boundaries** - Use `mod.rs` to organize and re-export
- **Appropriate visibility** - Use `pub` only when needed for the API
- **Document public APIs** - Add doc comments for public items
- **Organize imports** - Group by source, separate with blank lines

### ❌ Don't

- **Don't create unnecessary subdirectories** - Keep it flat when simple
- **Don't use generic names** - Avoid `utils.rs`, `misc.rs`, `common.rs`
- **Don't mix concerns** - Keep related code together, separate unrelated code
- **Don't over-expose** - Keep internals private, use `pub(crate)` for crate-internal
- **Don't skip documentation** - Document public APIs and complex types
- **Don't use wildcard imports** - Except for trait imports in specific contexts

## Examples from Cranelift

### Good Example: Single Concept File

**File**: `cranelift/codegen/src/ir/types.rs`

```rust
//! Common types for the Cranelift code generator.

use core::fmt;

/// The type of an SSA value.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type(u16);

pub const INVALID: Type = Type(0);
// ... type constants ...

impl Type {
    pub fn lane_type(self) -> Self { /* ... */ }
    // ... methods ...
}
```

**Why it's good:**

- Single concept: the `Type` type
- Main type at top
- Clear, descriptive name
- Well-documented

### Good Example: Module Organization

**Structure**: `cranelift/codegen/src/isa/riscv32/`

```
isa/riscv32/
├── mod.rs          # ISA backend definition
├── abi.rs          # ABI implementation
├── inst/           # Instruction definitions
│   ├── mod.rs      # Instruction module organization
│   ├── args.rs     # Instruction arguments
│   ├── emit.rs     # Instruction emission
│   ├── regs.rs     # Register definitions
│   └── imms.rs     # Immediate values
└── lower.rs        # Lowering from CLIF
```

**Why it's good:**

- Clear hierarchy by domain
- Related functionality grouped (all instruction code in `inst/`)
- One concept per file
- Logical organization with `mod.rs` for coordination

### Good Example: Main Concept at Top

**File**: `cranelift/codegen/src/ir/function.rs`

```rust
//! Intermediate representation of a function.

// Imports...

/// A function represents a sequence of instructions...
pub struct Function {
    // Main struct fields...
}

impl Function {
    // Methods...
}
```

**Why it's good:**

- Main concept (`Function`) immediately after imports
- Supporting types follow
- Clear organization

## References

- [The Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Book: Modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
- [Rust Book: Documentation](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments)

