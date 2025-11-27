use cad_core::{Entity, Point};

pub trait Tool {
    fn name(&self) -> &str;
    fn active(&self) -> bool;
    
    // Events
    fn mouse_down(&mut self, pos: Point) -> Option<ToolAction>;
    fn mouse_move(&mut self, pos: Point) -> Option<ToolAction>;
    fn mouse_up(&mut self, pos: Point) -> Option<ToolAction>;
    
    // Preview
    fn get_preview(&self) -> Option<Entity>;
}

pub enum ToolAction {
    Commit(Entity),
    Cancel,
    None,
}
