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
    type RenderContext;
    fn init(&mut self, config: &Self::Config, ctx: &InitContext) -> Result<(), Error>
    fn update(&mut self, ctx: &Self::RenderContext) -> Result<(), Error>
    fn destroy(&mut self) -> Result<(), Error>
  }

  // Each node type implements with its specific context:
  // TextureNodeRuntime: RenderContext = TextureRenderContext
  // ShaderNodeRuntime: RenderContext = ShaderRenderContext
  // FixtureNodeRuntime: RenderContext = FixtureRenderContext
  // OutputNodeRuntime: RenderContext = OutputRenderContext

runtime/contexts.rs
  InitContext<'a> { project_config: &'a ProjectConfig }
    Methods: get_texture_config, get_shader_config, etc.

  BaseRenderContext {
    delta_ms: u32,
    total_ms: u32,
  }

  ShaderRenderContext<'a> {
    base: BaseRenderContext,
    textures: &'a mut HashMap<TextureId, TextureNodeRuntime>,
  }
    Methods: get_texture_mut(texture_id: TextureId) -> Option<&mut Texture>

  FixtureRenderContext<'a> {
    base: BaseRenderContext,
    textures: &'a HashMap<TextureId, TextureNodeRuntime>,
    outputs: &'a mut HashMap<OutputId, OutputNodeRuntime>,
  }
    Methods:
      get_texture(texture_id: TextureId) -> Option<&Texture>
      get_output_mut(output_id: OutputId) -> Option<&mut OutputNodeRuntime>

  OutputRenderContext {
    base: BaseRenderContext,
    // No access to other nodes needed
  }

  TextureRenderContext {
    base: BaseRenderContext,
    // No access to other nodes needed
  }

nodes/*/config.rs
  OutputNodeConfig, TextureNodeConfig, ShaderNodeConfig, FixtureNodeConfig
  (FixtureNodeConfig adds texture_id field)

nodes/*/runtime.rs
  OutputNodeRuntime { handle: Option<Box<dyn OutputHandle>>, pixel_count, bytes_per_pixel, status }
  TextureNodeRuntime { texture: Texture, status }
  ShaderNodeRuntime { executable: Option<Box<dyn GlslExecutable>>, texture_id, status }
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
    Methods:
      init(output_provider: &dyn OutputProvider) -> Result<(), Error>
      update(delta_ms: u32, output_provider: &mut dyn OutputProvider) -> Result<(), Error>
      # Creates type-specific contexts and calls node.update() with appropriate context
      set_status, get_status

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
- `update()`: Update state using type-specific render context
- `destroy()`: Cleanup resources

The trait uses two associated types:

- `Config`: The config type for this node
- `RenderContext`: The specific render context type this node needs

This ensures type safety - each node can only access what it needs, and the compiler enforces this.

### Contexts

**InitContext**: Provides read-only access to project config during initialization. Used for dependency validation.

**Type-Specific Render Contexts**: Each node type has its own render context with only the access it needs:

- **TextureRenderContext**: Only timing (no other node access needed)
- **ShaderRenderContext**: Timing + mutable access to textures (for writing rendered pixels)
- **FixtureRenderContext**: Timing + read-only access to textures + mutable access to outputs
- **OutputRenderContext**: Only timing (no other node access needed)

This approach:

- Provides type safety (each node can only access what it needs)
- Avoids borrow checker issues (contexts are created per-node with appropriate borrows)
- Makes dependencies explicit in the type system

### Node Runtimes

- **TextureNodeRuntime**: Wraps a `Texture` instance
- **ShaderNodeRuntime**: Stores compiled `Box<dyn GlslExecutable>` (None if compilation failed). `GlslJitModule` implements `GlslExecutable` trait.
- **FixtureNodeRuntime**: Precomputes sampling kernels in `init()`, samples textures and writes to outputs in `update()` via `FixtureRenderContext` (which provides mutable access to outputs)
- **OutputNodeRuntime**: Holds firmware-specific `OutputHandle` for writing LED data

### Project Runtime

`ProjectRuntime` manages the lifecycle of all nodes:

- `init()`: Initializes nodes in order (textures → shaders → fixtures → outputs), allows partial failures
- `update(delta_ms)`: Updates nodes in hard-coded order (shaders → fixtures → outputs), updates `total_ms`. Creates appropriate type-specific contexts for each node:
  - Shaders get `ShaderRenderContext` with mutable texture access (for writing rendered pixels)
  - Fixtures get `FixtureRenderContext` with read-only texture access and mutable output access
  - Outputs get `OutputRenderContext` with no other node access
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
