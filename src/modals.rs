use std::num::ParseFloatError;

use super::SimplePaintApp;
use crate::draw::canvas::Canvas;
use egui::Margin;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct InitialModal {
    pub active: bool,
    width: String,
    height: String,
}

impl Default for InitialModal {
    fn default() -> Self {
        Self {
            active: true,
            width: String::new(),
            height: String::new(),
        }
    }
}

impl InitialModal {
    fn validate_dimensions(&self) -> Result<(f32, f32), ParseFloatError> {
        let parsed_width = self.width.parse::<f32>()?;
        let parsed_height = self.height.parse::<f32>()?;

        Ok((parsed_width, parsed_height))
    }
}

pub fn initial_modal(ctx: &egui::Context, app: &mut SimplePaintApp) {
    egui::Window::new("Initialize Modal")
        .max_width(150.0)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .frame(
            egui::Frame::new()
                .inner_margin(Margin::symmetric(40, 20))
                .fill(egui::Color32::from_hex("#ebeded").unwrap_or_default())
                .corner_radius(10.0),
        )
        .show(ctx, |ui| {
            ui.set_max_height(400.0);
            ui.vertical(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Canvas Size").size(16.0), // Set the font size in points
                    );
                });
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    let width_text = egui::TextEdit::singleline(&mut app.initial_modal.width)
                        .desired_width(40.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(width_text);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    let height_text = egui::TextEdit::singleline(&mut app.initial_modal.height)
                        .desired_width(40.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(height_text);
                    });
                });
                ui.add_space(10.0);
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        if ui.add(egui::Button::new("New")).clicked() {
                            let Ok(dimensions) = app.initial_modal.validate_dimensions() else {
                                app.initial_modal.width = String::new();
                                app.initial_modal.height = String::new();
                                return;
                            };
                            let (width, height) = dimensions;
                            app.canvas = Canvas::new(egui::Vec2::new(width, height));
                            app.initial_modal.active = false;
                        }
                    },
                );
                ui.separator();
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        if ui.add(egui::Button::new("Default")).clicked() {
                            app.initial_modal.active = false;
                        }
                    },
                );
            });
        });
}
