//! Debug UI for visualizing textures, mappings, and LEDs

use egui::{Color32, ColorImage, Image, Painter, TextureHandle, Ui};
use lp_core::nodes::fixture::{FixtureNode, Mapping};
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

/// Generate a color for a fixture based on its ID
fn fixture_color(fixture_id: u32) -> Color32 {
    // Generate distinct colors for different fixtures
    let hue = (fixture_id as f32 * 137.508) % 360.0; // Golden angle for distribution
    let (r, g, b) = hsv_to_rgb(hue / 360.0, 0.8, 0.9);
    Color32::from_rgb(r, g, b)
}

/// Convert HSV to RGB (simple approximation)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Draw mapping overlay on a texture
fn draw_mapping_overlay(
    painter: &Painter,
    texture_rect: egui::Rect,
    _texture_width: u32,
    _texture_height: u32,
    fixture_id: u32,
    mapping: &Mapping,
    show_labels: bool,
) {
    let color = fixture_color(fixture_id);
    let stroke_color = Color32::from_rgb(255, 255, 255); // White outline for visibility

    // Convert normalized coordinates [0, 1] to screen coordinates
    let center_x = texture_rect.left() + mapping.center[0] * texture_rect.width();
    let center_y = texture_rect.top() + mapping.center[1] * texture_rect.height();

    // Convert normalized radius to screen coordinates
    // Radius is in normalized texture space, so multiply by texture dimension
    let radius_pixels = mapping.radius * texture_rect.width().min(texture_rect.height());

    let center = egui::pos2(center_x, center_y);

    // Draw circle outline (radius)
    painter.circle_stroke(center, radius_pixels, egui::Stroke::new(1.0, stroke_color));

    // Draw center point
    painter.circle_filled(center, 3.0, color);
    painter.circle_stroke(center, 3.0, egui::Stroke::new(1.0, stroke_color));

    // Draw label if requested
    if show_labels {
        let label = format!("Ch{}", mapping.channel);
        painter.text(
            center + egui::Vec2::new(radius_pixels + 5.0, 0.0),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::monospace(10.0),
            color,
        );
    }
}

/// Render texture visualization in egui with optional mapping overlay
pub fn render_texture(
    ui: &mut Ui,
    texture_id: u32,
    texture: &TextureNode,
    texture_data: Option<&[u8]>,
    show_mappings: bool,
    project: Option<&ProjectConfig>,
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

            // Create a response area for the image to get its rect
            let image_response = ui.add(
                Image::new(&texture_handle).max_size(egui::Vec2::new(display_width, display_height))
            );

            // Draw mapping overlay if enabled
            if show_mappings {
                if let Some(project) = project {
                    // Find all fixtures and overlay their mappings
                    for (fixture_id, fixture) in &project.nodes.fixtures {
                        match fixture {
                            FixtureNode::CircleList { mapping, .. } => {
                                // Draw each mapping in this fixture
                                for mapping_item in mapping {
                                    draw_mapping_overlay(
                                        ui.painter(),
                                        image_response.rect,
                                        width,
                                        height,
                                        *fixture_id,
                                        mapping_item,
                                        true, // Show labels
                                    );
                                }
                            }
                        }
                    }
                }
            }
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

    // Toggle for showing mappings
    let mut show_mappings = true; // Could be stored in app state for persistence
    ui.checkbox(&mut show_mappings, "Show mapping overlay");

    if show_mappings && !project.nodes.fixtures.is_empty() {
        ui.label(format!("Found {} fixture(s)", project.nodes.fixtures.len()));
    }

    ui.separator();

    // Display each texture
    for (id, texture) in &project.nodes.textures {
        ui.group(|ui| {
            render_texture(ui, *id, texture, None, show_mappings, Some(project));
        });
        ui.separator();
    }
}

