//! Runtime contexts for node access

use crate::nodes::id::{FixtureId, OutputId, ShaderId, TextureId};
use crate::nodes::fixture::FixtureNode;
use crate::nodes::output::OutputNode;
use crate::nodes::shader::ShaderNode;
use crate::nodes::texture::TextureNode;
use crate::project::config::ProjectConfig;
use crate::runtime::frame_time::FrameTime;
use crate::util::Texture;
use hashbrown::HashMap;

// Forward declarations - these will be implemented in later phases
pub struct ShaderNodeRuntime;
pub struct FixtureNodeRuntime;

// OutputNodeRuntime is now implemented
use crate::nodes::output::OutputNodeRuntime;

// TextureNodeRuntime is now implemented
use crate::nodes::texture::TextureNodeRuntime;

/// Initialization context providing read-only access to project configuration
pub struct InitContext<'a> {
    project_config: &'a ProjectConfig,
}

impl<'a> InitContext<'a> {
    /// Create a new initialization context
    pub fn new(project_config: &'a ProjectConfig) -> Self {
        Self { project_config }
    }

    /// Get texture configuration by ID
    pub fn get_texture_config(&self, id: TextureId) -> Option<&TextureNode> {
        let id_u32: u32 = id.into();
        self.project_config.nodes.textures.get(&id_u32)
    }

    /// Get shader configuration by ID
    pub fn get_shader_config(&self, id: ShaderId) -> Option<&ShaderNode> {
        let id_u32: u32 = id.into();
        self.project_config.nodes.shaders.get(&id_u32)
    }

    /// Get fixture configuration by ID
    pub fn get_fixture_config(&self, id: FixtureId) -> Option<&FixtureNode> {
        let id_u32: u32 = id.into();
        self.project_config.nodes.fixtures.get(&id_u32)
    }

    /// Get output configuration by ID
    pub fn get_output_config(&self, id: OutputId) -> Option<&OutputNode> {
        let id_u32: u32 = id.into();
        self.project_config.nodes.outputs.get(&id_u32)
    }
}

/// Render context for shader nodes
///
/// Provides mutable access to textures for writing rendered pixels.
pub struct ShaderRenderContext<'a> {
    pub time: FrameTime,
    pub textures: &'a mut HashMap<TextureId, TextureNodeRuntime>,
}

impl<'a> ShaderRenderContext<'a> {
    /// Create a new shader render context
    pub fn new(
        time: FrameTime,
        textures: &'a mut HashMap<TextureId, TextureNodeRuntime>,
    ) -> Self {
        Self { time, textures }
    }

    /// Get mutable access to a texture
    ///
    /// Returns None if the texture doesn't exist.
    pub fn get_texture_mut(&mut self, texture_id: TextureId) -> Option<&mut Texture> {
        self.textures.get_mut(&texture_id).map(|rt| rt.texture_mut())
    }
}

/// Render context for fixture nodes
///
/// Provides read-only access to textures and mutable access to outputs.
pub struct FixtureRenderContext<'a> {
    pub time: FrameTime,
    pub textures: &'a HashMap<TextureId, TextureNodeRuntime>,
    pub outputs: &'a mut HashMap<OutputId, OutputNodeRuntime>,
}

impl<'a> FixtureRenderContext<'a> {
    /// Create a new fixture render context
    pub fn new(
        time: FrameTime,
        textures: &'a HashMap<TextureId, TextureNodeRuntime>,
        outputs: &'a mut HashMap<OutputId, OutputNodeRuntime>,
    ) -> Self {
        Self {
            time,
            textures,
            outputs,
        }
    }

    /// Get read-only access to a texture
    ///
    /// Returns None if the texture doesn't exist.
    pub fn get_texture(&self, texture_id: TextureId) -> Option<&Texture> {
        self.textures.get(&texture_id).map(|rt| rt.texture())
    }

    /// Get mutable access to an output node runtime
    ///
    /// Returns None if the output doesn't exist.
    pub fn get_output_mut(&mut self, output_id: OutputId) -> Option<&mut OutputNodeRuntime> {
        self.outputs.get_mut(&output_id)
    }
}

/// Render context for output nodes
///
/// Provides only timing information.
pub struct OutputRenderContext {
    pub time: FrameTime,
}

impl OutputRenderContext {
    /// Create a new output render context
    pub fn new(time: FrameTime) -> Self {
        Self { time }
    }
}

/// Render context for texture nodes
///
/// Provides only timing information (textures don't update themselves).
pub struct TextureRenderContext {
    pub time: FrameTime,
}

impl TextureRenderContext {
    /// Create a new texture render context
    pub fn new(time: FrameTime) -> Self {
        Self { time }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use crate::project::config::{Nodes, ProjectConfig};

    #[test]
    fn test_init_context_get_configs() {
        use hashbrown::HashMap;
        use crate::nodes::{ShaderNode, TextureNode};

        let mut project = ProjectConfig {
            uid: "test".to_string(),
            name: "Test".to_string(),
            nodes: Nodes {
                outputs: HashMap::new(),
                textures: HashMap::new(),
                shaders: HashMap::new(),
                fixtures: HashMap::new(),
            },
        };

        // Add a texture
        project.nodes.textures.insert(
            1,
            TextureNode::Memory {
                size: [64, 64],
                format: "RGB8".to_string(),
            },
        );

        // Add a shader
        project.nodes.shaders.insert(
            2,
            ShaderNode::Single {
                glsl: "vec4 main() { return vec4(1.0); }".to_string(),
                texture_id: 1,
            },
        );

        let ctx = InitContext::new(&project);

        // Test getting texture config
        let texture_config = ctx.get_texture_config(TextureId(1));
        assert!(texture_config.is_some());
        if let Some(TextureNode::Memory { size, format }) = texture_config {
            assert_eq!(*size, [64, 64]);
            assert_eq!(format, "RGB8");
        }

        // Test getting shader config
        let shader_config = ctx.get_shader_config(ShaderId(2));
        assert!(shader_config.is_some());

        // Test getting non-existent config
        assert!(ctx.get_texture_config(TextureId(999)).is_none());
    }

    #[test]
    fn test_render_contexts_creation() {
        let frame_time = FrameTime::new(16, 1000);
        let mut textures: HashMap<TextureId, TextureNodeRuntime> = HashMap::new();
        let mut outputs: HashMap<OutputId, OutputNodeRuntime> = HashMap::new();

        // Test ShaderRenderContext
        let _shader_ctx = ShaderRenderContext::new(frame_time, &mut textures);

        // Test FixtureRenderContext
        let _fixture_ctx = FixtureRenderContext::new(frame_time, &textures, &mut outputs);

        // Test OutputRenderContext
        let _output_ctx = OutputRenderContext::new(frame_time);

        // Test TextureRenderContext
        let _texture_ctx = TextureRenderContext::new(frame_time);
    }
}
