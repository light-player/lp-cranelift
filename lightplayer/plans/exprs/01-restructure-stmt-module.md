# Phase 1: Restructure Statement Module

## Goal

Break out `stmt.rs` into a clean module structure with per-statement files, following the pattern established in the `expr` module and aligning with Clang's organization.

## Current State

- Single large file: `lightplayer/crates/lp-glsl/src/codegen/stmt.rs` (672 lines)
- All statement types mixed together
- Hard to navigate and maintain
- No clear separation of concerns

## Target Structure

```
stmt/
  mod.rs              - Main dispatch (emit_statement), public API
  compound.rs         - Compound statements { ... }
  if.rs               - If statements (emit_if_stmt)
  loops.rs            - Loop dispatch + shared helpers (translate_condition)
  loop_for.rs         - For loops (emit_loop_for_stmt)
  loop_while.rs       - While loops (emit_loop_while_stmt)
  loop_do_while.rs    - Do-while loops (emit_loop_do_while_stmt)
  return.rs           - Return statements (emit_return_stmt)
  break.rs            - Break statements (emit_break_stmt)
  continue.rs         - Continue statements (emit_continue_stmt)
  declaration.rs      - Variable declarations (emit_declaration)
                        + parse_type_specifier, emit_initializer helpers
  expr.rs             - Expression statements (emit_expr_stmt)
```

## Design Decisions

