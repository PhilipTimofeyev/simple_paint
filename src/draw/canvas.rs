use egui::{Pos2, Rect};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Canvas {
    pub canvas_viewport: Rect,
    pub canvas_area: Rect,
    pub strokes: Vec<SingleStroke>,
    pub segments: Vec<Segment>,
    pub last_cursor_pos: Option<Pos2>,
    pub zoom: f32,
}

impl Canvas {
    pub fn new(canvas_size: egui::Vec2) -> Self {
        let canvas_dimensions = canvas_size.to_pos2();
        let initial_zoom = 0.85;
        let canvas_viewport = build_viewport(canvas_size, initial_zoom);

        Self {
            canvas_viewport,
            canvas_area: Rect::from_min_max(Pos2::default(), canvas_dimensions),
            strokes: Vec::default(),
            segments: Vec::default(),
            last_cursor_pos: None,
            zoom: initial_zoom,
        }
    }

    // Ratio between canvas size and viewport size is zoom level
    pub fn update_zoom(&mut self) {
        let canvas_size = self.canvas_area.size();
        let viewport_size = self.canvas_viewport.size();

        self.zoom = canvas_size.x / viewport_size.x;
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Action {
    AddStroke {
        stroke: SingleStroke,
    },
    RemoveStroke {
        stroke: SingleStroke,
        index: usize,
    },
    ModifyStroke {
        before: Option<SingleStroke>,
        after: Option<SingleStroke>,
        index: usize,
    },
}

impl Action {
    pub fn execute(&self, canvas: &mut Canvas) {
        match self {
            Self::AddStroke { stroke } => {
                canvas.strokes.push(stroke.clone());
            }
            Self::RemoveStroke { stroke: _, index } => {
                canvas.strokes.remove(*index);
            }
            Self::ModifyStroke {
                before: _,
                after,
                index,
            } => {
                if let Some(after) = after {
                    canvas.strokes[*index] = after.clone();
                }
            }
        }
    }

    pub fn undo(&self, canvas: &mut Canvas) {
        match self {
            Self::AddStroke { stroke } => {
                canvas.strokes.pop();
            }
            Self::RemoveStroke { stroke, index } => {
                canvas.strokes.insert(*index, stroke.clone());
            }
            Self::ModifyStroke {
                before,
                after: _,
                index,
            } => {
                if let Some(before) = before {
                    canvas.strokes[*index] = before.clone();
                }
            }
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SingleStroke {
    pub stroke: egui::Stroke,
    pub points: Vec<Segment>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Copy, Clone)]
pub struct Segment {
    pub segment: [Pos2; 2],
}

impl Segment {
    pub fn new(a: Pos2, b: Pos2) -> Self {
        Self { segment: [a, b] }
    }
}

pub fn build_viewport(canvas_size: egui::Vec2, zoom: f32) -> Rect {
    // let canvas_size = egui::vec2(dimensions.0 as f32, dimensions.1 as f32);
    let center = canvas_size / 2.0;

    let view_size = canvas_size / zoom;

    Rect::from_center_size(center.to_pos2(), view_size)
}
