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

    pub fn add(&self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }

    pub fn sub(&self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }

    pub fn scale(&self, factor: f32) -> Point {
        Point::new(self.x * factor, self.y * factor)
    }

    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Point {
        let l = self.len();
        if l == 0.0 {
            Point::new(0.0, 0.0)
        } else {
            self.scale(1.0 / l)
        }
    }

    // 垂直ベクトル (90度回転)
    pub fn normal(&self) -> Point {
        Point::new(-self.y, self.x)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entity {
    Line { start: Point, end: Point },
    Circle { center: Point, radius: f32 },
    Rect { p1: Point, p2: Point },
    Polyline { points: Vec<Point> },
}

#[derive(Default)]
pub struct GeometryStore {
    pub entities: Vec<Entity>,
    pub temp_entities: Vec<Entity>,
}

impl GeometryStore {
    pub fn add_line(&mut self, start: Point, end: Point) {
        self.entities.push(Entity::Line { start, end });
    }

    pub fn find_nearest_entity(&self, pos: Point, threshold: f32) -> Option<usize> {
        let mut best_idx = None;
        let mut min_dist = threshold;

        for (i, entity) in self.entities.iter().enumerate() {
            let dist = match entity {
                Entity::Line { start, end } => {
                    let ab = end.sub(*start);
                    let ap = pos.sub(*start);
                    let ab_len_sq = ab.x * ab.x + ab.y * ab.y;
                    if ab_len_sq == 0.0 {
                        ap.len()
                    } else {
                        let t = (ap.x * ab.x + ap.y * ab.y) / ab_len_sq;
                        let t = t.max(0.0).min(1.0);
                        let closest = start.add(ab.scale(t));
                        pos.sub(closest).len()
                    }
                }
                _ => f32::MAX, // TODO: Support Circle/Rect
            };

            if dist < min_dist {
                min_dist = dist;
                best_idx = Some(i);
            }
        }
        best_idx
    }
}
