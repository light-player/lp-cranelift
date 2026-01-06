//! Project configuration builder with fluent API

use alloc::{format, string::String, string::ToString};
use hashbrown::HashMap;

use crate::error::Error;
use crate::nodes::{
    FixtureId, FixtureNode, OutputId, OutputNode, ShaderId, ShaderNode, TextureId, TextureNode,
};
use crate::project::config::{Nodes, ProjectConfig};

/// Builder for creating ProjectConfig with a fluent API
pub struct ProjectBuilder {
    uid: String,
    name: String,
    next_id: u32,
    nodes: Nodes,
}

impl ProjectBuilder {
    /// Create a new ProjectBuilder with default values
    pub fn new() -> Self {
        Self {
            uid: "default".to_string(),
            name: "Untitled Project".to_string(),
            next_id: 1,
            nodes: Nodes {
                outputs: HashMap::new(),
                textures: HashMap::new(),
                shaders: HashMap::new(),
                fixtures: HashMap::new(),
            },
        }
    }

    /// Create a new ProjectBuilder with test defaults (uid="test", name="Test")
    ///
    /// Convenience method for tests to reduce boilerplate.
    pub fn new_test() -> Self {
        Self {
            uid: "test".to_string(),
            name: "Test".to_string(),
            next_id: 1,
            nodes: Nodes {
                outputs: HashMap::new(),
                textures: HashMap::new(),
                shaders: HashMap::new(),
                fixtures: HashMap::new(),
            },
        }
    }

    /// Set the project UID (fluent)
    pub fn with_uid(mut self, uid: String) -> Self {
        self.uid = uid;
        self
    }

    /// Set the project name (fluent)
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Add a texture node and return its auto-generated ID
    pub fn add_texture(mut self, config: TextureNode) -> (Self, TextureId) {
        let id = TextureId(self.next_id);
        self.next_id += 1;
        self.nodes.textures.insert(u32::from(id), config);
        (self, id)
    }

    /// Add a shader node and return its auto-generated ID
    pub fn add_shader(mut self, config: ShaderNode) -> (Self, ShaderId) {
        let id = ShaderId(self.next_id);
        self.next_id += 1;
        self.nodes.shaders.insert(u32::from(id), config);
        (self, id)
    }

    /// Add an output node and return its auto-generated ID
    pub fn add_output(mut self, config: OutputNode) -> (Self, OutputId) {
        let id = OutputId(self.next_id);
        self.next_id += 1;
        self.nodes.outputs.insert(u32::from(id), config);
        (self, id)
    }

    /// Add a fixture node and return its auto-generated ID
    pub fn add_fixture(mut self, config: FixtureNode) -> (Self, FixtureId) {
        let id = FixtureId(self.next_id);
        self.next_id += 1;
        self.nodes.fixtures.insert(u32::from(id), config);
        (self, id)
    }

    /// Build the ProjectConfig, validating that all referenced IDs exist
    pub fn build(self) -> Result<ProjectConfig, Error> {
        // Validate that all referenced IDs exist
        // Check shader texture_id references
        for (shader_id, shader) in &self.nodes.shaders {
            match shader {
                ShaderNode::Single { texture_id, .. } => {
                    let texture_id_u32 = u32::from(*texture_id);
                    if !self.nodes.textures.contains_key(&texture_id_u32) {
                        return Err(Error::Validation(format!(
                            "Shader {} references non-existent texture {}",
                            shader_id, texture_id_u32
                        )));
                    }
                }
            }
        }

        // Check fixture output_id and texture_id references
        for (fixture_id, fixture) in &self.nodes.fixtures {
            match fixture {
                FixtureNode::CircleList {
                    output_id,
                    texture_id,
                    ..
                } => {
                    let output_id_u32: u32 = (*output_id).into();
                    if !self.nodes.outputs.contains_key(&output_id_u32) {
                        return Err(Error::Validation(format!(
                            "Fixture {} references non-existent output {}",
                            fixture_id, output_id_u32
                        )));
                    }
                    let texture_id_u32: u32 = (*texture_id).into();
                    if !self.nodes.textures.contains_key(&texture_id_u32) {
                        return Err(Error::Validation(format!(
                            "Fixture {} references non-existent texture {}",
                            fixture_id, texture_id_u32
                        )));
                    }
                }
            }
        }

        Ok(ProjectConfig {
            uid: self.uid,
            name: self.name,
            nodes: self.nodes,
        })
    }
}

