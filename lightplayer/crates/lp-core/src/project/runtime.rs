//! Project runtime - manages lifecycle of all nodes

use alloc::{collections::BTreeMap, format, string::String};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::nodes::id::{FixtureId, OutputId, ShaderId, TextureId};
use crate::nodes::{FixtureNodeRuntime, OutputNodeRuntime, ShaderNodeRuntime, TextureNodeRuntime};
use crate::project::config::ProjectConfig;
use crate::runtime::contexts::{
    FixtureRenderContext, InitContext, OutputRenderContext, ShaderRenderContext,
};
use crate::runtime::frame_time::FrameTime;
use crate::runtime::lifecycle::NodeLifecycle;
use crate::traits::OutputProvider;

/// Project runtime - manages lifecycle of all node runtimes
pub struct ProjectRuntime {
    #[allow(dead_code)] // Used for serialization via get_runtime_nodes
    uid: String,
    frame_time: FrameTime,
    textures: HashMap<TextureId, TextureNodeRuntime>,
    shaders: HashMap<ShaderId, ShaderNodeRuntime>,
    fixtures: HashMap<FixtureId, FixtureNodeRuntime>,
    outputs: HashMap<OutputId, OutputNodeRuntime>,
}

impl ProjectRuntime {
    /// Create a new empty runtime for a project
    pub fn new(uid: String) -> Self {
        Self {
            uid,
            frame_time: FrameTime::new(0, 0),
            textures: HashMap::new(),
            shaders: HashMap::new(),
            fixtures: HashMap::new(),
            outputs: HashMap::new(),
        }
    }

    /// Initialize all nodes from configuration
    ///
    /// Initializes nodes in order: textures → shaders → fixtures → outputs
    /// Allows partial failures (nodes handle their own failures)
    pub fn init(
        &mut self,
        config: &ProjectConfig,
        output_provider: &dyn OutputProvider,
    ) -> Result<(), Error> {
        let init_ctx = InitContext::new(config);

        // Initialize textures
        for (id_u32, texture_config) in &config.nodes.textures {
            let texture_id = TextureId(*id_u32);
            let mut texture_runtime = TextureNodeRuntime::new();
            if let Err(e) = texture_runtime.init(texture_config, &init_ctx) {
                // Log error but continue - node status is set internally
                let _ = e;
            }
            self.textures.insert(texture_id, texture_runtime);
        }

        // Initialize shaders
        for (id_u32, shader_config) in &config.nodes.shaders {
            let shader_id = ShaderId(*id_u32);
            let mut shader_runtime = ShaderNodeRuntime::new();
            if let Err(e) = shader_runtime.init(shader_config, &init_ctx) {
                // Log error but continue - node status is set internally
                let _ = e;
            }
            self.shaders.insert(shader_id, shader_runtime);
        }

        // Initialize fixtures
        for (id_u32, fixture_config) in &config.nodes.fixtures {
            let fixture_id = FixtureId(*id_u32);
            let mut fixture_runtime = FixtureNodeRuntime::new();
            if let Err(e) = fixture_runtime.init(fixture_config, &init_ctx) {
                // Log error but continue - node status is set internally
                let _ = e;
            }
            self.fixtures.insert(fixture_id, fixture_runtime);
        }

        // Initialize outputs and create LED handles
        for (id_u32, output_config) in &config.nodes.outputs {
            let output_id = OutputId(*id_u32);
            let mut output_runtime = OutputNodeRuntime::new();
            if let Err(e) = output_runtime.init(output_config, &init_ctx) {
                // Log error but continue - node status is set internally
                let _ = e;
            } else {
                // Create LED output handle via OutputProvider
                match output_provider.create_output(output_config, Some(output_id)) {
                    Ok(handle) => {
                        output_runtime.set_handle(handle);
                    }
                    Err(e) => {
                        // Set error status but continue
                        let _ = e;
                    }
                }
            }
            self.outputs.insert(output_id, output_runtime);
        }

        Ok(())
    }

