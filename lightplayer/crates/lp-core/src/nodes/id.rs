//! Type-safe node ID wrappers

use serde::{Deserialize, Serialize};

/// Texture node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextureId(pub u32);

/// Output node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OutputId(pub u32);

/// Shader node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShaderId(pub u32);

/// Fixture node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FixtureId(pub u32);

// From/Into implementations for u32

impl From<u32> for TextureId {
    fn from(value: u32) -> Self {
        TextureId(value)
    }
}

impl From<TextureId> for u32 {
    fn from(value: TextureId) -> Self {
        value.0
    }
}

impl From<u32> for OutputId {
    fn from(value: u32) -> Self {
        OutputId(value)
    }
}

impl From<OutputId> for u32 {
    fn from(value: OutputId) -> Self {
        value.0
    }
}

impl From<u32> for ShaderId {
    fn from(value: u32) -> Self {
        ShaderId(value)
    }
}

impl From<ShaderId> for u32 {
    fn from(value: ShaderId) -> Self {
        value.0
    }
}

impl From<u32> for FixtureId {
    fn from(value: u32) -> Self {
        FixtureId(value)
    }
}

impl From<FixtureId> for u32 {
    fn from(value: FixtureId) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_id_serialization() {
        let texture_id = TextureId(42);
        let json = serde_json::to_string(&texture_id).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn test_id_deserialization() {
        let json = "42";
        let texture_id: TextureId = serde_json::from_str(json).unwrap();
        assert_eq!(texture_id, TextureId(42));
    }

    #[test]
    fn test_id_conversions() {
        let id = TextureId(42);
        let u32_val: u32 = id.into();
        assert_eq!(u32_val, 42);
        
        let id2: TextureId = 42.into();
        assert_eq!(id2, TextureId(42));
    }

    #[test]
    fn test_all_id_types() {
        let texture_id = TextureId(1);
        let output_id = OutputId(2);
        let shader_id = ShaderId(3);
        let fixture_id = FixtureId(4);

        assert_eq!(u32::from(texture_id), 1);
        assert_eq!(u32::from(output_id), 2);
        assert_eq!(u32::from(shader_id), 3);
        assert_eq!(u32::from(fixture_id), 4);
    }
}

