use crate::draw::canvas;
use crate::utils;
use egui::{Response, Stroke, Vec2};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SimplePaintApp {
    canvas: canvas::Canvas,
    stroke_type: Stroke,
    tool: Tool,
    // #[serde(skip)] // This how you opt-out of serialization of a field
}

impl Default for SimplePaintApp {
    fn default() -> Self {
        Self {
            canvas: canvas::Canvas::default(),
            stroke_type: egui::Stroke::new(2.0, egui::Color32::BLACK),
            tool: Tool::Pen,
        }
    }
}

impl SimplePaintApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        // eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {

        Default::default()
        // }
    }

    fn draw(&mut self, response: &Response, painter: &egui::Painter) {
        if let Some(pen_position) = response.interact_pointer_pos() {
            if response.dragged() {
                if let Some(prev) = self.canvas.last_cursor_pos {
                    // Modify this to change resolution. Less tiny segments = lower resolution
                    if prev.distance(pen_position) > 0.0 {
                        self.canvas
                            .segments
                            .push(canvas::Segment::new(prev, pen_position));
                    }
                }

                self.canvas.last_cursor_pos = Some(pen_position);
            }

            // draw strokes in realtime
            for seg in &self.canvas.segments {
                painter.line_segment(seg.segment, self.stroke_type);
            }

            if response.drag_stopped() {
                if !self.canvas.segments.is_empty() {
                    self.canvas.strokes.push(canvas::SingleStroke {
                        stroke: self.stroke_type,
                        points: std::mem::take(&mut self.canvas.segments),
                    });
                }
                self.canvas.last_cursor_pos = None;
            }
        }
    }

    fn erase(&mut self, response: &Response) {
        if response.dragged() {
            if let Some(eraser_pos) = response.interact_pointer_pos() {
                for stroke in &mut self.canvas.strokes {
                    stroke.points.retain(|seg| {
                        utils::cursor_to_segment_distance(eraser_pos, *seg) > self.stroke_type.width
                    });
                }
            }
        }
    }
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
enum Tool {
    Pen,
    Erase,
}

impl eframe::App for SimplePaintApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::TopBottomPanel::top("tool panel")
            .frame(egui::Frame::new().fill(egui::Color32::LIGHT_GRAY))
            .resizable(false)
            .max_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(ui.available_width() / 2.4);
                    ui.color_edit_button_srgba(&mut self.stroke_type.color);
                    ui.selectable_value(
                        &mut self.tool,
                        Tool::Pen,
                        egui::RichText::new("Pen")
                            .size(14.0)
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.selectable_value(
                        &mut self.tool,
                        Tool::Erase,
                        egui::RichText::new("Eraser")
                            .size(14.0)
                            .text_style(egui::TextStyle::Monospace),
                    );
                    ui.add(egui::Slider::new(&mut self.stroke_type.width, 0.5..=12.0));
                    if ui.add(egui::Button::new("Undo")).clicked() {
                        self.canvas.strokes.pop();
                    }
                })
                // });
                //
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new().fill(egui::Color32::DARK_GRAY), // .inner_margin(egui::Margin::symmetric(0, 15)),
            )
            .show(ctx, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                let scene = egui::Scene::new().zoom_range(0.0..=10.0);
                let scene_response = scene.show(ui, &mut self.canvas.rect, |ui| {
                    ui.allocate_painter(
                        Vec2 {
                            x: 2000.0,
                            y: 1500.0,
                        },
                        egui::Sense::drag(),
                    )
                });

                let (response, painter) = scene_response.inner;

                painter.rect_filled(self.canvas.rect, 0.0, egui::Color32::WHITE);

                if ui.ui_contains_pointer() {
                    match self.tool {
                        Tool::Pen => {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
                            self.draw(&response, &painter);
                        }
                        Tool::Erase => self.erase(&response),
                    }
                }
                for stroke in &self.canvas.strokes {
                    for segment in &stroke.points {
                        painter.line_segment(segment.segment, stroke.stroke);
                    }
                }
                // ui.separator();
                //
                // ui.add(egui::github_link_file!(
                //     "https://github.com/emilk/eframe_template/blob/main/",
                //     "Source code."
                // ));

                // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                //     powered_by_egui_and_eframe(ui);
                //     egui::warn_if_debug_build(ui);
                // });
            });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