    /// Update all nodes
    ///
    /// Updates nodes in order: shaders → fixtures → outputs
    /// Updates frame_time: total_ms += delta_ms, delta_ms = delta_ms
    pub fn update(
        &mut self,
        delta_ms: u32,
        _output_provider: &dyn OutputProvider,
    ) -> Result<(), Error> {
        // Update frame time
        self.frame_time.total_ms += delta_ms;
        self.frame_time.delta_ms = delta_ms;

        // Update shaders (write to textures)
        for shader_runtime in self.shaders.values_mut() {
            let mut ctx = ShaderRenderContext::new(self.frame_time, &mut self.textures);
            if let Err(_e) = shader_runtime.update(&mut ctx) {
                // Error status is set internally
            }
        }

        // Update fixtures (sample textures, write to outputs)
        for fixture_runtime in self.fixtures.values_mut() {
            let mut ctx =
                FixtureRenderContext::new(self.frame_time, &self.textures, &mut self.outputs);
            if let Err(_e) = fixture_runtime.update(&mut ctx) {
                // Error status is set internally
            }
        }

        // Update outputs (send buffer to hardware)
        for output_runtime in self.outputs.values_mut() {
            let mut ctx = OutputRenderContext::new(self.frame_time);
            if let Err(_e) = output_runtime.update(&mut ctx) {
                // Error status is set internally
            }
        }

        Ok(())
    }

    /// Destroy all nodes
    ///
    /// Calls destroy() on all nodes in reverse order: outputs → fixtures → shaders → textures
    pub fn destroy(&mut self) -> Result<(), Error> {
        // Destroy outputs
        for output_runtime in self.outputs.values_mut() {
            let _ = output_runtime.destroy();
        }

        // Destroy fixtures
        for fixture_runtime in self.fixtures.values_mut() {
            let _ = fixture_runtime.destroy();
        }

        // Destroy shaders
        for shader_runtime in self.shaders.values_mut() {
            let _ = shader_runtime.destroy();
        }

        // Destroy textures
        for texture_runtime in self.textures.values_mut() {
            let _ = texture_runtime.destroy();
        }

        Ok(())
    }

    /// Get runtime nodes status for serialization
    ///
    /// Derives RuntimeNodes from runtime instances, converting type-safe IDs to u32
    pub fn get_runtime_nodes(&self) -> RuntimeNodes {
        let mut runtime_nodes = RuntimeNodes {
            outputs: HashMap::new(),
            textures: HashMap::new(),
            shaders: HashMap::new(),
            fixtures: HashMap::new(),
        };

        // Collect status from texture runtimes
        for (id, runtime) in &self.textures {
            runtime_nodes
                .textures
                .insert(u32::from(*id), runtime.status().clone());
        }

        // Collect status from shader runtimes
        for (id, runtime) in &self.shaders {
            runtime_nodes
                .shaders
                .insert(u32::from(*id), runtime.status().clone());
        }

        // Collect status from fixture runtimes
        for (id, runtime) in &self.fixtures {
            runtime_nodes
                .fixtures
                .insert(u32::from(*id), runtime.status().clone());
        }

        // Collect status from output runtimes
        for (id, runtime) in &self.outputs {
            runtime_nodes
                .outputs
                .insert(u32::from(*id), runtime.status().clone());
        }

        runtime_nodes
    }

    /// Get the status for a node
    pub fn get_status(&self, node_type: NodeType, node_id: u32) -> Option<&NodeStatus> {
        match node_type {
            NodeType::Output => self.outputs.get(&OutputId(node_id)).map(|r| r.status()),
            NodeType::Texture => self.textures.get(&TextureId(node_id)).map(|r| r.status()),
            NodeType::Shader => self.shaders.get(&ShaderId(node_id)).map(|r| r.status()),
            NodeType::Fixture => self.fixtures.get(&FixtureId(node_id)).map(|r| r.status()),
        }
    }

    /// Get a texture runtime by ID
    pub fn get_texture(&self, id: TextureId) -> Option<&TextureNodeRuntime> {
        self.textures.get(&id)
    }

    /// Get a shader runtime by ID
    pub fn get_shader(&self, id: ShaderId) -> Option<&ShaderNodeRuntime> {
        self.shaders.get(&id)
    }

    /// Get a fixture runtime by ID
    pub fn get_fixture(&self, id: FixtureId) -> Option<&FixtureNodeRuntime> {
        self.fixtures.get(&id)
    }

    /// Get an output runtime by ID
    pub fn get_output(&self, id: OutputId) -> Option<&OutputNodeRuntime> {
        self.outputs.get(&id)
    }
}

