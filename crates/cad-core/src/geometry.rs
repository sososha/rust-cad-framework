use crate::primitives::Point;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct EntityId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entity {
    Line {
        p1: Point,
        p2: Point,
    },
    Circle {
        center: Point,
        radius: f32,
    },
    // Add more entities as needed
}
