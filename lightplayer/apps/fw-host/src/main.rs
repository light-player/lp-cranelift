mod filesystem;
mod led_output;
mod transport;

use eframe::egui;
use filesystem::HostFilesystem;
use led_output::{render_leds, HostLedOutput};
use std::path::PathBuf;
use transport::HostTransport;

fn main() -> eframe::Result<()> {
    // Initialize filesystem with current directory as base
    let _fs = HostFilesystem::new(PathBuf::from("."));
    // Initialize transport (stdio)
    let _transport = HostTransport::new();
    // Initialize LED output (128 LEDs, RGB = 3 bytes per pixel)
    let led_output = HostLedOutput::new(128, 3);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("LightPlayer Host Firmware"),
        ..Default::default()
    };

    eframe::run_native(
        "LightPlayer Host",
        options,
        Box::new(move |_cc| Ok(Box::new(LightPlayerApp::new(led_output)))),
    )
}

struct LightPlayerApp {
    led_output: HostLedOutput,
}

impl LightPlayerApp {
    fn new(led_output: HostLedOutput) -> Self {
        Self { led_output }
    }
}

impl eframe::App for LightPlayerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LightPlayer Host Firmware");
            ui.separator();
            ui.label("Host firmware simulator for LightPlayer");
            ui.label("LED Visualization:");
            ui.separator();

            // Render LEDs
            render_leds(ui, &self.led_output);
        });
    }
}

