# Extract Full CLIF IR from Compiled Module

## Problem

Currently `format_clif_ir()` only outputs the `main` function's CLIF IR. The user needs the full CLIF IR from the **final compiled module** (ObjectModule), not from ClifModule, so they can paste it into CLIF filetests for validation. This ensures we see the actual IR that's being compiled, catching any bugs that might be hidden in intermediate representations.

## Solution

1. Refactor `compile_clif_to_elf` to split into `build_object_module()` method on `ClifModule` that builds ObjectModule and extracts CLIF
2. During `link_into()`, extract Functions from Context and immediately format them to CLIF text strings
3. Generate ELF bytes from ObjectModule, then drop the ObjectModule (we don't need it after ELF generation)
4. Store only the formatted CLIF strings in `GlslEmulatorModule` (both pre-transform and post-transform)
5. Update `format_clif_ir()` to simply return the stored CLIF strings

**Note**: ObjectModule is only needed temporarily to generate ELF bytes. After that, we only need the CLIF strings for debugging output. Functions must be extracted from Context during `define_function()` before `clear_context()` is called. By formatting CLIF immediately during linking, we avoid storing Function objects and just store the formatted strings we need.

## Implementation Steps

### 1. Modify `link_into()` to format and return CLIF strings

- Update `link_into()` signature to return `Result<(HashMap<String, FuncId>, String), GlslError>` where the String is the formatted CLIF for all functions
- After each `define_function()` call but before `clear_context()`, extract `ctx.func` and format it using `write_function()` from `cranelift_codegen`
- Format all user functions first (sorted by name), then main function
- Add function name comments before each function (e.g., `; function add_float:` or `; function main:`) for clarity
- Concatenate all formatted functions into a single CLIF string
- Return both the name_to_id mapping and the formatted CLIF string
- **Important**: Format raw CLIF (no `//` prefix on each line) - this is for pasting into `.clif` filetests, not for embedding in GLSL test files

### 2. Add `build_object_module()` method to `ClifModule`

- Move the ObjectModule creation logic from `compile_clif_to_elf()` into a new method `build_object_module()` on `ClifModule`
- This method should call `link_into()` which returns `(HashMap<String, FuncId>, String)` where String is the formatted CLIF
- Build the ObjectModule, call `finish()` and `emit()` to get ELF bytes
- Return `(Vec<u8>, String)` where Vec<u8> is ELF bytes and String is formatted CLIF
- Drop the ObjectModule after generating ELF (we don't need it anymore)

### 3. Update `compile_clif_to_elf()` to use `build_object_module()`

- Simply call `module.build_object_module()` which returns `(Vec<u8>, String)` (ELF bytes and CLIF)
- Return only the ELF bytes (CLIF string will be stored separately in GlslEmulatorModule)

### 4. Update `link_glsl_for_emulator()` signature and implementation

- Accept both `original_module: ClifModule` and `transformed_module: ClifModule`
- Format CLIF directly from `original_module` using a helper function (similar to `format_clif_module()` but without `//` prefix - raw CLIF for filetests)
- Call `build_object_module()` for `transformed_module` to get `(Vec<u8>, String)` (ELF bytes and formatted CLIF from actual compiled IR)
- Use the transformed module's ELF bytes for emulator (as before)
- Store both formatted CLIF strings in `GlslEmulatorModule`
- **Note**: The original module CLIF should be formatted without `//` prefix (raw CLIF), matching the format from `link_into()` for consistency

### 5. Update `GlslEmulatorModule` struct

- Replace `main_function_ir` and `original_main_function_ir` fields with:
  - `transformed_clif: Option<String>` - Formatted CLIF string for all functions after transformation
  - `original_clif: Option<String>` - Formatted CLIF string for all functions before transformation
- We don't need to store ObjectModules - they're only used temporarily to generate ELF bytes

### 6. Simplify `format_clif_ir()` method

- Since CLIF strings are already formatted and stored, `format_clif_ir()` becomes a trivial getter
- Implementation: `(self.original_clif.clone(), self.transformed_clif.clone())`
- No formatting logic needed - just return the stored strings
- Keep the method in the trait for the interface, but it's now just returning stored values
- The CLIF strings are already in the correct format (suitable for pasting into `.clif` filetest)

### 7. Update `glsl_emu_riscv32()` to pass both modules

- Keep `original_module` (before transformation) and `module` (after transformation)
- Pass both to `link_glsl_for_emulator(original_module, module, ...)`
- Note: We pass original_module so we can format its CLIF directly (no ObjectModule build needed)

### 8. Update filetest module to handle full CLIF output

- **No changes needed to `test_run/mod.rs`** - it already calls `format_clif_ir()` correctly
- The output format in `test_run/mod.rs` already adds headers: `=== CLIF IR (BEFORE transformation) ===` and `=== CLIF IR (AFTER transformation) ===`
- Ensure CLIF strings formatted in `link_into()` include function names/headers for clarity
- Format should be ready to paste into `.clif` filetest (raw CLIF, not prefixed with `//` on each line)
- Consider adding function name comments (e.g., `; function add_float:` or similar) to make it clear which function is which

## Files to Modify

- `lightplayer/crates/lp-glsl/src/ir/clif_module.rs` - Add `build_object_module()` method and `format_clif_module()` method (or reuse from test_compile.rs)
- `lightplayer/crates/lp-glsl/src/compiler/link.rs` - Refactor `compile_clif_to_elf()` and update `link_glsl_for_emulator()`
- `lightplayer/crates/lp-glsl/src/backend/emu.rs` - Store CLIF strings, update `format_clif_ir()`
- `lightplayer/crates/lp-glsl/src/compiler/mod.rs` - Pass both original and transformed modules to linker
- `lightplayer/crates/lp-glsl-filetests/src/test_run/mod.rs` - **No changes needed** (already handles output correctly)

## Notes

- **ObjectModule doesn't store Function IR** - it only stores compiled machine code, signatures, and symbols
- **ObjectModule is only needed temporarily** - we use it to generate ELF bytes, then drop it
- Functions must be extracted from `ctx.func` during `define_function()` before `clear_context()` is called
- By formatting CLIF immediately during linking, we:
- Avoid storing large Function objects
- Avoid storing ObjectModules (only needed for ELF generation)
- Format once when we have access to the Function, not later
- Store exactly what we need for output (formatted strings)
- Functions extracted from Context represent the exact IR that gets compiled to machine code
- CLIF strings are formatted during linking, ensuring we capture the actual IR being compiled
- Output format matches CLIF filetest format (suitable for pasting into `.clif` filetest)