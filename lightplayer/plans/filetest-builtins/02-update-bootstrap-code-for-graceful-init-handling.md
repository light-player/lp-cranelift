# Phase 2: Update bootstrap code for graceful _init handling

## Goal

Modify the builtins-app bootstrap code to handle missing user _init gracefully with clear messages.

## Changes

1. **Update `main()` function in `lp-builtins-app/src/main.rs`**:
   - Change panic message to check for sentinel values (0xDEADBEEF or 0)
   - If `__USER_MAIN_PTR` is at sentinel value, print "no user _init specified. halting." and halt
   - If `__USER_MAIN_PTR` is set, print "jumping to user _init at <address>" before jumping
   - Update variable names and comments to use "_init" terminology

2. **Update print messages**:
   - Change "user_main_ptr" references to "user_init_ptr" in print statements
   - Update all related comments and documentation

## Files to Modify

- `lightplayer/apps/lp-builtins-app/src/main.rs`

## Success Criteria

- Bootstrap code handles missing user _init gracefully (no panic)
- Clear messages printed: "jumping to user _init at <address>" or "no user _init specified. halting."
- Code compiles and runs correctly
- Tests pass with updated behavior

