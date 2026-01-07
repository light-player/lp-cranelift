//! LightPlayer Application - main entry point for firmware

use alloc::{format, string::ToString, vec::Vec};
use crate::app::Platform;
use crate::error::Error;
use crate::project::{config::ProjectConfig, runtime::ProjectRuntime};

/// LightPlayer Application
///
/// Main entry point for firmware. Manages project lifecycle, handles messages,
/// and coordinates runtime updates.
pub struct LpApp {
    /// Platform-specific trait implementations
    pub platform: Platform,
    /// Current project config (None if no project loaded)
    config: Option<ProjectConfig>,
    /// Current project runtime (None if no project loaded)
    runtime: Option<ProjectRuntime>,
}

impl LpApp {
    /// Create a new LpApp with the provided platform
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            config: None,
            runtime: None,
        }
    }

    /// Create a default project with basic setup
    ///
    /// Creates a project with:
    /// - One output: 128 LEDs on GPIO 4 (ws2812)
    /// - One texture: 64x64 RGB8
    /// - One shader: rotating color wheel animation
    /// - One fixture: circle-list mapping LEDs in a grid pattern
    pub fn create_default_project() -> ProjectConfig {
        use hashbrown::HashMap;
        use crate::nodes::id::{OutputId, TextureId};
        use crate::nodes::{FixtureNode, Mapping, OutputNode, ShaderNode, TextureNode};
        use crate::project::config::{Nodes, ProjectConfig};

        // Create default project
        let mut project = ProjectConfig {
            uid: "default".to_string(),
            name: "Default Project".to_string(),
            nodes: Nodes {
                outputs: HashMap::new(),
                textures: HashMap::new(),
                shaders: HashMap::new(),
                fixtures: HashMap::new(),
            },
        };

        // Add output: 128 LEDs on GPIO 4
        project.nodes.outputs.insert(
            1,
            OutputNode::GpioStrip {
                chip: "ws2812".to_string(),
                gpio_pin: 4,
                count: 128,
            },
        );

        // Add texture: 64x64 RGB8
        project.nodes.textures.insert(
            2,
            TextureNode::Memory {
                size: [64, 64],
                format: "RGB8".to_string(),
            },
        );

        // Add shader: rotating color wheel animation
        project.nodes.shaders.insert(
            3,
            ShaderNode::Single {
                glsl: r#"
vec4 main(vec2 fragCoord, vec2 outputSize, float time) {
    // Center of texture
    vec2 center = outputSize * 0.5;
    
    // Direction from center to fragment
    vec2 dir = fragCoord - center;
    
    // Calculate angle (atan2 gives angle in [-PI, PI])
    float angle = atan(dir.y, dir.x);
    
    // Rotate angle with time (full rotation every 4 seconds)
    angle = angle + time * 0.5;
    
    // Normalize angle to [0, 1] for hue
    float hue = (angle / (2.0 * 3.14159) + 1.0) * 0.5;
    
    // Distance from center (normalized)
    float dist = length(dir) / (min(outputSize.x, outputSize.y) * 0.5);
    
    // Create color wheel: hue rotates, saturation and value vary with distance
    // Convert HSV to RGB (simplified)
    float c = 1.0 - abs(dist - 0.5) * 2.0; // Saturation based on distance
    float x = c * (1.0 - abs(mod(hue * 6.0, 2.0) - 1.0));
    float m = 0.8 - dist * 0.3; // Value (brightness)
    
    vec3 rgb;
    if (hue < 1.0/6.0) {
        rgb = vec3(c, x, 0.0);
    } else if (hue < 2.0/6.0) {
        rgb = vec3(x, c, 0.0);
    } else if (hue < 3.0/6.0) {
        rgb = vec3(0.0, c, x);
    } else if (hue < 4.0/6.0) {
        rgb = vec3(0.0, x, c);
    } else if (hue < 5.0/6.0) {
        rgb = vec3(x, 0.0, c);
    } else {
        rgb = vec3(c, 0.0, x);
    }
    
    return vec4((rgb + m - 0.4) * m, 1.0);
}
"#
                .to_string(),
                texture_id: TextureId(2),
            },
        );

        // Add fixture: circle-list mapping
        // Map LEDs in a grid pattern across the 64x64 texture
        let mut mapping = Vec::new();
        let led_count = 128;
        let cols = 16; // 16 columns
        let rows = led_count / cols; // 8 rows

        for i in 0..led_count {
            let row = i / cols;
            let col = i % cols;
            // Map to normalized coordinates [0, 1]
            let x = (col as f32 + 0.5) / cols as f32;
            let y = (row as f32 + 0.5) / rows as f32;

            mapping.push(Mapping {
                channel: i as u32,
                center: [x, y],
                radius: 0.05, // Small sampling radius
            });
        }

        project.nodes.fixtures.insert(
            4,
            FixtureNode::CircleList {
                output_id: OutputId(1),
                texture_id: TextureId(2),
                channel_order: "rgb".to_string(),
                mapping,
            },
        );

        project
    }

    /// Load a project from the filesystem
    ///
    /// Reads the project file, parses it as JSON, creates a ProjectRuntime,
    /// and initializes all nodes. If a project is already loaded, it is destroyed
    /// before loading the new one.
    ///
    /// If the project file doesn't exist, creates a default project, saves it,
    /// and loads it.
    pub fn load_project(&mut self, path: &str) -> Result<(), Error> {
        // Destroy existing runtime if present
        if let Some(mut runtime) = self.runtime.take() {
            let _ = runtime.destroy();
        }

        // Check if project file exists
        let config = if self.platform.fs.file_exists(path)? {
            // Read project file
            let data = self.platform.fs.read_file(path)?;
            let json = alloc::string::String::from_utf8(data)
                .map_err(|e| Error::Filesystem(format!("Invalid UTF-8 in {}: {}", path, e)))?;

            // Parse project config
            serde_json::from_str(&json)
                .map_err(|e| Error::Validation(format!("Invalid project file {}: {}", path, e)))?
        } else {
            // Create default project
            let default_config = Self::create_default_project();

            // Save default project to filesystem
            let json = serde_json::to_string_pretty(&default_config)
                .map_err(|e| Error::Serialization(format!("Failed to serialize project: {}", e)))?;
            self.platform.fs.write_file(path, json.as_bytes())?;

            default_config
        };

        // Create runtime
        let mut runtime = ProjectRuntime::new(config.uid.clone());

        // Initialize runtime with nodes
        runtime.init(&config, self.platform.output.as_ref())?;

        // Store config and runtime
        self.config = Some(config);
        self.runtime = Some(runtime);

        Ok(())
    }

    /// Process incoming messages and update runtime
    ///
    /// Processes all incoming messages, updates the runtime if loaded,
    /// and returns any outgoing messages.
    pub fn tick(
        &mut self,
        delta_ms: u32,
        incoming: &[crate::app::MsgIn],
    ) -> Result<alloc::vec::Vec<crate::app::MsgOut>, Error> {
        let mut outgoing = alloc::vec::Vec::new();

        // Process incoming messages
        for msg in incoming {
            match self.handle_message(msg.clone()) {
                Ok(mut msgs) => outgoing.append(&mut msgs),
                Err(e) => {
                    // Log error but continue processing
                    let _ = e;
                }
            }
        }

        // Update runtime if loaded
        if let Some(ref mut runtime) = self.runtime {
            let _ = runtime.update(delta_ms, self.platform.output.as_ref());
        }

        Ok(outgoing)
    }

    /// Handle a single incoming message
    fn handle_message(
        &mut self,
        msg: crate::app::MsgIn,
    ) -> Result<alloc::vec::Vec<crate::app::MsgOut>, Error> {
        let mut outgoing = alloc::vec::Vec::new();

        match msg {
            crate::app::MsgIn::UpdateProject { project } => {
                // Save project to filesystem
                let json = serde_json::to_string(&project)
                    .map_err(|e| Error::Serialization(format!("Failed to serialize project: {}", e)))?;
                self.platform.fs.write_file("project.json", json.as_bytes())?;

                // Load the project (this will initialize the runtime)
                self.load_project("project.json")?;
            }
            crate::app::MsgIn::GetProject => {
                // Get current project config
                if let Some(ref config) = self.config {
                    outgoing.push(crate::app::MsgOut::Project {
                        project: config.clone(),
                    });
                }
            }
            crate::app::MsgIn::Log { level, message } => {
                // Log message (for now, just ignore - firmware can handle logging)
                let _ = (level, message);
            }
        }

        Ok(outgoing)
    }

    /// Get a reference to the current project config
    pub fn config(&self) -> Option<&ProjectConfig> {
        self.config.as_ref()
    }

    /// Get a reference to the current project runtime
    pub fn runtime(&self) -> Option<&ProjectRuntime> {
        self.runtime.as_ref()
    }
}

