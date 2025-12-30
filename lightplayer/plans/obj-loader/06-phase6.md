# Phase 6: Implement Symbol Map Merging

## Goal
Implement `merge_symbol_maps()` helper to merge base and object symbol maps (base takes precedence).

## Changes Required

### 1. Implement `merge_symbol_maps()` in `symbols.rs`
- Input: `base_map: &HashMap<String, u32>`, `obj_map: &HashMap<String, u32>`
- Output: `HashMap<String, u32>`

### 2. Merge logic
- Clone base map (base symbols take precedence)
- Iterate object map
- For each symbol: `merged.entry(name).or_insert(addr)` (only add if not in base)

### 3. Handle conflicts
- Base symbols always win (already handled by `or_insert`)
- Object symbols only added if not in base
- Log conflicts if needed (for debugging)

## Implementation Details

- Simple merge: `let mut merged = base_map.clone();`
- Add object symbols: `for (name, addr) in obj_map { merged.entry(name.clone()).or_insert(*addr); }`
- No special conflict handling needed (base wins by design)

## Testing
- Test merging with no conflicts
- Test merging with conflicts (verify base wins)
- Test merging with undefined symbols
- Verify all symbols present in merged map

## Success Criteria
- Merged map contains all base symbols
- Merged map contains object symbols not in base
- Base symbols take precedence in conflicts

