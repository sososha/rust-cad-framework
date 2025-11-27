use crate::primitives::Point;
use crate::geometry::Entity;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapType {
    Endpoint,
    Midpoint,
    Center,
}

#[derive(Debug, Clone, Copy)]
pub struct SnapPoint {
    pub point: Point,
    pub snap_type: SnapType,
}

pub const SNAP_DISTANCE: f32 = 10.0; // Pixel distance threshold

impl Entity {
    pub fn get_snap_points(&self) -> Vec<SnapPoint> {
        match self {
            Entity::Line { p1, p2 } => vec![
                SnapPoint { point: *p1, snap_type: SnapType::Endpoint },
                SnapPoint { point: *p2, snap_type: SnapType::Endpoint },
                SnapPoint { 
                    point: Point::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0), 
                    snap_type: SnapType::Midpoint 
                },
            ],
            Entity::Circle { center, .. } => vec![
                SnapPoint { point: *center, snap_type: SnapType::Center },
                // Could add quadrants here
            ],
        }
    }
}

pub fn find_closest_snap_point(
    entities: &[Entity], 
    cursor_pos: Point, 
    threshold: f32
) -> Option<SnapPoint> {
    let mut closest_snap: Option<SnapPoint> = None;
    let mut min_dist_sq = threshold * threshold;

    for entity in entities {
        for snap in entity.get_snap_points() {
            let dist_sq = (snap.point.x - cursor_pos.x).powi(2) + (snap.point.y - cursor_pos.y).powi(2);
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                closest_snap = Some(snap);
            }
        }
    }

    closest_snap
}
