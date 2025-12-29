# Phase 10: Integration with `lp-glsl`

## Goal

Generate integration code to `lp-glsl/src/backend/builtins/`, update fixed32 transform to use new registry with macro-based binary CLIF loading.

## Steps

### 10.1 Update generated registry code

- Ensure generated registry code uses `include_bclif!()` macro from `lp-glsl-builtins-loader`
- Verify paths are correct relative to generated registry file
- Test that registry code compiles

### 10.2 Create integration module

- Generate `lp-glsl/src/backend/builtins/mod.rs` - Main builtins module
- Re-export registry and other generated code
- Ensure module structure is correct

### 10.3 Update fixed32 transform

- Modify `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/instructions.rs`:
  - Remove `Fixed32Builtin` enum
  - Use generated `BuiltinId` enum instead
  - Update `CallConversionState` to use new registry
- Modify `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/transform.rs`:
  - Update to use new registry for loading builtins
  - Remove old manual CLIF generation code
- Modify `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/converters/math.rs`:
  - Update `convert_sqrt` to use new `BuiltinId::Fixed32SqrtRecip`

### 10.4 Update dependencies

- Add `lp-glsl-builtins-loader` dependency to `lp-glsl/Cargo.toml`
- Ensure `cranelift-codegen` has `enable-serde` feature enabled
- Ensure `postcard` dependency is available

### 10.5 Test integration

- Verify `lp-glsl` compiles with new registry
- Verify macro expands correctly
- Verify builtin loading works
- Test that `sqrt` function call uses new builtin

## Files to Create/Modify

### New Files (Generated)
- `lightplayer/crates/lp-glsl/src/backend/builtins/mod.rs` - Integration module

### Modified Files
- `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/instructions.rs` - Use `BuiltinId`
- `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/transform.rs` - Use registry
- `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/converters/math.rs` - Use `BuiltinId::Fixed32SqrtRecip`
- `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/builtins.rs` - Remove or update (old manual generation)
- `lightplayer/crates/lp-glsl/Cargo.toml` - Add dependencies

## Success Criteria

- `lp-glsl` compiles with new builtin system
- `Fixed32Builtin` enum is removed
- `BuiltinId` enum is used throughout
- Registry loads builtins using macro
- `sqrt` function call works with new builtin
- Old manual CLIF generation code is removed

## Notes

- This phase replaces the old `Fixed32Builtin` system entirely
- Registry code is generated, not hand-written
- Macro handles binary CLIF loading at compile time
- Integration should be seamless for fixed32 transform

