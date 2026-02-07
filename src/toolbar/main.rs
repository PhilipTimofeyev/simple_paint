use super::super::SimplePaintApp;
use crate::draw::canvas;
use egui::{InnerResponse, Margin};

#[derive(PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Tool {
    Pen,
    Erase,
}

pub fn toolbar(app: &mut SimplePaintApp, ui: &mut egui::Ui) -> InnerResponse<()> {
    egui::Frame::NONE
        .fill(egui::Color32::from_hex("#dedede").unwrap_or_default())
        .corner_radius(10.0)
        .show(ui, |ui| {
            ui.add_space(30.0);

            // Undo
            if ui.add(egui::Button::new("Undo")).clicked() {
                app.history.undo(&mut app.canvas);
            }

            // Redo
            if ui.add(egui::Button::new("Redo")).clicked() {
                app.history.redo(&mut app.canvas);
            }

            // Color Palette
            ui.color_edit_button_srgba(&mut app.stroke_type.color);

            // TOOLS
            tool_frame(app, ui);

            // Pen Width
            egui::Frame::NONE.show(ui, |ui| {
                ui.label("Width");
                ui.add(egui::Slider::new(&mut app.stroke_type.width, 0.5..=50.0));
            });

            // Zoom
            egui::Frame::NONE
                .inner_margin(Margin::symmetric(30, 0))
                .show(ui, |ui| {
                    ui.label("Zoom");
                    let zoom = egui::DragValue::new(&mut app.canvas.zoom)
                        .range(0.01..=10.0)
                        .speed(0.01)
                        .custom_formatter(|n, _| {
                            let n = n * 100.0;
                            format!("{n:.0}%")
                        });
                    let zoom_response = ui.add(zoom);

                    if zoom_response.dragged() {
                        app.canvas.canvas_viewport =
                            canvas::build_viewport(app.canvas.canvas_area.size(), app.canvas.zoom);
                    }
                });
        })
}

fn tool_frame(app: &mut SimplePaintApp, ui: &mut egui::Ui) {
    egui::Frame::NONE
        .stroke(egui::Stroke::new(
            1.5,
            egui::Color32::from_hex("#b8b8b8").unwrap_or_default(),
        ))
        .corner_radius(1.0)
        .outer_margin(Margin::symmetric(20, 0))
        .show(ui, |ui| {
            egui::Grid::new("tool grid").show(ui, |ui| {
                // Pen
                ui.selectable_value(
                    &mut app.tool,
                    Tool::Pen,
                    egui::RichText::new("Pen")
                        // .size(14.0)
                        .text_style(egui::TextStyle::Monospace),
                );
                // Eraser
                ui.selectable_value(
                    &mut app.tool,
                    Tool::Erase,
                    egui::RichText::new("Eraser")
                        // .size(14.0)
                        .text_style(egui::TextStyle::Monospace),
                );
            })
        });
}
