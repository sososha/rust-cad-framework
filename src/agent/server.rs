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
            let start = payload.args.get("start").and_then(|v| v.as_array());
            let end = payload.args.get("end").and_then(|v| v.as_array());
            
            if let (Some(s), Some(e)) = (start, end) {
                if s.len() == 2 && e.len() == 2 {
                    let start_point = crate::geometry::primitives::Point::new(s[0].as_f64().unwrap() as f32, s[1].as_f64().unwrap() as f32);
                    let end_point = crate::geometry::primitives::Point::new(e[0].as_f64().unwrap() as f32, e[1].as_f64().unwrap() as f32);
                    state.geometry.add_line(start_point, end_point);
                    return Json(CommandResponse { status: "ok".into(), message: "Line added".into() });
                }
            }
        }
        "draw_circle" => {
            let center = payload.args.get("center").and_then(|v| v.as_array());
            let radius = payload.args.get("radius").and_then(|v| v.as_f64());
            
            if let (Some(c), Some(r)) = (center, radius) {
                if c.len() == 2 {
                     let center_point = crate::geometry::primitives::Point::new(c[0].as_f64().unwrap() as f32, c[1].as_f64().unwrap() as f32);
                     state.geometry.entities.push(crate::geometry::primitives::Entity::Circle {
                         center: center_point,
                         radius: r as f32,
                     });
                     return Json(CommandResponse { status: "ok".into(), message: "Circle added".into() });
                }
            }
        }
        "draw_rect" => {
             let p1 = payload.args.get("p1").and_then(|v| v.as_array());
             let p2 = payload.args.get("p2").and_then(|v| v.as_array());
             
             if let (Some(s), Some(e)) = (p1, p2) {
                 if s.len() == 2 && e.len() == 2 {
                     let p1_point = crate::geometry::primitives::Point::new(s[0].as_f64().unwrap() as f32, s[1].as_f64().unwrap() as f32);
                     let p2_point = crate::geometry::primitives::Point::new(e[0].as_f64().unwrap() as f32, e[1].as_f64().unwrap() as f32);
                     state.geometry.entities.push(crate::geometry::primitives::Entity::Rect {
                         p1: p1_point,
                         p2: p2_point,
                     });
                     return Json(CommandResponse { status: "ok".into(), message: "Rect added".into() });
                 }
             }
        }
        "draw_double_line" => {
            let start = payload.args.get("start").and_then(|v| v.as_array());
            let end = payload.args.get("end").and_then(|v| v.as_array());
            let width = payload.args.get("width").and_then(|v| v.as_f64()).unwrap_or(20.0) as f32;
            
            if let (Some(s), Some(e)) = (start, end) {
                if s.len() == 2 && e.len() == 2 {
                    let start_point = crate::geometry::primitives::Point::new(s[0].as_f64().unwrap() as f32, s[1].as_f64().unwrap() as f32);
                    let end_point = crate::geometry::primitives::Point::new(e[0].as_f64().unwrap() as f32, e[1].as_f64().unwrap() as f32);
                    
                    let dir = end_point.sub(start_point);
                    let normal = dir.normal().normalize();
                    let offset = normal.scale(width / 2.0);
                    
                    let p1_start = start_point.add(offset);
                    let p1_end = end_point.add(offset);
                    let p2_start = start_point.sub(offset);
                    let p2_end = end_point.sub(offset);
                    
                    state.geometry.entities.push(crate::geometry::primitives::Entity::Line { start: p1_start, end: p1_end });
                    state.geometry.entities.push(crate::geometry::primitives::Entity::Line { start: p2_start, end: p2_end });
                    
                    return Json(CommandResponse { status: "ok".into(), message: "Double Line added".into() });
                }
            }
        }
        "offset_entity" => {
            let id = payload.args.get("id").and_then(|v| v.as_u64()).map(|v| v as usize);
            let distance = payload.args.get("distance").and_then(|v| v.as_f64()).unwrap_or(20.0) as f32;
            let direction = payload.args.get("direction").and_then(|v| v.as_array());
            
            if let Some(idx) = id {
                 if let Some(crate::geometry::primitives::Entity::Line { start, end }) = state.geometry.entities.get(idx) {
                     let ab = end.sub(*start);
                     let n = ab.normal().normalize();
                     
                     let offset_vec = if let Some(d) = direction {
                         if d.len() == 2 {
                             let dir_p = crate::geometry::primitives::Point::new(d[0].as_f64().unwrap() as f32, d[1].as_f64().unwrap() as f32);
                             let ap = dir_p.sub(*start);
                             let sign = if ap.x * n.x + ap.y * n.y > 0.0 { 1.0 } else { -1.0 };
                             n.scale(distance * sign)
                         } else {
                             n.scale(distance)
                         }
                     } else {
                         n.scale(distance)
                     };
                     
                     let new_start = start.add(offset_vec);
                     let new_end = end.add(offset_vec);
                     state.geometry.entities.push(crate::geometry::primitives::Entity::Line { start: new_start, end: new_end });
                     return Json(CommandResponse { status: "ok".into(), message: "Entity offset".into() });
                 }
            }
        }
        "delete_entity" => {
            let id = payload.args.get("id").and_then(|v| v.as_u64()).map(|v| v as usize);
            
            if let Some(idx) = id {
                if idx < state.geometry.entities.len() {
                    state.geometry.entities.remove(idx);
                    return Json(CommandResponse { status: "ok".into(), message: "Entity deleted".into() });
                } else {
                    return Json(CommandResponse { status: "error".into(), message: "Index out of bounds".into() });
                }
            }
        }
        "draw_polyline" => {
            let points_arg = payload.args.get("points").and_then(|v| v.as_array());
            
            if let Some(pts) = points_arg {
                let mut points = Vec::new();
                for p in pts {
                    if let Some(arr) = p.as_array() {
                        if arr.len() == 2 {
                            let x = arr[0].as_f64().unwrap_or(0.0) as f32;
                            let y = arr[1].as_f64().unwrap_or(0.0) as f32;
                            points.push(crate::geometry::primitives::Point::new(x, y));
                        }
                    }
                }
                
                if points.len() >= 2 {
                    state.geometry.entities.push(crate::geometry::primitives::Entity::Polyline { points });
                    return Json(CommandResponse { status: "ok".into(), message: "Polyline added".into() });
                }
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
