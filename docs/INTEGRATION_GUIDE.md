# Integration Guide: egui + wgpu + winit

> **対象**: egui, wgpu, winit を統合してCADを作る開発者
> 
> **目的**: 3つのライブラリの完全統合手順

---

## 📚 Table of Contents
1. [Integration Overview](#1-integration-overview)
2. [Event Handling](#2-event-handling)
3. [Rendering Integration](#3-rendering-integration)
4. [Mouse Coordinate Conversion](#4-mouse-coordinate-conversion)
5. [Complete Example](#5-complete-example)

---

## 1. Integration Overview

### 1.1 役割分担

```
winit  → ウィンドウ & イベント
wgpu   → 3D/2D レンダリング
egui   → UI (ツールパレット、プロパティパネル)
```

### 1.2 依存関係

```toml
[dependencies]
winit = "0.29"
wgpu = "0.18"
egui = "0.24"
egui-wgpu = "0.24"
egui-winit = "0.24"
```

---

## 2. Event Handling

### 2.1 イベントフロー

```
winit Event
  ↓
egui-winit (UI用)
  ↓
CAD Tools (UI が処理しなかった場合)
```

### 2.2 実装

```rust
use winit::event::*;
use egui_winit::State as EguiWinitState;

struct App {
    egui_state: EguiWinitState,
    tool_manager: ToolManager,
}

impl App {
    pub fn input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        // 1. egui にイベントを渡す
        let response = self.egui_state.on_event(&self.egui_ctx, event);
        
        if response.consumed {
            // egui が処理した（UIクリックなど）
            return true;
        }
        
        // 2. CAD ツールにイベントを渡す
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    let pos = self.cursor_position;
                    let world_pos = self.screen_to_world(pos);
                    
                    match state {
                        ElementState::Pressed => {
                            self.tool_manager.mouse_down(world_pos);
                        }
                        ElementState::Released => {
                            self.tool_manager.mouse_up(world_pos);
                        }
                    }
                    return true;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = *position;
                let world_pos = self.screen_to_world(*position);
                self.tool_manager.mouse_move(world_pos);
                return true;
            }
            _ => {}
        }
        
        false
    }
}
```

---

## 3. Rendering Integration

### 3.1 レンダリング順序

```
1. wgpu: CAD ビューポート描画
2. egui: UI 描画
3. Present
```

### 3.2 実装

```rust
use egui_wgpu::Renderer as EguiRenderer;

struct App {
    // wgpu
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    
    // egui
    egui_ctx: egui::Context,
    egui_state: EguiWinitState,
    egui_renderer: EguiRenderer,
    
    // CAD
    cad_renderer: CadRenderer,
}

impl App {
    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // 1. CAD ビューポート描画
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("CAD Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });
            
            self.cad_renderer.render(&mut render_pass);
        }
        
        // 2. egui UI 描画
        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            self.ui(ctx);
        });
        
        self.egui_state.handle_platform_output(window, &self.egui_ctx, full_output.platform_output);
        
        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes);
        
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }
        
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // 既存の内容を保持
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });
            
            self.egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }
        
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        
        // 3. Submit & Present
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}
```

---

## 4. Mouse Coordinate Conversion

### 4.1 Screen → World 変換

```rust
use cgmath::{Matrix4, Vector4, Point2};

struct Camera {
    view_matrix: Matrix4<f32>,
    projection_matrix: Matrix4<f32>,
    viewport_size: (f32, f32),
}

impl Camera {
    pub fn screen_to_world(&self, screen_pos: winit::dpi::PhysicalPosition<f64>) -> Point2<f32> {
        let (width, height) = self.viewport_size;
        
        // 1. Screen → NDC (Normalized Device Coordinates)
        let ndc_x = (2.0 * screen_pos.x as f32 / width) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y as f32 / height); // Y軸反転
        
        // 2. NDC → Clip Space
        let clip_pos = Vector4::new(ndc_x, ndc_y, 0.0, 1.0);
        
        // 3. Clip Space → View Space
        let inv_projection = self.projection_matrix.invert().unwrap();
        let view_pos = inv_projection * clip_pos;
        
        // 4. View Space → World Space
        let inv_view = self.view_matrix.invert().unwrap();
        let world_pos = inv_view * view_pos;
        
        Point2::new(world_pos.x, world_pos.y)
    }
    
    pub fn world_to_screen(&self, world_pos: Point2<f32>) -> winit::dpi::PhysicalPosition<f64> {
        let (width, height) = self.viewport_size;
        
        // 1. World → View
        let view_pos = self.view_matrix * Vector4::new(world_pos.x, world_pos.y, 0.0, 1.0);
        
        // 2. View → Clip
        let clip_pos = self.projection_matrix * view_pos;
        
        // 3. Clip → NDC
        let ndc_x = clip_pos.x / clip_pos.w;
        let ndc_y = clip_pos.y / clip_pos.w;
        
        // 4. NDC → Screen
        let screen_x = (ndc_x + 1.0) * width / 2.0;
        let screen_y = (1.0 - ndc_y) * height / 2.0;
        
        winit::dpi::PhysicalPosition::new(screen_x as f64, screen_y as f64)
    }
}
```

---

### 4.2 2D Orthographic の簡易版

```rust
impl Camera {
    pub fn screen_to_world_2d(&self, screen_pos: winit::dpi::PhysicalPosition<f64>) -> Point2<f32> {
        let (width, height) = self.viewport_size;
        
        // Orthographic の場合、単純な線形変換
        let world_x = (screen_pos.x as f32 - width / 2.0) / self.zoom + self.position.x;
        let world_y = (height / 2.0 - screen_pos.y as f32) / self.zoom + self.position.y;
        
        Point2::new(world_x, world_y)
    }
}
```

---

## 5. Complete Example

### 5.1 完全な統合コード

```rust
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use egui_winit::State as EguiWinitState;
use egui_wgpu::Renderer as EguiRenderer;

struct IntegratedApp {
    // Window
    window: Window,
    
    // wgpu
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    
    // egui
    egui_ctx: egui::Context,
    egui_state: EguiWinitState,
    egui_renderer: EguiRenderer,
    
    // CAD
    camera: Camera,
    geometry_store: GeometryStore,
    tool_manager: ToolManager,
    
    // State
    cursor_position: winit::dpi::PhysicalPosition<f64>,
}

impl IntegratedApp {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        
        // wgpu 初期化
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();
        
        let (device, queue) = adapter.request_device(&Default::default(), None).await.unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        
        // egui 初期化
        let egui_ctx = egui::Context::default();
        let egui_state = EguiWinitState::new(&window);
        let egui_renderer = EguiRenderer::new(&device, surface_format, None, 1);
        
        Self {
            window,
            surface,
            device,
            queue,
            config,
            egui_ctx,
            egui_state,
            egui_renderer,
            camera: Camera::new(size.width as f32, size.height as f32),
            geometry_store: GeometryStore::new(),
            tool_manager: ToolManager::new(),
            cursor_position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        }
    }
    
    fn input(&mut self, event: &WindowEvent) -> bool {
        // egui 優先
        let response = self.egui_state.on_event(&self.egui_ctx, event);
        if response.consumed {
            return true;
        }
        
        // CAD イベント処理
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    let world_pos = self.camera.screen_to_world(self.cursor_position);
                    match state {
                        ElementState::Pressed => self.tool_manager.mouse_down(world_pos),
                        ElementState::Released => self.tool_manager.mouse_up(world_pos),
                    }
                    return true;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = *position;
                let world_pos = self.camera.screen_to_world(*position);
                self.tool_manager.mouse_move(world_pos);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        self.camera.zoom(*y);
                    }
                    _ => {}
                }
                return true;
            }
            _ => {}
        }
        
        false
    }
    
    fn ui(&mut self, ctx: &egui::Context) {
        // Tool Palette
        egui::SidePanel::left("tools").show(ctx, |ui| {
            ui.heading("Tools");
            if ui.button("Line").clicked() {
                self.tool_manager.set_tool(Tool::Line);
            }
            if ui.button("Circle").clicked() {
                self.tool_manager.set_tool(Tool::Circle);
            }
        });
        
        // Property Panel
        egui::SidePanel::right("properties").show(ctx, |ui| {
            ui.heading("Properties");
            // プロパティ表示
        });
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        
        let mut encoder = self.device.create_command_encoder(&Default::default());
        
        // CAD 描画
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("CAD Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });
            
            // CAD レンダリング
        }
        
        // egui 描画
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            self.ui(ctx);
        });
        
        self.egui_state.handle_platform_output(&self.window, &self.egui_ctx, full_output.platform_output);
        
        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes);
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }
        
        self.egui_renderer.update_buffers(&self.device, &self.queue, &mut encoder, &paint_jobs, &screen_descriptor);
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });
            
            self.egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    
    let mut app = pollster::block_on(IntegratedApp::new(window));
    
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { ref event, .. } => {
                if !app.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(size) => app.resize(*size),
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                app.render().ok();
            }
            Event::MainEventsCleared => {
                app.window.request_redraw();
            }
            _ => {}
        }
    });
}
```

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
