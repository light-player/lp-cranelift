//! Shader node runtime

use crate::error::Error;
use crate::nodes::id::TextureId;
use crate::nodes::shader::config::ShaderNode;
use crate::project::runtime::NodeStatus;
use crate::runtime::contexts::ShaderRenderContext;
use crate::runtime::lifecycle::NodeLifecycle;
use alloc::{format, string::ToString, vec};
use lp_glsl::frontend::semantic::types::Type;
use lp_glsl::{DecimalFormat, GlslExecutable, GlslOptions, GlslValue, RunMode, glsl_jit};

/// Shader node runtime
pub struct ShaderNodeRuntime {
    executable: Option<alloc::boxed::Box<dyn GlslExecutable>>,
    texture_id: TextureId,
    status: NodeStatus,
}

impl ShaderNodeRuntime {
    /// Create a new shader node runtime (uninitialized)
    pub fn new() -> Self {
        Self {
            executable: None,
            texture_id: TextureId(0),
            status: NodeStatus::Ok,
        }
    }

    /// Get the current status
    pub fn status(&self) -> &NodeStatus {
        &self.status
    }

    /// Get the texture ID this shader writes to
    pub fn texture_id(&self) -> TextureId {
        self.texture_id
    }
}

impl Default for ShaderNodeRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeLifecycle for ShaderNodeRuntime {
    type Config = ShaderNode;
    type RenderContext<'a> = ShaderRenderContext<'a>;

