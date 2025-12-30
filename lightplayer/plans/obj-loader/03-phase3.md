# Phase 3: Implement Object Layout Calculation

## Goal
Implement `calculate_object_layout()` to compute placement addresses for object file sections after the base executable.

## Changes Required

### 1. Implement `calculate_object_layout()` in `layout.rs`
- Input: `obj: &object::File`, `base_code_end: u32`, `base_ram_end: u32`
- Output: `Result<ObjectLayout, String>`

### 2. Define `ObjectLayout` struct
```rust
pub struct ObjectLayout {
    pub text_placement: u32,  // Where to place .text (after base code_end)
    pub data_placement: u32,  // Where to place .data (after base ram_end)
}
```

### 3. Calculate placement
- Find `.text` section size in object file
- Find `.data` section size in object file
- `text_placement = base_code_end` (aligned to 4 bytes)
- `data_placement = base_ram_end` (aligned to 4 bytes, relative to RAM start)

### 4. Handle edge cases
- Object file has no `.text` section
- Object file has no `.data` section
- Alignment requirements (4-byte alignment)

## Implementation Details

- Use `object::File` to iterate sections
- Find sections by name (`.text`, `.data`)
- Calculate sizes: `section.size()`
- Align addresses: `(addr + 3) & !3`

## Testing
- Test with object file containing `.text` and `.data`
- Test with object file missing `.text` or `.data`
- Test alignment (ensure addresses are 4-byte aligned)
- Verify placement is after base executable

## Success Criteria
- `calculate_object_layout()` returns correct placement addresses
- Addresses are properly aligned
- Handles missing sections gracefully

