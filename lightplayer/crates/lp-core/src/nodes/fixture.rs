//! Fixture node type definitions

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

/// Fixture node types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum FixtureNode {
    #[serde(rename = "circle-list")]
    CircleList {
        output_id: u32,
        channel_order: String,
        mapping: Vec<Mapping>,
    },
}

/// Mapping from shader output to LED channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub channel: u32,
    pub center: [f32; 2],
    pub radius: f32,
}