1. **Naming**: Use `emit_*_stmt` pattern (e.g., `emit_if_stmt`, `emit_loop_while_stmt`) to align with Clang. Loop statements prefixed with `loop_` (e.g., `emit_loop_for_stmt`, `emit_loop_while_stmt`, `emit_loop_do_while_stmt`)
2. **Public API**: Rename `translate_statement` to `emit_stmt` (matching Clang's `EmitStmt`) as main entry point in `mod.rs`
3. **Individual Functions**: Make all statement handlers public for clarity and testing
4. **Helpers**:
   - `translate_condition` → shared helper in `loops.rs` (used by if/while/for)
   - `parse_type_specifier` and `emit_initializer` → stay in `declaration.rs`
5. **Dispatch Only**: `mod.rs` contains only dispatch logic, no implementation
6. **Loop Organization**: Separate files for each loop type, shared helpers in `loops.rs`

## Implementation Steps

### Step 1: Create Module Structure

1. Create `lightplayer/crates/lp-glsl/src/codegen/stmt/` directory
2. Create `mod.rs` with:

   - Module declarations
   - Public `emit_statement` function (renamed from `translate_statement`)
   - Dispatch logic only

3. Create individual statement files:
   - `compound.rs` - Compound statements
   - `if.rs` - If statements
   - `loops.rs` - Loop dispatch + `translate_condition` helper
   - `loop_for.rs` - For loops
   - `loop_while.rs` - While loops
   - `loop_do_while.rs` - Do-while loops
   - `return.rs` - Return statements
   - `break.rs` - Break statements
   - `continue.rs` - Continue statements
   - `declaration.rs` - Variable declarations + helpers
   - `expr.rs` - Expression statements

### Step 2: Extract Compound Statements

1. Move `translate_compound` to `compound.rs`
2. Rename to `pub fn emit_compound_stmt`
3. Update `mod.rs` to call it

### Step 3: Extract If Statements

1. Move `translate_selection` to `if.rs`
2. Rename to `pub fn emit_if_stmt`
3. Update `mod.rs` to call it
4. Ensure it follows Clang's pattern

### Step 4: Extract Loops

1. Create `loops.rs`:

   - Move `translate_condition` helper here
   - Create `pub fn emit_iteration_stmt` dispatch function
   - Export loop functions

2. Create `loop_while.rs`:

   - Move `translate_while_loop` to `emit_loop_while_stmt`
   - Make it public

3. Create `loop_do_while.rs`:

   - Move `translate_do_while_loop` to `emit_loop_do_while_stmt`
   - Make it public

4. Create `loop_for.rs`:

   - Move `translate_for_loop` to `emit_loop_for_stmt`
   - Make it public

5. Update `mod.rs` to call `emit_iteration_stmt` from `loops.rs`

### Step 5: Extract Jump Statements

1. Create `return.rs`:

   - Move `translate_return` to `pub fn emit_return_stmt`
   - Make it public

2. Create `break.rs`:

   - Move `translate_break` to `pub fn emit_break_stmt`
   - Make it public

3. Create `continue.rs`:

   - Move `translate_continue` to `pub fn emit_continue_stmt`
   - Make it public

4. Create `jump.rs` (or keep dispatch in mod.rs):
   - Move `translate_jump` dispatch here
   - Or keep in `mod.rs` since it's simple dispatch

### Step 6: Extract Declarations

1. Create `declaration.rs`:
   - Move `translate_declaration` to `pub fn emit_declaration`
   - Move `parse_type_specifier` helper here
   - Move `translate_initializer` to `pub fn emit_initializer`
   - Make all public

### Step 7: Extract Expression Statements

1. Create `expr.rs`:
   - Extract expression statement handling from `translate_simple_statement`
   - Create `pub fn emit_expr_stmt`
   - Handle empty statements

### Step 8: Update Main Dispatch

1. Update `mod.rs`:
   - Import all statement modules
   - Rename `translate_statement` to `emit_statement` (matching Clang's `EmitStmt`)
   - Dispatch to appropriate `emit_*_stmt` functions
   - Keep internal dispatch helpers as needed

### Step 9: Update Imports

1. Update `lightplayer/crates/lp-glsl/src/codegen/mod.rs`:

   - Change `pub mod stmt;` to `pub mod stmt;` (no change needed, module structure handles it)

2. Update all files that import from `stmt`:
   - Update calls from `translate_statement` to `emit_statement`
   - Files to update:
     - `compiler/glsl_compiler.rs` (2 call sites)
     - `intrinsics/compiler.rs` (1 call site)
     - Internal recursive calls in `stmt/` module (7 call sites)

### Step 10: Test and Verify

1. Run all tests:

   ```bash
   scripts/glsl-filetests.sh vec4/
   ```

2. Verify no regressions
3. Verify code compiles
4. Check that structure is clean and maintainable

## Files to Create

- `lightplayer/crates/lp-glsl/src/codegen/stmt/mod.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/compound.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/if.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loops.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_for.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_while.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_do_while.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/return.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/break.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/continue.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/declaration.rs`
- `lightplayer/crates/lp-glsl/src/codegen/stmt/expr.rs`

## Files to Delete

- `lightplayer/crates/lp-glsl/src/codegen/stmt.rs` (after migration complete)

## Function Mapping

| Current Function          | New Location          | New Name                        |
| ------------------------- | --------------------- | ------------------------------- |
| `translate_statement`     | `mod.rs`              | `emit_statement`                |
| `translate_compound`      | `compound.rs`         | `emit_compound_stmt`            |
| `translate_selection`     | `if.rs`               | `emit_if_stmt`                  |
| `translate_iteration`     | `loops.rs`            | `emit_iteration_stmt`           |
| `translate_while_loop`    | `loop_while.rs`       | `emit_loop_while_stmt`          |
| `translate_do_while_loop` | `loop_do_while.rs`    | `emit_loop_do_while_stmt`       |
| `translate_for_loop`      | `loop_for.rs`         | `emit_loop_for_stmt`            |
| `translate_condition`     | `loops.rs`            | `translate_condition` (helper)  |
| `translate_jump`          | `mod.rs` or `jump.rs` | `emit_jump_stmt`                |
| `translate_return`        | `return.rs`           | `emit_return_stmt`              |
| `translate_break`         | `break.rs`            | `emit_break_stmt`               |
| `translate_continue`      | `continue.rs`         | `emit_continue_stmt`            |
| `translate_declaration`   | `declaration.rs`      | `emit_declaration`              |
| `parse_type_specifier`    | `declaration.rs`      | `parse_type_specifier` (helper) |
| `translate_initializer`   | `declaration.rs`      | `emit_initializer`              |
| Expression handling       | `expr.rs`             | `emit_expr_stmt`                |

## Test Cases

- All existing tests should continue to pass
- Verify no regressions
- Test each statement type individually

## Acceptance Criteria

- [ ] Module structure created
- [ ] All statement types extracted to separate files
- [ ] Functions renamed to `emit_*_stmt` pattern
- [ ] All functions are public
- [ ] `mod.rs` contains only dispatch logic
- [ ] Helpers organized appropriately
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] Structure is clean and maintainable

## Verification

Run all tests:

```bash
scripts/glsl-filetests.sh vec4/
```

Expected result: All tests pass, code is better organized.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: restructure stmt module into per-statement files"
```

## Notes

- This is a pure refactoring - no behavior changes
- Keep existing function signatures where possible
- Use `emit_*_stmt` naming to align with Clang patterns
- Make functions public for clarity and potential testing
- Keep helpers close to where they're used
