# Phase 6: Register Host Functions in Emulator

## Goal

Add host function declarations and linking in emulator codegen so emulator mode can call host functions from `lp-builtins-app`.

## Tasks

1. Update `lp-glsl/src/backend/builtins/registry.rs`:
   - Add host functions to builtin registry (or create separate host registry)
   - Declare `__host_debug` and `__host_println` as external symbols

2. Update `lp-glsl/src/backend/codegen/emu.rs`:
   - Declare host functions when creating module
   - Use `Linkage::Import` (same as builtins)
   - Linker will resolve from `lp-builtins-app`

3. Ensure symbol names match:
   - Declared names must match `lp-builtins-app` exports
   - Use `__host_debug` and `__host_println` consistently

4. Update `lp-glsl/src/backend/codegen/builtins_linker.rs` if needed:
   - Verify host functions are in symbol map after linking
   - Check that addresses are non-zero (defined)

## Success Criteria

- Host functions declared in emulator codegen
- Symbols resolve correctly from `lp-builtins-app`
- `host::debug!` and `host::println!` work in emulator execution
- Output appears via syscalls to host
- No undefined symbol errors

