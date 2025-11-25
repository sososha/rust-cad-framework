# wgpu Complete Guide: From Zero to Rendering

> **対象**: wgpu を使ってCADレンダリングを実装する開発者
> 
> **目的**: wgpu の初期化から描画まで、全ステップを完全理解

---

## 📚 Table of Contents
1. [wgpu Overview](#1-wgpu-overview)
2. [Initialization Pipeline](#2-initialization-pipeline)
3. [Shaders (WGSL)](#3-shaders-wgsl)
4. [Render Pipeline](#4-render-pipeline)
5. [Drawing](#5-drawing)
6. [Advanced Techniques](#6-advanced-techniques)

---

## 1. wgpu Overview

### 1.1 wgpu とは

**wgpu**: WebGPU の Rust 実装、クロスプラットフォームGPU API

```
wgpu → Vulkan (Linux/Windows)
     → Metal (macOS/iOS)
     → DirectX 12 (Windows)
     → WebGPU (Browser)
```

### 1.2 主要コンポーネント

```
Instance → Adapter → Device → Queue
                  ↓
                Surface → SwapChain
```

---

## 2. Initialization Pipeline

### 2.1 Instance 作成

```rust
let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    backends: wgpu::Backends::all(), // すべてのバックエンド
    dx12_shader_compiler: Default::default(),
});
```

**backends の選択肢**:
- `Backends::all()` - 全て（推奨）
- `Backends::VULKAN` - Vulkan のみ
- `Backends::METAL` - Metal のみ
- `Backends::DX12` - DirectX 12 のみ

---

### 2.2 Surface 作成

```rust
let surface = unsafe { instance.create_surface(&window) }.unwrap();
```

**注意**: `unsafe` だが、winit の Window を渡す限り安全

---

### 2.3 Adapter 取得

```rust
let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
    power_preference: wgpu::PowerPreference::HighPerformance,
    compatible_surface: Some(&surface),
    force_fallback_adapter: false,
}).await.unwrap();
```

**PowerPreference**:
- `HighPerformance` - 専用GPU優先（CAD推奨）
- `LowPower` - 統合GPU優先（バッテリー重視）

---

### 2.4 Device & Queue 取得

```rust
let (device, queue) = adapter.request_device(
    &wgpu::DeviceDescriptor {
        features: wgpu::Features::empty(),
        limits: wgpu::Limits::default(),
        label: Some("Device"),
    },
    None, // Trace path
).await.unwrap();
```

**Features** (必要に応じて):
```rust
features: wgpu::Features::POLYGON_MODE_LINE // ワイヤーフレーム
    | wgpu::Features::MULTI_DRAW_INDIRECT   // 間接描画
```

---

### 2.5 Surface Configuration

```rust
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
    present_mode: wgpu::PresentMode::Fifo, // VSync
    alpha_mode: surface_caps.alpha_modes[0],
    view_formats: vec![],
};

surface.configure(&device, &config);
```

**PresentMode**:
- `Fifo` - VSync（推奨）
- `Immediate` - VSync なし（低遅延）
- `Mailbox` - トリプルバッファリング

---

## 3. Shaders (WGSL)

### 3.1 基本構造

```wgsl
// Vertex Shader Input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

// Vertex Shader Output = Fragment Shader Input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// Vertex Shader
@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// Fragment Shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
```

---

### 3.2 Uniform Buffer

```wgsl
struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 0.0, 1.0);
    out.color = model.color;
    return out;
}
```

**Rust 側**:
```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Camera Buffer"),
    contents: bytemuck::cast_slice(&[camera_uniform]),
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
});
```

---

## 4. Render Pipeline

### 4.1 Vertex Buffer Layout

```rust
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
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
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

---

### 4.2 Bind Group Layout

```rust
let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    entries: &[
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    ],
    label: Some("camera_bind_group_layout"),
});

let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    layout: &bind_group_layout,
    entries: &[
        wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }
    ],
    label: Some("camera_bind_group"),
});
```

---

### 4.3 Pipeline 作成

```rust
let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Render Pipeline"),
    layout: Some(&pipeline_layout),
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
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })],
    }),
    primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::LineList, // 線描画
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None, // 2Dなのでカリングなし
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
    },
    depth_stencil: Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }),
    multisample: wgpu::MultisampleState {
        count: 4, // 4x MSAA
        mask: !0,
        alpha_to_coverage_enabled: false,
    },
    multiview: None,
});
```

**PrimitiveTopology**:
- `LineList` - 線（2頂点ごと）
- `LineStrip` - 連続線
- `TriangleList` - 三角形（3頂点ごと）

---

## 5. Drawing

### 5.1 基本的な描画ループ

```rust
pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    // 1. Surface から Texture を取得
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    
    // 2. Command Encoder 作成
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });
    
    // 3. Render Pass
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        });
        
        // Pipeline & Bind Group 設定
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        
        // Vertex Buffer 設定
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        
        // 描画
        render_pass.draw(0..self.num_vertices, 0..1);
    }
    
    // 4. Command 送信
    self.queue.submit(std::iter::once(encoder.finish()));
    
    // 5. Present
    output.present();
    
    Ok(())
}
```

---

### 5.2 動的 Vertex Buffer

```rust
fn update_vertices(&mut self, vertices: &[Vertex]) {
    // Buffer を再作成
    self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    self.num_vertices = vertices.len() as u32;
}

// または write_buffer で更新
fn update_vertices_fast(&mut self, vertices: &[Vertex]) {
    self.queue.write_buffer(
        &self.vertex_buffer,
        0,
        bytemuck::cast_slice(vertices)
    );
}
```

---

## 6. Advanced Techniques

### 6.1 Index Buffer

```rust
let indices: &[u16] = &[
    0, 1, 2,  // 三角形1
    2, 3, 0,  // 三角形2
];

let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Index Buffer"),
    contents: bytemuck::cast_slice(indices),
    usage: wgpu::BufferUsages::INDEX,
});

// 描画
render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
```

---

### 6.2 Instancing

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    position: [f32; 2],
    scale: f32,
}

let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Instance Buffer"),
    contents: bytemuck::cast_slice(&instance_data),
    usage: wgpu::BufferUsages::VERTEX,
});

// Vertex Buffer Layout に追加
wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Instance, // Instance ごと
    attributes: &[...],
}

// 描画
render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
render_pass.draw(0..num_vertices, 0..num_instances);
```

---

### 6.3 Compute Shader

```wgsl
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    data[id.x] = data[id.x] * 2.0;
}
```

```rust
let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("Compute Pipeline"),
    layout: Some(&compute_pipeline_layout),
    module: &shader,
    entry_point: "main",
});

let mut encoder = device.create_command_encoder(&Default::default());
{
    let mut compute_pass = encoder.begin_compute_pass(&Default::default());
    compute_pass.set_pipeline(&compute_pipeline);
    compute_pass.set_bind_group(0, &bind_group, &[]);
    compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
}
queue.submit(std::iter::once(encoder.finish()));
```

---

## 📊 wgpu チートシート

| 操作 | コード |
|------|--------|
| **Buffer 作成** | `device.create_buffer_init(...)` |
| **Texture 作成** | `device.create_texture(...)` |
| **Shader ロード** | `device.create_shader_module(...)` |
| **Pipeline 作成** | `device.create_render_pipeline(...)` |
| **描画** | `render_pass.draw(...)` |
| **Buffer 更新** | `queue.write_buffer(...)` |

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
