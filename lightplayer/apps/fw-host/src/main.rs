mod debug_ui;
mod filesystem;
mod led_output;
mod output_provider;
mod transport;

use debug_ui::{render_fixtures_panel, render_shaders_panel, render_textures_panel};
use eframe::egui;
use filesystem::HostFilesystem;
use led_output::render_leds;
use lp_core::app::{LpApp, MsgIn, MsgOut, Platform};
use lp_core::error::Error;
use lp_core::protocol::message::parse_command;
use lp_core::traits::{Filesystem, Transport};
use output_provider::HostOutputProvider;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use transport::HostTransport;

/// Wrapper around HostOutputProvider to use it as a trait object
struct HostOutputProviderWrapper {
    inner: Arc<HostOutputProvider>,
}

impl lp_core::traits::OutputProvider for HostOutputProviderWrapper {
    fn create_output(
        &self,
        config: &lp_core::nodes::output::config::OutputNode,
        output_id: Option<lp_core::nodes::id::OutputId>,
    ) -> Result<std::boxed::Box<dyn lp_core::traits::LedOutput>, lp_core::error::Error> {
        self.inner.create_output(config, output_id)
    }
}

/// Collect messages from transport and convert to MsgIn
fn collect_messages(transport: &Arc<Mutex<dyn Transport>>) -> Vec<MsgIn> {
    let mut messages = Vec::new();

    // Try to receive messages (non-blocking, collect all available)
    loop {
        let message = {
            let mut transport = transport.lock().unwrap();
            transport.receive_message()
        };

        match message {
            Ok(msg) => match parse_command(&msg) {
                Ok(command) => {
                    let msg_in: MsgIn = command.into();
                    messages.push(msg_in);
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

    messages
}

/// Handle outgoing messages from LpApp
fn handle_outgoing_messages(
    messages: Vec<MsgOut>,
    transport: &Arc<Mutex<dyn Transport>>,
) -> Result<(), Error> {
    for msg_out in messages {
        match msg_out {
            MsgOut::Project { project } => {
                // Send project via transport
                let json = serde_json::to_string(&project)
                    .map_err(|e| Error::Serialization(format!("{}", e)))?;
                let mut transport = transport.lock().unwrap();
                transport.send_message(&json)?;
            }
        }
    }
    Ok(())
}

fn main() -> eframe::Result<()> {
    // Initialize filesystem with current directory as base
    let fs: Box<dyn Filesystem> = Box::new(HostFilesystem::new(PathBuf::from(".")));
    // Initialize output provider (store separately for UI access)
    let output_provider = Arc::new(HostOutputProvider::new());
    // Create Platform (wrap in a boxed trait object)
    let platform_output_provider: Box<dyn lp_core::traits::OutputProvider> =
        Box::new(HostOutputProviderWrapper {
            inner: Arc::clone(&output_provider),
        });
    let platform = Platform::new(fs, platform_output_provider);
    // Create LpApp
    let mut lp_app = LpApp::new(platform);

    // Load project (will create default if not found)
    // Wrap in catch_unwind to handle panics from cranelift (e.g., unimplemented features on macOS)
    let load_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        lp_app.load_project("project.json")
    }));

    match load_result {
        Ok(Ok(())) => {
            // Success
        }
        Ok(Err(e)) => {
            eprintln!("Failed to load project: {}", e);
        }
        Err(_) => {
            eprintln!(
                "Project loading panicked (possibly due to unimplemented platform features in cranelift)"
            );
            eprintln!("This is a known issue on macOS - shader compilation may not work");
            // Continue anyway - the app will show errors in the debug UI
        }
    }

    // Initialize transport (stdio)
    let transport: Arc<Mutex<dyn Transport>> = Arc::new(Mutex::new(HostTransport::new()));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("LightPlayer Host Firmware"),
        ..Default::default()
    };

    eframe::run_native(
        "LightPlayer Host",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(AppState {
                lp_app,
                transport,
                output_provider,
                selected_led: None,
                last_frame_time: None,
                frame_count: 0,
                fps_history: Vec::new(),
            }))
        }),
    )
}

struct AppState {
    lp_app: LpApp,
    transport: Arc<Mutex<dyn Transport>>,
    output_provider: Arc<HostOutputProvider>,
    selected_led: Option<usize>,
    last_frame_time: Option<Instant>,
    frame_count: u64,
    fps_history: Vec<f32>, // Store last N frame times for average FPS
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Calculate delta_ms from frame timestamps
        let now = Instant::now();
        let delta_ms = if let Some(last_time) = self.last_frame_time {
            let delta = now.duration_since(last_time);
            delta.as_millis().min(u32::MAX as u128) as u32
        } else {
            0 // First frame, no delta
        };
        self.last_frame_time = Some(now);