/// Collection of runtime status for all node types (for serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeNodes {
    #[serde(
        serialize_with = "serialize_u32_map",
        deserialize_with = "deserialize_u32_map"
    )]
    pub outputs: HashMap<u32, NodeStatus>,
    #[serde(
        serialize_with = "serialize_u32_map",
        deserialize_with = "deserialize_u32_map"
    )]
    pub textures: HashMap<u32, NodeStatus>,
    #[serde(
        serialize_with = "serialize_u32_map",
        deserialize_with = "deserialize_u32_map"
    )]
    pub shaders: HashMap<u32, NodeStatus>,
    #[serde(
        serialize_with = "serialize_u32_map",
        deserialize_with = "deserialize_u32_map"
    )]
    pub fixtures: HashMap<u32, NodeStatus>,
}

/// Status of a node at runtime
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum NodeStatus {
    #[serde(rename = "Ok")]
    Ok,
    #[serde(rename = "Error")]
    Error {
        #[serde(rename = "statusMessage")]
        status_message: String,
    },
}

/// Node type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Output,
    Texture,
    Shader,
    Fixture,
}

/// Serialize HashMap<u32, T> with string keys
fn serialize_u32_map<S, T>(map: &HashMap<u32, T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: Serialize,
{
    let string_map: BTreeMap<String, &T> = map.iter().map(|(k, v)| (format!("{}", k), v)).collect();
    string_map.serialize(serializer)
}

/// Deserialize HashMap<u32, T> from string keys
fn deserialize_u32_map<'de, D, T>(deserializer: D) -> Result<HashMap<u32, T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    let string_map: BTreeMap<String, T> = BTreeMap::deserialize(deserializer)?;
    Ok(string_map
        .into_iter()
        .filter_map(|(k, v)| k.parse::<u32>().ok().map(|id| (id, v)))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::texture::formats;
    use crate::nodes::{FixtureNode, Mapping, OutputNode, ShaderNode, TextureNode};
    use crate::traits::{LedOutput, OutputProvider};
    use alloc::string::ToString;
    use alloc::vec;

    // Mock OutputProvider for testing
    struct MockOutputProvider;

    impl OutputProvider for MockOutputProvider {
        fn create_output(
            &self,
            _config: &OutputNode,
            _output_id: Option<OutputId>,
        ) -> Result<alloc::boxed::Box<dyn LedOutput>, Error> {
            Ok(alloc::boxed::Box::new(MockLedOutput {
                pixel_count: 100,
                last_written: alloc::vec::Vec::new(),
            }))
        }
    }

    // Mock LedOutput for testing
    struct MockLedOutput {
        pixel_count: usize,
        last_written: alloc::vec::Vec<u8>,
    }

    impl LedOutput for MockLedOutput {
        fn write_pixels(&mut self, pixels: &[u8]) -> Result<(), Error> {
            self.last_written = pixels.to_vec();
            Ok(())
        }

        fn get_pixel_count(&self) -> usize {
            self.pixel_count
        }
    }

    #[test]
    fn test_project_runtime_new() {
        let runtime = ProjectRuntime::new("test-uid".to_string());
        assert_eq!(runtime.uid, "test-uid");
        assert_eq!(runtime.textures.len(), 0);
        assert_eq!(runtime.shaders.len(), 0);
        assert_eq!(runtime.fixtures.len(), 0);
        assert_eq!(runtime.outputs.len(), 0);
    }

    #[test]
    fn test_project_runtime_init() {
        let mut runtime = ProjectRuntime::new("test".to_string());
        let (builder, texture_id) = crate::project::builder::ProjectBuilder::new_test()
            .add_texture(TextureNode::Memory {
                size: [64, 64],
                format: formats::RGBA8.to_string(),
            });
        let (builder, _shader_id) = builder.add_shader(ShaderNode::Single {
            glsl: "vec4 main(vec2 fragCoord, vec2 outputSize, float time) { return vec4(1.0); }"
                .to_string(),
            texture_id,
        });
        let (builder, output_id) = builder.add_output(OutputNode::GpioStrip {
            chip: "ws2812".to_string(),
            gpio_pin: 18,
            count: 100,
        });
        let (builder, _fixture_id) = builder.add_fixture(FixtureNode::CircleList {
            output_id,
            texture_id,
            channel_order: "rgb".to_string(),
            mapping: vec![Mapping {
                channel: 0,
                center: [0.5, 0.5],
                radius: 0.1,
            }],
        });
        let config = builder.build().unwrap();

        let output_provider = MockOutputProvider;
        assert!(runtime.init(&config, &output_provider).is_ok());

        // Check that nodes were initialized
        assert_eq!(runtime.textures.len(), 1);
        assert_eq!(runtime.shaders.len(), 1);
        assert_eq!(runtime.fixtures.len(), 1);
        assert_eq!(runtime.outputs.len(), 1);
    }

    #[test]
    fn test_project_runtime_update() {
        let mut runtime = ProjectRuntime::new("test".to_string());
        let (builder, texture_id) = crate::project::builder::ProjectBuilder::new_test()
            .add_texture(TextureNode::Memory {
                size: [4, 4],
                format: formats::RGBA8.to_string(),
            });
        let (builder, _shader_id) = builder.add_shader(ShaderNode::Single {
            glsl: "vec4 main(vec2 fragCoord, vec2 outputSize, float time) { return vec4(0.5, 0.5, 0.5, 1.0); }"
                .to_string(),
            texture_id,
        });
        let config = builder.build().unwrap();

        let output_provider = MockOutputProvider;
        runtime.init(&config, &output_provider).unwrap();

        // Update with 16ms delta
        assert!(runtime.update(16, &output_provider).is_ok());

        // Check frame time was updated
        assert_eq!(runtime.frame_time.delta_ms, 16);
        assert_eq!(runtime.frame_time.total_ms, 16);
    }

    #[test]
    fn test_project_runtime_get_runtime_nodes() {
        let mut runtime = ProjectRuntime::new("test".to_string());
        let (builder, _texture_id) = crate::project::builder::ProjectBuilder::new_test()
            .add_texture(TextureNode::Memory {
                size: [64, 64],
                format: formats::RGBA8.to_string(),
            });
        let config = builder.build().unwrap();

        let output_provider = MockOutputProvider;
        runtime.init(&config, &output_provider).unwrap();

        let runtime_nodes = runtime.get_runtime_nodes();
        assert_eq!(runtime_nodes.textures.len(), 1);
    }

    #[test]
    fn test_project_runtime_destroy() {
        let mut runtime = ProjectRuntime::new("test".to_string());
        let (builder, _texture_id) = crate::project::builder::ProjectBuilder::new_test()
            .add_texture(TextureNode::Memory {
                size: [64, 64],
                format: formats::RGBA8.to_string(),
            });
        let config = builder.build().unwrap();

        let output_provider = MockOutputProvider;
        runtime.init(&config, &output_provider).unwrap();

        assert!(runtime.destroy().is_ok());
    }

    #[test]
    fn test_complete_project_lifecycle() {
        // Build project
        let (builder, texture_id) = crate::project::builder::ProjectBuilder::new_test()
            .add_texture(TextureNode::Memory {
                size: [8, 8],
                format: formats::RGBA8.to_string(),
            });
        let (builder, _shader_id) = builder.add_shader(ShaderNode::Single {
            glsl: "vec4 main(vec2 fragCoord, vec2 outputSize, float time) { return vec4(0.5, 0.5, 0.5, 1.0); }"
                .to_string(),
            texture_id,
        });
        let (builder, output_id) = builder.add_output(OutputNode::GpioStrip {
            chip: "ws2812".to_string(),
            gpio_pin: 18,
            count: 10,
        });
        let (builder, _fixture_id) = builder.add_fixture(FixtureNode::CircleList {
            output_id,
            texture_id,
            channel_order: "rgb".to_string(),
            mapping: vec![Mapping {
                channel: 0,
                center: [0.5, 0.5],
                radius: 0.1,
            }],
        });
        let config = builder.build().unwrap();

        // Init runtime
        let mut runtime = ProjectRuntime::new("test".to_string());
        let output_provider = MockOutputProvider;
        assert!(runtime.init(&config, &output_provider).is_ok());

        // Update multiple times
        assert!(runtime.update(16, &output_provider).is_ok());
        assert_eq!(runtime.frame_time.delta_ms, 16);
        assert_eq!(runtime.frame_time.total_ms, 16);

        assert!(runtime.update(16, &output_provider).is_ok());
        assert_eq!(runtime.frame_time.delta_ms, 16);
        assert_eq!(runtime.frame_time.total_ms, 32);

        // Destroy
        assert!(runtime.destroy().is_ok());
    }

    #[test]
    fn test_shader_fixture_output_pipeline() {
        // Build: texture → shader → fixture → output
        let (builder, texture_id) = crate::project::builder::ProjectBuilder::new_test()
            .add_texture(TextureNode::Memory {
                size: [4, 4],
                format: formats::RGBA8.to_string(),
            });
        let (builder, _shader_id) = builder.add_shader(ShaderNode::Single {
            glsl: "vec4 main(vec2 fragCoord, vec2 outputSize, float time) { return vec4(1.0, 0.0, 0.0, 1.0); }"
                .to_string(),
            texture_id,
        });
        let (builder, output_id) = builder.add_output(OutputNode::GpioStrip {
            chip: "ws2812".to_string(),
            gpio_pin: 18,
            count: 5,
        });
        let (builder, _fixture_id) = builder.add_fixture(FixtureNode::CircleList {
            output_id,
            texture_id,
            channel_order: "rgb".to_string(),
            mapping: vec![Mapping {
                channel: 0,
                center: [0.5, 0.5],
                radius: 0.2,
            }],
        });
        let config = builder.build().unwrap();

        // Init and update
        let mut runtime = ProjectRuntime::new("test".to_string());
        let output_provider = MockOutputProvider;
        runtime.init(&config, &output_provider).unwrap();
        runtime.update(16, &output_provider).unwrap();

        // Verify pipeline worked: shader wrote to texture, fixture sampled texture, output got data
        let runtime_nodes = runtime.get_runtime_nodes();
        // All nodes should be Ok status
        assert!(matches!(
            runtime_nodes.shaders.values().next(),
            Some(NodeStatus::Ok)
        ));
        assert!(matches!(
            runtime_nodes.fixtures.values().next(),
            Some(NodeStatus::Ok)
        ));
        assert!(matches!(
            runtime_nodes.outputs.values().next(),
            Some(NodeStatus::Ok)
        ));
    }

    #[test]
    fn test_multiple_fixtures_same_output() {
        // Build: one output, multiple fixtures
        let (builder, texture_id) = crate::project::builder::ProjectBuilder::new_test()
            .add_texture(TextureNode::Memory {
                size: [4, 4],
                format: formats::RGBA8.to_string(),
            });
        let (builder, output_id) = builder.add_output(OutputNode::GpioStrip {
            chip: "ws2812".to_string(),
            gpio_pin: 18,
            count: 10,
        });
        let (builder, _fixture1_id) = builder.add_fixture(FixtureNode::CircleList {
            output_id,
            texture_id,
            channel_order: "rgb".to_string(),
            mapping: vec![Mapping {
                channel: 0,
                center: [0.3, 0.3],
                radius: 0.1,
            }],
        });
        let (builder, _fixture2_id) = builder.add_fixture(FixtureNode::CircleList {
            output_id,
            texture_id,
            channel_order: "rgb".to_string(),
            mapping: vec![Mapping {
                channel: 1,
                center: [0.7, 0.7],
                radius: 0.1,
            }],
        });
        let config = builder.build().unwrap();

        let mut runtime = ProjectRuntime::new("test".to_string());
        let output_provider = MockOutputProvider;
        runtime.init(&config, &output_provider).unwrap();
        runtime.update(16, &output_provider).unwrap();

        // Both fixtures should have written to the same output
        let runtime_nodes = runtime.get_runtime_nodes();
        assert_eq!(runtime_nodes.fixtures.len(), 2);
        assert_eq!(runtime_nodes.outputs.len(), 1);
    }

    #[test]
    fn test_frame_time_tracking() {
        let (builder, texture_id) = crate::project::builder::ProjectBuilder::new_test()
            .add_texture(TextureNode::Memory {
                size: [4, 4],
                format: formats::RGBA8.to_string(),
            });
        let config = builder.build().unwrap();

        let mut runtime = ProjectRuntime::new("test".to_string());
        let output_provider = MockOutputProvider;
        runtime.init(&config, &output_provider).unwrap();

        // Initial state
        assert_eq!(runtime.frame_time.delta_ms, 0);
        assert_eq!(runtime.frame_time.total_ms, 0);

        // First update
        runtime.update(16, &output_provider).unwrap();
        assert_eq!(runtime.frame_time.delta_ms, 16);
        assert_eq!(runtime.frame_time.total_ms, 16);

        // Second update
        runtime.update(17, &output_provider).unwrap();
        assert_eq!(runtime.frame_time.delta_ms, 17);
        assert_eq!(runtime.frame_time.total_ms, 33);

        // Third update
        runtime.update(16, &output_provider).unwrap();
        assert_eq!(runtime.frame_time.delta_ms, 16);
        assert_eq!(runtime.frame_time.total_ms, 49);
    }
}
