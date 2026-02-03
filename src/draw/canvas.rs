use egui::{Pos2, Rect};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Canvas {
    pub rect: egui::Rect,
    pub strokes: Vec<SingleStroke>,
    pub segments: Vec<Segment>,
    pub last_cursor_pos: Option<Pos2>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            rect: Rect::NOTHING,
            strokes: Vec::default(),
            segments: Vec::default(),
            last_cursor_pos: None,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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
