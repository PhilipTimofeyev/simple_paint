use crate::draw::canvas::{self, SingleStroke};
use crate::modals;
use crate::toolbar::main::{Tool, toolbar};
use crate::utils;
use egui::{Response, Stroke};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SimplePaintApp {
    pub initial_modal: modals::InitialModal,
    pub canvas: canvas::Canvas,
    pub stroke_type: Stroke,
    pub tool: Tool,
    pub history: History,
    // #[serde(skip)] // This how you opt-out of serialization of a field
}

impl Default for SimplePaintApp {
    fn default() -> Self {
        Self {
            initial_modal: modals::InitialModal::default(),
            canvas: canvas::Canvas::new(egui::Vec2::new(1920.0, 1080.0)),
            stroke_type: egui::Stroke::new(8.0, egui::Color32::BLACK),
            tool: Tool::Pen,
            history: History::default(),
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
    }

    fn run(&mut self, action: canvas::Action) {
        action.execute(&mut self.canvas);
        self.history.undo.push(action);
        self.history.redo.clear();
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
                    let stroke = canvas::SingleStroke {
                        stroke: self.stroke_type,
                        points: std::mem::take(&mut self.canvas.segments),
                    };
                    self.run(canvas::Action::AddStroke { stroke });
                }
                self.canvas.last_cursor_pos = None;
            }
        }
    }

    fn erase(&mut self, response: &Response) {
        let mut erase_actions: Vec<canvas::Action> = Vec::new();

        if response.dragged() {
            if let Some(eraser_pos) = response.interact_pointer_pos() {
                // let mut retained_segments: Vec<canvas::Segment> = Vec::new();
                for (idx, stroke) in self.canvas.strokes.iter().enumerate() {
                    let mut erased = false;

                    let retained_segments: Vec<canvas::Segment> = stroke
                        .points
                        .iter()
                        .filter_map(|segment| {
                            let segment_eraser_distance =
                                utils::cursor_to_segment_distance(eraser_pos, segment);

                            if segment_eraser_distance <= self.stroke_type.width {
                                erased = true;
                                None
                            } else {
                                Some(*segment)
                            }
                        })
                        .collect();

                    if !erased {
                        continue;
                    }

                    let modified_stroke = SingleStroke {
                        stroke: stroke.stroke,
                        points: retained_segments.clone(),
                    };

                    erase_actions.push(canvas::Action::ModifyStroke {
                        before: Some(stroke.clone()),
                        after: Some(modified_stroke),
                        index: idx,
                    });
                }
            }
        }
        for action in erase_actions {
            self.run(action);
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
        if self.initial_modal.active {
            modals::initial_modal(ctx, self);
        }

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

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct History {
    undo: Vec<canvas::Action>,
    redo: Vec<canvas::Action>,
}

impl History {
    pub fn undo(&mut self, canvas: &mut canvas::Canvas) {
        if let Some(action) = self.undo.pop() {
            action.undo(canvas);
            self.redo.push(action);
        }
    }

    pub fn redo(&mut self, canvas: &mut canvas::Canvas) {
        if let Some(action) = self.redo.pop() {
            action.execute(canvas);
        }
    }
}
