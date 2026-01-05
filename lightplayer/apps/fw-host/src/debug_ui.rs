//! Debug UI for visualizing textures, mappings, and LEDs

use egui::{Color32, ColorImage, Image, TextureHandle, Ui};
use lp_core::nodes::texture::{formats, TextureNode};
use lp_core::project::config::ProjectConfig;

/// Generate placeholder texture data for visualization
/// In the future, this will use actual shader-rendered data
fn generate_placeholder_texture(
    width: u32,
    height: u32,
    format: &str,
) -> Vec<u8> {
    let bytes_per_pixel = formats::bytes_per_pixel(format).unwrap_or(3);
    let mut data = Vec::with_capacity((width * height * bytes_per_pixel as u32) as usize);

    for y in 0..height {
        for x in 0..width {
            match format {
                formats::RGB8 => {
                    // Create a simple gradient pattern
                    let r = ((x as f32 / width as f32) * 255.0) as u8;
                    let g = ((y as f32 / height as f32) * 255.0) as u8;
                    let b = 128u8;
                    data.push(r);
                    data.push(g);
                    data.push(b);
                }
                formats::RGBA8 => {
                    // Create a simple gradient pattern with alpha
                    let r = ((x as f32 / width as f32) * 255.0) as u8;
                    let g = ((y as f32 / height as f32) * 255.0) as u8;
                    let b = 128u8;
                    let a = 255u8;
                    data.push(r);
                    data.push(g);
                    data.push(b);
                    data.push(a);
                }
                formats::R8 => {
                    // Grayscale gradient
                    let gray = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;
                    data.push(gray);
                }
                _ => {
                    // Default: fill with zeros
                    for _ in 0..bytes_per_pixel {
                        data.push(0);
                    }
                }
            }
        }
    }

    data
}

/// Convert texture data to egui ColorImage
fn texture_data_to_color_image(
    data: &[u8],
    width: u32,
    height: u32,
    format: &str,
) -> ColorImage {
    let mut pixels = Vec::with_capacity((width * height) as usize);

    let bytes_per_pixel = formats::bytes_per_pixel(format).unwrap_or(3);

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * bytes_per_pixel as u32) as usize;
            if idx + bytes_per_pixel <= data.len() {
                let color = match format {
                    formats::RGB8 => {
                        let r = data[idx];
                        let g = data[idx + 1];
                        let b = data[idx + 2];
                        Color32::from_rgb(r, g, b)
                    }
                    formats::RGBA8 => {
                        let r = data[idx];
                        let g = data[idx + 1];
                        let b = data[idx + 2];
                        let a = data[idx + 3];
                        Color32::from_rgba_unmultiplied(r, g, b, a)
                    }
                    formats::R8 => {
                        let gray = data[idx];
                        Color32::from_gray(gray)
                    }
                    _ => Color32::BLACK,
                };
                pixels.push(color);
            } else {
                pixels.push(Color32::BLACK);
            }
        }
    }

    ColorImage {
        size: [width as usize, height as usize],
        pixels,
    }
}

/// Render texture visualization in egui
pub fn render_texture(
    ui: &mut Ui,
    texture_id: u32,
    texture: &TextureNode,
    texture_data: Option<&[u8]>,
) {
    match texture {
        TextureNode::Memory { size, format } => {
            let [width, height] = *size;

            // Get texture data (placeholder for now if not provided)
            let data = if let Some(data) = texture_data {
                data.to_vec()
            } else {
                generate_placeholder_texture(width, height, format)
            };

            // Convert to egui image
            let color_image = texture_data_to_color_image(&data, width, height, format);

            // Create texture handle (cached per frame)
            let texture_handle: TextureHandle = ui.ctx().load_texture(
                format!("texture_{}", texture_id),
                color_image,
                Default::default(),
            );

            // Display metadata
            ui.group(|ui| {
                ui.label(format!("Texture ID: {}", texture_id));
                ui.label(format!("Size: {}x{}", width, height));
                ui.label(format!("Format: {}", format));
                ui.label(format!(
                    "Bytes per pixel: {}",
                    formats::bytes_per_pixel(format).unwrap_or(0)
                ));
            });

            ui.separator();

            // Display texture image
            // Scale to fit available width, maintaining aspect ratio
            let available_width = ui.available_width();
            let scale = (available_width / width as f32).min(1.0);
            let display_width = width as f32 * scale;
            let display_height = height as f32 * scale;

            ui.add(Image::new(&texture_handle).max_size(egui::Vec2::new(display_width, display_height)));
        }
    }
}

/// Render all textures in a debug panel
pub fn render_textures_panel(ui: &mut Ui, project: &ProjectConfig) {
    ui.heading("Textures");
    ui.separator();

    if project.nodes.textures.is_empty() {
        ui.label("No textures defined");
        return;
    }

    // Display each texture
    for (id, texture) in &project.nodes.textures {
        ui.group(|ui| {
            render_texture(ui, *id, texture, None);
        });
        ui.separator();
    }
}