        // Update frame count
        self.frame_count += 1;

        // Calculate FPS (frames per second) and update history
        let current_fps = if delta_ms > 0 {
            1000.0 / delta_ms as f32
        } else {
            0.0
        };

        // Update FPS history (keep last 60 frames for average)
        self.fps_history.push(current_fps);
        if self.fps_history.len() > 60 {
            self.fps_history.remove(0);
        }

        // Collect messages from transport
        let incoming_messages = collect_messages(&self.transport);

        // Update runtime with tick() - processes messages and updates runtime
        match self.lp_app.tick(delta_ms, &incoming_messages, &[]) {
            Ok(outgoing) => {
                // Handle outgoing messages
                if let Err(e) = handle_outgoing_messages(outgoing, &self.transport) {
                    eprintln!("Error handling outgoing messages: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error in tick: {}", e);
            }
        }

        // Request repaint to keep loop running continuously
        // Use request_repaint_after to ensure we get regular updates even if nothing changes
        ctx.request_repaint_after(std::time::Duration::from_millis(16)); // ~60 FPS

        // Use a side panel for textures and main panel for LEDs
        egui::SidePanel::right("debug_panel")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Debug Panel");
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if let Some(project) = self.lp_app.config() {
                            // Show project info
                            ui.group(|ui| {
                                ui.label(format!("Project: {}", project.name));
                                ui.label(format!("UID: {}", project.uid));
                            });
                            ui.separator();

                            // Show textures
                            render_textures_panel(ui, project, self.lp_app.runtime());

                            ui.separator();

                            // Show shaders
                            render_shaders_panel(ui, project, self.lp_app.runtime());

                            ui.separator();

                            // Show fixtures
                            render_fixtures_panel(ui, project, self.lp_app.runtime());
                        } else {
                            ui.label("No project loaded");
                        }
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LightPlayer Host Firmware");
            ui.separator();

            // Frame statistics
            ui.group(|ui| {
                ui.heading("Frame Statistics");

                let current_fps = self.fps_history.last().copied().unwrap_or(0.0);
                let avg_fps = if !self.fps_history.is_empty() {
                    self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32
                } else {
                    0.0
                };

                ui.label(format!("Frame: {}", self.frame_count));
                ui.label(format!("Current FPS: {:.1}", current_fps));
                ui.label(format!("Average FPS: {:.1}", avg_fps));

                if let Some(runtime) = self.lp_app.runtime() {
                    let frame_time = runtime.frame_time();
                    ui.label(format!(
                        "Total Time: {:.2}s",
                        frame_time.total_ms as f32 / 1000.0
                    ));
                    ui.label(format!("Delta: {}ms", frame_time.delta_ms));
                } else {
                    ui.label("Runtime not initialized");
                }
            });
            ui.separator();

            // Show project info
            if let Some(project) = self.lp_app.config() {
                ui.label(format!("Project: {} ({})", project.name, project.uid));
            } else {
                ui.label("No project loaded");
            }

            ui.separator();

            // Show outputs - one section per output
            let outputs = self.output_provider.get_all_outputs();
            if outputs.is_empty() {
                ui.label("No outputs configured");
            } else {
                for (output_id, output_arc) in &outputs {
                    {
                        let output = output_arc.lock().unwrap();
                        let pixel_count = lp_core::traits::LedOutput::get_pixel_count(&*output);
                        ui.heading(format!(
                            "Output {} ({} LEDs)",
                            u32::from(*output_id),
                            pixel_count
                        ));
                    }
                    ui.separator();

                    // Render LEDs for this output in a contained panel
                    let output = output_arc.lock().unwrap();
                    let pixel_count = lp_core::traits::LedOutput::get_pixel_count(&*output);

                    // Calculate size needed for LED grid
                    let cols = (pixel_count as f32).sqrt().ceil() as usize;
                    let led_size = 25.0;
                    let spacing = 8.0;
                    let estimated_width = (cols as f32) * (led_size + spacing) - spacing;
                    let estimated_height =
                        ((pixel_count + cols - 1) / cols) as f32 * (led_size + spacing) - spacing;

                    // Create contained area with clipping
                    let available_width = ui.available_width();
                    let max_height = 400.0;
                    let allocated_size = egui::Vec2::new(
                        available_width.min(estimated_width + 20.0),
                        estimated_height.min(max_height) + 20.0,
                    );

                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.set_max_size(allocated_size);
                            ui.set_clip_rect(ui.max_rect());
                            render_leds(ui, &*output, self.selected_led);
                        });
                    drop(output);

                    ui.separator();
                }
            }
        });
    }
}
