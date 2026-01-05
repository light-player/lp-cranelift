# Architecture Design: LP-Core Runtime

## File Structure

```
lightplayer/crates/lp-core/src/
├── lib.rs
├── error.rs
├── nodes/
│   ├── mod.rs
│   ├── id.rs                    # NEW: Type-safe ID wrappers
│   ├── output.rs                # OutputNodeConfig (renamed from OutputNode)
│   ├── texture.rs               # TextureNodeConfig (renamed), Texture abstraction
│   ├── shader.rs                # ShaderNodeConfig (renamed)
│   └── fixture.rs               # FixtureNodeConfig (renamed, add texture_id)
├── project/
│   ├── mod.rs
│   ├── config.rs                # ProjectConfig, Nodes (with *NodeConfig)
│   └── runtime.rs               # ProjectRuntime (enhanced with runtime instances)
├── runtime/                      # NEW: Runtime types and lifecycle
│   ├── mod.rs
│   ├── lifecycle.rs             # NodeLifecycle trait
│   ├── contexts.rs              # InitContext, RenderContext
│   ├── output.rs                # OutputNodeRuntime
│   ├── texture.rs               # TextureNodeRuntime
│   ├── shader.rs                # ShaderNodeRuntime
│   └── fixture.rs               # FixtureNodeRuntime
├── builder.rs                    # NEW: ProjectBuilder
├── protocol.rs                   # Existing
├── traits.rs                     # Existing
└── util.rs                       # Existing
```

## Types and Functions Summary

### nodes/id.rs (NEW)

```
TextureId(u32)
OutputId(u32)
ShaderId(u32)
FixtureId(u32)

impl From<u32> for TextureId
impl From<TextureId> for u32
// ... similar for other IDs
```

### nodes/texture.rs

```
// Config (renamed from TextureNode)
enum TextureNodeConfig {
    Memory { size: [u32; 2], format: String }
}

// NEW: Runtime abstraction
struct Texture {
    width: u32,
    height: u32,
    format: String,
    data: Vec<u8>,  // or raw pointer for contiguous memory
}

impl Texture {
    fn new(width: u32, height: u32, format: String) -> Self
    fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]>
    fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4])
    fn sample(&self, u: f32, v: f32) -> Option<[u8; 4]>
    fn compute_all<F>(&mut self, f: F) where F: Fn(u32, u32) -> [u8; 4]
    // ... other helper methods
}
```

### nodes/fixture.rs

```
enum FixtureNodeConfig {
    CircleList {
        output_id: OutputId,      // Changed from u32
        texture_id: TextureId,     // NEW: Added texture_id
        channel_order: String,
        mapping: Vec<Mapping>,
    }
}
```

### runtime/lifecycle.rs (NEW)

```
trait NodeLifecycle {
    type Config;

    fn init(
        &mut self,
        config: &Self::Config,
        ctx: &InitContext,
    ) -> Result<(), Error>;

    fn update(
        &mut self,
        ctx: &RenderContext,
    ) -> Result<(), Error>;

    fn destroy(&mut self) -> Result<(), Error>;
}
```

### runtime/contexts.rs (NEW)

```
struct InitContext<'a> {
    project_config: &'a ProjectConfig,
    // Provides access to configs for validation
    // No access to runtimes (they don't exist yet)
}

impl InitContext {
    fn get_texture_config(&self, id: TextureId) -> Option<&TextureNodeConfig>
    fn get_shader_config(&self, id: ShaderId) -> Option<&ShaderNodeConfig>
    // ... similar for other node types
}

struct RenderContext<'a> {
    delta_ms: u32,
    total_ms: u32,
    // Provides access to runtime instances
    textures: &'a HashMap<TextureId, TextureNodeRuntime>,
    shaders: &'a HashMap<ShaderId, ShaderNodeRuntime>,
    fixtures: &'a HashMap<FixtureId, FixtureNodeRuntime>,
    outputs: &'a HashMap<OutputId, OutputNodeRuntime>,
}

impl RenderContext {
    fn get_texture(&self, id: TextureId) -> Option<&Texture>
    fn get_shader(&self, id: ShaderId) -> Option<&ShaderNodeRuntime>
    fn get_output(&mut self, id: OutputId) -> Option<&mut OutputNodeRuntime>
    // ... similar for other node types
}
```

### runtime/texture.rs (NEW)

```
struct TextureNodeRuntime {
    texture: Texture,
    status: NodeStatus,
}

impl NodeLifecycle for TextureNodeRuntime {
    type Config = TextureNodeConfig;

    fn init(&mut self, config: &Self::Config, ctx: &InitContext) -> Result<(), Error>
    fn update(&mut self, ctx: &RenderContext) -> Result<(), Error>
    fn destroy(&mut self) -> Result<(), Error>
}
```

### runtime/shader.rs (NEW)

```
struct ShaderNodeRuntime {
    executable: Option<JitExecutable>,  // Compiled shader code
    texture_id: TextureId,
    status: NodeStatus,
}

impl NodeLifecycle for ShaderNodeRuntime {
    type Config = ShaderNodeConfig;

    fn init(&mut self, config: &Self::Config, ctx: &InitContext) -> Result<(), Error>
    fn update(&mut self, ctx: &RenderContext) -> Result<(), Error>
    fn destroy(&mut self) -> Result<(), Error>
}
```

