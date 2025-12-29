# Phase 5: Add unit tests for fixed32 functions

## Goal

Add comprehensive unit tests for `fixed32/div`, `fixed32/mul`, and `fixed32/sqrt` functions.

## Steps

### 5.1 Add tests for `fixed32/div.rs`

- Test basic division cases
- Test division by zero (should saturate)
- Test edge cases (max values, min values)
- Test sign handling (positive/negative combinations)
- Test precision (compare against expected results)

### 5.2 Add tests for `fixed32/mul.rs`

- Test basic multiplication cases
- Test overflow cases (should saturate)
- Test underflow cases (should saturate)
- Test zero handling
- Test sign handling

### 5.3 Add tests for `fixed32/sqrt.rs`

- Test perfect squares
- Test non-perfect squares
- Test edge cases (x <= 0 should return 0)
- Test precision (compare against expected results)
- Test large values

### 5.4 Test structure

- Use `#[cfg(test)]` modules in each file
- Use helper functions for float-to-fixed conversion if needed
- Test both correctness and edge cases

## Files to Modify

- `lightplayer/crates/lp-builtins/src/fixed32/div.rs` (add `#[cfg(test)] mod tests`)
- `lightplayer/crates/lp-builtins/src/fixed32/mul.rs` (add `#[cfg(test)] mod tests`)
- `lightplayer/crates/lp-builtins/src/fixed32/sqrt.rs` (add `#[cfg(test)] mod tests`)

## Success Criteria

- All tests pass for native target
- Tests cover basic cases, edge cases, and error conditions
- Tests verify correctness (within expected precision for sqrt)

## Notes

- Focus on custom implementations (div, mul, sqrt) - these need thorough testing
- Use approximate equality for sqrt tests (precision may vary)
- Test saturation behavior for div and mul

