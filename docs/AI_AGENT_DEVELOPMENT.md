# AI Agent Testing & Development Guide

> **対象**: AIエージェントを使ってCADを開発・テストする開発者
> 
> **目的**: Agent API による自動テストとAI駆動開発の完全ガイド

---

## 📚 Table of Contents
1. [Agent API Architecture](#1-agent-api-architecture)
2. [Automated Testing with AI](#2-automated-testing-with-ai)
3. [AI-Driven Development Workflow](#3-ai-driven-development-workflow)
4. [Prompt Engineering for CAD](#4-prompt-engineering-for-cad)
5. [Complete Implementation](#5-complete-implementation)

---

## 1. Agent API Architecture

### 1.1 なぜ Agent API が必要か

**問題**: デスクトップアプリは GUI 自動テストが困難
- マウス座標が環境依存
- ウィンドウ位置が不安定
- スクリーンショット比較は脆弱

**解決**: HTTP API でプログラマティックに操作
```
AI Agent → HTTP Request → CAD Server → CAD Core
                                      ↓
                                   Response
```

---

### 1.2 Agent Server 実装

```rust
use axum::{
    routing::{get, post},
    Router, Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// アプリケーション状態（スレッド間で共有）
pub struct AppState {
    pub geometry: GeometryStore,
    pub camera: Camera,
    pub tool_manager: ToolManager,
}

pub type SharedState = Arc<Mutex<AppState>>;

// Agent Server
pub struct AgentServer {
    state: SharedState,
}

impl AgentServer {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
    
    pub async fn run(self, port: u16) {
        let app = Router::new()
            .route("/api/health", get(health_check))
            .route("/api/command", post(execute_command))
            .route("/api/state", get(get_state))
            .route("/api/entities", get(list_entities))
            .route("/api/screenshot", get(take_screenshot))
            .with_state(self.state);
        
        let addr = format!("127.0.0.1:{}", port);
        println!("Agent API listening on http://{}", addr);
        
        axum::Server::bind(&addr.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

// Health Check
async fn health_check() -> &'static str {
    "OK"
}

// Command Execution
#[derive(Deserialize)]
struct CommandRequest {
    action: String,
    args: serde_json::Value,
}

#[derive(Serialize)]
struct CommandResponse {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
}

async fn execute_command(
    axum::extract::State(state): axum::extract::State<SharedState>,
    Json(payload): Json<CommandRequest>,
) -> Json<CommandResponse> {
    let mut state = state.lock().unwrap();
    
    let result = match payload.action.as_str() {
        "draw_line" => {
            let start: [f32; 2] = serde_json::from_value(payload.args["start"].clone()).unwrap();
            let end: [f32; 2] = serde_json::from_value(payload.args["end"].clone()).unwrap();
            
            let entity = Entity::Line {
                p1: Point::new(start[0], start[1]),
                p2: Point::new(end[0], end[1]),
            };
            
            let id = state.geometry.add_entity(entity);
            
            CommandResponse {
                success: true,
                message: "Line created".to_string(),
                data: Some(serde_json::json!({ "id": id.data().as_ffi() })),
            }
        }
        "draw_circle" => {
            let center: [f32; 2] = serde_json::from_value(payload.args["center"].clone()).unwrap();
            let radius: f32 = serde_json::from_value(payload.args["radius"].clone()).unwrap();
            
            let entity = Entity::Circle {
                center: Point::new(center[0], center[1]),
                radius,
            };
            
            let id = state.geometry.add_entity(entity);
            
            CommandResponse {
                success: true,
                message: "Circle created".to_string(),
                data: Some(serde_json::json!({ "id": id.data().as_ffi() })),
            }
        }
        "delete_entity" => {
            let id_data: u64 = serde_json::from_value(payload.args["id"].clone()).unwrap();
            let id = EntityId::from(slotmap::KeyData::from_ffi(id_data));
            
            if state.geometry.remove_entity(id).is_some() {
                CommandResponse {
                    success: true,
                    message: "Entity deleted".to_string(),
                    data: None,
                }
            } else {
                CommandResponse {
                    success: false,
                    message: "Entity not found".to_string(),
                    data: None,
                }
            }
        }
        "set_tool" => {
            let tool_name: String = serde_json::from_value(payload.args["tool"].clone()).unwrap();
            state.tool_manager.set_tool(&tool_name);
            
            CommandResponse {
                success: true,
                message: format!("Tool set to {}", tool_name),
                data: None,
            }
        }
        _ => CommandResponse {
            success: false,
            message: format!("Unknown action: {}", payload.action),
            data: None,
        }
    };
    
    Json(result)
}

// Get State
#[derive(Serialize)]
struct StateResponse {
    entity_count: usize,
    camera_position: [f32; 2],
    active_tool: String,
}

async fn get_state(
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> Json<StateResponse> {
    let state = state.lock().unwrap();
    
    Json(StateResponse {
        entity_count: state.geometry.entity_count(),
        camera_position: [state.camera.position.x, state.camera.position.y],
        active_tool: state.tool_manager.active_tool_name(),
    })
}

// List Entities
#[derive(Serialize)]
struct EntityInfo {
    id: u64,
    entity_type: String,
    data: serde_json::Value,
}

async fn list_entities(
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> Json<Vec<EntityInfo>> {
    let state = state.lock().unwrap();
    
    let entities: Vec<EntityInfo> = state.geometry.entities.iter()
        .map(|(id, entity)| {
            EntityInfo {
                id: id.data().as_ffi(),
                entity_type: entity.type_name().to_string(),
                data: entity.to_json(),
            }
        })
        .collect();
    
    Json(entities)
}
```

---

## 2. Automated Testing with AI

### 2.1 AI テストエージェントの実装

```python
# test_agent.py
import requests
import json
from typing import List, Dict, Any

class CADTestAgent:
    def __init__(self, base_url: str = "http://localhost:9000"):
        self.base_url = base_url
        self.session = requests.Session()
    
    def health_check(self) -> bool:
        """CADアプリが起動しているか確認"""
        try:
            response = self.session.get(f"{self.base_url}/api/health")
            return response.status_code == 200
        except:
            return False
    
    def execute_command(self, action: str, args: Dict[str, Any]) -> Dict[str, Any]:
        """コマンドを実行"""
        response = self.session.post(
            f"{self.base_url}/api/command",
            json={"action": action, "args": args}
        )
        return response.json()
    
    def draw_line(self, start: List[float], end: List[float]) -> int:
        """線を描画"""
        result = self.execute_command("draw_line", {
            "start": start,
            "end": end
        })
        return result["data"]["id"]
    
    def draw_circle(self, center: List[float], radius: float) -> int:
        """円を描画"""
        result = self.execute_command("draw_circle", {
            "center": center,
            "radius": radius
        })
        return result["data"]["id"]
    
    def delete_entity(self, entity_id: int) -> bool:
        """エンティティを削除"""
        result = self.execute_command("delete_entity", {
            "id": entity_id
        })
        return result["success"]
    
    def get_state(self) -> Dict[str, Any]:
        """現在の状態を取得"""
        response = self.session.get(f"{self.base_url}/api/state")
        return response.json()
    
    def list_entities(self) -> List[Dict[str, Any]]:
        """全エンティティを取得"""
        response = self.session.get(f"{self.base_url}/api/entities")
        return response.json()
    
    # テストシナリオ
    def test_basic_drawing(self):
        """基本的な描画テスト"""
        print("Test: Basic Drawing")
        
        # 初期状態確認
        state = self.get_state()
        initial_count = state["entity_count"]
        
        # 線を描画
        line_id = self.draw_line([0, 0], [100, 100])
        print(f"  ✓ Line created: {line_id}")
        
        # エンティティ数確認
        state = self.get_state()
        assert state["entity_count"] == initial_count + 1
        print(f"  ✓ Entity count: {state['entity_count']}")
        
        # 円を描画
        circle_id = self.draw_circle([50, 50], 25)
        print(f"  ✓ Circle created: {circle_id}")
        
        # エンティティ数確認
        state = self.get_state()
        assert state["entity_count"] == initial_count + 2
        print(f"  ✓ Entity count: {state['entity_count']}")
        
        print("  ✅ Test passed!")
    
    def test_delete(self):
        """削除テスト"""
        print("Test: Delete Entity")
        
        # エンティティ作成
        line_id = self.draw_line([0, 0], [50, 50])
        
        # 削除
        success = self.delete_entity(line_id)
        assert success
        print(f"  ✓ Entity deleted: {line_id}")
        
        # 削除確認
        entities = self.list_entities()
        assert not any(e["id"] == line_id for e in entities)
        print("  ✅ Test passed!")

# 使用例
if __name__ == "__main__":
    agent = CADTestAgent()
    
    if not agent.health_check():
        print("❌ CAD application is not running")
        exit(1)
    
    print("✅ CAD application is running\n")
    
    agent.test_basic_drawing()
    agent.test_delete()
```

---

### 2.2 AI による自動テスト生成

```python
# ai_test_generator.py
import anthropic
import json

class AITestGenerator:
    def __init__(self, api_key: str):
        self.client = anthropic.Anthropic(api_key=api_key)
        self.agent = CADTestAgent()
    
    def generate_test(self, requirement: str) -> str:
        """要件からテストコードを生成"""
        prompt = f"""
You are a CAD testing expert. Generate Python test code for the following requirement:

Requirement: {requirement}

Available API:
- agent.draw_line(start, end) -> entity_id
- agent.draw_circle(center, radius) -> entity_id
- agent.delete_entity(entity_id) -> bool
- agent.get_state() -> dict
- agent.list_entities() -> list

Generate a test function that:
1. Uses the CADTestAgent API
2. Includes assertions
3. Has clear print statements

Return only the Python code.
"""
        
        response = self.client.messages.create(
            model="claude-3-5-sonnet-20241022",
            max_tokens=1024,
            messages=[{"role": "user", "content": prompt}]
        )
        
        return response.content[0].text
    
    def execute_generated_test(self, test_code: str):
        """生成されたテストを実行"""
        exec(test_code, {"agent": self.agent})

# 使用例
generator = AITestGenerator(api_key="your-api-key")

# AIにテストを生成させる
test_code = generator.generate_test(
    "Create a square using 4 lines and verify all 4 lines exist"
)

print("Generated Test:")
print(test_code)
print("\nExecuting...")

generator.execute_generated_test(test_code)
```

---

## 3. AI-Driven Development Workflow

### 3.1 開発フロー

```
1. 要件定義 (Human)
   ↓
2. 実装計画 (AI + Human)
   ↓
3. コード生成 (AI)
   ↓
4. レビュー (Human)
   ↓
5. テスト生成 (AI)
   ↓
6. 実行 & 検証 (AI + Human)
   ↓
7. デバッグ (AI + Human)
```

---

### 3.2 AI への指示テンプレート

#### Phase 1: アーキテクチャ設計
```
# Prompt Template

あなたはRust CADフレームワークの開発者です。
以下のドキュメントを参照して、[機能名]を実装してください。

## 参照ドキュメント
- docs/CAD_ARCHITECTURES.md
- docs/IMPLEMENTATION_DETAILS.md
- docs/GETTING_STARTED.md

## 要件
[具体的な要件]

## 制約
- Document-View パターンを使用
- slotmap で EntityId を管理
- wgpu でレンダリング

## 出力
1. 実装計画（Markdown）
2. 必要なファイルのリスト
3. 主要な構造体・関数のシグネチャ
```

#### Phase 2: 実装
```
# Prompt Template

以下の実装計画に基づいて、Rustコードを生成してください。

## 実装計画
[Phase 1 で生成された計画]

## 参照コード
- src/geometry/primitives.rs
- src/tools/mod.rs

## 出力
完全な実装コード（コメント付き）
```

#### Phase 3: テスト
```
# Prompt Template

以下の実装に対するテストを生成してください。

## 実装コード
[Phase 2 で生成されたコード]

## テスト要件
- Agent API を使用
- 正常系・異常系の両方
- アサーションを含む

## 出力
Python テストコード
```

---

### 3.3 実践例: AI と協働で機能実装

```markdown
# Example: "Offset Tool" の実装

## Step 1: Human → AI
「Offset Tool（複線ツール）を実装してください。
参照: docs/USAGE_PATTERNS.md の Domain-Specific CAD」

## Step 2: AI → Human (実装計画)
```rust
// 実装計画
struct OffsetTool {
    selected_entity: Option<EntityId>,
    offset_distance: f32,
}

impl Tool for OffsetTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        // 最も近いエンティティを選択
        self.selected_entity = state.geometry.find_nearest(pos, 10.0);
    }
    
    fn mouse_move(&mut self, pos: Point, state: &mut AppState) {
        // プレビュー表示
    }
    
    fn mouse_up(&mut self, pos: Point, state: &mut AppState) {
        // オフセットを確定
    }
}
```

## Step 3: Human レビュー
「OK、実装してください」

## Step 4: AI → Human (完全実装)
[完全なコード]

## Step 5: AI → Human (テスト)
```python
def test_offset_tool():
    agent = CADTestAgent()
    
    # 元の線を描画
    original_id = agent.draw_line([0, 0], [100, 0])
    
    # Offset Tool を選択
    agent.execute_command("set_tool", {"tool": "offset"})
    
    # オフセット実行
    offset_id = agent.execute_command("offset_entity", {
        "id": original_id,
        "distance": 10.0
    })
    
    # 検証
    entities = agent.list_entities()
    assert len(entities) == 2
    print("✅ Offset tool works!")
```

## Step 6: Human 実行 & 検証
```bash
python test_offset_tool.py
```
```

---

## 4. Prompt Engineering for CAD

### 4.1 効果的なプロンプト

#### ❌ 悪い例
```
「CADを作って」
```
**問題**: 曖昧すぎる

#### ✅ 良い例
```
「Rust CAD Framework の docs/ を参照して、
以下の仕様で Line Tool を実装してください：

1. マウスダウンで始点を記録
2. マウスムーブでプレビュー表示
3. マウスアップで線を確定

参照:
- docs/GETTING_STARTED.md (基本構造)
- docs/WGPU_COMPLETE_GUIDE.md (レンダリング)
- src/tools/mod.rs (既存のTool trait)

出力:
- src/tools/line_tool.rs の完全な実装
- テストコード (Python)
```

---

### 4.2 段階的な指示

```markdown
# Phase 1: 理解
「docs/CAD_ARCHITECTURES.md を読んで、
Document-View パターンの利点を3つ挙げてください」

# Phase 2: 設計
「Document-View パターンで Selection System を設計してください。
出力: 構造体定義とメソッドシグネチャ」

# Phase 3: 実装
「上記の設計を実装してください。
参照: docs/PRACTICAL_IMPLEMENTATION.md の Selection System」

# Phase 4: テスト
「実装した Selection System のテストを生成してください。
Agent API を使用してください」
```

---

## 5. Complete Implementation

### 5.1 統合例: AI テスト可能なCAD

```rust
// main.rs
use std::sync::{Arc, Mutex};
use std::thread;

#[tokio::main]
async fn main() {
    // アプリケーション状態
    let state = Arc::new(Mutex::new(AppState::new()));
    
    // Agent Server を別スレッドで起動
    let server_state = state.clone();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = AgentServer::new(server_state);
            server.run(9000).await;
        });
    });
    
    // メインアプリケーション起動
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    
    let mut app = App::new(window, state).await;
    
    event_loop.run(move |event, _, control_flow| {
        app.handle_event(event, control_flow);
    });
}
```

---

### 5.2 CI/CD 統合

```yaml
# .github/workflows/ai-test.yml
name: AI Agent Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build CAD
        run: cargo build --release
      
      - name: Start CAD with Agent API
        run: |
          ./target/release/my-cad --agent-api &
          sleep 5
      
      - name: Run AI Tests
        run: |
          pip install requests anthropic
          python tests/ai_agent_tests.py
      
      - name: Generate Test Report
        run: python tests/generate_report.py
```

---

## 📊 AI 開発のメリット

| 項目 | 従来 | AI駆動 | 改善率 |
|------|------|--------|--------|
| **実装速度** | 1週間 | 1日 | **7x** |
| **テストカバレッジ** | 60% | 95% | **1.6x** |
| **バグ発見** | 手動 | 自動 | **10x** |
| **ドキュメント** | 古い | 常に最新 | **∞** |

---

## 🎯 ベストプラクティス

1. ✅ **Agent API を最初から設計**
2. ✅ **ドキュメントを充実させる**（AIの参照元）
3. ✅ **段階的にAIに指示**（理解→設計→実装→テスト）
4. ✅ **人間がレビュー**（AIは完璧ではない）
5. ✅ **テストを自動化**（Agent API + Python）

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
