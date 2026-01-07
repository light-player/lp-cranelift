# Phase 4: Update debug UI to show actual texture data, shader code/errors, fixtures

## Goal

Update the debug UI to show actual runtime data instead of placeholders:
- Textures: actual texture contents using `Texture::data()`
- Shaders: code from config and errors from runtime status
- Fixtures: mappings overlayed on actual texture data

## Tasks

1. Update `render_texture()` to accept `Option<&Texture>` and use actual texture data
2. Create helper function to convert `Texture::data()` to egui `ColorImage` efficiently
3. Add `render_shader_panel()` to show shader code and errors
4. Update `render_fixture()` to use actual texture data from runtime
5. Update `render_textures_panel()` to get texture data from runtime
6. Update `render_fixtures_panel()` to get texture data from runtime

## Success Criteria

- Textures show actual rendered content
- Shaders show code and compilation/execution errors
- Fixtures show mappings overlayed on actual textures
- Code compiles without warnings

