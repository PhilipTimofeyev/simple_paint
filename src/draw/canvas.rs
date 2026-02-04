use egui::{Pos2, Rect};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Canvas {
    pub canvas_viewport: egui::Rect,
    pub canvas_area: Rect,
    pub strokes: Vec<SingleStroke>,
    pub segments: Vec<Segment>,
    pub last_cursor_pos: Option<Pos2>,
}

impl Canvas {
    pub fn new(dimensions: (u16, u16)) -> Self {
        let (x, y) = dimensions;
        let canvas_dimensions = Pos2::new(x as f32, y as f32);
        let (min, max) = Self::build_viewport(dimensions);
        Self {
            canvas_viewport: Rect::from_min_max(min, max),
            canvas_area: Rect::from_min_max(Pos2::default(), canvas_dimensions),
            strokes: Vec::default(),
            segments: Vec::default(),
            last_cursor_pos: None,
        }
    }

    fn build_viewport(dimensions: (u16, u16)) -> (Pos2, Pos2) {
        let (x, y) = dimensions;
        let min = Pos2::new(x as f32 / -2.0, y as f32 / -2.0);
        let max = Pos2::new(x as f32 * 1.5, y as f32 * 1.5);

        (min, max)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
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
