/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use egui::{Color32, CornerRadius, LayerId, Pos2, Rect, Sense, Stroke, Vec2};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    strokes: Vec<Vec<((Color32, f32), Pos2)>>,
    current_stroke: Vec<((Color32, f32), Pos2)>,
    stroke_type: (Color32, f32),
    // #[serde(skip)] // This how you opt-out of serialization of a field
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            strokes: Vec::default(),
            current_stroke: Vec::default(),
            stroke_type: (egui::Color32::BLACK, 2.0),
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

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Simple Paint");

            ui.horizontal(|ui| {
                ui.label("Brush color:");
                ui.color_edit_button_srgba(&mut self.stroke_type.0);
                ui.add(egui::Slider::new(&mut self.stroke_type.1, 0.5..=12.0).text("Brush Width"));
            });

            let size = Vec2::new(500.0, 400.0);
            let (response, painter) = ui.allocate_painter(size, egui::Sense::drag());
            let rect = response.rect;
            painter.rect_filled(rect, 0.0, egui::Color32::WHITE);

            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    self.current_stroke
                        .push(((self.stroke_type.0, self.stroke_type.1), pos));
                }
            }

            if response.drag_stopped() {
                let current_strokes = std::mem::take(&mut self.current_stroke);
                self.strokes.push(current_strokes);
            }

            // Maybe create a stroke struct that is an array of two Pos2 structs to prevent
            // needing stroke[0] and windows
            for stroke in self.current_stroke.windows(2) {
                painter.line_segment(
                    [stroke[0].1, stroke[1].1],
                    egui::Stroke::new(stroke[0].0 .1, stroke[0].0 .0),
                );
            }

            for strokes in &self.strokes {
                for stroke in strokes.windows(2) {
                    painter.line_segment(
                        [stroke[0].1, stroke[1].1],
                        egui::Stroke::new(stroke[0].0 .1, stroke[0].0 .0),
                    );
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
