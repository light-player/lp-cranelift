# Phase 2: Create Object Submodule Structure

## Goal
Create `elf_loader/object/` submodule with `mod.rs`, `layout.rs`, `sections.rs`, `symbols.rs`, and `relocations.rs`.

## Changes Required

### 1. Create directory structure
```
elf_loader/
  object/
    mod.rs
    layout.rs
    sections.rs
    symbols.rs
    relocations.rs
```

### 2. Create `mod.rs`
- Module declarations
- Public API exports (functions will be added in later phases)
- Re-export key types if needed

### 3. Create placeholder files
- `layout.rs`: Will contain `calculate_object_layout()` (Phase 3)
- `sections.rs`: Will contain `load_object_sections()` (Phase 4)
- `symbols.rs`: Will contain `build_object_symbol_map()` and `merge_symbol_maps()` (Phases 5-6)
- `relocations.rs`: Will contain object file relocation application (Phase 7)

### 4. Update `elf_loader/mod.rs`
- Add `pub mod object;` declaration

## Implementation Details

- Keep files minimal for now - just module structure
- Add necessary imports (`alloc`, `object`, `hashbrown`, etc.)
- Add placeholder function signatures with `todo!()` or `unimplemented!()`

## Testing
- Verify module compiles
- No tests needed yet (placeholders only)

## Success Criteria
- Module structure created
- All files compile
- Module can be imported from `elf_loader`

