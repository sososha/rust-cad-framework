use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entity {
    Line { start: Point, end: Point },
    // Circle, Rect, etc.
}

#[derive(Default)]
pub struct GeometryStore {
    pub entities: Vec<Entity>,
    pub temp_entity: Option<Entity>, // 描画中のプレビュー用
}

impl GeometryStore {
    pub fn add_line(&mut self, start: Point, end: Point) {
        self.entities.push(Entity::Line { start, end });
    }
}
