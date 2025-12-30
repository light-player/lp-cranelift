# Object Loader - Design Questions

## Architecture & Integration

### Q1: Unified vs Separate Modules

Should we:

- **Option A**: Create a new unified `loader` module that handles both ELF executables and object files?
- **Option B**: Extend the existing `elf_loader` module to also handle object files? ✅ **SELECTED**
- **Option C**: Create a separate `obj_loader` module that works alongside `elf_loader`?

**Considerations**:

- Code sharing between ELF and object loading
- API clarity and discoverability
- Future extensibility (loading multiple object files)

**Answer**: **Option B** - Extend the existing `elf_loader` module.

**Rationale**:

- `elf_loader` already has all the infrastructure we need (relocations, symbols, sections)
- Base executable's symbol map is naturally available for object file loading
- Maintains backward compatibility (existing `load_elf()` continues to work)
- Keeps related functionality co-located
- Enables future optimization of loading multiple object files without reloading base

### Q2: Module Structure

If we extend `elf_loader` or create a unified loader, what should the module structure look like?

- Should object file loading be a submodule (`elf_loader::object`)? ✅ **SELECTED**
- Should it be parallel modules (`elf_loader` and `obj_loader` sharing common code)?
- Should common functionality be extracted to a shared module?

**Answer**: Add an `object` submodule within `elf_loader`.

**Proposed Structure**:

```
elf_loader/
  mod.rs              - Public API (load_elf, load_object_file)
  parse.rs            - ELF parsing (shared)
  layout.rs           - Layout calculation (shared/extended)
  sections.rs         - Section loading (shared/extended)
  symbols.rs          - Symbol maps (shared)
  relocations/        - Relocation handling (shared/extended)
    mod.rs
    handlers.rs
    ...
  object/             - Object file specific logic (NEW)
    mod.rs            - Object file loading entry point
    layout.rs         - Object file layout calculation
    sections.rs       - Object file section loading
```

**Rationale**:

- Keeps object file logic separate but within the same module
- Allows sharing common code (relocations, symbols) without duplication
- Clear API: `elf_loader::load_elf()` and `elf_loader::load_object_file()`
- Can share internal helpers without exposing them publicly

### Q3: State Management

For the goal of loading multiple object files without reloading the base executable:

