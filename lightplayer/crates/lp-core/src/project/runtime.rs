//! Project runtime status tracking structures

use alloc::{collections::BTreeMap, format, string::String};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// Project runtime status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRuntime {
    pub uid: String,
    pub nodes: RuntimeNodes,
}

/// Collection of runtime status for all node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeNodes {
    #[serde(serialize_with = "serialize_u32_map", deserialize_with = "deserialize_u32_map")]
    pub outputs: HashMap<u32, NodeStatus>,
    #[serde(serialize_with = "serialize_u32_map", deserialize_with = "deserialize_u32_map")]
    pub textures: HashMap<u32, NodeStatus>,
    #[serde(serialize_with = "serialize_u32_map", deserialize_with = "deserialize_u32_map")]
    pub shaders: HashMap<u32, NodeStatus>,
    #[serde(serialize_with = "serialize_u32_map", deserialize_with = "deserialize_u32_map")]
    pub fixtures: HashMap<u32, NodeStatus>,
}

/// Status of a node at runtime
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum NodeStatus {
    #[serde(rename = "Ok")]
    Ok,
    #[serde(rename = "Warn")]
    Warn {
        #[serde(rename = "statusMessage")]
        status_message: String,
    },
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

impl ProjectRuntime {
    /// Create a new empty runtime for a project
    pub fn new(uid: String) -> Self {
        Self {
            uid,
            nodes: RuntimeNodes {
                outputs: HashMap::new(),
                textures: HashMap::new(),
                shaders: HashMap::new(),
                fixtures: HashMap::new(),
            },
        }
    }

    /// Set the status for a node
    pub fn set_status(&mut self, node_type: NodeType, node_id: u32, status: NodeStatus) {
        match node_type {
            NodeType::Output => {
                self.nodes.outputs.insert(node_id, status);
            }
            NodeType::Texture => {
                self.nodes.textures.insert(node_id, status);
            }
            NodeType::Shader => {
                self.nodes.shaders.insert(node_id, status);
            }
            NodeType::Fixture => {
                self.nodes.fixtures.insert(node_id, status);
            }
        }
    }

    /// Get the status for a node
    pub fn get_status(&self, node_type: NodeType, node_id: u32) -> Option<&NodeStatus> {
        match node_type {
            NodeType::Output => self.nodes.outputs.get(&node_id),
            NodeType::Texture => self.nodes.textures.get(&node_id),
            NodeType::Shader => self.nodes.shaders.get(&node_id),
            NodeType::Fixture => self.nodes.fixtures.get(&node_id),
        }
    }
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
    use alloc::string::ToString;

    #[test]
    fn test_runtime_creation() {
        let runtime = ProjectRuntime::new("UID12345".to_string());
        assert_eq!(runtime.uid, "UID12345");
        assert_eq!(runtime.nodes.outputs.len(), 0);
        assert_eq!(runtime.nodes.textures.len(), 0);
        assert_eq!(runtime.nodes.shaders.len(), 0);
        assert_eq!(runtime.nodes.fixtures.len(), 0);
    }

    #[test]
    fn test_set_get_status() {
        let mut runtime = ProjectRuntime::new("UID12345".to_string());

        runtime.set_status(
            NodeType::Output,
            1,
            NodeStatus::Ok,
        );
        runtime.set_status(
            NodeType::Shader,
            2,
            NodeStatus::Warn {
                status_message: "Shader compilation slow".to_string(),
            },
        );
        runtime.set_status(
            NodeType::Texture,
            3,
            NodeStatus::Error {
                status_message: "Texture format not supported".to_string(),
            },
        );

        assert!(matches!(
            runtime.get_status(NodeType::Output, 1),
            Some(NodeStatus::Ok)
        ));
        assert!(matches!(
            runtime.get_status(NodeType::Shader, 2),
            Some(NodeStatus::Warn { .. })
        ));
        assert!(matches!(
            runtime.get_status(NodeType::Texture, 3),
            Some(NodeStatus::Error { .. })
        ));
        assert_eq!(runtime.get_status(NodeType::Fixture, 999), None);
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut runtime = ProjectRuntime::new("UID12345".to_string());
        runtime.set_status(
            NodeType::Output,
            1,
            NodeStatus::Ok,
        );
        runtime.set_status(
            NodeType::Shader,
            2,
            NodeStatus::Warn {
                status_message: "Test warning".to_string(),
            },
        );

        let json = serde_json::to_string(&runtime).unwrap();
        let deserialized: ProjectRuntime = serde_json::from_str(&json).unwrap();

        assert_eq!(runtime.uid, deserialized.uid);
        assert_eq!(runtime.nodes.outputs.len(), deserialized.nodes.outputs.len());
        assert_eq!(runtime.nodes.shaders.len(), deserialized.nodes.shaders.len());
    }
}

