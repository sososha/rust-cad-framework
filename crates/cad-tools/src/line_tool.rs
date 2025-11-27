use crate::tool::{Tool, ToolAction};
use cad_core::{Entity, Point};

pub struct LineTool {
    start: Option<Point>,
    current: Point,
}

impl LineTool {
    pub fn new() -> Self {
        Self {
            start: None,
            current: Point::new(0.0, 0.0),
        }
    }
}

impl Tool for LineTool {
    fn name(&self) -> &str {
        "Line"
    }

    fn active(&self) -> bool {
        self.start.is_some()
    }

    fn mouse_down(&mut self, pos: Point) -> Option<ToolAction> {
        if let Some(start) = self.start {
            // Second click: Finish line
            let entity = Entity::Line { p1: start, p2: pos };
            self.start = None;
            Some(ToolAction::Commit(entity))
        } else {
            // First click: Start line
            self.start = Some(pos);
            self.current = pos;
            Some(ToolAction::None)
        }
    }

    fn mouse_move(&mut self, pos: Point) -> Option<ToolAction> {
        self.current = pos;
        Some(ToolAction::None)
    }

    fn mouse_up(&mut self, _pos: Point) -> Option<ToolAction> {
        None
    }

    fn get_preview(&self) -> Option<Entity> {
        if let Some(start) = self.start {
            Some(Entity::Line {
                p1: start,
                p2: self.current,
            })
        } else {
            None
        }
    }
}
