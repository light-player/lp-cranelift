# Design Issues Analysis - Second Pass

## New Issues Found

### 15. ID Types in Configs vs Runtime ✅ FIXED
**Problem**: Design uses type-safe IDs in runtime, but configs currently use `u32`.

**Solution**: Use type-safe IDs in configs with `#[serde(transparent)]` attribute. This allows serde to serialize/deserialize the inner `u32` value directly (which becomes a string in JSON). No conversion needed during init - configs and runtime both use type-safe IDs.

### 16. FixtureNodeConfig.texture_id Missing ✅ FIXED
**Problem**: Design said to add `texture_id` to `FixtureNodeConfig`, but current `FixtureNode` doesn't have it.

**Solution**: Add `texture_id: TextureId` to `FixtureNodeConfig::CircleList`. This is already documented in the design.

### 17. OutputNodeRuntime Buffer Access ✅ FIXED
**Problem**: Design showed `buffer: Vec<u8>` in `OutputNodeRuntime`, but wasn't clear how fixtures write to it.

**Solution**: Add `buffer_mut() -> &mut [u8]` method to `OutputNodeRuntime`. Fixtures access via `get_output_mut().buffer_mut()` to get mutable slice for writing.

### 18. Shader Execution Details ✅ FIXED
**Problem**: Design didn't specify how `ShaderNodeRuntime.update()` executes the shader.

**Solution**: Shader main signature: `vec4 main(vec2 fragCoord, vec2 outputSize, float time)`. During `init()`, validate GLSL has matching signature before compilation. During `update()`: iterate over all texture pixels, call shader with `[x, y]` as fragCoord, `[width, height]` as outputSize, and `time.total_ms` as time. Write result via `texture.set_pixel()`. Eventually will use uniforms/globals, but for now everything is passed as parameters.

### 19. Texture Initialization ✅ FIXED
**Problem**: Design didn't specify how `Texture` is created in `TextureNodeRuntime.init()`.

**Solution**: `Texture::new(width, height, format)` constructor allocates buffer using `Vec::with_capacity(width * height * bytes_per_pixel(format))`, initializes to zeros, and validates format. `bytes_per_pixel()` helper derives from format string. `TextureNodeRuntime.init()` creates texture via this constructor.

### 20. OutputHandle vs LedOutput Trait ✅ FIXED
**Problem**: Design introduced `OutputHandle` trait, but codebase already has `LedOutput` trait.

**Solution**: Use `LedOutput` trait (HAL-style LED hardware access). `OutputProvider.create_output()` sets up hardware (GPIO pin, etc.) based on config and returns `Box<dyn LedOutput>`. `LedOutput` is for built-in LED hardware; future outputs (UDP packets, etc.) will have different traits. `LedOutput` trait may need enhancement for setup/initialization - see issue #31.

### 21. RuntimeNodes Serialization with Type-Safe IDs ✅ NOT AN ISSUE
**Problem**: `RuntimeNodes` stores `HashMap<u32, NodeStatus>` for serialization, but runtime uses type-safe IDs.

**Resolution**: IDs implement `Into<u32>`, so conversion is straightforward when building `RuntimeNodes` from runtime instances.

### 22. ShaderNodeConfig.texture_id Type ✅ FIXED (via #15)
**Problem**: `ShaderNodeConfig` has `texture_id: u32` but runtime uses `TextureId`.

**Resolution**: Fixed by issue #15 - configs now use type-safe IDs.

### 23. FixtureNodeConfig.output_id and texture_id Types ✅ FIXED (via #15)
**Problem**: Same as #22 - configs use `u32`, runtimes use type-safe IDs.

**Resolution**: Fixed by issue #15 - configs now use type-safe IDs.

### 24. OutputNodeRuntime Buffer Size ✅ FIXED
**Problem**: `OutputNodeRuntime` has `buffer: Vec<u8>`, but how is size determined?

**Solution**: `bytes_per_pixel` is derived from `OutputNodeConfig` chip type (e.g., "ws2812" = 3 bytes RGB). Stored in runtime for convenience but could be derived when needed. Buffer allocated in `init()`: `Vec::with_capacity(pixel_count * bytes_per_pixel)`, initialized to zeros. `pixel_count` comes from `config.count`.

### 25. Multiple Fixtures Writing to Same Output ✅ FIXED
**Problem**: Multiple fixtures can write to the same output (same `output_id`). How do we handle this?

**Solution**: Multiple fixtures writing to the same output is a valid use case (fixtures can be strung together). Each fixture writes to specific channels/pixels based on its mapping. For now, no overlap validation - if mappings overlap, later fixtures overwrite earlier ones. Future: could add validation to ensure mappings don't overlap. This is acceptable for now.

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

### 28. NodeStatus in Runtime vs RuntimeNodes ✅ FIXED (via #7)
**Problem**: Design says runtime instances are source of truth, but `RuntimeNodes` is still needed for serialization.

**Resolution**: Fixed by issue #7 - removed separate `RuntimeNodes` field, derive when needed.

### 29. InitContext Access Pattern ✅ NOT AN ISSUE
**Problem**: `InitContext` provides access to configs, but how does a node validate its dependencies?

**Resolution**: Nodes use `InitContext.get_texture_config()` etc. to validate dependencies. This is fine.

### 30. Destroy() Method Usage
**Problem**: Design includes `destroy()` in lifecycle, but when is it called?
- On project unload?
- On node removal?
- Never (just drop)?

**Clarification Needed**: When and how is `destroy()` used?

### 31. LedOutput Trait Enhancement
**Problem**: `LedOutput` trait is currently simple (just `write_pixels` and `get_pixel_count`). User notes it should cover setup (GPIO pin, etc.) and be HAL-style for built-in LED hardware.

**Clarification Needed**: 
- Should `LedOutput` have setup/init methods?
- Or should setup be handled by `OutputProvider`?
- How should GPIO pin configuration work?
