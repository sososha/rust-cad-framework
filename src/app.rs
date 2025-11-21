use std::sync::Arc;
use winit::{window::Window, event::WindowEvent};
use parking_lot::Mutex;
use crate::canvas::renderer::Renderer;
use crate::canvas::camera::Camera;
use crate::geometry::primitives::GeometryStore;
use crate::tools::ToolManager;

// アプリケーションの共有状態 (Agentからもアクセス可能)
pub struct AppState {
    pub geometry: GeometryStore,
    pub camera: Camera,
    pub tool_manager: ToolManager,
    pub grid_size: f32,
    // 今後の拡張: Selection, Layers, etc.
}

pub struct CadApp {
    #[allow(dead_code)]
    window: Arc<Window>,
    renderer: Renderer,
    state: Arc<Mutex<AppState>>,
}

impl CadApp {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let renderer = Renderer::new(window.clone(), size.width, size.height).await;
        
        let state = Arc::new(Mutex::new(AppState {
            geometry: GeometryStore::default(),
            camera: Camera::new(size.width as f32, size.height as f32),
            tool_manager: ToolManager::default(),
            grid_size: 50.0, // デフォルトグリッドサイズ
        }));

        Self {
            window,
            renderer,
            state,
        }
    }

    pub fn handle(&self) -> Arc<Mutex<AppState>> {
        self.state.clone()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.resize(new_size);
            let mut state = self.state.lock();
            state.camera.update_viewport(new_size.width as f32, new_size.height as f32);
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        // eguiのイベント処理をここに入れる (後で実装)
        // if self.renderer.handle_egui_event(event) { return true; }

        let mut state = self.state.lock();
        
        // カメラ操作 (スペースキー or 中クリックドラッグ)
        if state.camera.handle_event(event) {
            return true;
        }

        // ツール操作
        // MutexGuardから個別のフィールドへの可変参照を取り出すことで、
        // Rustの借用チェッカーに「別々のデータを触っている」ことを伝える
        let AppState { geometry, camera, tool_manager, grid_size, .. } = &mut *state;
        tool_manager.handle_event(event, geometry, camera, *grid_size);
        
        false
    }

    pub fn update(&mut self) {
        // アニメーションや物理演算があればここで
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let state = self.state.lock();
        self.renderer.render(&state)
    }
    
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.renderer.size()
    }
}
