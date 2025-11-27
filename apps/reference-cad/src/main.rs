use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::sync::Arc;

mod app;
use app::App;

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Rust CAD Framework Reference App")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap());
    
    let mut app = pollster::block_on(App::new(window.clone()));
    
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.window.id() => {
                if !app.input(event) {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            app.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            // In winit 0.29, inner_size_writer handles resize automatically or we just ignore
                            // Actually we should use the suggested size if provided, but for now let's just trigger resize on next Resized event which usually follows
                        }
                        WindowEvent::RedrawRequested => {
                            app.update();
                            match app.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => app.resize(app.size()),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        // Handle Mouse Input with Snapping
                        WindowEvent::CursorMoved { position, .. } => {
                            if !app.egui_ctx.is_pointer_over_area() {
                                let screen_pos = cgmath::Vector2::new(position.x as f32, position.y as f32);
                                let world_vec = app.renderer.camera.screen_to_world(
                                    screen_pos,
                                    app.renderer.size.width as f32,
                                    app.renderer.size.height as f32
                                );
                                let mut point = cad_core::Point::new(world_vec.x, world_vec.y);
                                
                                // Snapping Logic
                                let visible_entities = app.document.lock().get_visible_entities();
                                let snap_threshold = 10.0 / app.renderer.camera.zoom;
                                let snap_point = cad_core::find_closest_snap_point(&visible_entities, point, snap_threshold);
                                
                                if let Some(snap) = snap_point {
                                    point = snap.point;
                                    app.active_snap_point = Some(snap.point);
                                } else {
                                    app.active_snap_point = None;
                                }

                                app.last_mouse_pos = Some(*position);
                                if app.is_panning {
                                    // Pan logic would need last_mouse_pos before update
                                    // But here we just update last_mouse_pos
                                    // Actual pan logic is usually in DeviceEvent or separate
                                    // Let's keep it simple: if panning, we use raw delta
                                } else {
                                    if let Some(action) = app.active_tool.mouse_move(point) {
                                        app.update_geometry_with_preview();
                                    }
                                }
                            }
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            if !app.egui_ctx.is_pointer_over_area() {
                                if *state == ElementState::Pressed && *button == MouseButton::Left {
                                    let position = app.last_mouse_pos.unwrap_or(winit::dpi::PhysicalPosition::new(0.0, 0.0));
                                    let screen_pos = cgmath::Vector2::new(position.x as f32, position.y as f32);
                                    let world_vec = app.renderer.camera.screen_to_world(
                                        screen_pos,
                                        app.renderer.size.width as f32,
                                        app.renderer.size.height as f32
                                    );
                                    let mut point = cad_core::Point::new(world_vec.x, world_vec.y);
                                    
                                    // Snapping Logic (Reuse active_snap_point if valid?)
                                    // Or re-calculate to be safe
                                    if let Some(snap_pos) = app.active_snap_point {
                                        point = snap_pos;
                                    }

                                    if let Some(action) = app.active_tool.mouse_down(point) {
                                        match action {
                                            cad_tools::ToolAction::Commit(entity) => {
                                                let command = Box::new(cad_tools::AddEntityCommand::new(entity));
                                                app.command_manager.execute(command, &mut app.document.lock());
                                                app.update_geometry_with_preview();
                                            }
                                            _ => {
                                                app.update_geometry_with_preview();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
