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
            "circle" => self.active_tool = Box::new(CircleTool::default()),
            "rect" => self.active_tool = Box::new(RectTool::default()),
            "double_line" => self.active_tool = Box::new(DoubleLineTool::default()),
            "offset" => self.active_tool = Box::new(OffsetTool::default()),
            "delete" => self.active_tool = Box::new(DeleteTool::default()),
            "polyline" => self.active_tool = Box::new(PolylineTool::default()),
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
                store.temp_entities.clear();
                store.temp_entities.push(Entity::Line { start, end: pos });
            }
        }
    }

    fn on_mouse_up(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(start) = self.start_pos {
                // 線を確定
                store.entities.push(Entity::Line { start, end: pos });
                store.temp_entities.clear();
            }
            self.start_pos = None;
            self.is_dragging = false;
        }
    }
}

// --- Circle Tool Implementation ---

#[derive(Default)]
struct CircleTool {
    center: Option<Point>,
    is_dragging: bool,
}

impl Tool for CircleTool {
    fn name(&self) -> &str { "circle" }

    fn on_mouse_down(&mut self, pos: Point, _store: &mut GeometryStore) {
        self.center = Some(pos);
        self.is_dragging = true;
    }

    fn on_mouse_move(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(center) = self.center {
                let radius = ((pos.x - center.x).powi(2) + (pos.y - center.y).powi(2)).sqrt();
                store.temp_entities.clear();
                store.temp_entities.push(Entity::Circle { center, radius });
            }
        }
    }

    fn on_mouse_up(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(center) = self.center {
                let radius = ((pos.x - center.x).powi(2) + (pos.y - center.y).powi(2)).sqrt();
                store.entities.push(Entity::Circle { center, radius });
                store.temp_entities.clear();
            }
            self.center = None;
            self.is_dragging = false;
        }
    }
}

// --- Rect Tool Implementation ---

#[derive(Default)]
struct RectTool {
    start_pos: Option<Point>,
    is_dragging: bool,
}

impl Tool for RectTool {
    fn name(&self) -> &str { "rect" }

    fn on_mouse_down(&mut self, pos: Point, _store: &mut GeometryStore) {
        self.start_pos = Some(pos);
        self.is_dragging = true;
    }

    fn on_mouse_move(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(start) = self.start_pos {
                store.temp_entities.clear();
                store.temp_entities.push(Entity::Rect { p1: start, p2: pos });
            }
        }
    }

    fn on_mouse_up(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(start) = self.start_pos {
                store.entities.push(Entity::Rect { p1: start, p2: pos });
                store.temp_entities.clear();
            }
            self.start_pos = None;
            self.is_dragging = false;
        }
    }
}

// --- Double Line Tool Implementation ---

pub struct DoubleLineTool {
    start_pos: Option<Point>,
    is_dragging: bool,
    width: f32,
}

impl Default for DoubleLineTool {
    fn default() -> Self {
        Self {
            start_pos: None,
            is_dragging: false,
            width: 20.0,
        }
    }
}

impl Tool for DoubleLineTool {
    fn name(&self) -> &str { "double_line" }

    fn on_mouse_down(&mut self, pos: Point, _store: &mut GeometryStore) {
        self.start_pos = Some(pos);
        self.is_dragging = true;
    }

    fn on_mouse_move(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(start) = self.start_pos {
                store.temp_entities.clear();
                
                let dir = pos.sub(start);
                let normal = dir.normal().normalize();
                let offset = normal.scale(self.width / 2.0);
                
                let p1_start = start.add(offset);
                let p1_end = pos.add(offset);
                let p2_start = start.sub(offset);
                let p2_end = pos.sub(offset);
                
                store.temp_entities.push(Entity::Line { start: p1_start, end: p1_end });
                store.temp_entities.push(Entity::Line { start: p2_start, end: p2_end });
            }
        }
    }

    fn on_mouse_up(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(start) = self.start_pos {
                let dir = pos.sub(start);
                let normal = dir.normal().normalize();
                let offset = normal.scale(self.width / 2.0);
                
                let p1_start = start.add(offset);
                let p1_end = pos.add(offset);
                let p2_start = start.sub(offset);
                let p2_end = pos.sub(offset);
                
                store.entities.push(Entity::Line { start: p1_start, end: p1_end });
                store.entities.push(Entity::Line { start: p2_start, end: p2_end });
                store.temp_entities.clear();
            }
            self.start_pos = None;
            self.is_dragging = false;
        }
    }
}

