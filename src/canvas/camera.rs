use winit::event::{WindowEvent, ElementState, MouseButton, MouseScrollDelta};
use cgmath::{Matrix4, Vector3, Point3};

pub struct Camera {
    pub position: Point3<f32>,
    pub zoom: f32,
    viewport_width: f32,
    viewport_height: f32,
    
    // 操作用フラグ
    pub is_panning: bool,
    pub last_mouse_pos: Option<(f64, f64)>,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            zoom: 1.0,
            viewport_width: width,
            viewport_height: height,
            is_panning: false,
            last_mouse_pos: None,
        }
    }

    pub fn update_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Middle {
                    self.is_panning = *state == ElementState::Pressed;
                    return true;
                }
                false
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                if self.is_panning {
                    if let Some((last_x, last_y)) = self.last_mouse_pos {
                        let dx = (x - last_x) as f32 / self.zoom;
                        let dy = (y - last_y) as f32 / self.zoom;
                        self.position.x -= dx;
                        self.position.y -= dy; // Y軸の向き注意 (通常CADはY-upだがwgpuはY-down等)
                    }
                }
                self.last_mouse_pos = Some((x, y));
                self.is_panning // パン中ならイベント消費
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let factor = match delta {
                    MouseScrollDelta::LineDelta(_, y) => 1.0 + y * 0.1,
                    MouseScrollDelta::PixelDelta(pos) => 1.0 + (pos.y as f32 * 0.001),
                };
                self.zoom *= factor;
                self.zoom = self.zoom.max(0.1).min(100.0);
                true
            }
            _ => false,
        }
    }

    // ワールド変換行列 (Rendererで使用)
    #[allow(dead_code)]
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::from_translation(Vector3::new(-self.position.x, -self.position.y, 0.0));
        let scale = Matrix4::from_scale(self.zoom);
        // 正射影 (Orthographic)
        let ortho = cgmath::ortho(
            0.0, self.viewport_width,
            self.viewport_height, 0.0, // 左上原点
            -1.0, 1.0
        );
        
        ortho * scale * view
    }

    // スクリーン座標 -> ワールド座標
    pub fn screen_to_world(&self, x: f32, y: f32) -> (f32, f32) {
        let wx = (x / self.zoom) + self.position.x;
        let wy = (y / self.zoom) + self.position.y;
        (wx, wy)
    }
}
