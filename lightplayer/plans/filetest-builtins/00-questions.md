# Questions for Filetest Builtins Integration

## Overview
We need to integrate the object file loading system into GLSL filetests, allowing direct function calls instead of requiring a main() wrapper. The process should be:
1. Link the ELF (builtins) and object file (GLSL compiled code)
2. Call bootstrap init code
3. Optionally call user main() if provided (not an error if missing)
4. Allow `call_bool`, `call_i32`, etc. in `exec/emu.rs` to call any function from the object file

## Questions

1. **Function Address Resolution**: ✅ **ANSWERED**: Store function addresses in `GlslEmulatorModule`. They should be populated during object file loading (as a product of calling the object linking function) and stored for fast lookup.

2. **Bootstrap Init Execution**: ✅ **ANSWERED**: Bootstrap init should run once when the emulator is created. The user function should be renamed from "main" to "user_init" (or similar) to reflect that it's one-time setup code, not a repeatedly-called function.

3. **User Init Handling**: ✅ **ANSWERED**: 
   - Rename "main" to "_init" (canonical name)
   - Look for "_init" symbol in object file
   - If present, update `__USER_MAIN_PTR` to point to it
   - If absent, leave `__USER_MAIN_PTR` at sentinel value (not an error)
   - Bootstrap code should handle gracefully: print "jumping to user _init at <address>" or "no user _init specified. halting."

4. **Function Signature Lookup**: ✅ **ANSWERED**: Store all function addresses in a `HashMap<String, u32>` populated during object file loading (from merged symbol map). Use existing `signatures` and `cranelift_signatures` maps that are already populated during GLSL compilation. When calling a function, look up its address from the address map and its signature from the existing signature maps.

5. **ABI Adaptation**: ✅ **ANSWERED**: No changes needed. The existing `call_function` method already accepts a `u32` address and doesn't care about the source. Relocations are applied during object file loading, so addresses from the merged symbol map are already final. The existing ABI handling (argument placement, return value extraction) should work as-is.

6. **Bootstrap Code Removal**: ✅ **ANSWERED**: 
   - Remove main() generation completely from bootstrap.rs
   - Remove tests that test main() itself (filetests are comprehensive enough)
   - Migrate all filetests at once (can break things during migration)

7. **Error Handling**: ✅ **ANSWERED**: 
   - Remove `validate_main_only` and `validate_no_args` restrictions
   - Return clear, descriptive errors via `Result` for:
     - Function doesn't exist in object file
     - Function signature mismatch
     - Bootstrap init failures
   - Validate function exists and signature matches before calling
   - Bootstrap init failures should be fatal (fail fast)

8. **Test Migration**: ✅ **ANSWERED**: 
   - Remove main() test files (`function/main-*.glsl`, `function2/main-entry.glsl`)
   - Update all other filetests to call functions directly (remove main() wrapper generation)
   - Do this all at once (can break things during migration)
   - We'll test the `_init` concept later if needed