// --- Offset Tool Implementation ---

#[derive(Default)]
struct OffsetTool {
    selected_idx: Option<usize>,
    is_dragging: bool,
}

impl Tool for OffsetTool {
    fn name(&self) -> &str { "offset" }

    fn on_mouse_down(&mut self, pos: Point, store: &mut GeometryStore) {
        // 近くのエンティティを探す (閾値 20.0)
        if let Some(idx) = store.find_nearest_entity(pos, 20.0) {
            self.selected_idx = Some(idx);
            self.is_dragging = true;
        }
    }

    fn on_mouse_move(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(idx) = self.selected_idx {
                if let Some(Entity::Line { start, end }) = store.entities.get(idx) {
                    store.temp_entities.clear();
                    
                    // マウス位置までの距離と方向を計算
                    // Line vector
                    let ab = end.sub(*start);
                    // Point vector
                    let ap = pos.sub(*start);
                    
                    // Normal vector (normalized)
                    let n = ab.normal().normalize();
                    
                    // Dot product of AP and N gives the signed distance
                    let dist = ap.x * n.x + ap.y * n.y;
                    
                    // Offset vector
                    let offset = n.scale(dist);
                    
                    let new_start = start.add(offset);
                    let new_end = end.add(offset);
                    
                    store.temp_entities.push(Entity::Line { start: new_start, end: new_end });
                }
            }
        }
    }

    fn on_mouse_up(&mut self, pos: Point, store: &mut GeometryStore) {
        if self.is_dragging {
            if let Some(idx) = self.selected_idx {
                if let Some(Entity::Line { start, end }) = store.entities.get(idx) {
                    // 同じ計算をして確定
                    let ab = end.sub(*start);
                    let ap = pos.sub(*start);
                    let n = ab.normal().normalize();
                    let dist = ap.x * n.x + ap.y * n.y;
                    let offset = n.scale(dist);
                    
                    let new_start = start.add(offset);
                    let new_end = end.add(offset);
                    
                    store.entities.push(Entity::Line { start: new_start, end: new_end });
                    store.temp_entities.clear();
                }
            }
            self.selected_idx = None;
            self.is_dragging = false;
        }
    }
}

// --- Delete Tool Implementation ---

#[derive(Default)]
struct DeleteTool {
    target_idx: Option<usize>,
}

impl Tool for DeleteTool {
    fn name(&self) -> &str { "delete" }

    fn on_mouse_down(&mut self, pos: Point, store: &mut GeometryStore) {
        self.target_idx = store.find_nearest_entity(pos, 20.0);
    }

    fn on_mouse_move(&mut self, _pos: Point, _store: &mut GeometryStore) {
        // 将来的にはハイライト処理を入れる
    }

    fn on_mouse_up(&mut self, _pos: Point, store: &mut GeometryStore) {
        if let Some(idx) = self.target_idx {
            if idx < store.entities.len() {
                store.entities.remove(idx);
                println!("Deleted entity at index {}", idx);
            }
        }
        self.target_idx = None;
    }
}

// --- Polyline Tool Implementation ---

#[derive(Default)]
struct PolylineTool {
    points: Vec<Point>,
}

impl Tool for PolylineTool {
    fn name(&self) -> &str { "polyline" }

    fn on_mouse_down(&mut self, pos: Point, store: &mut GeometryStore) {
        // 終了判定: 最後の点に近い場所をクリックしたら終了
        if let Some(last) = self.points.last() {
            if pos.sub(*last).len() < 5.0 { // 5.0 world units threshold
                if self.points.len() >= 2 {
                    store.entities.push(Entity::Polyline { points: self.points.clone() });
                }
                self.points.clear();
                store.temp_entities.clear();
                return;
            }
        }
        
        self.points.push(pos);
    }

    fn on_mouse_move(&mut self, pos: Point, store: &mut GeometryStore) {
        if !self.points.is_empty() {
            let mut preview_points = self.points.clone();
            preview_points.push(pos);
            store.temp_entities.clear();
            store.temp_entities.push(Entity::Polyline { points: preview_points });
        }
    }

    fn on_mouse_up(&mut self, _pos: Point, _store: &mut GeometryStore) {
        // 何もしない (クリックで点を追加していくスタイル)
    }
}
