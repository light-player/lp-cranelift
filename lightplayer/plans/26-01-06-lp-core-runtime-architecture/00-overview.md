# Overview: LP-Core Runtime Architecture

## Goal

Refactor `lp-core` to separate configuration from runtime state, add lifecycle management for nodes, and create a runtime system that can execute projects.

## Key Changes

1. **Config vs Runtime Separation**: Rename all `*Node` types to `*NodeConfig` and create corresponding `*NodeRuntime` types
2. **Type-Safe IDs**: Create `TextureId`, `OutputId`, `ShaderId`, `FixtureId` newtype wrappers
3. **Texture Abstraction**: Create low-level `Texture` utility in `util/texture.rs` for pixel buffer management
4. **Node Lifecycle**: Implement `NodeLifecycle` trait with `init()`, `update()`, `destroy()` methods
5. **Contexts**: Create `InitContext` (for initialization) and `RenderContext` (for updates with timing)
6. **Project Runtime**: Enhance `ProjectRuntime` to store runtime instances and manage lifecycle
7. **Project Builder**: Create fluent API builder for constructing test projects
8. **Error Handling**: Remove `Warn` status, use only `Ok`/`Error`

## Architecture

### File Structure (Feature-First)

- `nodes/` - Node definitions organized by type
  - `id.rs` - Type-safe ID wrappers
  - `output/`, `texture/`, `shader/`, `fixture/` - Each with `config.rs` and `runtime.rs`
- `util/texture.rs` - Low-level texture abstraction
- `runtime/` - Runtime infrastructure
  - `lifecycle.rs` - `NodeLifecycle` trait
  - `contexts.rs` - `InitContext` and `RenderContext`
- `project/` - Project-level structures
  - `config.rs` - `ProjectConfig` with `*NodeConfig`
  - `runtime.rs` - `ProjectRuntime` with runtime instances
- `builder.rs` - `ProjectBuilder` fluent API

### Key Design Decisions

- **Lifecycle**: Trait-based with associated `Config` type for type safety
- **Contexts**: Separate contexts for init (config access) and render (runtime access + timing)
- **Partial Failure**: All nodes initialize in default state, failures tracked via `NodeStatus`
- **Update Order**: Hard-coded order (shaders → fixtures → outputs) in `ProjectRuntime::update()`
- **Firmware Abstraction**: `OutputProvider` trait allows firmware to provide output handles
- **Frame Timing**: Simple `delta_ms` and `total_ms` (milliseconds) in `RenderContext`

## Success Criteria

- All node configs renamed to `*NodeConfig`
- Type-safe IDs implemented
- `Texture` abstraction created in `util/texture.rs`
- All node runtimes implement `NodeLifecycle`
- `ProjectRuntime` can initialize and update projects
- `ProjectBuilder` can construct valid projects
- Tests can be written for basic project execution

