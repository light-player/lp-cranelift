# Relocations Module Rewrite Plan

## Overview

Rewrite `lightplayer/crates/lp-riscv-tools/src/elf_loader/relocations.rs` with a clean, maintainable architecture that properly handles RISC-V relocations, especially GOT (Global Offset Table) entries for PIC (Position-Independent Code).

## Current Problems

1. **Mixed concerns**: Section layout, GOT tracking, and relocation application are intertwined
2. **Duplicated logic**: ROM vs RAM handling is repeated multiple times
3. **GOT entry guessing**: Creating GOT entries by guessing locations (e.g., "12 bytes after auipc")
4. **Multiple passes**: Sections are iterated multiple times for different purposes
5. **Debug noise**: Debug statements scattered throughout making it hard to trace issues
6. **Special cases**: `__USER_MAIN_PTR` handling mixed into general logic

## Architecture

### Module Structure

```
relocations/
├── mod.rs          # Main entry point, public API
├── phase1.rs       # Phase 1: Scan and analyze relocations
├── phase2.rs       # Phase 2: Apply relocations
├── got.rs          # GOT entry tracking and management
├── handlers.rs     # Individual relocation type handlers
└── section.rs      # Section address resolution (VMA/LMA)
```

### Two-Phase Approach

**Phase 1: Analysis Pass**
- Scan all sections and relocations
- Identify GOT entries (R_RISCV_32 relocations that initialize GOT entries)
- Compute GOT entry locations explicitly (don't guess)
- Build relocation dependency graph
- Generate comprehensive debug output showing:
  - All relocations found per section
  - GOT entries identified and their locations
  - Relocation dependencies
  - Section address mappings (VMA/LMA)

**Phase 2: Application Pass**
- Apply relocations in dependency order
- Use known GOT entry addresses from Phase 1
- Patch instructions with correct offsets
- Generate debug output showing:
  - Each relocation being applied
  - Before/after instruction values
  - Computed offsets and addresses

## Relocation Types to Handle

1. **R_RISCV_CALL_PLT** (17): Function call via PLT (auipc+jalr pair)
2. **R_RISCV_GOT_HI20** (19): GOT high 20 bits (for auipc instruction)
3. **R_RISCV_PCREL_HI20** (20): PC-relative high 20 bits (may be used for GOT)
4. **R_RISCV_PCREL_LO12_I** (21, 24): PC-relative low 12 bits (for lw instruction)
5. **R_RISCV_32** (1): 32-bit absolute relocation (used for GOT entry initialization)

## Implementation Plan

### Phase 1: Create Module Structure

1. **Create `relocations/mod.rs`**
   - Public `apply_relocations()` function (main entry point)
   - Re-export necessary types
   - Coordinate Phase 1 and Phase 2

2. **Create `relocations/section.rs`**
   - `SectionAddressInfo` struct: VMA, LMA, buffer slice info
   - `resolve_section_addresses()`: Build map of section name → address info
   - Handle ROM vs RAM sections
   - Handle `.data` sections with LMA in ROM, VMA in RAM

3. **Create `relocations/got.rs`**
   - `GotEntry` struct: symbol name, address, initialized flag
   - `GotTracker` struct: HashMap<String, GotEntry>
   - `identify_got_entries()`: Scan R_RISCV_32 relocations to find GOT entries
   - `get_got_entry()`: Lookup GOT entry by symbol name
   - Debug output: List all GOT entries found

### Phase 2: Implement Phase 1 Analysis

4. **Create `relocations/phase1.rs`**
   - `RelocationInfo` struct: offset, type, target symbol, section, etc.
   - `analyze_relocations()`: Scan all sections, collect relocation info
   - Identify GOT entries from R_RISCV_32 relocations
   - Build dependency graph (GOT_HI20 depends on GOT entry existing)
   - Generate comprehensive debug output:
     ```
     === Phase 1: Relocation Analysis ===
     Section '.text' (VMA: 0x0, LMA: 0x0):
       Relocation at 0x184c: R_RISCV_PCREL_HI20 → '__lp_fixed32_sqrt'
       Relocation at 0x1850: R_RISCV_PCREL_LO12_I → '.L0_20'
     Section '.data' (VMA: 0x80000000, LMA: 0xa18):
       Relocation at 0x0: R_RISCV_32 → '_user_main'
     
     === GOT Entries Identified ===
     '__lp_fixed32_sqrt': R_RISCV_32 at 0x1858 in '.text' (initialized with 0xfa4)
     ```

### Phase 3: Implement Relocation Handlers

5. **Create `relocations/handlers.rs`**
   - `RelocationContext`: Contains all context needed for relocation (buffer, addresses, GOT tracker, etc.)
   - Individual handler functions:
     - `handle_call_plt()`: Patch auipc+jalr pair
     - `handle_got_hi20()`: Patch auipc for GOT access
     - `handle_pcrel_hi20()`: Patch auipc (regular or GOT)
     - `handle_pcrel_lo12_i()`: Patch lw instruction
     - `handle_abs32()`: Write absolute address (GOT entry initialization)
   - Each handler is a pure function: `(context, reloc_info) -> Result<(), String>`
   - Each handler generates debug output showing what it's doing

### Phase 4: Implement Phase 2 Application

6. **Create `relocations/phase2.rs`**
   - `apply_relocations()`: Iterate through analyzed relocations
   - Apply in dependency order (GOT entries first, then references)
   - For each relocation:
     - Look up handler by type
     - Create relocation context
     - Call handler
     - Log result
   - Generate debug output:
     ```
     === Phase 2: Applying Relocations ===
     Applying R_RISCV_32 at 0x1858 in '.text':
       Target: '__lp_fixed32_sqrt' (0xfa4)
       Writing 0x00000fa4 to offset 0x1858
       ✓ GOT entry initialized
     
     Applying R_RISCV_PCREL_HI20 at 0x184c in '.text':
       Target: '__lp_fixed32_sqrt' (GOT entry at 0x1858)
       PC: 0x184c, GOT entry: 0x1858, offset: 0xc
       Instruction: 0x00000617 → 0x00000617 (hi20=0x0)
       ✓ auipc patched
     ```

### Phase 5: Integration and Testing

7. **Update `relocations/mod.rs`**
   - Wire everything together
   - Call Phase 1, then Phase 2
   - Handle errors with clear messages

8. **Update `elf_loader/mod.rs`**
   - Ensure it still calls `relocations::apply_relocations()` correctly
   - No changes should be needed to the public API

9. **Test with existing test**
   - Run `test_load_and_run_bootstrap_app`
   - Verify GOT relocations work correctly
   - Verify debug output is helpful for debugging

## Key Design Decisions

1. **GOT Entry Location**: Don't guess. Identify R_RISCV_32 relocations that initialize GOT entries, record their exact addresses.

2. **PCREL_HI20 for GOT**: When R_RISCV_PCREL_HI20 targets an external symbol (starts with `__lp_`), treat it as GOT access. Use the GOT entry address from Phase 1.

3. **PCREL_LO12_I for GOT**: When the immediate is 12 (typical for GOT), compute offset to GOT entry (auipc_addr + 12), not to auipc label.

4. **Section Address Resolution**: Extract to separate module to avoid duplication. Handle ROM/RAM distinction cleanly.

5. **Debug Output**: Make Phase 1 output comprehensive and easy to read. Use consistent formatting. Show all relevant addresses and offsets.

## Testing Strategy

1. **Unit tests for handlers**: Test each relocation handler independently
2. **Integration test**: Use existing `test_load_and_run_bootstrap_app`
3. **Debug output verification**: Ensure Phase 1 output shows all relocations and GOT entries clearly

## Migration Notes

- The old `relocations.rs` is currently empty (stub), so this is a fresh implementation
- Preserve the public API: `apply_relocations(obj, rom, ram, symbol_map) -> Result<(), String>`
- Keep compatibility with existing `sections.rs` and `symbols.rs` modules

