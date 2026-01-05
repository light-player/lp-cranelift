mod filesystem;
mod transport;

use eframe::egui;
use filesystem::HostFilesystem;
use std::path::PathBuf;
use transport::HostTransport;

fn main() -> eframe::Result<()> {
    // Initialize filesystem with current directory as base
    let _fs = HostFilesystem::new(PathBuf::from("."));
    // Initialize transport (stdio)
    let _transport = HostTransport::new();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("LightPlayer Host Firmware"),
        ..Default::default()
    };

    eframe::run_native(
        "LightPlayer Host",
        options,
        Box::new(|_cc| Ok(Box::new(LightPlayerApp::default()))),
    )
}

struct LightPlayerApp {
    // App state will be added in later phases
}

impl Default for LightPlayerApp {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for LightPlayerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LightPlayer Host Firmware");
            ui.separator();
            ui.label("Host firmware simulator for LightPlayer");
            ui.label("This will be used for development and testing.");
        });
    }
}

