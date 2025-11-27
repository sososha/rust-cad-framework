use axum::{
    routing::{get, post},
    Router, Json, extract::State,
};
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use cad_core::Document;

// Shared state between App and Agent Server
// For simplicity, we might need to share the whole App state, but App is not thread-safe easily.
// We will use a shared Document for query, and a command queue for commands.
// Or we can use a channel to send commands to the main thread.
// For Query, we need read access to Document.
// Let's define a SharedState struct.

#[derive(Clone)]
pub struct AgentState {
    pub document: Arc<Mutex<Document>>,
    pub command_sender: tokio::sync::mpsc::UnboundedSender<AgentCommand>,
}

#[derive(Debug, Deserialize)]
pub enum AgentCommand {
    // Define commands that agent can send
    SelectTool(String),
    DrawLine { x1: f32, y1: f32, x2: f32, y2: f32 },
    Undo,
    Redo,
}

pub async fn start_server(state: AgentState) {
    let app = Router::new()
        .route("/query", get(handle_query))
        .route("/command", post(handle_command))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000").await.unwrap();
    println!("Agent Server listening on 127.0.0.1:9000");
    axum::serve(listener, app).await.unwrap();
}

async fn handle_query(State(state): State<AgentState>) -> Json<Document> {
    let doc = state.document.lock();
    Json(doc.clone())
}

#[derive(Deserialize)]
struct CommandRequest {
    command: String,
    args: Option<serde_json::Value>,
}

async fn handle_command(State(state): State<AgentState>, Json(payload): Json<CommandRequest>) -> Json<String> {
    let cmd = match payload.command.as_str() {
        "undo" => AgentCommand::Undo,
        "redo" => AgentCommand::Redo,
        "select_tool" => {
            let tool_name = payload.args.and_then(|v| v.get("name").and_then(|s| s.as_str()).map(|s| s.to_string())).unwrap_or_default();
            AgentCommand::SelectTool(tool_name)
        },
        "draw_line" => {
            // Parse args for coordinates
            // Simplified for now
            AgentCommand::DrawLine { x1: 0.0, y1: 0.0, x2: 100.0, y2: 100.0 } 
        },
        _ => return Json("Unknown command".to_string()),
    };

    state.command_sender.send(cmd).unwrap();
    Json("Command sent".to_string())
}
