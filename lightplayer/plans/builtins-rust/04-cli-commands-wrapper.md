# Phase 4: Implement CLI tool commands and wrapper script

## Goal

Create `generate-clif` and `generate-binaries` commands with filetest-style error handling (basic structure, can be run incrementally). Create `scripts/build-builtins.sh` wrapper with workspace defaults. Errors should reference the wrapper script.

## Steps

### 4.1 Implement command structure

- Use `clap` to define two subcommands: `generate-clif` and `generate-binaries`
- Define shared arguments:
  - `--builtins-src` - Path to builtins source directory
  - `--output-dir` - Output directory for generated files
  - `--codegen-dir` - Directory for `lp-glsl` integration code
- Define optional arguments:
  - `--target` - Target triple (default: `riscv32imac-unknown-none-elf`)
  - `--verbose` - Verbose output
  - `--force` - Force regeneration
  - `--validate-level` - Validation strictness level

### 4.2 Implement filetest-style error handling

- Create error collection structure
- Implement summary mode (default):
  - Process all builtins
  - Collect errors
  - Show summary at end with commands to run in detail mode
- Implement detail mode:
  - Show full error details as we go
  - Summarize at end (so `tail` catches them)
- Use error codes (like compiler) for easy grepping
- Errors should reference `scripts/build-builtins.sh` wrapper

### 4.3 Create wrapper script

- Create `scripts/build-builtins.sh`
- Use workspace defaults for paths:
  - `--builtins-src` defaults to `lightplayer/crates/lp-glsl-builtins-src`
  - `--output-dir` defaults to `lightplayer/crates/lp-glsl-builtins-tool/src/generated`
  - `--codegen-dir` defaults to `lightplayer/crates/lp-glsl/src/backend/builtins`
- Make script executable
- Add usage documentation

### 4.4 Implement basic command workflows

- `generate-clif` command:
  - Check for nightly Rust (give helpful error if missing)
  - Basic structure (full implementation in Phase 5)
  - Can be run incrementally
- `generate-binaries` command:
  - Basic structure (full implementation in Phase 8)
  - Can be run incrementally

## Files to Create/Modify

### New Files
- `scripts/build-builtins.sh` - Wrapper script

### Modified Files
- `lightplayer/crates/lp-glsl-builtins-tool/src/main.rs` - Implement CLI commands
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/mod.rs` - Add error handling utilities

## Success Criteria

- Two commands exist: `generate-clif` and `generate-binaries`
- Commands accept arguments and show help text
- Filetest-style error handling is implemented (summary and detail modes)
- Wrapper script exists and works with workspace defaults
- Errors reference the wrapper script

## Notes

- Commands can be stubs initially - full implementation comes in later phases
- Error handling should match existing filetest patterns
- Wrapper script should be similar to `scripts/glsl-filetests.sh`

