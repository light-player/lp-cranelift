# Phase 7: Generate registry code and filetests

## Goal

Generate `BuiltinId` enum and `BuiltinRegistry` with dependency detection. Generate textual CLIF to both `lp-glsl/src/backend/builtins/clif/` and `cranelift/filetests/filetests/32bit/builtins/` with formal expectations.

## Steps

### 7.1 Implement dependency detection

- Parse CLIF functions to find calls to other `__lp_*` functions
- Build dependency graph: function → set of dependencies
- Compute transitive closure for each function
- Order functions by dependency (topological sort)

### 7.2 Generate `BuiltinId` enum

- Create enum generation code in `src/generator/registry.rs`
- For each discovered builtin:
  - Strip `__lp_` prefix
  - Split on `_`
  - Convert each segment to PascalCase
  - Join segments → enum variant name
- Example: `__lp_fixed32_sqrt_recip` → `Fixed32SqrtRecip`

### 7.3 Generate `BuiltinRegistry` struct

- Create registry struct with `load_builtin()` method
- For each builtin, generate macro invocation:
  ```rust
  BuiltinId::Fixed32SqrtRecip => include_bclif!("clif/__lp_fixed32_sqrt_recip.clif").load(),
  ```
- Paths are relative to generated registry file location
- Handle dependency ordering (load dependencies first if needed)

### 7.4 Generate textual CLIF files

- Write validated/transformed CLIF functions to `.clif` files
- Output locations:
  - `lp-glsl/src/backend/builtins/clif/` - For integration
  - `cranelift/filetests/filetests/32bit/builtins/` - For filetests
- Include function name and signature in CLIF

### 7.5 Generate CLIF filetests with expectations

- Transform test expectations (from Phase 2) to CLIF runtest format
- Generate `test run` commands for each test case
- Include expected results in filetest format
- Write to `cranelift/filetests/filetests/32bit/builtins/`

### 7.6 Integrate into `generate-clif` workflow

- Add registry generation step
- Add CLIF file generation step
- Add filetest generation step
- Ensure all steps run in correct order

## Files to Create/Modify

### New Files (Generated)
- `lightplayer/crates/lp-glsl/src/backend/builtins/registry.rs` - Generated registry code
- `lightplayer/crates/lp-glsl/src/backend/builtins/clif/*.clif` - Generated CLIF files
- `cranelift/filetests/filetests/32bit/builtins/*.clif` - Generated filetests

### Modified Files
- `lightplayer/crates/lp-glsl-builtins-tool/src/generator/registry.rs` - Implement generation logic
- `lightplayer/crates/lp-glsl-builtins-tool/src/main.rs` - Integrate registry generation

## Success Criteria

- `BuiltinId` enum is generated with correct variants
- `BuiltinRegistry` is generated with `load_builtin()` method
- Textual CLIF files are generated in both locations
- CLIF filetests are generated with test expectations
- Dependency detection works correctly
- `sqrt_recip` appears in registry and filetests

## Notes

- Registry code uses `include_bclif!()` macro (to be implemented in Phase 9)
- Paths must be correct relative to generated registry file
- Filetests should be runnable with existing Cranelift filetest infrastructure