### runtime/fixture.rs (NEW)

```
struct SamplingKernel {
    center: [f32; 2],
    radius: f32,
    samples: Vec<(f32, f32, f32)>,  // (u, v, weight)
}

struct FixtureNodeRuntime {
    output_id: OutputId,
    texture_id: TextureId,
    kernels: Vec<SamplingKernel>,  // Precomputed in init()
    channel_order: String,
    status: NodeStatus,
}

impl NodeLifecycle for FixtureNodeRuntime {
    type Config = FixtureNodeConfig;

    fn init(&mut self, config: &Self::Config, ctx: &InitContext) -> Result<(), Error>
    fn update(&mut self, ctx: &RenderContext) -> Result<(), Error>
    fn destroy(&mut self) -> Result<(), Error>
}
```

### runtime/output.rs (NEW)

```
struct OutputNodeRuntime {
    handle: Option<OutputHandle>,  // Firmware-specific handle (trait object?)
    pixel_count: usize,
    bytes_per_pixel: usize,
    status: NodeStatus,
}

// Trait for firmware-specific output (implemented by firmware)
trait OutputHandle {
    fn write_pixels(&mut self, pixels: &[u8]) -> Result<(), Error>;
}

impl NodeLifecycle for OutputNodeRuntime {
    type Config = OutputNodeConfig;

    fn init(&mut self, config: &Self::Config, ctx: &InitContext) -> Result<(), Error>
    fn update(&mut self, ctx: &RenderContext) -> Result<(), Error>
    fn destroy(&mut self) -> Result<(), Error>
}
```

### project/runtime.rs

```
struct ProjectRuntime {
    uid: String,
    total_ms: u32,  // NEW: Frame timing

    // Config reference (for init)
    config: Option<ProjectConfig>,

    // Runtime instances
    textures: HashMap<TextureId, TextureNodeRuntime>,
    shaders: HashMap<ShaderId, ShaderNodeRuntime>,
    fixtures: HashMap<FixtureId, FixtureNodeRuntime>,
    outputs: HashMap<OutputId, OutputNodeRuntime>,

    // Status tracking (existing)
    nodes: RuntimeNodes,  // Status only, for serialization
}

impl ProjectRuntime {
    fn new(uid: String) -> Self
    fn init(&mut self, config: &ProjectConfig, output_provider: &dyn OutputProvider) -> Result<(), Error>
    fn update(&mut self, delta_ms: u32, output_provider: &mut dyn OutputProvider) -> Result<(), Error>

    // Status management
    fn set_status(&mut self, node_type: NodeType, node_id: u32, status: NodeStatus)
    fn get_status(&self, node_type: NodeType, node_id: u32) -> Option<&NodeStatus>
}

// NEW: Trait for firmware to provide output handles
trait OutputProvider {
    fn create_output(&self, config: &OutputNodeConfig) -> Result<Box<dyn OutputHandle>, Error>;
}
```

### project/config.rs

```
struct ProjectConfig {
    uid: String,
    name: String,
    nodes: Nodes,
}

struct Nodes {
    outputs: HashMap<u32, OutputNodeConfig>,    // Changed from OutputNode
    textures: HashMap<u32, TextureNodeConfig>, // Changed from TextureNode
    shaders: HashMap<u32, ShaderNodeConfig>,     // Changed from ShaderNode
    fixtures: HashMap<u32, FixtureNodeConfig>,   // Changed from FixtureNode
}
```

### builder.rs (NEW)

```
struct ProjectBuilder {
    uid: Option<String>,
    name: Option<String>,
    next_id: u32,
    nodes: Nodes,
}

impl ProjectBuilder {
    fn new() -> Self
    fn with_uid(mut self, uid: String) -> Self
    fn with_name(mut self, name: String) -> Self

    fn add_texture(mut self, config: TextureNodeConfig) -> (Self, TextureId)
    fn add_shader(mut self, texture_id: TextureId, config: ShaderNodeConfig) -> (Self, ShaderId)
    fn add_output(mut self, config: OutputNodeConfig) -> (Self, OutputId)
    fn add_fixture(mut self, output_id: OutputId, texture_id: TextureId, config: FixtureNodeConfig) -> (Self, FixtureId)

    fn build(self) -> Result<ProjectConfig, Error>
}
```

### project/runtime.rs - NodeStatus (UPDATED)

```
enum NodeStatus {
    Ok,
    Error { status_message: String },
    // Warn removed
}
```

## Key Design Decisions

1. **Type Safety**: All IDs are newtype wrappers for compile-time safety
2. **Separation**: Clear separation between config (serializable) and runtime (stateful)
3. **Contexts**: InitContext for initialization, RenderContext for updates
4. **Lifecycle**: Trait-based approach with associated Config type
5. **Partial Failure**: All nodes initialize in default state, failures tracked via status
6. **Firmware Abstraction**: OutputProvider trait allows firmware to provide output handles
7. **Update Order**: Hard-coded order (shaders → fixtures → outputs) in ProjectRuntime::update()
