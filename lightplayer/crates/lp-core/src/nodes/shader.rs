//! Shader node type definitions

use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Shader node types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum ShaderNode {
    #[serde(rename = "Single")]
    Single {
        glsl: String,
        texture_id: u32,
    },
}

