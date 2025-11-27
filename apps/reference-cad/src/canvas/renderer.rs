use winit::window::Window;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use crate::app::AppState;
use crate::geometry::primitives::Entity;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window).unwrap();
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
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
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // シェーダーとパイプラインの作成 (簡易版)
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

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
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    }
                ],
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
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size: winit::dpi::PhysicalSize::new(width, height),
            render_pipeline,
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

    pub fn render(&mut self, state: &AppState) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // 頂点データの構築 (本来はバッファをキャッシュすべきだが、簡単のため毎フレーム再構築)
        let mut vertices: Vec<[f32; 2]> = Vec::new();
        
        // 座標変換: World -> Screen -> NDC (-1.0 ~ 1.0)
        // ここでは簡易的にNDC変換をCPUで行う (本来はVertex ShaderでUniform Bufferを使う)
        let transform = |p: crate::geometry::primitives::Point| -> [f32; 2] {
            // 簡易実装: ズームとパンを適用してNDCへ
            let x = (p.x - state.camera.position.x) * state.camera.zoom;
            let y = (p.y - state.camera.position.y) * state.camera.zoom;
            
            // Screen (0~W, 0~H) -> NDC (-1~1, 1~-1)
            let ndc_x = (x / self.size.width as f32) * 2.0 - 1.0;
            let ndc_y = -((y / self.size.height as f32) * 2.0 - 1.0); // Y-flip
            [ndc_x, ndc_y]
        };

        // --- Grid Rendering ---
        let grid_size = state.grid_size;
        let (left, top) = state.camera.screen_to_world(0.0, 0.0);
        let (right, bottom) = state.camera.screen_to_world(self.size.width as f32, self.size.height as f32);
        
        let start_x = (left / grid_size).floor() as i32;
        let end_x = (right / grid_size).ceil() as i32;
        let start_y = (top / grid_size).floor() as i32;
        let end_y = (bottom / grid_size).ceil() as i32;

        for i in start_x..=end_x {
            let x = i as f32 * grid_size;
            vertices.push(transform(crate::geometry::primitives::Point::new(x, top)));
            vertices.push(transform(crate::geometry::primitives::Point::new(x, bottom)));
        }
        for i in start_y..=end_y {
            let y = i as f32 * grid_size;
            vertices.push(transform(crate::geometry::primitives::Point::new(left, y)));
            vertices.push(transform(crate::geometry::primitives::Point::new(right, y)));
        }

        // エンティティ描画用クロージャ
        let mut add_entity = |entity: &Entity| {
            match entity {
                Entity::Line { start, end } => {
                    vertices.push(transform(*start));
                    vertices.push(transform(*end));
                }
                Entity::Rect { p1, p2 } => {
                    // 4本の線分を描画
                    let p3 = crate::geometry::primitives::Point::new(p2.x, p1.y);
                    let p4 = crate::geometry::primitives::Point::new(p1.x, p2.y);
                    
                    vertices.push(transform(*p1)); vertices.push(transform(p3));
                    vertices.push(transform(p3)); vertices.push(transform(*p2));
                    vertices.push(transform(*p2)); vertices.push(transform(p4));
                    vertices.push(transform(p4)); vertices.push(transform(*p1));
                }
                Entity::Circle { center, radius } => {
                    // 円を線分近似
                    let segments = 64;
                    for i in 0..segments {
                        let theta1 = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
                        let theta2 = ((i + 1) as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
                        
                        let p1 = crate::geometry::primitives::Point::new(
                            center.x + radius * theta1.cos(),
                            center.y + radius * theta1.sin(),
                        );
                        let p2 = crate::geometry::primitives::Point::new(
                            center.x + radius * theta2.cos(),
                            center.y + radius * theta2.sin(),
                        );
                        vertices.push(transform(p1));
                        vertices.push(transform(p2));
                    }
                }
                Entity::Polyline { points } => {
                    for i in 0..points.len().saturating_sub(1) {
                        vertices.push(transform(points[i]));
                        vertices.push(transform(points[i+1]));
                    }
                }
            }
        };

        // 確定したエンティティ
        for entity in &state.geometry.entities {
            add_entity(entity);
        }

        // プレビュー中のエンティティ
        for entity in &state.geometry.temp_entities {
            add_entity(entity);
        }

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1, g: 0.1, b: 0.1, a: 1.0, // Dark Gray Background
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !vertices.is_empty() {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..vertices.len() as u32, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
}
