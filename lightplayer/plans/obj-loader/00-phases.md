# Object File Loader - Implementation Phases

## Phase 1: Extend ElfLoadInfo
Extend `ElfLoadInfo` with `symbol_map`, `code_end`, and `ram_end` fields, and update `load_elf()` to populate them.

## Phase 2: Create Object Submodule Structure
Create `elf_loader/object/` submodule with `mod.rs`, `layout.rs`, `sections.rs`, `symbols.rs`, and `relocations.rs`.

## Phase 3: Implement Object Layout Calculation
Implement `calculate_object_layout()` to compute placement addresses for object file sections after the base executable.

## Phase 4: Implement Object Section Loading
Implement `load_object_sections()` to copy object file sections into the base executable's code/ram buffers at calculated addresses.

## Phase 5: Implement Object Symbol Map Building
Implement `build_object_symbol_map()` to build a symbol map with addresses adjusted for section placement.

## Phase 6: Implement Symbol Map Merging
Implement `merge_symbol_maps()` helper to merge base and object symbol maps (base takes precedence).

## Phase 7: Implement Object Relocation Application
Implement object file relocation application by reusing existing relocation handlers with the merged symbol map.

## Phase 8: Implement load_object_file Entry Point
Implement `load_object_file()` entry point that orchestrates layout, section loading, symbol building, merging, and relocation.

## Phase 9: Add __USER_MAIN_PTR Update Logic
Add `__USER_MAIN_PTR` update logic in `load_object_file()` when a `main` symbol is found.

## Phase 10: Add Unit Tests
Add unit tests for object file loading, including base + object file scenarios.

## Phase 11: Remove executable_linker Module
Remove `executable_linker` module and update any remaining references to use the new object loader.

