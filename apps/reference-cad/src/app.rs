use cad_core::{Entity, Point, Document, CommandManager};
use cad_tools::{Tool, LineTool, ToolAction};
use cad_rendering::Renderer;
use cad_ui::{ToolPalette, UIAction};
use cad_io::{CADSerializer, JSONSerializer};
use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    window::Window,
};
use egui_wgpu::ScreenDescriptor;

use std::sync::Arc;

use cad_agent_interface::{AgentState, AgentCommand};
use parking_lot::Mutex;

pub struct App {
    pub window: Arc<Window>,
    pub renderer: Renderer,
    pub document: Arc<Mutex<Document>>, // Changed to Arc<Mutex<>> for sharing
    pub command_manager: CommandManager,
    pub is_panning: bool,
    pub last_mouse_pos: Option<winit::dpi::PhysicalPosition<f64>>,
    pub active_tool: Box<dyn Tool>,
    pub active_snap_point: Option<Point>,
    
    // Agent Interface
    pub agent_command_receiver: tokio::sync::mpsc::UnboundedReceiver<AgentCommand>,
    
    // UI
    pub egui_ctx: egui::Context,
    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,
    pub tool_palette: ToolPalette,
}

impl App {
    pub async fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(&window).await;
        
        let document = Arc::new(Mutex::new(Document::new()));
        let command_manager = CommandManager::new();

        // Agent Interface Setup
        let (agent_sender, agent_receiver) = tokio::sync::mpsc::unbounded_channel();
        let agent_state = AgentState {
            document: document.clone(),
            command_sender: agent_sender,
        };
        
        // Spawn Agent Server
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(cad_agent_interface::start_server(agent_state));
        });

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
            document,
            command_manager,
            is_panning: false,
            last_mouse_pos: None,
            active_tool: Box::new(LineTool::new()),
            active_snap_point: None,
            agent_command_receiver: agent_receiver,
            egui_ctx,
            egui_state,
            egui_renderer,
            tool_palette: ToolPalette::new(),
        };
        
        app.renderer.update_geometry(&app.document.lock().get_visible_entities());
        app
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // 1. Pass event to UI
        let response = self.egui_state.on_window_event(&self.window, event);
        if response.consumed {
            return true;
        }

        // Handle Panning (Middle Mouse / Space + Drag)
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Middle {
                    self.is_panning = *state == ElementState::Pressed;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.is_panning {
                    if let Some(last_pos) = self.last_mouse_pos {
                        let dx = position.x - last_pos.x;
                        let dy = position.y - last_pos.y;
                        self.renderer.camera.pan(dx as f32, dy as f32);
                    }
                }
                self.last_mouse_pos = Some(*position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let zoom_factor = match delta {
                    MouseScrollDelta::LineDelta(_, y) => 1.0 - y * 0.1,
                    MouseScrollDelta::PixelDelta(pos) => 1.0 - pos.y as f32 * 0.001,
                };
                self.renderer.camera.zoom(zoom_factor);
            }
            _ => {}
        }

        false
    }

    pub fn update_geometry_with_preview(&mut self) {
        let mut display_entities = self.document.lock().get_visible_entities();
        if let Some(preview) = self.active_tool.get_preview() {
            display_entities.push(preview);
        }
        // Render Snap Indicator
        if let Some(snap_pos) = self.active_snap_point {
            // Draw a small circle for snap point
            display_entities.push(Entity::Circle { center: snap_pos, radius: 5.0 / self.renderer.camera.zoom });
        }
        self.renderer.update_geometry(&display_entities);
    }

    pub fn update(&mut self) {
        // Handle Agent Commands
        while let Ok(command) = self.agent_command_receiver.try_recv() {
            match command {
                AgentCommand::Undo => {
                    self.command_manager.undo(&mut self.document.lock());
                    self.renderer.update_geometry(&self.document.lock().get_visible_entities());
                }
                AgentCommand::Redo => {
                    self.command_manager.redo(&mut self.document.lock());
                    self.renderer.update_geometry(&self.document.lock().get_visible_entities());
                }
                AgentCommand::SelectTool(name) => {
                    if name == "Line" {
                        self.active_tool = Box::new(LineTool::new());
                        self.tool_palette.selected_tool = "Line".to_string();
                    } else if name == "Circle" {
                        // Circle tool not implemented yet, but switch logic here
                        self.tool_palette.selected_tool = "Circle".to_string();
                    }
                }
                AgentCommand::DrawLine { x1, y1, x2, y2 } => {
                    // Direct manipulation for testing
                    use cad_core::{Entity, Point};
                    let line = Entity::Line {
                        p1: Point::new(x1, y1),
                        p2: Point::new(x2, y2),
                    };
                    let command = Box::new(cad_tools::AddEntityCommand::new(line));
                    self.command_manager.execute(command, &mut self.document.lock());
                    self.renderer.update_geometry(&self.document.lock().get_visible_entities());
                }
            }
        }
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
        
        let action = self.tool_palette.show(&self.egui_ctx);
        cad_ui::LayerManager::show(&self.egui_ctx, &mut self.document.lock());
        
        match action {
            UIAction::SelectTool(name) => {
                if name == "Line" {
                    self.active_tool = Box::new(LineTool::new());
                }
            }
            UIAction::Save => {
                let serializer = JSONSerializer;
                // Saving visible entities for now. Ideally we save the whole Document structure.
                // But CADSerializer expects &[Entity].
                // We should update CADSerializer to support Document or save flattened list.
                // For backward compatibility with current serializer, saving flattened list.
                if let Err(e) = serializer.save(&self.document.lock().get_visible_entities(), std::path::Path::new("drawing.json")) {
                    eprintln!("Failed to save: {}", e);
                } else {
                    println!("Saved to drawing.json");
                }
            }
            UIAction::Load => {
                let serializer = JSONSerializer;
                match serializer.load(std::path::Path::new("drawing.json")) {
                    Ok(loaded_entities) => {
                        // Loading flat entities into a new document structure
                        // For now, put them all in the default layer
                        let mut doc = self.document.lock();
                        *doc = Document::new();
                        if let Some(layer) = doc.layers.get_mut(0) {
                            layer.entities = loaded_entities;
                        }
                        self.renderer.update_geometry(&doc.get_visible_entities());
                        println!("Loaded from drawing.json");
                    }
                    Err(e) => eprintln!("Failed to load: {}", e),
                }
            }
            UIAction::Undo => {
                let mut doc = self.document.lock();
                self.command_manager.undo(&mut doc);
                self.renderer.update_geometry(&doc.get_visible_entities());
            }
            UIAction::Redo => {
                let mut doc = self.document.lock();
                self.command_manager.redo(&mut doc);
                self.renderer.update_geometry(&doc.get_visible_entities());
            }
            UIAction::None => {}
        }
        
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
