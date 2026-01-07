# Questions: FW-Host Rework

## Overview

Rework the fw-host app to:

1. Finish LpApp work by moving functionality from `app.rs` into either LpApp (create_default_project) or main.rs (message handling)
2. Separate and focus on two main functions:
   - Simulate ESP32 firmware: instantiate LpApp and show simulated output LEDs in egui
   - Debug info: show project state (nodes, textures, shaders, fixtures) in readonly debug UI

## Questions

### 1. Output Tracking

Currently `HostOutputProvider::create_output()` creates new `HostLedOutput` instances but doesn't track them. We need to display outputs in the UI when they're created.

**Question:** How should we track outputs created by `OutputProvider::create_output()`?

**Answer:** Option A - Modify `HostOutputProvider` to store a mapping of `OutputId -> Arc<Mutex<HostLedOutput>>` and provide accessor methods. This keeps output management in one place and makes it easy to access for UI rendering.

### 2. create_default_project Location

The `create_default_project()` function in `app.rs` creates a default project configuration.

**Question:** Where should `create_default_project()` live?

**Answer:** Move to `lp-core/src/app/lp_app.rs` and integrate into the standard app initialization. If there is no project.json on the filesystem, use the default one (which can later be saved).

### 3. Message Handling

Currently `app.rs` has `handle_command()`, `tick()`, and `process_messages()` methods that handle transport messages and convert them to `MsgIn`.

**Question:** Where should message handling (transport -> MsgIn conversion) live?

**Answer:** Handle messages in main.rs directly, perhaps in a helper function in another file if needed.

### 4. Output Display Organization

We need to display outputs assuming RGB-order LEDs starting at channel 0. Each call to `OutputProvider::create_output` should create a new section in the UI.

**Question:** How should we organize the output display UI?

**Answer:** Option A - One section per output, showing all LEDs for that output, clearly labeled with OutputId.

### 5. Debug UI Data Access

The debug UI needs to show:

- Textures: actual texture contents (from `TextureNodeRuntime::texture()`)
- Shaders: code (from `ProjectConfig`) and errors (from `ShaderNodeRuntime::status()`)
- Fixtures: mapping overlayed on texture (from `ProjectConfig` and `TextureNodeRuntime`)

**Question:** Do we need any additional accessor methods on `LpApp`, `ProjectRuntime`, or node runtimes?

**Answer:** We may need accessor methods on `ProjectRuntime` to get node runtimes by ID (e.g., `get_texture()`, `get_shader()`, etc.) if the HashMap fields are not accessible from outside the module.

### 6. Channel Order Interpretation

Outputs need to display LEDs assuming RGB-order starting at channel 0.

**Question:** How do we determine the channel order for each output?

**Answer:** For now, assume RGB order (hardcoded). Keep it simple for now, we'll improve it later.

### 7. Texture Data Access for Debug UI

The debug UI needs to show actual texture contents (not placeholders).

**Question:** How do we get texture data from runtime for display?

**Answer:** Use `Texture::data()` to get raw pixel data (`&[u8]`), then use egui's native image features (e.g., `ColorImage`) to convert and display efficiently. This avoids going pixel by pixel. Access via `runtime.textures.get(&texture_id).map(|t| t.texture().data())`.
