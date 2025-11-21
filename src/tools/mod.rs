use winit::event::{WindowEvent, ElementState, MouseButton};
use crate::geometry::primitives::{GeometryStore, Point, Entity};
use crate::geometry::snapping::Snapper;
use crate::canvas::camera::Camera;

pub trait Tool {
    fn on_mouse_down(&mut self, pos: Point, store: &mut GeometryStore);
    fn on_mouse_move(&mut self, pos: Point, store: &mut GeometryStore);
    fn on_mouse_up(&mut self, pos: Point, store: &mut GeometryStore);
    #[allow(dead_code)]
    fn name(&self) -> &str;
}

pub struct ToolManager {
    active_tool: Box<dyn Tool + Send + Sync>,
}

impl Default for ToolManager {
    fn default() -> Self {
        Self {
            active_tool: Box::new(LineTool::default()),
        }
    }
}

impl ToolManager {
    pub fn set_active_tool(&mut self, name: &str) {
        match name {
            "line" => self.active_tool = Box::new(LineTool::default()),
            _ => println!("Unknown tool: {}", name),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent, store: &mut GeometryStore, camera: &Camera, grid_size: f32) {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    // マウス位置はイベントに含まれないため、Cameraのlast_mouse_posを使うか、
                    // CursorMovedで保存しておく必要があるが、ここでは簡易的にCameraから取得する設計とする
                    // (実際にはApp側で位置を渡す方が良い)
                    if let Some((mx, my)) = camera.last_mouse_pos {
                        let (wx, wy) = camera.screen_to_world(mx as f32, my as f32);
                        let raw_pos = Point::new(wx, wy);
                        let pos = Snapper::snap(raw_pos, grid_size);
                        
                        match state {
                            ElementState::Pressed => self.active_tool.on_mouse_down(pos, store),
                            ElementState::Released => self.active_tool.on_mouse_up(pos, store),
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (wx, wy) = camera.screen_to_world(position.x as f32, position.y as f32);
                let raw_pos = Point::new(wx, wy);
                let pos = Snapper::snap(raw_pos, grid_size);
                self.active_tool.on_mouse_move(pos, store);
            }
            _ => {}
        }
    }
}

// --- Line Tool Implementation ---

#[derive(Default)]
struct LineTool {
    start_pos: Option<Point>,
    is_dragging: bool,
}

impl Tool for LineTool {
    fn name(&self) -> &str { "line" }

    fn on_mouse_down(&mut self, pos: Point, _store: &mut GeometryStore) {
        self.start_pos = Some(pos);
        self.is_dragging = true;
    }

    fn on_mouse_move(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(start) = self.start_pos {
                // プレビュー更新
                store.temp_entity = Some(Entity::Line { start, end: pos });
            }
        }
    }

    fn on_mouse_up(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(start) = self.start_pos {
                // 線を確定
                store.add_line(start, pos);
                store.temp_entity = None;
            }
            self.start_pos = None;
            self.is_dragging = false;
        }
    }
}
