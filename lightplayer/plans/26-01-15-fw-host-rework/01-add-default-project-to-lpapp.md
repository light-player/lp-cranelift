# Phase 1: Add create_default_project to LpApp and integrate into initialization

## Goal

Move `create_default_project()` from `app.rs` to `LpApp` and integrate it into the initialization flow. If no `project.json` exists, create and use the default project.

## Tasks

1. Read `app.rs` to get the `create_default_project()` implementation
2. Move `create_default_project()` to `lp-core/src/app/lp_app.rs` as an associated function
3. Modify `LpApp::load_project()` to check if file exists, and if not, create default project
4. Update `LpApp::load_project()` to save the default project to `project.json` if it was created
5. Remove the `init()` logic from `app.rs` (will be handled by `load_project()`)

## Success Criteria

- `create_default_project()` is in `LpApp` as an associated function
- `LpApp::load_project()` creates default project if `project.json` doesn't exist
- Default project is saved to filesystem when created
- Code compiles without warnings

