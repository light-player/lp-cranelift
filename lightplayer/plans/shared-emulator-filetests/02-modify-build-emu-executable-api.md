# Phase 2: Modify build_emu_executable API

## Goal

Modify `build_emu_executable` to optionally accept a `SharedEmulatorContext`, enabling shared emulator usage while maintaining backward compatibility.

## Implementation Steps

1. **Update `build_emu_executable` signature**
   - Add optional parameter: `shared_context: Option<&mut SharedEmulatorContext>`
   - When `None`: use existing code path (backward compatible)
   - When `Some`: use shared context path

2. **Implement shared context path**
   - When `shared_context` is provided:
     - Skip loading builtins executable (already loaded)
     - Call `shared_context.link_object_file()` with compiled ELF bytes
     - Get updated symbol map from shared context
     - Create emulator using `shared_context.create_emulator()`
     - Skip bootstrap init (already done)
     - Build function address map from shared context's symbol map
   - Build `GlslEmulatorModule` with the emulator instance

3. **Keep existing path unchanged**
   - When `shared_context` is `None`, use all existing logic:
     - Load builtins executable
     - Link object file
     - Create emulator
     - Run bootstrap init
     - Build function address map

4. **Update call sites**
   - `GlModule<ObjectModule>::build_executable()` - pass `None` for now (backward compatible)
   - Will be updated in Phase 3 to pass shared context

5. **Handle trap information**
   - Traps are still collected from compiled functions
   - Pass traps to `create_emulator()` when using shared context
   - Traps are function-relative offsets, convert to absolute addresses using symbol map

## Success Criteria

- `build_emu_executable` compiles with optional shared context parameter
- Existing code path (no shared context) works exactly as before
- New code path (with shared context) creates emulator without bootstrap init
- Function address map is built correctly from shared symbol map
- Trap information is handled correctly
- All code compiles without warnings (except unused code that will be used later)
- Existing tests still pass (backward compatibility)

## Notes

- Maintain backward compatibility - existing callers should work unchanged
- Function address map should filter for function symbols (addresses < 0x80000000)
- Trap offsets need to be converted to absolute addresses using function addresses from symbol map

