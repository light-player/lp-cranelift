# Build Issue: minimal-lexical no_std Support

## Problem

The `esp32c6-glsl-jit` build fails because `minimal-lexical` v0.2.1 (a transitive dependency via `nom` → `glsl` parser) doesn't properly support `no_std` compilation for `riscv32imac-unknown-none-elf` target.

### Error Details

```
error[E0463]: can't find crate for `std`
error[E0277]: the trait bound `StackVec: core::cmp::Eq` is not satisfied
error: could not compile `minimal-lexical` (lib) due to 119 previous errors
```

### Dependency Chain

```
esp32c6-glsl-jit
└── glsl (git: Yona-Appletree/glsl-parser, branch: feature/spans)
    └── nom_locate v4.2.0 (with std feature enabled)
        └── nom v7.1.3 (with std feature)
            └── minimal-lexical v0.2.1 (requires std, has no_std bugs)
```

## Root Cause

1. The `glsl` parser enables `nom_locate`'s default features, which includes `std`
2. `nom_locate` with `std` feature enables `nom`'s `std` feature
3. `nom` with `std` feature pulls in `minimal-lexical` with `std` requirements
4. `minimal-lexical` v0.2.1 has bugs when compiled for `no_std` targets (missing trait implementations, std dependencies)

## Solutions

### Option 1: Fork and Fix glsl Parser (Recommended)

Fork the `glsl` parser repository and modify its `Cargo.toml` to not enable `nom_locate`'s `std` feature:

```toml
[dependencies.nom_locate]
version = "4.2.0"
default-features = false
features = ["alloc"]  # Use alloc instead of std
```

Then update `apps/esp32c6-glsl-jit/Cargo.toml`:

```toml
glsl = { git = "https://github.com/YOUR_USERNAME/glsl-parser.git", branch = "feature/spans-no-std" }
```

### Option 2: Fork and Fix minimal-lexical

Fork `minimal-lexical` and fix its `no_std` support by:
- Adding missing trait implementations (`PartialEq`, `Eq`, `PartialOrd`, `Ord` for `StackVec`)
- Removing `std` dependencies
- Adding proper `#![no_std]` support

Then patch it in `Cargo.toml`:

```toml
[patch.crates-io]
minimal-lexical = { git = "https://github.com/YOUR_USERNAME/minimal-lexical", branch = "no-std-fix" }
```

### Option 3: Use Different GLSL Parser

Find or create a GLSL parser that properly supports `no_std` without requiring `nom` with `std` features.

### Option 4: Wait for Upstream Fixes

Monitor upstream repositories for fixes:
- `minimal-lexical`: https://github.com/rust-lang/minimal-lexical
- `nom`: https://github.com/rust-bakery/nom
- `glsl` parser: https://github.com/Yona-Appletree/glsl-parser

## Current Status

- ✅ Release profile optimizations added
- ✅ Size analysis tools created
- ✅ Dependency analysis completed
- ❌ Build blocked by `minimal-lexical` no_std issue

## Next Steps

1. Choose one of the solutions above
2. Implement the fix
3. Test the build
4. Once build succeeds, run size analysis:
   ```bash
   ./analyze-esp32c6-size.sh
   cargo bloat --release --target riscv32imac-unknown-none-elf --package esp32c6-glsl-jit -n 50
   ```



