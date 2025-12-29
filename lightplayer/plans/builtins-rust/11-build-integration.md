# Phase 11: Build integration

## Goal

Add `build.rs` to `lp-glsl` to invoke `lp-glsl-builtins-tool` during build, ensure generated files are fresh.

## Steps

### 11.1 Create build script

- Create `lightplayer/crates/lp-glsl/build.rs`
- Invoke `lp-glsl-builtins-tool` with appropriate arguments
- Use workspace-relative paths for builtins source and output directories
- Handle tool execution errors gracefully

### 11.2 Determine when to regenerate

- Check if generated files exist and are up-to-date
- Compare timestamps of source files vs generated files
- Regenerate if source is newer or if `--force` is set
- Or always regenerate (simpler, ensures freshness)

### 11.3 Handle tool dependencies

- Ensure `lp-glsl-builtins-tool` is built before running
- Handle case where tool doesn't exist (helpful error message)
- Check for nightly Rust requirement (for `generate-clif`)

### 11.4 Integrate with Cargo build

- Use `std::process::Command` to invoke tool
- Set appropriate environment variables
- Capture and report errors
- Ensure build fails if tool fails

### 11.5 Test build integration

- Run `cargo build` on `lp-glsl`
- Verify tool is invoked
- Verify generated files are created/updated
- Verify compilation succeeds with generated code

## Files to Create/Modify

### New Files
- `lightplayer/crates/lp-glsl/build.rs` - Build script

### Modified Files
- `lightplayer/crates/lp-glsl/Cargo.toml` - Ensure `build.rs` is specified (if needed)

## Success Criteria

- `build.rs` invokes `lp-glsl-builtins-tool` during build
- Generated files are created/updated before compilation
- Build fails gracefully if tool fails
- Generated code is always fresh
- Build works in CI and local environments

## Notes

- Build script should use workspace-relative paths
- Consider caching generated files (but ensure freshness)
- Tool invocation should be fast (don't regenerate unnecessarily)
- Error messages should be helpful (reference wrapper script)

