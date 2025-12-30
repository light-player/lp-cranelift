# Object File Loader - Overview

## Goal

Build an object file loader that loads PIC (Position-Independent Code) relocatable object files into the emulator *after* a base executable has been loaded. This enables loading user code dynamically without needing to link everything into a single executable.

## Key Design Principles

1. **Explicit State**: No hidden state - return state from `load_elf()`, pass explicitly to `load_object_file()`
2. **Mutate Buffers**: Object loader mutates the base executable's code/ram Vecs directly, extending them as needed
3. **Merge Symbol Maps**: Base and object symbol maps are merged before relocations (base symbols take precedence)
4. **Reuse Existing Code**: Leverage existing relocation handlers, section loading patterns, and symbol building logic
5. **Sequential Placement**: Object files are placed sequentially after the base executable (non-overlapping)

## Architecture

- **Module Structure**: `elf_loader/object/` submodule containing object-file-specific logic
- **API**: `load_object_file()` function that takes mutable references to code/ram/symbol_map
- **Return Types**: `ObjectLoadInfo` with placement addresses and symbol map
- **State Tracking**: `ElfLoadInfo` extended with `symbol_map`, `code_end`, `ram_end`

## Things to Keep in Mind Between Phases

1. **Backward Compatibility**: `load_elf()` API must remain unchanged - only extend `ElfLoadInfo` with new optional fields
2. **Symbol Resolution**: Always merge symbol maps before applying relocations - base symbols win conflicts
3. **Address Calculation**: Object file symbols are section-relative - must adjust based on where sections are placed
4. **GOT Entries**: Object files may have GOT entries - use same GOT tracker mechanism as base executable
5. **Relocation Dependencies**: Reuse existing two-phase relocation approach (analyze, then apply in dependency order)
6. **Memory Layout**: Track where base executable ends (`code_end`, `ram_end`) to place object files sequentially
7. **Error Handling**: Fail fast on undefined symbols, return detailed errors for memory overflow
8. **Testing**: Each phase should be testable independently - add tests as you go

## Success Criteria

- Can load a PIC object file after loading base executable
- Object file's symbols resolve correctly (both internal and external to base)
- Relocations apply correctly (PC-relative, GOT, absolute)
- Multiple object files can be loaded sequentially
- Base executable remains unchanged (no relocation reapplication needed)
- Tests demonstrate faster iteration (base loaded once, multiple object files tested)

