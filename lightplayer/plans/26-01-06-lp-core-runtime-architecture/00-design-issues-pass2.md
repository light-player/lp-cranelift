# Design Issues Analysis - Second Pass

## New Issues Found

### 15. ID Types in Configs vs Runtime ✅ FIXED
**Problem**: Design uses type-safe IDs in runtime, but configs currently use `u32`.

**Solution**: Use type-safe IDs in configs with `#[serde(transparent)]` attribute. This allows serde to serialize/deserialize the inner `u32` value directly (which becomes a string in JSON). No conversion needed during init - configs and runtime both use type-safe IDs.

### 16. FixtureNodeConfig.texture_id Missing ✅ FIXED
**Problem**: Design said to add `texture_id` to `FixtureNodeConfig`, but current `FixtureNode` doesn't have it.

**Solution**: Add `texture_id: TextureId` to `FixtureNodeConfig::CircleList`. This is already documented in the design.

### 17. OutputNodeRuntime Buffer Access
**Problem**: Design shows `buffer: Vec<u8>` in `OutputNodeRuntime`, but how do fixtures write to it? They get `&mut OutputNodeRuntime` via `get_output_mut()`. Do they:
- Write directly: `output.buffer[channel] = value`?
- Use a method: `output.write_channel(channel, value)`?

**Clarification Needed**: How should fixtures write to output buffers?

### 18. Shader Execution Details
**Problem**: Design doesn't specify how `ShaderNodeRuntime.update()` executes the shader:
- How does it call `executable.call_*()`?
- What parameters does it pass? (fragCoord? time?)
- Does it iterate over all pixels in the texture?
- How does it write results to texture?

**Clarification Needed**: Define the shader execution flow.

### 19. Texture Initialization
**Problem**: Design doesn't specify how `Texture` is created in `TextureNodeRuntime.init()`:
- How is buffer allocated? (`Vec::with_capacity()` based on width*height*bytes_per_pixel?)
- What's the initial buffer state? (zeros? uninitialized?)
- How do we get `bytes_per_pixel` from format string?

**Clarification Needed**: Define texture initialization process.

### 20. OutputHandle vs LedOutput Trait
**Problem**: Design introduces `OutputHandle` trait, but codebase already has `LedOutput` trait in `traits/led_output.rs`. Are these:
- The same thing (rename `LedOutput` to `OutputHandle`)?
- Different (one for firmware abstraction, one for runtime)?
- Should `OutputHandle` wrap `LedOutput`?

**Clarification Needed**: Relationship between `OutputHandle` and existing `LedOutput` trait.

### 21. RuntimeNodes Serialization with Type-Safe IDs
**Problem**: `RuntimeNodes` stores `HashMap<u32, NodeStatus>` for serialization, but runtime uses type-safe IDs (`TextureId`, etc.). `get_runtime_nodes()` needs to convert IDs to `u32`:
- How do we extract `u32` from `TextureId`? (via `Into<u32>`)
- Do we need separate maps per node type, or one unified map?

**Current Design**: `RuntimeNodes` has separate maps per type, which is good. Just need to convert IDs when building.

### 22. ShaderNodeConfig.texture_id Type
**Problem**: `ShaderNodeConfig` has `texture_id: u32` (for JSON), but `ShaderNodeRuntime` has `texture_id: TextureId`. Need to convert during init.

**Clarification**: This is fine, just need to document the conversion happens in `init()`.

### 23. FixtureNodeConfig.output_id and texture_id Types
**Problem**: Same as #22 - configs use `u32`, runtimes use type-safe IDs. Need conversion.

### 24. OutputNodeRuntime Buffer Size
**Problem**: `OutputNodeRuntime` has `buffer: Vec<u8>`, but how is size determined?
- From `OutputNodeConfig.count` and `bytes_per_pixel`?
- What's `bytes_per_pixel`? (3 for RGB, 4 for RGBA?)
- Should it be stored in runtime or derived from config?

**Clarification Needed**: How is buffer size determined and stored?

### 25. Multiple Fixtures Writing to Same Output
**Problem**: Multiple fixtures can write to the same output (same `output_id`). How do we handle this?
- Do fixtures overwrite each other?
- Do we accumulate/blend values?
- Is there a write order dependency?

**Current Design**: Says "if fixture doesn't update, stays same as last frame" - implies overwriting is fine.

**Clarification Needed**: Define behavior when multiple fixtures write to same output.

### 26. Shader Texture Access
**Problem**: `ShaderNodeRuntime` has `texture_id: TextureId`, but how does the shader access it? The shader GLSL code doesn't have texture access built-in yet. For now:
- Shaders render pixel-by-pixel and write via `Texture::set_pixel()`?
- Or do we need to pass texture as a parameter somehow?

**Clarification Needed**: How do shaders access texture data (if at all) in the initial implementation?

### 27. Time Struct Location
**Problem**: `Time` struct is shown in `runtime/contexts.rs`, but `ProjectRuntime` also uses it. Should it be:
- In `runtime/contexts.rs` (shared)?
- In `project/runtime.rs` (where it's used)?
- In a separate `util/time.rs`?

**Clarification Needed**: Where should `Time` struct live?

### 28. NodeStatus in Runtime vs RuntimeNodes
**Problem**: Design says runtime instances are source of truth, but `RuntimeNodes` is still needed for serialization. The current `ProjectRuntime` structure has `nodes: RuntimeNodes` which conflicts with the new design.

**Clarification**: Need to remove `nodes: RuntimeNodes` from `ProjectRuntime` struct definition in the design.

### 29. InitContext Access Pattern
**Problem**: `InitContext` provides access to configs, but how does a node validate its dependencies? For example:
- `ShaderNodeRuntime.init()` needs to check that `texture_id` exists in config
- `FixtureNodeRuntime.init()` needs to check that `output_id` and `texture_id` exist

**Clarification**: This seems fine - nodes can use `InitContext.get_texture_config()` etc. to validate.

### 30. Destroy() Method Usage
**Problem**: Design includes `destroy()` in lifecycle, but when is it called?
- On project unload?
- On node removal?
- Never (just drop)?

**Clarification Needed**: When and how is `destroy()` used?

