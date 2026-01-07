# Design: FW-Host Rework

## Overview

Rework fw-host to finish LpApp work and separate concerns:
1. Simulate ESP32 firmware: instantiate LpApp and show simulated output LEDs
2. Debug info: show project state (nodes, textures, shaders, fixtures) in readonly debug UI

## File Structure

```
lp-core/src/app/
└── lp_app.rs                    # MODIFY: Add create_default_project() and integrate into init

lp-core/src/project/
└── runtime.rs                   # MODIFY: Add accessor methods for node runtimes

fw-host/src/
├── app.rs                       # DELETE: Move functionality elsewhere
├── main.rs                      # MODIFY: Handle messages, show outputs/debug UI
├── output_provider.rs           # MODIFY: Track outputs in HashMap
├── debug_ui.rs                  # MODIFY: Show actual texture data, shader code/errors, fixtures
├── led_output.rs                # EXISTING: No changes
├── filesystem.rs                # EXISTING: No changes
└── transport.rs                 # EXISTING: No changes (may add helper for message handling)
```

## Code Structure

### lp-core/src/app/lp_app.rs

**New Methods:**
- `create_default_project() -> ProjectConfig` - Creates default project config
- Modify `load_project()` - If no project.json exists, create default project

**Changes:**
- Integrate default project creation into initialization flow
- If `project.json` doesn't exist, create default project and use it (can be saved later)

### lp-core/src/project/runtime.rs

**New Methods:**
- `get_texture(&self, id: TextureId) -> Option<&TextureNodeRuntime>`
- `get_shader(&self, id: ShaderId) -> Option<&ShaderNodeRuntime>`
- `get_fixture(&self, id: FixtureId) -> Option<&FixtureNodeRuntime>`
- `get_output(&self, id: OutputId) -> Option<&OutputNodeRuntime>`

**Changes:**
- Add accessor methods to get node runtimes by ID (fields are private HashMaps)

### fw-host/src/output_provider.rs

**New Fields:**
- `outputs: HashMap<OutputId, Arc<Mutex<HostLedOutput>>>` - Track created outputs

**New Methods:**
- `get_output(&self, id: OutputId) -> Option<Arc<Mutex<HostLedOutput>>>`
- `get_all_outputs(&self) -> HashMap<OutputId, Arc<Mutex<HostLedOutput>>>`

**Changes:**
- Modify `create_output()` to store created outputs in HashMap
- Return `Arc<Mutex<HostLedOutput>>` instead of `Box<dyn LedOutput>` (or wrap it)

### fw-host/src/main.rs

**New Structure:**
- Remove `FwHostApp` wrapper
- Direct `LpApp` instance
- Direct `HostOutputProvider` instance (for accessing outputs)
- Message handling helper function (or inline in update loop)

**Changes:**
- Handle transport messages directly, convert to `MsgIn`, call `LpApp::tick()`
- Show output sections: one per output, displaying LEDs assuming RGB order
- Show debug panel: textures (actual data), shaders (code + errors), fixtures (mappings on textures)

### fw-host/src/debug_ui.rs

**New Functions:**
- `render_texture_with_data(ui, texture_id, texture_node, texture_data)` - Show actual texture contents
- `render_shader_panel(ui, shader_id, shader_node, shader_runtime)` - Show code and errors
- `render_fixture_with_texture(ui, fixture_id, fixture_node, texture_runtime)` - Show mapping on texture

**Changes:**
- Use `Texture::data()` to get raw pixel data
- Convert to egui `ColorImage` efficiently (not pixel-by-pixel)
- Show shader code from config, errors from runtime status
- Show fixture mappings overlayed on actual texture data

## New Functions and Types

### lp-core/src/app/lp_app.rs
- `create_default_project() -> ProjectConfig` - Creates default project with output, texture, shader, fixture

### lp-core/src/project/runtime.rs
- `get_texture(&self, id: TextureId) -> Option<&TextureNodeRuntime>`
- `get_shader(&self, id: ShaderId) -> Option<&ShaderNodeRuntime>`
- `get_fixture(&self, id: FixtureId) -> Option<&FixtureNodeRuntime>`
- `get_output(&self, id: OutputId) -> Option<&OutputNodeRuntime>`

### fw-host/src/output_provider.rs
- `get_output(&self, id: OutputId) -> Option<Arc<Mutex<HostLedOutput>>>`
- `get_all_outputs(&self) -> &HashMap<OutputId, Arc<Mutex<HostLedOutput>>>`

### fw-host/src/main.rs (or helper module)
- `handle_transport_messages(transport) -> Vec<MsgIn>` - Convert transport messages to MsgIn

### fw-host/src/debug_ui.rs
- `texture_data_to_color_image(texture_data, width, height, format) -> ColorImage` - Efficient conversion
- `render_shader_panel(ui, shader_id, shader_config, shader_runtime)` - Show shader code and errors
- `render_fixture_with_texture(ui, fixture_id, fixture_config, texture_runtime)` - Show mapping overlay

## Key Design Decisions

1. **Output Tracking**: `HostOutputProvider` tracks outputs in HashMap for UI access
2. **Default Project**: Integrated into `LpApp::load_project()` - creates if missing
3. **Message Handling**: In main.rs, convert transport -> MsgIn -> LpApp::tick()
4. **Output Display**: One section per output, RGB order (hardcoded for now)
5. **Debug UI**: Read-only, shows actual runtime data (textures, shader errors, fixture mappings)
6. **Texture Display**: Use `Texture::data()` + egui `ColorImage` for efficient rendering

