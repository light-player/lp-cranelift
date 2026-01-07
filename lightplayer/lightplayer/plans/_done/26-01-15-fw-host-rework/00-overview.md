# Overview: FW-Host Rework

## Goal

Rework the fw-host app to finish LpApp work and separate concerns:
1. Simulate ESP32 firmware: instantiate LpApp and show simulated output LEDs in egui
2. Debug info: show project state (nodes, textures, shaders, fixtures) in readonly debug UI

## Changes

- Move `create_default_project()` from `app.rs` to `LpApp` and integrate into initialization
- Add accessor methods to `ProjectRuntime` for accessing node runtimes
- Modify `HostOutputProvider` to track outputs in HashMap for UI access
- Update debug UI to show actual runtime data (textures, shader code/errors, fixtures)
- Move message handling from `app.rs` to `main.rs`
- Delete `app.rs` entirely
- Update main.rs to show output sections (one per output) and debug panel

## Success Criteria

- `app.rs` is deleted
- `create_default_project()` is in `LpApp` and used during initialization
- Outputs are tracked in `HostOutputProvider` and displayed in UI
- Debug UI shows actual texture data, shader code/errors, and fixture mappings
- Message handling happens in main.rs
- All code compiles without warnings
- Tests pass

