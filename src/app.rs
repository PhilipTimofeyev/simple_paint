/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use egui::{Color32, CornerRadius, LayerId, Pos2, Rect, Response, Sense, Stroke, Vec2};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    strokes: Vec<SingleStroke>,
    current_stroke: Vec<Pos2>,
    stroke_type: Stroke,
    tool: Tool,
    // #[serde(skip)] // This how you opt-out of serialization of a field
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            strokes: Vec::default(),
            current_stroke: Vec::default(),
            stroke_type: egui::Stroke::new(2.0, egui::Color32::BLACK),
            tool: Tool::Pen,
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

    fn draw(&mut self, painter: &egui::Painter, response: &Response) {
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                if self.current_stroke.len() != 2 {
                    self.current_stroke.push(pos);
                } else {
                    let single_stroke = SingleStroke {
                        points: [self.current_stroke[0], self.current_stroke[1]],
                        stroke: self.stroke_type,
                    };
                    self.strokes.push(single_stroke);
                    self.current_stroke.remove(0);
                }
            }
        }

        if response.drag_stopped() {
            self.current_stroke.clear();
        }
    }

    // fn erase(&mut self, response: &Response) {
    //     let mut updated_strokes: Vec<Vec<(Stroke, Pos2)>> = Vec::new();
    //     let mut strokes: Vec<(Stroke, Pos2)> = Vec::new();
    //
    //     if response.dragged() {
    //         if let Some(pos) = response.interact_pointer_pos() {
    //             for stroke in &self.strokes {
    //                 for (stroke, point) in stroke {
    //                     if point.distance(pos) > 2.0 {
    //                         strokes.push((*stroke, *point));
    //                     } else if strokes.len() > 1 {
    //                         let working_stroke = std::mem::take(&mut strokes);
    //                         updated_strokes.push(working_stroke);
    //                     }
    //                 }
    //                 let working_stroke = std::mem::take(&mut strokes);
    //                 updated_strokes.push(working_stroke);
    //             }
    //         }
    //     }
    //     if updated_strokes.len() > 1 {
    //         // println!("{:?}", updated_strokes);
    //         self.strokes = updated_strokes;
    //     }
    // }
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
enum Tool {
    Pen,
    Erase,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct SingleStroke {
    stroke: egui::Stroke,
    points: [Pos2; 2],
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
                ui.label("Color:");
                ui.color_edit_button_srgba(&mut self.stroke_type.color);
                ui.label("Pen width:");
                ui.add(egui::Slider::new(&mut self.stroke_type.width, 0.5..=12.0));
                if ui.add(egui::Button::new("Undo")).clicked() {
                    self.strokes.pop();
                }

                ui.selectable_value(&mut self.tool, Tool::Pen, "Pen");
                ui.selectable_value(&mut self.tool, Tool::Erase, "Erase");
                // ui.add(label("SelectableLabel", "SelectableLabel"));
            });

            let size = Vec2::new(500.0, 400.0);
            let (response, painter) = ui.allocate_painter(size, egui::Sense::drag());
            let rect = response.rect;
            painter.rect_filled(rect, 0.0, egui::Color32::WHITE);

            match self.tool {
                Tool::Pen => self.draw(&painter, &response),
                Tool::Erase => todo!(),
            }

            // Move to draw canvas function
            for strokes in &self.strokes {
                painter.line_segment(strokes.points, strokes.stroke);
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
