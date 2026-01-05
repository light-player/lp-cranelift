mod app;
mod debug_ui;
mod filesystem;
mod led_output;
mod transport;

use app::LightPlayerApp as AppLogic;
use debug_ui::render_textures_panel;
use eframe::egui;
use filesystem::HostFilesystem;
use led_output::{render_leds, HostLedOutput};
use lp_core::traits::{Filesystem, LedOutput, Transport};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use transport::HostTransport;

fn main() -> eframe::Result<()> {
    // Initialize filesystem with current directory as base
    let fs: Box<dyn Filesystem> = Box::new(HostFilesystem::new(PathBuf::from(".")));
    // Initialize transport (stdio)
    let transport: Arc<Mutex<dyn Transport>> = Arc::new(Mutex::new(HostTransport::new()));
    // Initialize LED output (128 LEDs, RGB = 3 bytes per pixel)
    let led_output: Arc<Mutex<dyn LedOutput>> =
        Arc::new(Mutex::new(HostLedOutput::new(128, 3)));

    // Create application logic
    let mut app_logic = AppLogic::new(fs, Arc::clone(&transport), Arc::clone(&led_output));
    if let Err(e) = app_logic.init() {
        eprintln!("Failed to initialize application: {}", e);
    }

    // Get LED output for visualization (we need to clone the inner data)
    // For now, we'll create a separate instance for visualization
    let led_output_viz = HostLedOutput::new(128, 3);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("LightPlayer Host Firmware"),
        ..Default::default()
    };

    eframe::run_native(
        "LightPlayer Host",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(AppState {
                app_logic,
                led_output: led_output_viz,
            }))
        }),
    )
}

struct AppState {
    app_logic: AppLogic,
    led_output: HostLedOutput,
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process incoming messages (non-blocking)
        if let Err(e) = self.app_logic.process_messages() {
            eprintln!("Error processing messages: {}", e);
        }

        // Use a side panel for textures and main panel for LEDs
        egui::SidePanel::right("debug_panel")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Debug Panel");
                ui.separator();

                if let Some(project) = self.app_logic.project() {
                    // Show project info
                    ui.group(|ui| {
                        ui.label(format!("Project: {}", project.name));
                        ui.label(format!("UID: {}", project.uid));
                    });
                    ui.separator();

                    // Show textures
                    render_textures_panel(ui, project);
                } else {
                    ui.label("No project loaded");
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LightPlayer Host Firmware");
            ui.separator();

            // Show project info
            if let Some(project) = self.app_logic.project() {
                ui.label(format!("Project: {} ({})", project.name, project.uid));
            } else {
                ui.label("No project loaded");
            }

            ui.separator();
            ui.label("LED Visualization:");
            ui.separator();

            // Render LEDs
            render_leds(ui, &self.led_output);
        });
    }
}
