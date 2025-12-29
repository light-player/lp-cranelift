# Phase 5: Implement CLIF extraction

## Goal

Compile `lp-glsl-builtins-src` with Cranelift backend, parse CLIF files, extract `__lp_*` functions.

## Steps

### 5.1 Implement compilation to CLIF

- Create `compile_to_clif()` function in `src/generator/extract.rs`
- Use `rustc +nightly -Zcodegen-backend=cranelift` to compile builtins source
- Set appropriate flags:
  - `-C opt-level=2`
  - `-C panic=abort`
  - `-C overflow-checks=off`
  - `-C debuginfo=0`
  - `-C link-dead-code=off`
  - `-C codegen-units=1`
  - `-C target-feature=+crt-static`
  - Target: `riscv32imac-unknown-none-elf` (or from `--target` arg)
- Capture CLIF output (written to `.clif` files in output directory)

### 5.2 Implement CLIF parsing

- Create `parse_clif_file()` function
- Use `cranelift-reader::parse_functions()` to parse CLIF text
- Handle parsing errors gracefully
- Return collection of `Function` objects

### 5.3 Implement function discovery

- Create `discover_builtins()` function
- Extract all functions matching `__lp_*` pattern from parsed CLIF
- Build mapping: function name → `Function` object
- Handle cases where expected functions are missing

### 5.4 Integrate with CLI command

- Wire up `generate-clif` command to use extraction functions
- Output discovered functions (for debugging)
- Handle errors appropriately (filetest-style)

### 5.5 Test with `sqrt_recip`

- Run `generate-clif` command
- Verify `__lp_fixed32_sqrt_recip` is discovered
- Verify CLIF is parsed correctly
- Check function signature matches expectations

## Files to Create/Modify

### Modified Files
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/extract.rs` - Implement extraction logic
- `lightplayer/crates/lp-glsl-builtins-tool/src/main.rs` - Wire up `generate-clif` command

## Success Criteria

- `generate-clif` command compiles builtins source to CLIF
- CLIF files are parsed correctly
- `__lp_*` functions are discovered and extracted
- `sqrt_recip` builtin is found and parsed
- Errors are handled with filetest-style reporting

## Notes

- Reference `lp-glsl-builtins-src/build.sh` for compilation flags
- CLIF files are written by `rustc` to a `.clif` subdirectory
- Function discovery happens after parsing (not from source files)

