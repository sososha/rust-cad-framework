use cad_tools::{Tool, LineTool, ToolAction};
use cad_core::{Entity, Point};
use cad_rendering::Renderer;
use cad_ui::{ToolPalette, UIComponent};
use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    window::Window,
};
use egui_wgpu::ScreenDescriptor;

use std::sync::Arc;

pub struct App {
    window: Arc<Window>,
    renderer: Renderer,
    entities: Vec<Entity>,
    is_panning: bool,
    last_mouse_pos: Option<winit::dpi::PhysicalPosition<f64>>,
    active_tool: Box<dyn Tool>,
    
    // UI
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    tool_palette: ToolPalette,
}

impl App {
    pub async fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(&window).await;
        
        let entities = Vec::new();

        // UI Setup
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );
        
        let egui_renderer = egui_wgpu::Renderer::new(
            &renderer.device,
            renderer.config.format,
            None,
            1,
        );

        let mut app = Self {
            window,
            renderer,
            entities,
            is_panning: false,
            last_mouse_pos: None,
            active_tool: Box::new(LineTool::new()),
            egui_ctx,
            egui_state,
            egui_renderer,
            tool_palette: ToolPalette::new(),
        };
        
        app.renderer.update_geometry(&app.entities);
        app
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let _ = self.egui_state.on_window_event(&self.window, event);
        
        if self.egui_ctx.wants_pointer_input() || self.egui_ctx.wants_keyboard_input() {
            return true; 
        }

        match event {
            WindowEvent::MouseWheel { delta, .. } => {
                let zoom_factor = match delta {
                    MouseScrollDelta::LineDelta(_, y) => 1.0 + y * 0.1,
                    MouseScrollDelta::PixelDelta(pos) => 1.0 + pos.y as f32 * 0.001,
                };
                self.renderer.camera.zoom *= zoom_factor;
                self.renderer.camera.zoom = self.renderer.camera.zoom.max(0.1).min(100.0);
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Middle {
                    self.is_panning = *state == ElementState::Pressed;
                    return true;
                }
                
                if *button == MouseButton::Left && *state == ElementState::Pressed {
                    if let Some(pos) = self.last_mouse_pos {
                        let world_pos = self.renderer.camera.screen_to_world(
                            cgmath::Vector2::new(pos.x as f32, pos.y as f32),
                            self.renderer.size.width as f32,
                            self.renderer.size.height as f32
                        );
                        let point = Point::new(world_pos.x, world_pos.y);
                        
                        if let Some(action) = self.active_tool.mouse_down(point) {
                            match action {
                                ToolAction::Commit(entity) => {
                                    self.entities.push(entity);
                                    self.update_geometry_with_preview();
                                }
                                _ => {
                                    self.update_geometry_with_preview();
                                }
                            }
                        }
                    }
                    return true;
                }
                false
            }
            WindowEvent::CursorMoved { position, .. } => {
                let current_pos = *position;
                if self.is_panning {
                    if let Some(last_pos) = self.last_mouse_pos {
                        let dx = (current_pos.x - last_pos.x) as f32;
                        let dy = (current_pos.y - last_pos.y) as f32;
                        
                        let zoom = self.renderer.camera.zoom;
                        let pan_speed = 2.0 / self.renderer.size.height as f32 / zoom;
                        
                        self.renderer.camera.pan.x += dx * pan_speed;
                        self.renderer.camera.pan.y -= dy * pan_speed;
                    }
                } else {
                    // Tool Hover
                    let world_pos = self.renderer.camera.screen_to_world(
                        cgmath::Vector2::new(current_pos.x as f32, current_pos.y as f32),
                        self.renderer.size.width as f32,
                        self.renderer.size.height as f32
                    );
                    let point = Point::new(world_pos.x, world_pos.y);
                    self.active_tool.mouse_move(point);
                    self.update_geometry_with_preview();
                }
                self.last_mouse_pos = Some(current_pos);
                true
            }
            _ => false,
        }
    }

    fn update_geometry_with_preview(&mut self) {
        let mut display_entities = self.entities.clone();
        if let Some(preview) = self.active_tool.get_preview() {
            display_entities.push(preview);
        }
        self.renderer.update_geometry(&display_entities);
    }

    pub fn update(&mut self) {
        // Update logic here
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.renderer.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 1. Render Scene
        let mut encoder = self.renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // We need to split render_scene to take encoder or just call it.
        // Renderer::render_scene was not implemented yet, I only had render().
        // Let's assume I need to call renderer.render_pass(&view, &mut encoder)
        // But wait, renderer.render() does everything including present.
        // I need to refactor renderer to separate rendering from presentation if I want to compose.
        // Or I can just do everything here since I have access to renderer fields (they are public or I can make them public).
        // Actually renderer fields are not all public.
        // Let's modify Renderer to expose a render_to_view method or similar.
        // For now, let's just copy the render logic here since I have access to most things?
        // No, renderer fields are private in crate (except the ones I made public).
        // I should add a method to Renderer to render the scene to a view using an encoder.
        
        // Let's assume I added `render_scene` to Renderer. I will add it next.
        self.renderer.render_scene(&view, &mut encoder);

        // 2. Render UI
        let raw_input = self.egui_state.take_egui_input(&self.window);
        self.egui_ctx.begin_frame(raw_input);
        
        self.tool_palette.show(&self.egui_ctx);
        
        // Tool Switching Logic
        if self.tool_palette.selected_tool == "Line" && self.active_tool.name() != "Line" {
             self.active_tool = Box::new(LineTool::new());
        }
        // Add Circle tool switching when implemented
        
        let full_output = self.egui_ctx.end_frame();
        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, self.egui_ctx.pixels_per_point());
        
        self.egui_state.handle_platform_output(&self.window, full_output.platform_output);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.renderer.config.width, self.renderer.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.renderer.device, &self.renderer.queue, *id, image_delta);
        }
        
        self.egui_renderer.update_buffers(
            &self.renderer.device,
            &self.renderer.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Load the scene
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            self.egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }
        
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.renderer.size
    }
}
