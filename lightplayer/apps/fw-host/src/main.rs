use eframe::egui;

fn main() -> eframe::Result<()> {
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

