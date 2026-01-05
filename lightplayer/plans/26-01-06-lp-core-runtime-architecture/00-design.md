# Architecture Design: LP-Core Runtime

## File Structure

```
lp-core/src/
├── nodes/
│   ├── id.rs                    # NEW: TextureId, OutputId, ShaderId, FixtureId
│   ├── output/
│   │   ├── mod.rs
│   │   ├── config.rs           # OutputNodeConfig (renamed from output.rs)
│   │   └── runtime.rs          # OutputNodeRuntime
│   ├── texture/
│   │   ├── mod.rs
│   │   ├── config.rs           # TextureNodeConfig (renamed)
│   │   └── runtime.rs          # TextureNodeRuntime
│   ├── shader/
│   │   ├── mod.rs
│   │   ├── config.rs           # ShaderNodeConfig (renamed)
│   │   └── runtime.rs          # ShaderNodeRuntime
│   └── fixture/
│       ├── mod.rs
│       ├── config.rs           # FixtureNodeConfig (renamed, add texture_id)
│       └── runtime.rs          # FixtureNodeRuntime
├── util/
│   └── texture.rs              # NEW: Texture core abstraction (low-level)
├── runtime/
│   ├── lifecycle.rs             # NodeLifecycle trait
│   └── contexts.rs              # InitContext, RenderContext
├── project/
│   ├── config.rs                # ProjectConfig, Nodes
│   └── runtime.rs               # ProjectRuntime
└── builder.rs                   # NEW: ProjectBuilder
```

## Types Summary

```
nodes/id.rs
  TextureId(u32), OutputId(u32), ShaderId(u32), FixtureId(u32)
  From/Into conversions

util/texture.rs
  Texture { width, height, format, data: Vec<u8> }
  Methods: get_pixel, set_pixel, sample, compute_all

runtime/lifecycle.rs
  trait NodeLifecycle {
    type Config;
    fn init(&mut self, config: &Self::Config, ctx: &InitContext) -> Result<(), Error>
    fn update(&mut self, ctx: &RenderContext) -> Result<(), Error>
    fn destroy(&mut self) -> Result<(), Error>
  }

runtime/contexts.rs
  InitContext<'a> { project_config: &'a ProjectConfig }
    Methods: get_texture_config, get_shader_config, etc.
  
  RenderContext<'a> {
    delta_ms: u32,
    total_ms: u32,
    textures: &'a HashMap<TextureId, TextureNodeRuntime>,
    shaders: &'a HashMap<ShaderId, ShaderNodeRuntime>,
    fixtures: &'a HashMap<FixtureId, FixtureNodeRuntime>,
    outputs: &'a HashMap<OutputId, OutputNodeRuntime>,
  }
    Methods: get_texture, get_shader, get_output (mutable), etc.

nodes/*/config.rs
  OutputNodeConfig, TextureNodeConfig, ShaderNodeConfig, FixtureNodeConfig
  (FixtureNodeConfig adds texture_id field)

nodes/*/runtime.rs
  OutputNodeRuntime { handle: Option<OutputHandle>, pixel_count, bytes_per_pixel, status }
  TextureNodeRuntime { texture: Texture, status }
  ShaderNodeRuntime { executable: Option<JitExecutable>, texture_id, status }
  FixtureNodeRuntime { output_id, texture_id, kernels: Vec<SamplingKernel>, channel_order, status }
  
  All implement NodeLifecycle

project/runtime.rs
  ProjectRuntime {
    uid: String,
    total_ms: u32,
    config: Option<ProjectConfig>,
    textures: HashMap<TextureId, TextureNodeRuntime>,
    shaders: HashMap<ShaderId, ShaderNodeRuntime>,
    fixtures: HashMap<FixtureId, FixtureNodeRuntime>,
    outputs: HashMap<OutputId, OutputNodeRuntime>,
    nodes: RuntimeNodes,  # Status tracking for serialization
  }
    Methods: init(), update(delta_ms), set_status, get_status
  
  trait OutputProvider {
    fn create_output(&self, config: &OutputNodeConfig) -> Result<Box<dyn OutputHandle>, Error>
  }
  
  trait OutputHandle {
    fn write_pixels(&mut self, pixels: &[u8]) -> Result<(), Error>
  }
  
  enum NodeStatus { Ok, Error { status_message: String } }  # Warn removed

builder.rs
  ProjectBuilder {
    uid, name, next_id, nodes
  }
    Methods: new(), with_uid(), with_name(), add_texture() -> (Self, TextureId),
            add_shader(texture_id) -> (Self, ShaderId), add_output() -> (Self, OutputId),
            add_fixture(output_id, texture_id) -> (Self, FixtureId), build() -> Result
```

## Design Details

### Type-Safe IDs

All node references use newtype wrappers (`TextureId`, `OutputId`, etc.) instead of raw `u32` for compile-time type safety. IDs implement `From<u32>` and `Into<u32>` for conversion.

### Texture Abstraction

The `Texture` struct in `util/texture.rs` is a low-level utility for managing pixel buffers. It provides:
- Fixed-size buffer (not resizable)
- Format metadata (RGB8, RGBA8, R8)
- Sampling methods (get_pixel, sample with normalized coordinates)
- Helper methods like `compute_all` for batch operations

This will eventually move to `lp-builtins` as part of the core GLSL system.

### Node Lifecycle

All node runtimes implement `NodeLifecycle` trait with:
- `init()`: Initialize from config, validate dependencies, allocate resources
- `update()`: Update state (render shaders, sample textures, write outputs)
- `destroy()`: Cleanup resources

The trait uses an associated `Config` type so each runtime has typed access to its specific config.

### Contexts

**InitContext**: Provides read-only access to project config during initialization. Used for dependency validation.

**RenderContext**: Provides read/write access to runtime instances during updates. Includes frame timing (`delta_ms`, `total_ms`). Methods return `Option` to handle missing or failed nodes gracefully.

### Node Runtimes

- **TextureNodeRuntime**: Wraps a `Texture` instance
- **ShaderNodeRuntime**: Stores compiled `JitExecutable` (None if compilation failed)
- **FixtureNodeRuntime**: Precomputes sampling kernels in `init()`, samples textures and writes to outputs in `update()`
- **OutputNodeRuntime**: Holds firmware-specific `OutputHandle` for writing LED data

### Project Runtime

`ProjectRuntime` manages the lifecycle of all nodes:
- `init()`: Initializes nodes in order (textures → shaders → fixtures → outputs), allows partial failures
- `update(delta_ms)`: Updates nodes in hard-coded order (shaders → fixtures → outputs), updates `total_ms`
- Tracks status for serialization via `RuntimeNodes`

### Project Builder

Fluent API for constructing test projects:
- Auto-generates IDs
- Methods return IDs for linking (e.g., `add_shader(texture_id)`)
- Validates at `build()` time
- Returns `Result<ProjectConfig, Error>`

### Error Handling

- Removed `Warn` status, only `Ok`/`Error`
- Lifecycle methods return `Result<(), Error>`
- Errors update `NodeStatus` in runtime
- Partial failures allowed - project can init even if some nodes fail

