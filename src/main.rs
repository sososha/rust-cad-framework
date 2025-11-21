use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use std::sync::Arc;

mod app;
mod canvas;
mod geometry;
mod tools;
mod agent;

use app::CadApp;

#[tokio::main]
async fn main() {
    // ログ出力の初期化 (必要なら)
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Rust CAD Framework")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)
        .unwrap());

    // アプリケーションの初期化
    let mut app = CadApp::new(window.clone()).await;

    // エージェントサーバーの起動 (Featureフラグで制御)
    #[cfg(feature = "agent")]
    {
        let app_handle = app.handle();
        tokio::spawn(async move {
            agent::server::run_server(app_handle).await;
        });
        println!("Agent server listening on http://localhost:9000");
    }

    // イベントループ
    event_loop.run(move |event, target| {
        // エージェントからの入力があれば処理する (簡易的な実装)
        // 実際にはUserEventを使うのが良いが、今回はpollで対応
        
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !app.handle_window_event(event) {
                    match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::Resized(physical_size) => {
                            app.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            app.update();
                            match app.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => app.resize(app.size()),
                                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                                Err(e) => eprintln!("{:?}", e),
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
