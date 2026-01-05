//! Main application logic

use lp_core::error::Error;
use lp_core::protocol::{parse_command, Command, LogLevel};
use lp_core::project::{config::ProjectConfig, runtime::ProjectRuntime};
use lp_core::traits::{Filesystem, LedOutput, Transport};
use std::sync::{Arc, Mutex};

pub struct LightPlayerApp {
    filesystem: Box<dyn Filesystem>,
    transport: Arc<Mutex<dyn Transport>>,
    led_output: Arc<Mutex<dyn LedOutput>>,
    project: Option<ProjectConfig>,
    runtime: Option<ProjectRuntime>,
}

impl LightPlayerApp {
    pub fn new(
        filesystem: Box<dyn Filesystem>,
        transport: Arc<Mutex<dyn Transport>>,
        led_output: Arc<Mutex<dyn LedOutput>>,
    ) -> Self {
        Self {
            filesystem,
            transport,
            led_output,
            project: None,
            runtime: None,
        }
    }

    /// Initialize the application
    pub fn init(&mut self) -> Result<(), Error> {
        // Try to load project.json on startup
        if self.filesystem.file_exists("project.json")? {
            match self.load_project() {
                Ok(_) => {
                    self.log(LogLevel::Info, "Loaded project from project.json");
                }
                Err(e) => {
                    self.log(
                        LogLevel::Warn,
                        &format!("Failed to load project.json: {}", e),
                    );
                    // Create default project on load failure
                    self.create_default_project();
                }
            }
        } else {
            self.log(LogLevel::Info, "No project.json found, starting with empty project");
            self.create_default_project();
        }

        Ok(())
    }

    /// Create a default project with basic setup
    fn create_default_project(&mut self) {
        use hashbrown::HashMap;
        use lp_core::nodes::{
            FixtureNode, Mapping, OutputNode, ShaderNode, TextureNode,
        };
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

        // Add shader: simple gray shader
        project.nodes.shaders.insert(
            3,
            ShaderNode::Single {
                glsl: r#"
vec4 main(in vec2 fragCoord) {
    return vec4(0.5, 0.5, 0.5, 1.0);
}
"#
                .to_string(),
                texture_id: 2,
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
                output_id: 1,
                channel_order: "rgb".to_string(),
                mapping,
            },
        );

        self.project = Some(project.clone());
        self.runtime = Some(ProjectRuntime::new(project.uid.clone()));
        self.log(LogLevel::Info, "Created default project");
    }

    /// Load project from filesystem
    pub fn load_project(&mut self) -> Result<(), Error> {
        let data = self.filesystem.read_file("project.json")?;
        let json = String::from_utf8(data)
            .map_err(|e| Error::Filesystem(format!("Invalid UTF-8 in project.json: {}", e)))?;
        let project: ProjectConfig = serde_json::from_str(&json)
            .map_err(|e| Error::Validation(format!("Invalid project.json: {}", e)))?;

        self.project = Some(project.clone());
        // Initialize runtime with project UID
        self.runtime = Some(ProjectRuntime::new(project.uid.clone()));

        Ok(())
    }

    /// Save project to filesystem
    pub fn save_project(&self) -> Result<(), Error> {
        let project = self
            .project
            .as_ref()
            .ok_or_else(|| Error::Validation("No project to save".to_string()))?;

        let json = serde_json::to_string_pretty(project)
            .map_err(|e| Error::Serialization(format!("Failed to serialize project: {}", e)))?;
        self.filesystem.write_file("project.json", json.as_bytes())?;

        Ok(())
    }

    /// Handle a command
    pub fn handle_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::UpdateProject { project } => {
                self.project = Some(project.clone());
                // Initialize runtime
                self.runtime = Some(ProjectRuntime::new(project.uid.clone()));
                // Save to filesystem
                self.save_project()?;
                self.log(LogLevel::Info, "Project updated and saved");
            }
            Command::GetProject => {
                if let Some(ref project) = self.project {
                    let json = serde_json::to_string(project)
                        .map_err(|e| Error::Serialization(format!("{}", e)))?;
                    let mut transport = self.transport.lock().unwrap();
                    transport.send_message(&json)?;
                } else {
                    self.log(LogLevel::Warn, "No project loaded");
                }
            }
            Command::Log { level, message } => {
                // Log messages are handled by the sender, but we can echo them
                eprintln!("[{}] {}", format!("{:?}", level).to_lowercase(), message);
            }
        }
        Ok(())
    }

    /// Process incoming messages from transport
    pub fn process_messages(&mut self) -> Result<(), Error> {
        // Try to receive a message (non-blocking check)
        // Note: In a real implementation, this would be async or use a separate thread
        // For now, we'll handle this in the egui update loop
        let message = {
            let mut transport = self.transport.lock().unwrap();
            transport.receive_message()
        };

        match message {
            Ok(msg) => {
                match parse_command(&msg) {
                    Ok(command) => {
                        self.handle_command(command)?;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse command: {}", e);
                    }
                }
            }
            Err(e) => {
                // In a real implementation, we'd handle this more gracefully
                // For now, just log the error
                eprintln!("Error receiving message: {}", e);
            }
        }
        Ok(())
    }

    /// Log a message
    pub fn log(&self, level: LogLevel, message: &str) {
        eprintln!("[{}] {}", format!("{:?}", level).to_lowercase(), message);
    }

    /// Get a reference to the project
    pub fn project(&self) -> Option<&ProjectConfig> {
        self.project.as_ref()
    }

    /// Get a reference to the runtime
    pub fn runtime(&self) -> Option<&ProjectRuntime> {
        self.runtime.as_ref()
    }
}

