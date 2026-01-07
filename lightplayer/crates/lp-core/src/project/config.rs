//! Project configuration structures

use alloc::{collections::BTreeMap, string::String};
use serde::{Deserialize, Serialize};

use crate::nodes::{FixtureNode, OutputNode, ShaderNode, TextureNode};

/// Project configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub uid: String,
    pub name: String,
    pub nodes: Nodes,
}

/// Collection of all node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nodes {
    pub outputs: BTreeMap<String, OutputNode>,
    pub textures: BTreeMap<String, TextureNode>,
    pub shaders: BTreeMap<String, ShaderNode>,
    pub fixtures: BTreeMap<String, FixtureNode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::Mapping;
    use alloc::{string::ToString, vec};

    #[test]
    fn test_serialize_deserialize() {
        let mut config = ProjectConfig {
            uid: "UID12345".to_string(),
            name: "Test Project".to_string(),
            nodes: Nodes {
                outputs: BTreeMap::new(),
                textures: BTreeMap::new(),
                shaders: BTreeMap::new(),
                fixtures: BTreeMap::new(),
            },
        };

        config.nodes.outputs.insert(
            "/src/output.output".to_string(),
            OutputNode::GpioStrip {
                chip: "ws2812".to_string(),
                gpio_pin: 4,
                count: 128,
            },
        );

        config.nodes.textures.insert(
            "/src/texture.texture".to_string(),
            TextureNode::Memory {
                size: [64, 64],
                format: "RGB8".to_string(),
            },
        );

        config.nodes.shaders.insert(
            "/src/shader.shader".to_string(),
            ShaderNode::Single {
                glsl: "void main() {}".to_string(),
                texture_id: crate::nodes::id::TextureId("/src/texture.texture".to_string()),
            },
        );

        config.nodes.fixtures.insert(
            "/src/fixture.fixture".to_string(),
            FixtureNode::CircleList {
                output_id: crate::nodes::id::OutputId("/src/output.output".to_string()),
                texture_id: crate::nodes::id::TextureId("/src/texture.texture".to_string()),
                channel_order: "rgb".to_string(),
                mapping: vec![Mapping {
                    channel: 0,
                    center: [0.5, 0.5],
                    radius: 0.1,
                }],
            },
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProjectConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.uid, deserialized.uid);
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.nodes.outputs.len(), deserialized.nodes.outputs.len());
        assert_eq!(
            config.nodes.textures.len(),
            deserialized.nodes.textures.len()
        );
        assert_eq!(config.nodes.shaders.len(), deserialized.nodes.shaders.len());
        assert_eq!(
            config.nodes.fixtures.len(),
            deserialized.nodes.fixtures.len()
        );
    }
}
