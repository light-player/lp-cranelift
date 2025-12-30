# Phase 7: Implement Object Relocation Application

## Goal
Implement object file relocation application by reusing existing relocation handlers with the merged symbol map.

## Changes Required

### 1. Implement object file relocation application in `relocations.rs`
- Input: `obj: &object::File`, `code: &mut [u8]`, `ram: &mut [u8]`, `merged_symbol_map: &HashMap<String, u32>`, `section_placement: &ObjectSectionPlacement`
- Output: `Result<(), String>`

### 2. Reuse existing relocation infrastructure
- Use `relocations::phase1::analyze_relocations()` to analyze object file relocations
- Use `relocations::phase2::apply_relocations_phase2()` to apply relocations
- Pass merged symbol map to relocation handlers

### 3. Adjust relocation addresses
- Relocations in object file are section-relative
- Adjust relocation addresses: `final_addr = section_placement + reloc_offset`
- Create `RelocationContext` with adjusted addresses

### 4. Handle GOT entries
- Create `GotTracker` for object file relocations
- Use existing GOT handling logic
- GOT entries placed in object file's memory region

## Implementation Details

- Parse relocations: `obj.section_by_name(".rela.text")` etc.
- Build relocation info: similar to `phase1::analyze_relocations()`
- Adjust addresses: add section placement offset
- Apply relocations: use existing handlers with merged symbol map

## Testing
- Test PC-relative relocations (PCREL_HI20/LO12)
- Test GOT relocations (GOT_HI20)
- Test absolute relocations (R_RISCV_32)
- Test relocations referencing base executable symbols
- Test relocations referencing object file symbols

## Success Criteria
- Relocations are applied correctly
- Symbols resolve correctly (both base and object)
- GOT entries work correctly

