# Phase 4: Implement Object Section Loading

## Goal
Implement `load_object_sections()` to copy object file sections into the base executable's code/ram buffers at calculated addresses.

## Changes Required

### 1. Implement `load_object_sections()` in `sections.rs`
- Input: `obj: &object::File`, `code: &mut Vec<u8>`, `ram: &mut Vec<u8>`, `layout: &ObjectLayout`
- Output: `Result<ObjectSectionPlacement, String>`

### 2. Define `ObjectSectionPlacement` struct
```rust
pub struct ObjectSectionPlacement {
    pub text_start: u32,
    pub text_size: usize,
    pub data_start: u32,
    pub data_size: usize,
}
```

### 3. Load sections
- Extend `code` Vec if needed to fit `.text` section
- Extend `ram` Vec if needed to fit `.data` section
- Copy `.text` section data to `code` at `layout.text_placement`
- Copy `.data` section data to `ram` at `layout.data_placement` (relative to RAM start)
- Zero-initialize `.bss` sections if present

### 4. Handle section types
- `.text` → code buffer
- `.data` → ram buffer
- `.rodata` → code buffer (if present)
- `.bss` → ram buffer (zero-initialized)

## Implementation Details

- Use `section.data()` to get section contents
- Extend Vecs: `code.resize(new_size, 0)` or `code.extend_from_slice()`
- Copy data: `code[offset..offset+size].copy_from_slice(data)`
- Track actual placement addresses for return value

## Testing
- Test loading `.text` section into code buffer
- Test loading `.data` section into ram buffer
- Test Vec extension (verify buffers grow as needed)
- Test with multiple sections
- Verify data is copied correctly (compare bytes)

## Success Criteria
- Sections are copied to correct addresses
- Buffers are extended as needed
- Return value contains correct placement info

