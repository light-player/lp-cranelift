# Phase 6: Set up ELF generation

## Goal

Create `build.rs` script to compile `lp-builtins` to static library (`.a`) for `riscv32imac-unknown-none-elf` target.

## Steps

### 6.1 Create build.rs script

- Create `lightplayer/crates/lp-builtins/build.rs`
- Use `std::process::Command` to run `cargo build`
- Target: `riscv32imac-unknown-none-elf`
- Output: static library (`.a` file)

### 6.2 Configure build output

- Set `OUT_DIR` environment variable or use default target directory
- Locate generated `.a` file in `target/riscv32imac-unknown-none-elf/release/` (or debug)
- Copy or reference the `.a` file for linking

### 6.3 Add build dependencies

- Add `build-dependencies` to `Cargo.toml` if needed
- May need `cc` crate for linking, or just use `cargo build` command

### 6.4 Test build

- Run `cargo build --target riscv32imac-unknown-none-elf` manually first
- Verify `.a` file is generated
- Verify it contains expected symbols (`nm` or `objdump`)

## Implementation Approach

Option 1: Use `cargo build` command in `build.rs`:
```rust
std::process::Command::new("cargo")
    .args(&["build", "--target", "riscv32imac-unknown-none-elf", "--release"])
    .current_dir(workspace_root)
    .status()?;
```

Option 2: Use `cc` crate to compile directly (more control but more complex)

## Files to Create

- `lightplayer/crates/lp-builtins/build.rs`

## Files to Modify

- `lightplayer/crates/lp-builtins/Cargo.toml` (add `build-dependencies` if needed)

## Success Criteria

- `build.rs` successfully compiles `lp-builtins` to `.a` file
- `.a` file contains `__lp_fixed32_*` symbols
- Build integrates with Cargo's build system

## Notes

- May need to install `riscv32imac-unknown-none-elf` target: `rustup target add riscv32imac-unknown-none-elf`
- Consider whether to build in `build.rs` or have separate build step
- `.a` file location: `target/riscv32imac-unknown-none-elf/release/liblp_builtins.a`