impl Default for ProjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::ToString, vec};
    use crate::nodes::texture::formats;

    #[test]
    fn test_project_builder_new() {
        let builder = ProjectBuilder::new();
        assert_eq!(builder.uid, "default");
        assert_eq!(builder.name, "Untitled Project");
        assert_eq!(builder.next_id, 1);
    }

    #[test]
    fn test_project_builder_new_test() {
        let builder = ProjectBuilder::new_test();
        assert_eq!(builder.uid, "test");
        assert_eq!(builder.name, "Test");
        assert_eq!(builder.next_id, 1);
    }

    #[test]
    fn test_fluent_api() {
        let builder = ProjectBuilder::new()
            .with_uid("test-uid".to_string())
            .with_name("Test Project".to_string());
        assert_eq!(builder.uid, "test-uid");
        assert_eq!(builder.name, "Test Project");
    }

    #[test]
    fn test_add_texture() {
        let (builder, texture_id) = ProjectBuilder::new().add_texture(TextureNode::Memory {
            size: [64, 64],
            format: formats::RGB8.to_string(),
        });
        assert_eq!(u32::from(texture_id), 1);
        assert_eq!(builder.nodes.textures.len(), 1);
    }

    #[test]
    fn test_add_shader() {
        let (builder, texture_id) = ProjectBuilder::new().add_texture(TextureNode::Memory {
            size: [64, 64],
            format: formats::RGB8.to_string(),
        });
        let (builder, shader_id) = builder.add_shader(ShaderNode::Single {
            glsl: "vec4 main(vec2 fragCoord, vec2 outputSize, float time) { return vec4(1.0); }"
                .to_string(),
            texture_id,
        });
        assert_eq!(u32::from(shader_id), 2);
        assert_eq!(builder.nodes.shaders.len(), 1);
    }

    #[test]
    fn test_add_output() {
        let (builder, output_id) = ProjectBuilder::new().add_output(OutputNode::GpioStrip {
            chip: "gpiochip0".to_string(),
            gpio_pin: 18,
            count: 100,
        });
        assert_eq!(u32::from(output_id), 1);
        assert_eq!(builder.nodes.outputs.len(), 1);
    }

    #[test]
    fn test_add_fixture() {
        let (builder, output_id) = ProjectBuilder::new().add_output(OutputNode::GpioStrip {
            chip: "gpiochip0".to_string(),
            gpio_pin: 18,
            count: 100,
        });
        let (builder, fixture_id) = builder.add_fixture(FixtureNode::CircleList {
            output_id,
            texture_id: TextureId(1),
            channel_order: "RGB".to_string(),
            mapping: vec![],
        });
        assert_eq!(u32::from(fixture_id), 2);
        assert_eq!(builder.nodes.fixtures.len(), 1);
    }

    #[test]
    fn test_build_validates_shader_texture_reference() {
        let builder = ProjectBuilder::new();
        let (builder, _shader_id) = builder.add_shader(ShaderNode::Single {
            glsl: "vec4 main(vec2 fragCoord, vec2 outputSize, float time) { return vec4(1.0); }"
                .to_string(),
            texture_id: TextureId(999), // Non-existent texture
        });
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_build_validates_fixture_output_reference() {
        let builder = ProjectBuilder::new();
        let (builder, _fixture_id) = builder.add_fixture(FixtureNode::CircleList {
            output_id: OutputId(999), // Non-existent output
            texture_id: TextureId(1),
            channel_order: "RGB".to_string(),
            mapping: vec![],
        });
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_build_validates_fixture_texture_reference() {
        let builder = ProjectBuilder::new();
        let (builder, output_id) = builder.add_output(OutputNode::GpioStrip {
            chip: "ws2812".to_string(),
            gpio_pin: 18,
            count: 100,
        });
        let (builder, _fixture_id) = builder.add_fixture(FixtureNode::CircleList {
            output_id,
            texture_id: TextureId(999), // Non-existent texture
            channel_order: "RGB".to_string(),
            mapping: vec![],
        });
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_build_success() {
        let (builder, texture_id) = ProjectBuilder::new()
            .with_uid("test".to_string())
            .with_name("Test".to_string())
            .add_texture(TextureNode::Memory {
                size: [64, 64],
                format: formats::RGB8.to_string(),
            });
        let (builder, _shader_id) = builder.add_shader(ShaderNode::Single {
            glsl: "vec4 main(vec2 fragCoord, vec2 outputSize, float time) { return vec4(1.0); }"
                .to_string(),
            texture_id,
        });
        let config = builder.build().unwrap();
        assert_eq!(config.uid, "test");
        assert_eq!(config.name, "Test");
        assert_eq!(config.nodes.textures.len(), 1);
        assert_eq!(config.nodes.shaders.len(), 1);
    }
}

