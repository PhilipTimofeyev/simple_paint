use crate::draw::canvas;
use crate::toolbar::main::{Tool, toolbar};
use crate::utils;
use egui::{Response, Stroke};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SimplePaintApp {
    initial_modal: InitialModal,
    pub canvas: canvas::Canvas,
    pub stroke_type: Stroke,
    pub tool: Tool,
    // #[serde(skip)] // This how you opt-out of serialization of a field
}

impl Default for SimplePaintApp {
    fn default() -> Self {
        Self {
            initial_modal: InitialModal::default(),
            canvas: canvas::Canvas::new(egui::Vec2::new(1920.0, 1080.0)),
            stroke_type: egui::Stroke::new(8.0, egui::Color32::BLACK),
            tool: Tool::Pen,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct InitialModal {
    active: bool,
    width: String,
    height: String,
}

impl InitialModal {
    fn default() -> Self {
        Self {
            active: true,
            width: String::new(),
            height: String::new(),
        }
    }

    fn validate_dimensions(&self) -> (f32, f32) {
        let parsed_width = self.width.parse::<f32>().expect("Failed to parse to width");
        let parsed_height = self
            .height
            .parse::<f32>()
            .expect("Failed to parse to height");

        (parsed_width, parsed_height)
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

        if self.initial_modal.active {
            egui::Window::new("My Modal Window")
                .max_width(100.0)
                .title_bar(false)
                // .open(&mut self.initial_modal.active) // Links the window's close button to the state
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Dimensions").size(16.0), // Set the font size in points
                        );
                        ui.add_space(15.0);
                        ui.label("Width");
                        ui.text_edit_singleline(&mut self.initial_modal.width);
                        ui.label("Height");
                        ui.text_edit_singleline(&mut self.initial_modal.height);

                        if ui.add(egui::Button::new("New")).clicked() {
                            let (width, height) = self.initial_modal.validate_dimensions();
                            self.canvas = canvas::Canvas::new(egui::Vec2::new(width, height));
                            self.initial_modal.active = false;
                        }
                        if ui.add(egui::Button::new("Default")).clicked() {
                            self.initial_modal.active = false;
                        }
                    });
                });
        }

        egui::TopBottomPanel::top("tool panel")
            .frame(egui::Frame::new().fill(egui::Color32::from_hex("#adadad").unwrap_or_default()))
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_max_height(30.0);

                ui.horizontal_centered(|ui| {
                    let toolbar_width = 620.0;
                    ui.add_space((ui.available_width() / 2.0) - toolbar_width / 2.0);
                    toolbar(self, ui);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::DARK_GRAY))
            .show(ctx, |ui| {
                if self.initial_modal.active {
                    return;
                }
                let scene = egui::Scene::new().zoom_range(0.01..=10.0);
                let scene_response = scene.show(ui, &mut self.canvas.canvas_viewport, |ui| {
                    ui.allocate_painter(self.canvas.canvas_area.size(), egui::Sense::drag())
                });

                let (response, painter) = scene_response.inner;
                painter.rect_filled(self.canvas.canvas_area, 0.0, egui::Color32::WHITE);

                if response.hovered() {
                    match self.tool {
                        Tool::Pen => {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
                            self.draw(&response, &painter);
                        }
                        Tool::Erase => {
                            if let Some(pos) = response.hover_pos() {
                                painter.circle_stroke(
                                    pos,
                                    self.stroke_type.width,
                                    egui::Stroke::new(1.0, egui::Color32::BLACK),
                                );
                            }
                            self.erase(&response);
                        }
                    }
                }

                self.canvas.update_zoom();

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
