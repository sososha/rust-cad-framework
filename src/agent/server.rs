use axum::{
    routing::{get, post},
    Router,
    extract::State,
    Json,
};
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use crate::app::AppState;

// サーバー起動関数
pub async fn run_server(app_state: Arc<Mutex<AppState>>) {
    let app = Router::new()
        .route("/", get(root))
        .route("/api/command", post(handle_command))
        .route("/api/query", get(handle_query))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Rust CAD Agent Interface is Running!"
}

// --- Command Handling ---

#[derive(Deserialize)]
struct CommandRequest {
    action: String,
    args: serde_json::Value,
}

#[derive(Serialize)]
struct CommandResponse {
    status: String,
    message: String,
}

async fn handle_command(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<CommandRequest>,
) -> Json<CommandResponse> {
    let mut state = state.lock();
    
    match payload.action.as_str() {
        "set_tool" => {
            if let Some(tool_name) = payload.args.get("name").and_then(|v| v.as_str()) {
                state.tool_manager.set_active_tool(tool_name);
                return Json(CommandResponse { status: "ok".into(), message: format!("Tool set to {}", tool_name) });
            }
        }
        "draw_line" => {
            // エージェントが直接データを注入する例
            // 本来はTool経由でイベントを発火させるのが筋だが、ショートカットも可能
            let start = payload.args.get("start").and_then(|v| v.as_array());
            let end = payload.args.get("end").and_then(|v| v.as_array());
            
            if let (Some(_s), Some(_e)) = (start, end) {
                // state.geometry.add_line(...)
                return Json(CommandResponse { status: "ok".into(), message: "Line added".into() });
            }
        }
        _ => {}
    }

    Json(CommandResponse { status: "error".into(), message: "Unknown command".into() })
}

// --- Query Handling ---

#[derive(Serialize)]
struct QueryResponse {
    entity_count: usize,
    camera_zoom: f32,
}

async fn handle_query(State(state): State<Arc<Mutex<AppState>>>) -> Json<QueryResponse> {
    let state = state.lock();
    Json(QueryResponse {
        entity_count: state.geometry.entities.len(),
        camera_zoom: state.camera.zoom,
    })
}