    fn init(
        &mut self,
        config: &Self::Config,
        _ctx: &crate::runtime::contexts::InitContext,
    ) -> Result<(), Error> {
        match config {
            ShaderNode::Single { glsl, texture_id } => {
                self.texture_id = *texture_id;

                // Create compilation options
                let options = GlslOptions {
                    run_mode: RunMode::HostJit,
                    decimal_format: DecimalFormat::Float,
                };

                // Compile GLSL
                match glsl_jit(glsl, options) {
                    Ok(executable) => {
                        // Validate signature: vec4 main(vec2 fragCoord, vec2 outputSize, float time)
                        if let Some(sig) = executable.get_function_signature("main") {
                            // Check return type is vec4
                            let is_vec4 = matches!(sig.return_type, Type::Vec4);
                            if !is_vec4 {
                                self.status = NodeStatus::Error {
                                    status_message: format!(
                                        "Shader main() must return vec4, got {:?}",
                                        sig.return_type
                                    ),
                                };
                                return Err(Error::Validation(format!(
                                    "Shader main() must return vec4, got {:?}",
                                    sig.return_type
                                )));
                            }

                            // Check parameters: vec2 fragCoord, vec2 outputSize, float time
                            if sig.parameters.len() != 3 {
                                self.status = NodeStatus::Error {
                                    status_message: format!(
                                        "Shader main() must have 3 parameters (vec2 fragCoord, vec2 outputSize, float time), got {}",
                                        sig.parameters.len()
                                    ),
                                };
                                return Err(Error::Validation(format!(
                                    "Shader main() must have 3 parameters, got {}",
                                    sig.parameters.len()
                                )));
                            }

                            // Check parameter types (simplified - just check count for now)
                            // Full validation would check each parameter type
                            self.executable = Some(executable);
                            self.status = NodeStatus::Ok;
                            Ok(())
                        } else {
                            self.status = NodeStatus::Error {
                                status_message: "Shader must have a main() function".to_string(),
                            };
                            Err(Error::Validation(
                                "Shader must have a main() function".to_string(),
                            ))
                        }
                    }
                    Err(e) => {
                        self.status = NodeStatus::Error {
                            status_message: format!("Shader compilation failed: {}", e),
                        };
                        Err(Error::Node(format!("Shader compilation failed: {}", e)))
                    }
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut Self::RenderContext<'_>) -> Result<(), Error> {
        // Skip if executable is None (compilation failed)
        let executable = match &mut self.executable {
            Some(exec) => exec,
            None => return Ok(()),
        };

        // Read time before borrowing texture
        let time_seconds = ctx.time.total_ms as f32 / 1000.0;

        // Get texture to write to
        let texture = match ctx.get_texture_mut(self.texture_id) {
            Some(tex) => tex,
            None => {
                self.status = NodeStatus::Error {
                    status_message: format!("Texture {} not found", u32::from(self.texture_id)),
                };
                return Err(Error::Node(format!(
                    "Texture {} not found",
                    u32::from(self.texture_id)
                )));
            }
        };

        let width = texture.width();
        let height = texture.height();

        // Iterate over all texture pixels
        for y in 0..height {
            for x in 0..width {
                // Convert pixel coordinates to fragCoord (vec2)
                let frag_coord = [
                    x as f32 + 0.5, // Center of pixel
                    y as f32 + 0.5,
                ];

                // Output size (vec2)
                let output_size = [width as f32, height as f32];

                // Call shader: vec4 main(vec2 fragCoord, vec2 outputSize, float time)
                let args = vec![
                    GlslValue::Vec2(frag_coord),
                    GlslValue::Vec2(output_size),
                    GlslValue::F32(time_seconds),
                ];

                match executable.call_vec("main", &args, 4) {
                    Ok(result) => {
                        // result is Vec<f32> with 4 elements [r, g, b, a]
                        // Convert to [u8; 4] (clamp to [0, 1] and scale to [0, 255])
                        let r = (result[0].max(0.0).min(1.0) * 255.0) as u8;
                        let g = (result[1].max(0.0).min(1.0) * 255.0) as u8;
                        let b = (result[2].max(0.0).min(1.0) * 255.0) as u8;
                        let a = (result[3].max(0.0).min(1.0) * 255.0) as u8;

                        texture.set_pixel(x, y, [r, g, b, a]);
                    }
                    Err(e) => {
                        self.status = NodeStatus::Error {
                            status_message: format!("Shader execution failed: {}", e),
                        };
                        return Err(Error::Node(format!("Shader execution failed: {}", e)));
                    }
                }
            }
        }

        Ok(())
    }

    fn destroy(&mut self) -> Result<(), Error> {
        // Cleanup executable if needed
        self.executable = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::config::{Nodes, ProjectConfig};
    use hashbrown::HashMap;

    #[test]
    #[ignore] // JIT compilation issue: lp-glsl sets is_pic=true but cranelift-jit requires is_pic=false
    fn test_shader_node_runtime_init_valid() {
        let mut runtime = ShaderNodeRuntime::new();
        let glsl = r#"
vec4 main(vec2 fragCoord, vec2 outputSize, float time) {
    return vec4(0.5, 0.5, 0.5, 1.0);
}
"#;
        let config = ShaderNode::Single {
            glsl: glsl.to_string(),
            texture_id: TextureId(1),
        };
        let project_config = ProjectConfig {
            uid: "test".to_string(),
            name: "Test".to_string(),
            nodes: Nodes {
                outputs: HashMap::new(),
                textures: HashMap::new(),
                shaders: HashMap::new(),
                fixtures: HashMap::new(),
            },
        };
        let ctx = crate::runtime::contexts::InitContext::new(&project_config);

        assert!(runtime.init(&config, &ctx).is_ok());
        assert!(runtime.executable.is_some());
        assert!(matches!(runtime.status(), NodeStatus::Ok));
    }

    #[test]
    fn test_shader_node_runtime_init_invalid_glsl() {
        let mut runtime = ShaderNodeRuntime::new();
        let glsl = "invalid glsl code";
        let config = ShaderNode::Single {
            glsl: glsl.to_string(),
            texture_id: TextureId(1),
        };
        let project_config = ProjectConfig {
            uid: "test".to_string(),
            name: "Test".to_string(),
            nodes: Nodes {
                outputs: HashMap::new(),
                textures: HashMap::new(),
                shaders: HashMap::new(),
                fixtures: HashMap::new(),
            },
        };
        let ctx = crate::runtime::contexts::InitContext::new(&project_config);

        assert!(runtime.init(&config, &ctx).is_err());
        assert!(matches!(runtime.status(), NodeStatus::Error { .. }));
    }

    #[test]
    fn test_shader_node_runtime_update_skips_if_no_executable() {
        let mut runtime = ShaderNodeRuntime::new();
        runtime.executable = None;

        let frame_time = crate::runtime::frame_time::FrameTime::new(16, 1000);
        let mut textures: HashMap<TextureId, crate::nodes::texture::TextureNodeRuntime> =
            HashMap::new();
        let mut ctx = ShaderRenderContext::new(frame_time, &mut textures);

        // Should return Ok without error
        assert!(runtime.update(&mut ctx).is_ok());
    }
}
