//! Host LED output implementation with egui visualization

use lp_core::error::Error;
use lp_core::traits::LedOutput;
use std::sync::{Arc, Mutex};

/// Host LED output implementation that stores pixels and visualizes them in egui
pub struct HostLedOutput {
    pixels: Arc<Mutex<Vec<u8>>>,
    pixel_count: usize,
    bytes_per_pixel: usize,
}

impl HostLedOutput {
    /// Create a new host LED output
    ///
    /// `pixel_count` is the number of LEDs
    /// `bytes_per_pixel` is typically 3 (RGB) or 4 (RGBA)
    pub fn new(pixel_count: usize, bytes_per_pixel: usize) -> Self {
        Self {
            pixels: Arc::new(Mutex::new(vec![0; pixel_count * bytes_per_pixel])),
            pixel_count,
            bytes_per_pixel,
        }
    }

    /// Get a reference to the pixel data for rendering
    pub fn get_pixels(&self) -> Arc<Mutex<Vec<u8>>> {
        Arc::clone(&self.pixels)
    }

    /// Get bytes per pixel
    pub fn bytes_per_pixel(&self) -> usize {
        self.bytes_per_pixel
    }
}

impl LedOutput for HostLedOutput {
    fn write_pixels(&mut self, pixels: &[u8]) -> Result<(), Error> {
        let expected_len = self.pixel_count * self.bytes_per_pixel;
        if pixels.len() != expected_len {
            return Err(Error::Node(format!(
                "Invalid pixel data length: expected {} bytes, got {}",
                expected_len,
                pixels.len()
            )));
        }

        let mut pixel_data = self.pixels.lock().unwrap();
        pixel_data.copy_from_slice(pixels);
        Ok(())
    }

    fn get_pixel_count(&self) -> usize {
        self.pixel_count
    }
}

/// Helper to render LEDs in egui with enhanced visualization
pub fn render_leds(
    ui: &mut egui::Ui,
    led_output: &HostLedOutput,
    selected_led: Option<usize>,
) -> Option<usize> {
    let pixels = led_output.get_pixels();
    let pixel_data = pixels.lock().unwrap();
    let bytes_per_pixel = led_output.bytes_per_pixel();
    let pixel_count = led_output.get_pixel_count();

    // Calculate layout: try to make a roughly square grid
    let cols = (pixel_count as f32).sqrt().ceil() as usize;
    let rows = (pixel_count + cols - 1) / cols; // Ceiling division

    // Size of each LED circle (adjustable)
    let led_size = 25.0;
    let spacing = 8.0;

    // Calculate total size needed
    let total_width = (cols as f32) * (led_size + spacing) - spacing;
    let total_height = (rows as f32) * (led_size + spacing) - spacing;

    // Use scroll area for zoom/pan
    let mut clicked_led = None;
    egui::ScrollArea::both()
        .show(ui, |ui| {
            // Center the grid
            let available_size = ui.available_size();
            let start_x = (available_size.x - total_width) / 2.0;
            let start_y = (available_size.y - total_height) / 2.0;

            let mut y = start_y;
            for row in 0..rows {
                let mut x = start_x;
                for col in 0..cols {
                    let idx = row * cols + col;
                    if idx >= pixel_count {
                        break;
                    }

                    // Get pixel color
                    let pixel_start = idx * bytes_per_pixel;
                    let r = pixel_data[pixel_start];
                    let g = if bytes_per_pixel > 1 {
                        pixel_data[pixel_start + 1]
                    } else {
                        0
                    };
                    let b = if bytes_per_pixel > 2 {
                        pixel_data[pixel_start + 2]
                    } else {
                        0
                    };

                    let color = egui::Color32::from_rgb(r, g, b);
                    let center = egui::pos2(x + led_size / 2.0, y + led_size / 2.0);
                    let radius = led_size / 2.0;

                    // Create interactive area for this LED
                    let response = ui.allocate_rect(
                        egui::Rect::from_center_size(center, egui::Vec2::new(led_size, led_size)),
                        egui::Sense::click(),
                    );

                    // Check if clicked
                    if response.clicked() {
                        clicked_led = Some(idx);
                    }

                    // Check if hovered or selected
                    let is_selected = selected_led == Some(idx);
                    let is_hovered = response.hovered();

                    // Draw LED circle with highlight if selected/hovered
                    let stroke_width = if is_selected { 3.0 } else if is_hovered { 2.0 } else { 1.0 };
                    let stroke_color = if is_selected {
                        egui::Color32::YELLOW
                    } else if is_hovered {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_gray(100)
                    };

                    ui.painter().circle_filled(center, radius, color);
                    ui.painter().circle_stroke(center, radius, egui::Stroke::new(stroke_width, stroke_color));

                    // Draw LED index label (small, inside circle if space allows)
                    if led_size > 20.0 {
                        let label = format!("{}", idx);
                        let font_size = (led_size * 0.3).min(10.0);
                        let text_color = if (r as u16 + g as u16 + b as u16) > 384 {
                            egui::Color32::BLACK // Dark text on light background
                        } else {
                            egui::Color32::WHITE // Light text on dark background
                        };
                        ui.painter().text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::monospace(font_size),
                            text_color,
                        );
                    }

                    // Show tooltip with LED details
                    if is_hovered || is_selected {
                        response.on_hover_ui(|ui| {
                            ui.set_max_width(200.0);
                            ui.label(format!("LED #{}", idx));
                            ui.separator();
                            ui.label(format!("RGB: ({}, {}, {})", r, g, b));
                            ui.label(format!("Hex: #{:02X}{:02X}{:02X}", r, g, b));
                            if bytes_per_pixel > 3 {
                                let a = pixel_data[pixel_start + 3];
                                ui.label(format!("Alpha: {}", a));
                            }
                        });
                    }

                    x += led_size + spacing;
                }
                y += led_size + spacing;
            }
        });

    clicked_led
}

