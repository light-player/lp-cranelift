# Filetest Builtins Integration - Overview

## Goal

Integrate the object file loading system into GLSL filetests, allowing direct function calls instead of requiring a main() wrapper. This enables filetests to call user functions directly (e.g., `add_float(1.5, 2.5)`) without generating a main() wrapper.

## Key Changes

1. **Rename main to _init**: The user initialization function is renamed from "main" to "_init" to reflect that it's one-time setup code, not a repeatedly-called function.

2. **Object file loading integration**: GLSL compilation will now:
   - Compile GLSL to an object file
   - Load the builtins executable (ELF)
   - Load the GLSL object file into the emulator
   - Store function addresses from the merged symbol map

3. **Bootstrap init execution**: When the emulator is created, bootstrap init code runs once:
   - Initializes .bss and .data sections
   - Optionally calls user `_init` if present (gracefully handles missing)
   - Emulator is then ready for direct function calls

4. **Direct function calls**: The `call_*` methods (`call_bool`, `call_i32`, etc.) will:
   - Support calling any function from the object file (not just main)
   - Look up function addresses from the stored address map
   - Use existing signature maps for ABI handling
   - Return clear errors for missing functions or signature mismatches

5. **Filetest changes**: Filetests will:
   - No longer generate main() wrappers
   - Call functions directly from `// run:` directives
   - Remove tests that specifically test main() requirements

## Architecture

- **Object Loader**: Already implemented, needs to look for "_init" instead of "main"
- **Bootstrap Code**: Needs graceful handling of missing user _init
- **GlslEmulatorModule**: Needs function address map, needs to run bootstrap init
- **Filetest Bootstrap**: Remove main() generation logic
- **Filetest Execution**: Parse and call functions directly from run directives

## Success Criteria

- All filetests pass without main() wrappers
- Functions can be called directly by name
- User _init runs if present, gracefully skipped if missing
- Clear error messages for missing functions or signature mismatches
- Code is clean, well-documented, and maintainable

