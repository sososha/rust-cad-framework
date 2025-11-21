use crate::geometry::primitives::Point;

pub struct Snapper;

impl Snapper {
    pub fn snap(pos: Point, grid_size: f32) -> Point {
        // グリッドスナップ
        let snapped_x = (pos.x / grid_size).round() * grid_size;
        let snapped_y = (pos.y / grid_size).round() * grid_size;
        
        // ここに「既存の点へのスナップ」ロジックを追加できる
        // if distance(pos, endpoint) < threshold { return endpoint; }

        Point::new(snapped_x, snapped_y)
    }
}
