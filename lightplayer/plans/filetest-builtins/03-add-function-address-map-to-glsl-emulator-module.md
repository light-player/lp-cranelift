# Phase 3: Add function address map to GlslEmulatorModule

## Goal

Add a `HashMap<String, u32>` to `GlslEmulatorModule` to store function addresses from object file loading.

## Changes

1. **Add `function_addresses` field to `GlslEmulatorModule`**:
   - Add `function_addresses: HashMap<String, u32>` field
   - Initialize as empty HashMap in constructor

2. **Update constructor/creation code**:
   - Ensure `function_addresses` is initialized when module is created
   - Consider if we need to populate it during compilation or later during object loading

## Files to Modify

- `lightplayer/crates/lp-glsl/src/exec/emu.rs` (GlslEmulatorModule struct)

## Success Criteria

- `function_addresses` field added to `GlslEmulatorModule`
- Field is properly initialized
- Code compiles without warnings
- No functionality broken (addresses will be populated in next phase)

