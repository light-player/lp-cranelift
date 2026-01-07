//! LightPlayer Application - main entry point for firmware

use crate::app::Platform;
use crate::error::Error;
use crate::project::{config::ProjectConfig, runtime::ProjectRuntime};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

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
    /// Creates a project with only top-level metadata (uid and name).
    /// Nodes will be loaded from the filesystem separately via ProjectLoader.
    pub fn create_default_project() -> ProjectConfig {
        ProjectConfig {
            uid: "default".to_string(),
            name: "Default Project".to_string(),
        }
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
        file_changes: &[crate::app::FileChange],
    ) -> Result<alloc::vec::Vec<crate::app::MsgOut>, Error> {
        let mut outgoing = alloc::vec::Vec::new();

        // Process file changes (placeholder - will be implemented in Phase 9)
        for change in file_changes {
            // TODO: Implement file change handling in Phase 9
            let _ = change;
        }

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
                let json = serde_json::to_string(&project).map_err(|e| {
                    Error::Serialization(format!("Failed to serialize project: {}", e))
                })?;
                self.platform
                    .fs
                    .write_file("project.json", json.as_bytes())?;

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
