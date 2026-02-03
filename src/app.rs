/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use egui::{
    Align2, Color32, CornerRadius, LayerId, Margin, Pos2, Rect, Response, Sense, Stroke, Vec2,
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    strokes: Vec<SingleStroke>,
    segments: Vec<Segment>,
    last_pos: Option<Pos2>,
    stroke_type: Stroke,
    tool: Tool,
    rect: egui::Rect,
    // #[serde(skip)] // This how you opt-out of serialization of a field
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            strokes: Vec::default(),
            segments: Vec::default(),
            last_pos: None,
            stroke_type: egui::Stroke::new(2.0, egui::Color32::BLACK),
            tool: Tool::Pen,
            rect: Rect::NOTHING,
        }
    }
}

impl TemplateApp {
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
                if let Some(prev) = self.last_pos {
                    // Modify this to change resolution. Less tiny segments = lower resolution
                    if prev.distance(pen_position) > 0.0 {
                        self.segments.push(Segment::new(prev, pen_position));
                    }
                }
                self.last_pos = Some(pen_position);
            }

            // draw strokes in realtime
            for seg in &self.segments {
                painter.line_segment(seg.segment, self.stroke_type);
            }

            if response.drag_stopped() {
                if !self.segments.is_empty() {
                    self.strokes.push(SingleStroke {
                        stroke: self.stroke_type,
                        points: std::mem::take(&mut self.segments),
                    });
                }
                self.last_pos = None;
            }
        }
    }

    fn erase(&mut self, response: &Response) {
        if response.dragged() {
            if let Some(eraser_pos) = response.interact_pointer_pos() {
                for stroke in &mut self.strokes {
                    stroke.points.retain(|seg| {
                        cursor_to_segment_distance(eraser_pos, *seg) > self.stroke_type.width
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

fn cursor_to_segment_distance(cursor_pos: Pos2, segment: Segment) -> f32 {
    let [endpoint_a, endpoint_b] = segment.segment;
    let vector_ab = endpoint_b - endpoint_a;
    let vector_ac = cursor_pos - endpoint_a;

    let t = (vector_ac.dot(vector_ab) / vector_ab.dot(vector_ab)).clamp(0.0, 1.0);
    let closest_point = endpoint_a + vector_ab * t;

    cursor_pos.distance(closest_point)
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct SingleStroke {
    stroke: egui::Stroke,
    points: Vec<Segment>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Copy, Clone)]
struct Segment {
    segment: [Pos2; 2],
}

impl Segment {
    fn new(a: Pos2, b: Pos2) -> Self {
        Self { segment: [a, b] }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        // if true {
        //     egui::Window::new("Mo")
        //         .collapsible(false)
        //         .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        //         .resizable(false)
        //         // .open(&mut self.window_open)
        //         .show(ctx, |ui| {
        //             ui.label("contents");
        //         });
        // }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

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
                        self.strokes.pop();
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
                let scene_response = scene.show(ui, &mut self.rect, |ui| {
                    ui.allocate_painter(
                        Vec2 {
                            x: 2000.0,
                            y: 1500.0,
                        },
                        egui::Sense::drag(),
                    )
                });

                let (response, painter) = scene_response.inner;

                painter.rect_filled(self.rect, 0.0, egui::Color32::WHITE);

                if ui.ui_contains_pointer() {
                    match self.tool {
                        Tool::Pen => {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
                            self.draw(&response, &painter);
                        }
                        Tool::Erase => self.erase(&response),
                    }
                }
                for stroke in &self.strokes {
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
