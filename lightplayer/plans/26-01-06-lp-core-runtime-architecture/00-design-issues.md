# Design Issues Analysis

## Critical Issues

### 1. Shader Writing to Textures
**Problem**: `ShaderRenderContext.get_texture()` returns `Option<&Texture>` (immutable), but shaders need to WRITE to textures.

**Current Design**:
```
ShaderRenderContext {
  textures: &'a HashMap<TextureId, TextureNodeRuntime>,
}
Methods: get_texture(texture_id: TextureId) -> Option<&Texture>
```

**Issue**: Shaders render TO textures, so they need mutable access.

**Solution Options**:
- Option A: `get_texture_mut()` returns `Option<&mut Texture>` - but this conflicts with immutable borrow of HashMap
- Option B: Shaders return rendered data, ProjectRuntime writes to textures
- Option C: `ShaderRenderContext` has mutable access to textures HashMap

### 2. FixtureRenderContext Borrow Checker
**Problem**: `FixtureRenderContext` has `&'a mut HashMap<OutputId, OutputNodeRuntime>`, but when we call `fixture.update(&mut fixture, ctx)` we're mutably borrowing both the fixture AND the outputs HashMap.

**Current Design**:
```
FixtureRenderContext<'a> {
  textures: &'a HashMap<TextureId, TextureNodeRuntime>,
  outputs: &'a mut HashMap<OutputId, OutputNodeRuntime>,
}
```

**Issue**: How does `ProjectRuntime::update()` create this context? It needs to mutably borrow `self.outputs` while also having `&mut self.fixtures`.

**Solution**: Need to clarify how contexts are created - probably need to borrow outputs separately, or use a different pattern.

### 3. JitExecutable Type Mismatch ✅ FIXED
**Problem**: Design mentioned `JitExecutable` but codebase has `GlslJitModule` and `GlslExecutable` trait.

**Solution**: Changed to `Option<Box<dyn GlslExecutable>>`. `GlslJitModule` implements `GlslExecutable` trait, allowing flexibility for different backends.

### 4. Texture Extraction from TextureNodeRuntime ✅ FIXED
**Problem**: `ShaderRenderContext` stores `HashMap<TextureId, TextureNodeRuntime>`, but `get_texture_mut()` returns `&mut Texture`. How do we extract Texture from TextureNodeRuntime?

**Solution**: Added `texture()` and `texture_mut()` methods on `TextureNodeRuntime`. `ShaderRenderContext::get_texture_mut()` uses `textures.get_mut().map(|rt| rt.texture_mut())`.

## Design Gaps

### 5. SamplingKernel Definition Missing ✅ FIXED
**Problem**: Design mentioned `Vec<SamplingKernel>` but didn't define the struct.

**Solution**: Changed to single `SamplingKernel` per fixture (not Vec). Kernel has `radius` (normalized, same for all pixels) and `samples: Vec<SamplePoint>` with relative offsets and weights. One kernel is precomputed in `init()` and reused for all mapping points at their respective centers.

### 6. OutputProvider in update() ✅ FIXED
**Problem**: Design showed `update(delta_ms, output_provider: &mut dyn OutputProvider)` but outputs are already initialized.

**Solution**: Removed `OutputProvider` from `update()`. Outputs are created in `init()`, so `update()` only needs `delta_ms`.

### 7. NodeStatus Synchronization ✅ FIXED
**Problem**: Current `ProjectRuntime` had separate `RuntimeNodes` for serialization, but new design has `status` in each runtime struct.

**Solution**: Removed separate `RuntimeNodes` field. Runtime instances are the source of truth. `get_runtime_nodes()` derives `RuntimeNodes` from runtime instances when needed for serialization. `set_status()` and `get_status()` operate directly on runtime instances.

### 8. Texture Format Handling ✅ FIXED
**Problem**: Design didn't specify how shaders know texture format when writing, or how fixtures know format when reading.

**Solution**: Shaders always return vec4 (RGBA). `Texture::set_pixel()` writes based on format: RGB8 writes first 3 bytes, R8 writes first byte, RGBA8 writes all 4 bytes. `Texture` provides `format()` and `bytes_per_pixel()` methods for querying format. Fixtures can query format when sampling.

### 9. Error Cascading
**Problem**: If a shader fails to render (executable is None), what happens to fixtures that depend on that texture? Do they skip, fail, or use previous frame's data?

**Current Design**: Says "partial failures allowed" but doesn't specify behavior.

**Clarification Needed**: Define error handling strategy for dependent nodes.

### 10. OutputNodeRuntime.update() Purpose
**Problem**: Design shows `OutputNodeRuntime` has `update()` method, but what does it do? Outputs are written to by fixtures.

**Clarification Needed**: 
- Does `update()` write to hardware?
- Or does it just prepare the buffer?
- When does hardware actually get updated?

### 11. BaseRenderContext vs Individual Contexts
**Problem**: Design shows `BaseRenderContext` with timing, but individual contexts have `base: BaseRenderContext`. This means contexts don't have direct access to timing fields.

**Clarification Needed**: Should contexts have `delta_ms` and `total_ms` directly, or access via `base.delta_ms`?

### 12. ProjectRuntime.config Field
**Problem**: Design shows `config: Option<ProjectConfig>` in `ProjectRuntime`, but it's only needed during `init()`. Why keep it?

**Clarification Needed**: 
- Is it needed for re-initialization?
- Or can we drop it after init?

## Minor Issues

### 13. Builder API Return Types
**Problem**: Builder methods return `(Self, Id)` tuples. This is awkward - can't chain easily.

**Example**: `let (builder, tex_id) = builder.add_texture(...); let (builder, shader_id) = builder.add_shader(tex_id, ...);`

**Better**: Could use `&mut self` and return just the ID, or use a different pattern.

### 14. Texture in util/ vs nodes/texture/
**Problem**: `Texture` is in `util/texture.rs` but `TextureNodeRuntime` uses it. Should there be a relationship or import?

**Clarification**: This seems fine, just noting the separation.

