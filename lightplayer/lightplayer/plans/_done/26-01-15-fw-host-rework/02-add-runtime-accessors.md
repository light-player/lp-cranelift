# Phase 2: Add accessor methods to ProjectRuntime for node runtimes

## Goal

Add accessor methods to `ProjectRuntime` to get node runtimes by ID, since the HashMap fields are private.

## Tasks

1. Add `get_texture(&self, id: TextureId) -> Option<&TextureNodeRuntime>` to `ProjectRuntime`
2. Add `get_shader(&self, id: ShaderId) -> Option<&ShaderNodeRuntime>` to `ProjectRuntime`
3. Add `get_fixture(&self, id: FixtureId) -> Option<&FixtureNodeRuntime>` to `ProjectRuntime`
4. Add `get_output(&self, id: OutputId) -> Option<&OutputNodeRuntime>` to `ProjectRuntime`

## Success Criteria

- All four accessor methods are implemented
- Methods return references to node runtimes
- Code compiles without warnings

