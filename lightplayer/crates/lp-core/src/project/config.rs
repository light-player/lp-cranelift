//! Project configuration structures

use alloc::{collections::BTreeMap, format, string::String};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::nodes::{FixtureNode, Mapping, OutputNode, ShaderNode, TextureNode};

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
    #[serde(serialize_with = "serialize_u32_map", deserialize_with = "deserialize_u32_map")]
    pub outputs: HashMap<u32, OutputNode>,
    #[serde(serialize_with = "serialize_u32_map", deserialize_with = "deserialize_u32_map")]
    pub textures: HashMap<u32, TextureNode>,
    #[serde(serialize_with = "serialize_u32_map", deserialize_with = "deserialize_u32_map")]
    pub shaders: HashMap<u32, ShaderNode>,
    #[serde(serialize_with = "serialize_u32_map", deserialize_with = "deserialize_u32_map")]
    pub fixtures: HashMap<u32, FixtureNode>,
}


/// Serialize HashMap<u32, T> with string keys
fn serialize_u32_map<S, T>(map: &HashMap<u32, T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: Serialize,
{
    let string_map: BTreeMap<String, &T> = map
        .iter()
        .map(|(k, v)| (format!("{}", k), v))
        .collect();
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
    use alloc::{string::ToString, vec};

    #[test]
    fn test_serialize_deserialize() {
        let mut config = ProjectConfig {
            uid: "UID12345".to_string(),
            name: "Test Project".to_string(),
            nodes: Nodes {
                outputs: HashMap::new(),
                textures: HashMap::new(),
                shaders: HashMap::new(),
                fixtures: HashMap::new(),
            },
        };

        config.nodes.outputs.insert(
            1,
            OutputNode::GpioStrip {
                chip: "ws2812".to_string(),
                gpio_pin: 4,
                count: 128,
            },
        );

        config.nodes.textures.insert(
            2,
            TextureNode::Memory {
                size: [64, 64],
                format: "RGB8".to_string(),
            },
        );

        config.nodes.shaders.insert(
            3,
            ShaderNode::Single {
                glsl: "void main() {}".to_string(),
                texture_id: 2,
            },
        );

        config.nodes.fixtures.insert(
            4,
            FixtureNode::CircleList {
                output_id: 1,
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
        assert_eq!(config.nodes.textures.len(), deserialized.nodes.textures.len());
        assert_eq!(config.nodes.shaders.len(), deserialized.nodes.shaders.len());
        assert_eq!(config.nodes.fixtures.len(), deserialized.nodes.fixtures.len());
    }
}
