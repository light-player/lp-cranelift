//! Main application logic

use lp_core::app::{LpApp, MsgIn, MsgOut, Platform};
use lp_core::error::Error;
use lp_core::project::{config::ProjectConfig, runtime::ProjectRuntime};
use lp_core::protocol::LogLevel;
use lp_core::traits::{Filesystem, LedOutput};
use std::sync::{Arc, Mutex};

use crate::output_provider::HostOutputProvider;

pub struct FwHostApp {
    lp_app: LpApp,
    led_output: Arc<Mutex<dyn LedOutput>>, // Keep for UI visualization
}

impl FwHostApp {
    pub fn new(filesystem: Box<dyn Filesystem>, led_output: Arc<Mutex<dyn LedOutput>>) -> Self {
        // Create HostOutputProvider
        let output_provider = Box::new(HostOutputProvider::new());

        // Create Platform
        let platform = Platform::new(filesystem, output_provider);

        // Create LpApp
        let lp_app = LpApp::new(platform);

        Self { lp_app, led_output }
    }

    /// Initialize the application
    pub fn init(&mut self) -> Result<(), Error> {
        // Try to load project.json on startup
        if self.lp_app.platform.fs.file_exists("project.json")? {
            match self.lp_app.load_project("project.json") {
                Ok(_) => {
                    self.log(LogLevel::Info, "Loaded project from project.json");
                }
                Err(e) => {
                    self.log(
                        LogLevel::Warn,
                        &format!("Failed to load project.json: {}", e),
                    );
                    // Create default project on load failure
                    self.create_default_project()?;
                }
            }
        } else {
            self.log(
                LogLevel::Info,
                "No project.json found, starting with template project",
            );
            self.create_default_project()?;
        }

        Ok(())
    }

    /// Create a default project with basic setup
    fn create_default_project(&mut self) -> Result<(), Error> {
        use hashbrown::HashMap;
        use lp_core::nodes::id::{OutputId, TextureId};
        use lp_core::nodes::{FixtureNode, Mapping, OutputNode, ShaderNode, TextureNode};
        use lp_core::project::config::{Nodes, ProjectConfig};

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

        // Save project to filesystem
        let json = serde_json::to_string_pretty(&project)
            .map_err(|e| Error::Serialization(format!("Failed to serialize project: {}", e)))?;
        self.lp_app
            .platform
            .fs
            .write_file("project.json", json.as_bytes())?;

        // Load the project via LpApp
        self.lp_app.load_project("project.json")?;
        self.log(LogLevel::Info, "Created default project");

        Ok(())
    }

    /// Load project from filesystem
    pub fn load_project(&mut self) -> Result<(), Error> {
        self.lp_app.load_project("project.json")
    }

    /// Save project to filesystem
    pub fn save_project(&self) -> Result<(), Error> {
        let project = self
            .lp_app
            .config()
            .ok_or_else(|| Error::Validation("No project to save".to_string()))?;

        let json = serde_json::to_string_pretty(project)
            .map_err(|e| Error::Serialization(format!("Failed to serialize project: {}", e)))?;
        self.lp_app
            .platform
            .fs
            .write_file("project.json", json.as_bytes())?;

        Ok(())
    }

    /// Handle a command
    pub fn handle_command(&mut self, command: Command) -> Result<(), Error> {
        // Convert Command to MsgIn
        let msg_in: MsgIn = command.into();

        // Process via LpApp
        let mut msgs = vec![msg_in];
        let outgoing = self.lp_app.tick(0, &msgs)?;

        // Handle outgoing messages
        for msg_out in outgoing {
            match msg_out {
                MsgOut::Project { project } => {
                    // Send project via transport
                    let json = serde_json::to_string(&project)
                        .map_err(|e| Error::Serialization(format!("{}", e)))?;
                    let mut transport = self.transport.lock().unwrap();
                    transport.send_message(&json)?;
                }
            }
        }

        Ok(())
    }

    /// Process incoming messages from transport and update runtime
    ///
    /// Reads messages from transport, converts them to MsgIn, calls LpApp::tick(),
    /// and handles outgoing messages. Also updates the runtime with the given delta_ms.
    pub fn tick(&mut self, delta_ms: u32) -> Result<(), Error> {
        // Collect incoming messages from transport
        let mut incoming_messages = Vec::new();

        // Try to receive messages (non-blocking, collect all available)
        loop {
            let message = {
                let mut transport = self.transport.lock().unwrap();
                transport.receive_message()
            };

            match message {
                Ok(msg) => match parse_command(&msg) {
                    Ok(command) => {
                        let msg_in: MsgIn = command.into();
                        incoming_messages.push(msg_in);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse command: {}", e);
                        break; // Stop on parse error
                    }
                },
                Err(e) => {
                    // "No message available" means no more messages
                    if !e.to_string().contains("No message available") {
                        eprintln!("Error receiving message: {}", e);
                    }
                    break;
                }
            }
        }

        // Call LpApp::tick() with delta_ms and messages
        let outgoing = self.lp_app.tick(delta_ms, &incoming_messages)?;

        // Handle outgoing messages
        for msg_out in outgoing {
            match msg_out {
                MsgOut::Project { project } => {
                    // Send project via transport
                    let json = serde_json::to_string(&project)
                        .map_err(|e| Error::Serialization(format!("{}", e)))?;
                    let mut transport = self.transport.lock().unwrap();
                    transport.send_message(&json)?;
                }
            }
        }

        Ok(())
    }

    /// Process incoming messages from transport (legacy method, kept for compatibility)
    pub fn process_messages(&mut self) -> Result<(), Error> {
        // Just call tick with 0 delta - this will process messages but not update runtime
        // In practice, this should be replaced with tick() calls
        self.tick(0)
    }

    /// Log a message
    pub fn log(&self, level: LogLevel, message: &str) {
        eprintln!("[{}] {}", format!("{:?}", level).to_lowercase(), message);
    }

    /// Get a reference to the project
    pub fn project(&self) -> Option<&ProjectConfig> {
        self.lp_app.config()
    }

    /// Get a reference to the runtime
    pub fn runtime(&self) -> Option<&ProjectRuntime> {
        self.lp_app.runtime()
    }
}