- Should we maintain state (base executable's code/ram buffers, symbol map) in a loader struct?
- Or should we return this state and pass it to subsequent object file loads? ✅ **SELECTED**
- How do we handle the case where we want to reload the base executable?

**Answer**: Return state from `load_elf()` and pass it explicitly to `load_object_file()`.

**Proposed Approach**:

```rust
// Load base executable - returns state
let base_info = load_elf(base_bytes)?;

// Load object file - takes base state
let obj_info = load_object_file(
    obj_bytes,
    &base_info,  // Contains code, ram, symbol_map, etc.
)?;

// Load another object file - can reuse same base state
let obj_info2 = load_object_file(obj_bytes2, &base_info)?;
```

**State Structure**:

```rust
pub struct BaseLoadState {
    pub code: Vec<u8>,
    pub ram: Vec<u8>,
    pub symbol_map: HashMap<String, u32>,
    pub code_end: u32,  // Where base executable's code ends
    pub ram_end: u32,   // Where base executable's RAM ends
}
```

**Rationale**:

- Simple: no hidden state, explicit data flow
- Flexible: caller controls when to reload the base
- Testable: easy to test object loading independently
- Future-friendly: can add builder/stateful API later if needed

**Note**: Memory cleaning (re-zeroing) between loads may be desired later, but not a primary concern now.

## API Design

### Q4: Return Types

What should the API return?

**Answer**: Extend `ElfLoadInfo` with symbol map and end addresses, return separate `ObjectLoadInfo` for object files.

**Proposed Structure**:

```rust
// Base executable loading - extended with symbol map
pub struct ElfLoadInfo {
    pub code: Vec<u8>,
    pub ram: Vec<u8>,
    pub entry_point: u32,
    pub symbol_map: HashMap<String, u32>,  // NEW: needed for object files
    pub code_end: u32,  // NEW: where code sections end
    pub ram_end: u32,   // NEW: where RAM sections end
}

// Object file loading - returns info about what was loaded
pub struct ObjectLoadInfo {
    pub main_address: Option<u32>,  // Address of 'main' function if found
    pub symbol_map: HashMap<String, u32>,  // Object file's symbols
    pub text_start: u32,  // Where object file's .text was placed
    pub data_start: u32,  // Where object file's .data was placed
}
```

**Function Signature**:

```rust
pub fn load_object_file(
    obj_file_bytes: &[u8],
    code: &mut Vec<u8>,           // Mutate base's code buffer
    ram: &mut Vec<u8>,             // Mutate base's ram buffer
    symbol_map: &mut HashMap<String, u32>,  // Add object file's symbols
) -> Result<ObjectLoadInfo, String>
```

**Usage Pattern**:

```rust
// Load base
let mut base_info = load_elf(base_bytes)?;

// Load object file - mutates base_info's buffers
let obj_info = load_object_file(
    obj_bytes,
    &mut base_info.code,
    &mut base_info.ram,
    &mut base_info.symbol_map,
)?;

// Create emulator with updated memory
let emu = Riscv32Emulator::new(base_info.code, base_info.ram);
```

**Rationale**:

- Clear separation: base vs object file information
- Reusable: base state can be reused for multiple object files
- Extensible: easy to add more object files later
- Backward compatible: `ElfLoadInfo` extends existing struct
- Mutates Vecs directly: caller manages buffers, loader extends them

### Q5: Function Signatures

Should we have:

- **Option A**: Separate functions (`load_elf()`, `load_object_file()`) ✅ **SELECTED**
- **Option B**: Unified function (`load(base_elf, object_files)`)
- **Option C**: Builder pattern (`Loader::new().load_base().load_object().finish()`)

**Answer**: Separate functions for clarity and flexibility.

**Proposed API**:

```rust
// Existing function (extended)
pub fn load_elf(elf_data: &[u8]) -> Result<ElfLoadInfo, String>

// New function
pub fn load_object_file(
    obj_file_bytes: &[u8],
    code: &mut Vec<u8>,
    ram: &mut Vec<u8>,
    symbol_map: &mut HashMap<String, u32>,
) -> Result<ObjectLoadInfo, String>
```

**Usage Pattern**:

```rust
let mut base = load_elf(base_bytes)?;
let obj1 = load_object_file(obj1_bytes, &mut base.code, &mut base.ram, &mut base.symbol_map)?;
let obj2 = load_object_file(obj2_bytes, &mut base.code, &mut base.ram, &mut base.symbol_map)?;
```

**Rationale**:

- Clear separation: base executable vs object file loading
- Flexible: can load base once, then multiple object files
- Simple: no builder pattern overhead
- Backward compatible: doesn't change existing `load_elf()` API
- Explicit: caller controls the sequence

### Q6: Buffer Ownership

For loading multiple object files:

- Should the loader own the code/ram buffers and extend them as needed?
- Or should the caller manage buffers and pass mutable references? ✅ **SELECTED**
- How do we handle buffer resizing when loading multiple object files?

**Answer**: Caller manages buffers, passes mutable references. Loader extends buffers as needed.

**Proposed Approach**:

- `load_object_file()` takes `&mut Vec<u8>` for code and ram
- Loader extends Vecs using `Vec::extend()` or `Vec::resize()` as needed
- Caller can pre-allocate space if desired, or let Vecs grow dynamically
- Symbol map also passed as `&mut HashMap<String, u32>` to accumulate symbols

**Rationale**:

- Simple: no ownership transfer, clear data flow
- Flexible: caller can pre-allocate or let grow
- Efficient: Vecs grow automatically, no manual size calculations needed
- Consistent: matches how `load_elf()` currently works (returns Vecs)

## Code Sharing

### Q7: Relocation Handlers

The existing relocation handlers in `elf_loader/relocations/handlers.rs`:

- Should we reuse them directly for object files? ✅ **YES**
- Do they need modification to support resolving symbols from multiple symbol maps? ✅ **NO - merge maps first**
- Should we extract them to a shared module? ✅ **NO - already shared**

**Answer**: Reuse handlers directly, merge symbol maps before calling.

**Proposed Approach**:

1. Build object file's symbol map (with adjusted addresses for where sections are loaded)
2. Merge with base symbol map (base symbols take precedence)
3. Pass merged map to existing relocation handlers

**Implementation**:

```rust
// In object file loader
fn merge_symbol_maps(
    base_map: &HashMap<String, u32>,
    obj_map: &HashMap<String, u32>,
) -> HashMap<String, u32> {
    let mut merged = base_map.clone();
    // Add object file symbols (base symbols take precedence)
    for (name, addr) in obj_map {
        merged.entry(name.clone()).or_insert(*addr);
    }
    merged
}
```

**Rationale**:

- No handler changes needed - they already accept a symbol map
- Simple: merge maps before relocations
- Clear: symbol resolution happens in one place (merge function)
- Flexible: can extend to multiple object files later
- Handlers remain unchanged, reducing risk

### Q8: Section Loading

The existing `elf_loader/sections.rs` handles section loading:

- Can we reuse this code for object files, or is it too specific to executables? ✅ **TOO SPECIFIC - create new function**
- Should we extract common section loading logic to a shared module? ✅ **EXTRACT HELPERS IF USEFUL**
- How do we handle the difference: executables have fixed addresses, object files need addresses calculated? ✅ **NEW FUNCTION WITH CALCULATED ADDRESSES**

**Answer**: Create object-file-specific section loading, extract common helpers if useful.

**Analysis**:

- `load_sections()` is executable-specific:
  - Uses fixed section addresses from ELF
  - Handles LMA vs VMA
  - Uses symbol addresses to infer VMA for sections with address 0
  - Handles `.data` LMA via `__data_source_start`
- Object files need different logic:
  - Sections have no fixed addresses (relocatable)
  - Must calculate placement addresses (after base executable)
  - Must adjust symbol addresses based on placement

**Proposed Approach**:

1. Create new `load_object_sections()` in `elf_loader/object/sections.rs`
2. Extract common helpers if useful (e.g., `copy_section_data()`, `is_loadable_section()`)
3. Keep executable-specific logic in `elf_loader/sections.rs`

**Implementation Structure**:

```rust
// In elf_loader/object/sections.rs
pub fn load_object_sections(
    obj: &object::File,
    code: &mut Vec<u8>,  // Base executable's code buffer
    ram: &mut Vec<u8>,   // Base executable's ram buffer
    code_start: u32,     // Where to place object file's .text
    ram_start: u32,      // Where to place object file's .data
) -> Result<ObjectSectionPlacement, String>
```

**Rationale**:

- Clear separation: executable vs object file logic
- Reusable: common helpers can be shared
- Simple: no complex refactoring of existing code
- Maintainable: each loader handles its own case
- Pragmatic: "make it work, then make it good"

### Q9: Symbol Map Building

The existing `elf_loader/symbols.rs` builds symbol maps:

- Can we reuse this for object files? ✅ **NO - create new function**
- Do we need separate functions for base executable vs object file symbol maps? ✅ **YES - separate function**
- Should symbol map building be extracted to a shared module? ✅ **SIMILAR STRUCTURE, DIFFERENT CALCULATION**

**Answer**: Create object-file-specific symbol map builder with adjusted addresses.

**Analysis**:

- `build_symbol_map()` is executable-specific:
  - Uses `text_base` to calculate offsets for ROM symbols
  - Uses absolute addresses for RAM symbols
  - Symbols already have final addresses
- Object files need different logic:
  - Symbols have section-relative addresses (not absolute)
  - Must adjust addresses based on where sections are placed
  - Need to know where `.text` and `.data` sections are placed

**Proposed Approach**:

1. Create new `build_object_symbol_map()` in `elf_loader/object/symbols.rs`
2. Similar structure to `build_symbol_map()` but:
   - Takes section placement addresses (where `.text` and `.data` are loaded)
   - Adjusts symbol addresses: `final_addr = section_placement + symbol_offset`
   - Handles ROM vs RAM symbols based on section kind

**Implementation**:

```rust
// In elf_loader/object/symbols.rs
pub fn build_object_symbol_map(
    obj: &object::File,
    text_placement: u32,  // Where .text section was placed
    data_placement: u32,  // Where .data section was placed
) -> HashMap<String, u32>
```

**Rationale**:

- Similar logic, different address calculation
- Clear: object file logic separate but similar
- Reusable: can share symbol collection logic if needed
- Simple: no complex refactoring

### Q10: Layout Calculation

The existing `elf_loader/layout.rs` calculates memory layout:

- Can we extend it to also calculate object file layout? ✅ **SEPARATE FUNCTION**
- Or should object file layout be separate? ✅ **YES**
- How do we track where base executable ends to place object files? ✅ **TRACK IN ElfLoadInfo**

**Answer**: Separate layout calculation for object files, track base executable end in `ElfLoadInfo`.

**Proposed Approach**:

- `ElfLoadInfo` already tracks `code_end` and `ram_end` (from Q4)
- Create `calculate_object_layout()` in `elf_loader/object/layout.rs`:
  - Takes object file and base executable's `code_end`/`ram_end`
  - Calculates where to place object file's sections (after base)
  - Returns placement addresses for `.text` and `.data`

**Implementation**:

```rust
// In elf_loader/object/layout.rs
pub struct ObjectLayout {
    pub text_placement: u32,  // Where to place .text (after base code_end)
    pub data_placement: u32,  // Where to place .data (after base ram_end)
}

pub fn calculate_object_layout(
    obj: &object::File,
    base_code_end: u32,
    base_ram_end: u32,
) -> Result<ObjectLayout, String>
```

**Rationale**:

- Simple: separate concerns, clear data flow
- Flexible: can adjust placement strategy later
- Tracked: base end addresses in `ElfLoadInfo`

## Symbol Resolution

### Q11: Symbol Map Structure

For resolving symbols from both base executable and object files:

- Should we maintain separate symbol maps and check both? ✅ **NO**
- Or merge them into a single map (with conflict resolution)? ✅ **YES - merge (from Q7)**
- How do we handle symbol conflicts (same name in base and object)? ✅ **BASE TAKES PRECEDENCE**

**Answer**: Merge symbol maps, base symbols take precedence (already decided in Q7).

**Rationale**:

- Consistent with Q7 decision to merge maps before relocations
- Simple: single lookup, no multiple checks
- Base symbols win: ensures base executable's symbols aren't overridden

### Q12: Symbol Lookup Order

When resolving a symbol during relocation:

- Check object file's symbols first, then base executable? ✅ **NO**
- Or always prefer base executable symbols? ✅ **YES - base first (via merge)**
- Should this be configurable? ✅ **NO - not needed**

**Answer**: Base symbols take precedence (via merge order in Q7).

**Rationale**:

- Consistent with Q11: base symbols win
- Simple: no configuration needed
- Safe: prevents accidental symbol overriding

### Q13: Multiple Object Files

If we support loading multiple object files:

- Can object file 2 reference symbols from object file 1? ✅ **YES - via merged symbol map**
- Or can object files only reference the base executable? ✅ **NO - can reference previous objects**
- How do we build the symbol map for multiple object files? ✅ **ACCUMULATE IN CALLER'S MAP**

**Answer**: Object files can reference symbols from previously loaded object files via accumulated symbol map.

**Proposed Approach**:

```rust
let mut base = load_elf(base_bytes)?;
let obj1 = load_object_file(obj1_bytes, &mut base.code, &mut base.ram, &mut base.symbol_map)?;
// obj1's symbols are now in base.symbol_map
let obj2 = load_object_file(obj2_bytes, &mut base.code, &mut base.ram, &mut base.symbol_map)?;
// obj2 can reference obj1's symbols via base.symbol_map
```

**Rationale**:

- Flexible: supports incremental loading
- Simple: symbol map accumulates naturally
- Enables: object files can depend on each other

## Memory Management

### Q14: Buffer Sizing

When loading object files:

- Should we pre-calculate total size needed (base + all objects) and allocate once? ✅ **NO**
- Or extend buffers dynamically as we load each object file? ✅ **YES - dynamic extension (from Q6)**
- What's the performance trade-off? ✅ **ACCEPTABLE - Vec grows efficiently**

**Answer**: Extend buffers dynamically (already decided in Q6).

**Rationale**:

- Simple: Vec grows automatically, no manual size calculations
- Flexible: caller can pre-allocate if desired
- Efficient: Vec reallocation is amortized O(1)
- Consistent: matches how `load_elf()` works

### Q15: Buffer Extension

If we extend buffers dynamically:

- How do we handle relocations in the base executable that reference absolute addresses? ✅ **BASE RELOCATIONS ALREADY RESOLVED**
- Do we need to reapply base executable relocations if we extend buffers? ✅ **NO**
- Or can we guarantee base executable's relocations are already resolved? ✅ **YES**

**Answer**: Base executable's relocations are already resolved before object file loading.

**Rationale**:

- Base executable is fully linked: all relocations applied
- Extending buffers doesn't change base addresses: base sections stay at same addresses
- Object file relocations reference base symbols: base symbols already have final addresses
- No need to reapply: base is immutable after loading

### Q16: Memory Layout Tracking

How do we track:

- Where base executable sections end? ✅ **ElfLoadInfo.code_end, ram_end**
- Where each object file's sections are placed? ✅ **ObjectLoadInfo.text_start, data_start**
- Available memory regions for new object files? ✅ **CALCULATE FROM END ADDRESSES**

**Answer**: Track in return structs, calculate placement from end addresses.

**Proposed Approach**:

- `ElfLoadInfo` tracks `code_end` and `ram_end` (from Q4)
- `ObjectLoadInfo` tracks `text_start` and `data_start` (from Q4)
- Calculate next placement: `next_code_start = max(base.code_end, last_obj.text_start + last_obj.text_size)`
- Update `ElfLoadInfo` after each object file load: `base.code_end = obj.text_start + obj.text_size`

**Rationale**:

- Simple: tracked in return values
- Clear: explicit addresses in structs
- Flexible: caller can track or we can update `ElfLoadInfo`

## Relocation Application

### Q17: Relocation Context

The existing relocation handlers use `RelocationContext`:

- Does it need modification to support multiple symbol maps? ✅ **NO**
- Should we extend it to check both object and base symbol maps? ✅ **NO - merge maps first (from Q7)**
- Or create a new context type for object file relocations? ✅ **NO - reuse existing**

**Answer**: Reuse existing `RelocationContext`, merge symbol maps before calling (from Q7).

**Rationale**:

- Consistent with Q7: merge maps, then use existing handlers
- No changes needed: handlers already accept merged symbol map
- Simple: no new context type

### Q18: GOT Entries

Object files may have GOT entries:

- Do we need a separate GOT tracker per object file? ✅ **NO - single tracker**
- Or can we share GOT entries across object files? ✅ **YES - single shared tracker**
- How do GOT entries in object files reference base executable symbols? ✅ **VIA MERGED SYMBOL MAP**

**Answer**: Single shared GOT tracker, references resolved via merged symbol map.

**Proposed Approach**:

- Create GOT tracker for object file's relocations (same as base executable)
- GOT entries reference symbols via merged symbol map (base + object)
- GOT entries placed in object file's memory region (after base)

**Rationale**:

- Consistent: same GOT mechanism as base executable
- Simple: single tracker, merged symbol resolution
- Flexible: GOT entries can reference base or object symbols

### Q19: Relocation Dependencies

Object file relocations may depend on:

- Other relocations in the same object file? ✅ **YES - use existing dependency handling**
- Symbols from the base executable (already resolved)? ✅ **YES - via merged symbol map**
- How do we handle the dependency ordering? ✅ **REUSE EXISTING TWO-PHASE APPROACH**

**Answer**: Reuse existing two-phase relocation approach (phase1: analyze, phase2: apply).

**Rationale**:

- Existing relocation system already handles dependencies (PCREL_HI20/LO12 pairs, GOT entries)
- Object file relocations are same types: same dependency rules apply
- Base symbols already resolved: merged symbol map has final addresses
- No changes needed: existing phase1/phase2 handles it

## Testing & Performance

### Q20: Test Performance

For the goal of faster tests (loading multiple object files without reloading base):

- Should we cache the base executable's loaded state? ✅ **YES - caller manages**
- How do we reset/clean up between tests? ✅ **CLEAR BUFFERS, KEEP BASE STATE**
- Should this be a separate "fast path" API? ✅ **NO - same API, caller reuses base**

**Answer**: Caller caches base state, clears buffers between tests, reuses base.

**Proposed Approach**:

```rust
// In test setup
let base = load_elf(base_bytes)?;

// In each test
let mut code = base.code.clone();
let mut ram = base.ram.clone();
let mut symbol_map = base.symbol_map.clone();
let obj = load_object_file(obj_bytes, &mut code, &mut ram, &mut symbol_map)?;
// ... test ...
```

**Rationale**:

- Simple: same API, caller manages caching
- Flexible: caller controls when to reload base
- Fast: base loaded once, reused across tests

### Q21: Test Isolation

If we load multiple object files in sequence:

- Do we need to isolate them (separate memory regions)? ✅ **NO - sequential placement**
- Or can they overlap/share memory? ✅ **NO - sequential, non-overlapping**
- How do we handle test failures without affecting subsequent tests? ✅ **CALLER CLONES BASE STATE**

**Answer**: Sequential placement (non-overlapping), caller clones base state for isolation.

**Rationale**:

- Sequential: object files placed after previous ones
- Non-overlapping: each object file gets its own memory region
- Isolation: caller clones base state, so test failures don't affect other tests
- Simple: no complex memory management needed

## Integration Points

### Q22: Emulator Integration

How does the emulator use the loader?

- Does it call `load_elf()` then `load_object_file()` separately? ✅ **YES**
- Or does it use a unified API? ✅ **NO - separate calls**
- How does it get the final code/ram buffers? ✅ **FROM ElfLoadInfo**

**Answer**: Separate calls, final buffers from `ElfLoadInfo`.

**Proposed Usage**:

```rust
let mut base = load_elf(base_bytes)?;
load_object_file(obj_bytes, &mut base.code, &mut base.ram, &mut base.symbol_map)?;
let emu = Riscv32Emulator::new(base.code, base.ram);
```

**Rationale**:

- Simple: explicit sequence, clear data flow
- Flexible: can load multiple object files
- Consistent: matches existing `load_elf()` usage

### Q23: \_\_USER_MAIN_PTR Update

When and how do we update `__USER_MAIN_PTR`?

- During object file loading? ✅ **YES - if object file has main**
- After all object files are loaded? ✅ **OR AFTER ALL LOADED**
- Should this be part of the loader, or separate? ✅ **PART OF LOADER**

**Answer**: Update during object file loading if `main` symbol found, or after all loaded.

**Proposed Approach**:

- `load_object_file()` checks for `main` symbol
- If found, updates `__USER_MAIN_PTR` in RAM (via symbol map lookup)
- Returns `main_address` in `ObjectLoadInfo` for caller reference

**Rationale**:

- Convenient: loader handles common case
- Explicit: `main_address` in return value
- Flexible: caller can override if needed

### Q24: Entry Point

What is the entry point for the combined system?

- Base executable's entry point? ✅ **YES - base entry point**
- Object file's main function? ✅ **CALLED VIA \_\_USER_MAIN_PTR**
- How do we handle multiple object files with multiple main functions? ✅ **LAST ONE WINS**

**Answer**: Base executable's entry point, calls object file's `main` via `__USER_MAIN_PTR`.

**Rationale**:

- Base executable provides runtime: entry point is base's
- Object file's `main` is called via `__USER_MAIN_PTR` (from Q23)
- Last object file's `main` wins: simple rule, matches linker behavior

## Error Handling

### Q25: Undefined Symbols

What happens if an object file references an undefined symbol?

- Fail fast with an error? ✅ **YES**
- Allow lazy resolution (defer error until symbol is actually used)? ✅ **NO**
- Should this be configurable? ✅ **NO**

**Answer**: Fail fast with error during relocation application.

**Rationale**:

- Safe: catch errors early, before execution
- Simple: no lazy resolution complexity
- Consistent: matches linker behavior (undefined symbols cause link errors)

### Q26: Symbol Conflicts

What happens if:

- Object file defines a symbol with the same name as base executable? ✅ **BASE WINS (from Q11)**
- Multiple object files define the same symbol? ✅ **FIRST ONE WINS**
- Should we allow overriding, or error? ✅ **SILENTLY RESOLVE - base/first wins**

**Answer**: Base symbols win, first object file symbol wins for conflicts.

**Rationale**:

- Consistent with Q11: base takes precedence
- Simple: no error handling for conflicts
- Matches linker behavior: first definition wins
- Silent resolution: no need to error on conflicts

### Q27: Memory Overflow

What happens if loading an object file would exceed available memory?

- Pre-check and fail before loading? ✅ **YES - check before loading**
- Extend buffers automatically (if possible)? ✅ **YES - Vec extends automatically**
- Return an error with details about what's needed? ✅ **YES - detailed error**

**Answer**: Pre-check placement, Vec extends automatically, return detailed error if needed.

**Proposed Approach**:

- Calculate object file size before loading
- Check if placement would exceed reasonable limits (e.g., 64MB code, 512MB RAM)
- Vec extends automatically, but check for overflow before placement
- Return error with required size if pre-check fails

**Rationale**:

- Safe: catch overflow before loading
- Efficient: Vec grows, but we check bounds
- Helpful: error message shows what's needed

## Migration & Cleanup

### Q28: executable_linker Removal

When we remove `executable_linker`:

- What code can be reused? ✅ **RELOCATION LOGIC (already in elf_loader)**
- What needs to be rewritten? ✅ **LINKING LOGIC (replaced by object loader)**
- Are there any features in `executable_linker` we need to preserve? ✅ **NONE - object loader replaces it**

**Answer**: Relocation logic already reused, linking logic replaced by object loader.

**Analysis**:

- `executable_linker` was trying to link object files into executables
- Object loader replaces this: loads object files directly into emulator
- Relocation logic already in `elf_loader/relocations/` - no need to reuse from linker
- No features to preserve: object loader is cleaner approach

**Migration**:

- Delete `executable_linker/` module
- Update tests to use `load_object_file()` instead of linking

### Q29: Backward Compatibility

Do we need to maintain backward compatibility with:

- Existing code that uses `elf_loader`? ✅ **YES - extend, don't break**
- Existing tests? ✅ **YES - keep working**
- Or can we break the API? ✅ **NO - extend ElfLoadInfo**

**Answer**: Maintain backward compatibility, extend `ElfLoadInfo` with new fields.

**Proposed Approach**:

- `load_elf()` API unchanged: still returns `ElfLoadInfo`
- `ElfLoadInfo` extended with `symbol_map`, `code_end`, `ram_end` (new fields)
- Existing code continues to work: old fields still present
- New `load_object_file()` is additive: doesn't affect existing code

**Rationale**:

- Safe: no breaking changes
- Incremental: can migrate gradually
- Compatible: existing tests still work

### Q30: Documentation

What documentation do we need?

- API documentation for the new loader? ✅ **YES - rustdoc**
- Migration guide from `executable_linker`? ✅ **YES - if needed**
- Examples of loading base + object files? ✅ **YES - in rustdoc**

**Answer**: Rustdoc for API, examples in doc comments, migration guide if needed.

**Proposed Documentation**:

- Rustdoc for `load_object_file()` with examples
- Example in `elf_loader/mod.rs` showing base + object loading
- Migration guide if `executable_linker` had external users (probably not)

**Rationale**:

- Standard: rustdoc is standard Rust documentation
- Examples: help users understand usage
- Migration: only if needed (probably internal only)
