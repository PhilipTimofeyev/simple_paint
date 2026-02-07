use crate::draw::canvas::Segment;
use egui::Pos2;

pub fn cursor_to_segment_distance(cursor_pos: Pos2, segment: &Segment) -> f32 {
    let [endpoint_a, endpoint_b] = segment.segment;
    let vector_ab = endpoint_b - endpoint_a;
    let vector_ac = cursor_pos - endpoint_a;

    let t = (vector_ac.dot(vector_ab) / vector_ab.dot(vector_ab)).clamp(0.0, 1.0);
    let closest_point = endpoint_a + vector_ab * t;

    cursor_pos.distance(closest_point)
}
