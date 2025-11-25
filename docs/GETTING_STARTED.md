# Getting Started: Build Your First CAD

> **対象**: Rust CAD Framework を使って初めてCADを作る開発者
> 
> **目的**: プロジェクト初期化から動作確認まで、最短ルートで完了

---

## 📚 Table of Contents
1. [Prerequisites](#1-prerequisites)
2. [Project Setup](#2-project-setup)
3. [Minimal Working Code](#3-minimal-working-code)
4. [First Tool Implementation](#4-first-tool-implementation)
5. [Next Steps](#5-next-steps)

---

## 1. Prerequisites

### 必要な環境
- **Rust**: 1.70 以上
- **OS**: Windows, macOS, Linux

### インストール確認
```bash
rustc --version
# rustc 1.70.0 以上であること
```

---

## 2. Project Setup

### 2.1 プロジェクト作成

```bash
cargo new my-cad
cd my-cad
```

### 2.2 依存関係 (`Cargo.toml`)

```toml
[package]
name = "my-cad"
version = "0.1.0"
edition = "2021"

[dependencies]
# Window & Event Loop
winit = "0.29"

# Rendering
wgpu = "0.18"
bytemuck = { version = "1.14", features = ["derive"] }

# UI
egui = "0.24"
egui-wgpu = "0.24"
egui-winit = "0.24"

# Math
cgmath = "0.18"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Utilities
slotmap = "1.0"
```

### 2.3 ディレクトリ構造

```
my-cad/
├── Cargo.toml
├── src/
│   ├── main.rs           # エントリーポイント
│   ├── app.rs            # アプリケーション状態
│   ├── geometry/
│   │   ├── mod.rs
│   │   └── primitives.rs # Point, Line, Entity
│   ├── rendering/
│   │   ├── mod.rs
│   │   ├── renderer.rs   # wgpu レンダラー
│   │   └── camera.rs     # カメラ
│   ├── ui/
│   │   ├── mod.rs
│   │   └── panels.rs     # UI パネル
│   └── tools/
│       ├── mod.rs
│       └── line_tool.rs  # ラインツール
└── assets/
    └── shaders/
        └── basic.wgsl     # シェーダー
```

---

## 3. Minimal Working Code

### 3.1 `src/main.rs` - エントリーポイント

```rust
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod app;
use app::App;

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("My CAD")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();
    
    let mut app = pollster::block_on(App::new(&window));
    
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !app.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            app.resize(*physical_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                app.update();
                match app.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => app.resize(app.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
```

### 3.2 `src/app.rs` - アプリケーション状態

```rust
use winit::{event::*, window::Window};
use wgpu::util::DeviceExt;

pub struct App {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    // Rendering
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        
        // wgpu 初期化
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();
        
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        
        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/basic.wgsl").into()),
        });
        
        // Render Pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        // 初期ジオメトリ（十字線）
        let vertices = vec![
            Vertex { position: [-0.5, 0.0], color: [1.0, 1.0, 1.0] },
            Vertex { position: [0.5, 0.0], color: [1.0, 1.0, 1.0] },
            Vertex { position: [0.0, -0.5], color: [1.0, 1.0, 1.0] },
            Vertex { position: [0.0, 0.5], color: [1.0, 1.0, 1.0] },
        ];
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices: vertices.len() as u32,
        }
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }
    
    pub fn update(&mut self) {
        // 更新処理
    }
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
```

### 3.3 `assets/shaders/basic.wgsl` - シェーダー

```wgsl
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
```

### 3.4 実行

```bash
cargo run
```

**期待される結果**: 黒い背景に白い十字線が表示される

---

## 4. First Tool Implementation

### 4.1 `src/geometry/primitives.rs`

```rust
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug)]
pub enum Entity {
    Line { p1: Point, p2: Point },
}
```

### 4.2 `src/tools/line_tool.rs`

```rust
use crate::geometry::primitives::{Point, Entity};

pub struct LineTool {
    start: Option<Point>,
}

impl LineTool {
    pub fn new() -> Self {
        Self { start: None }
    }
    
    pub fn mouse_down(&mut self, pos: Point) {
        if self.start.is_none() {
            self.start = Some(pos);
        } else {
            // 線を確定
            self.start = None;
        }
    }
    
    pub fn get_preview(&self, current_pos: Point) -> Option<Entity> {
        self.start.map(|start| Entity::Line {
            p1: start,
            p2: current_pos,
        })
    }
}
```

---

## 5. Next Steps

### 学習パス
1. ✅ Getting Started (このドキュメント)
2. → **wgpu Complete Guide** - レンダリングの詳細
3. → **Integration Guide** - egui統合
4. → **Practical Implementation** - 実践的な機能

### 機能追加
- [ ] egui UI の統合
- [ ] カメラ（パン・ズーム）
- [ ] 複数ツール
- [ ] Undo/Redo

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
