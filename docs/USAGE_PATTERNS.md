# Framework Usage Patterns & Examples

> **対象**: Rust CAD Framework を使ってアプリケーションを作る開発者
> 
> **目的**: 現実的な使用パターンと実装例を提示

---

## 📚 Table of Contents
1. [Usage Patterns Overview](#1-usage-patterns-overview)
2. [Pattern 1: Simple 2D Drawing App](#2-pattern-1-simple-2d-drawing-app)
3. [Pattern 2: Parametric CAD Tool](#3-pattern-2-parametric-cad-tool)
4. [Pattern 3: Collaborative Cloud CAD](#4-pattern-3-collaborative-cloud-cad)
5. [Pattern 4: Domain-Specific CAD](#5-pattern-4-domain-specific-cad)
6. [Pattern 5: AI-Assisted CAD](#6-pattern-5-ai-assisted-cad)

---

## 1. Usage Patterns Overview

### パターン分類

| パターン | 複雑度 | 用途 | 主要技術 |
|---------|--------|------|---------|
| Simple 2D Drawing | ⭐ | スケッチ、図面作成 | Immediate Mode |
| Parametric CAD | ⭐⭐⭐ | 機械設計、建築 | Constraint Solver + Undo/Redo |
| Collaborative Cloud | ⭐⭐⭐⭐ | チーム設計 | Event Sourcing + CRDT |
| Domain-Specific | ⭐⭐ | 回路図、配管図 | Custom Tools + Validation |
| AI-Assisted | ⭐⭐⭐⭐ | 生成設計 | Agent API + Reactive Graph |

---

## 2. Pattern 1: Simple 2D Drawing App

### 概要
**用途**: 簡単な図面作成、スケッチアプリ  
**例**: Inkscape風、簡易CAD

### アーキテクチャ
```
┌─────────────────┐
│   UI (egui)     │
├─────────────────┤
│  Tool Manager   │ ← Line, Circle, Rect
├─────────────────┤
│ Geometry Store  │ ← Vec<Entity>
├─────────────────┤
│ Renderer (wgpu) │
└─────────────────┘
```

### 実装例

```rust
// main.rs
use rust_cad_framework::*;

fn main() {
    let app = CadApp::builder()
        .window_size(1280, 720)
        .title("Simple Drawing App")
        .build();
    
    app.run();
}

// custom_app.rs
struct DrawingApp {
    framework: CadFramework,
    current_tool: ToolType,
}

impl DrawingApp {
    fn new() -> Self {
        let mut framework = CadFramework::new();
        
        // 基本ツールを登録
        framework.register_tool("line", LineTool::new());
        framework.register_tool("circle", CircleTool::new());
        framework.register_tool("rect", RectTool::new());
        
        Self {
            framework,
            current_tool: ToolType::Line,
        }
    }
    
    fn handle_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Line").clicked() {
                self.framework.set_active_tool("line");
            }
            if ui.button("Circle").clicked() {
                self.framework.set_active_tool("circle");
            }
            if ui.button("Rect").clicked() {
                self.framework.set_active_tool("rect");
            }
        });
        
        // プロパティパネル
        ui.separator();
        ui.label("Properties");
        if let Some(selected) = self.framework.get_selected_entity() {
            ui.label(format!("Type: {:?}", selected.entity_type()));
            ui.label(format!("Position: {:?}", selected.position()));
        }
    }
}
```

### ファイル保存

```rust
// JSON形式で保存
impl DrawingApp {
    fn save(&self, path: &Path) -> Result<()> {
        let document = self.framework.export_document();
        let json = serde_json::to_string_pretty(&document)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    fn load(&mut self, path: &Path) -> Result<()> {
        let json = std::fs::read_to_string(path)?;
        let document: CadDocument = serde_json::from_str(&json)?;
        self.framework.import_document(document);
        Ok(())
    }
}
```

---

## 3. Pattern 2: Parametric CAD Tool

### 概要
**用途**: 機械部品設計、パラメトリックモデリング  
**例**: FreeCAD風、SolidWorks風

### アーキテクチャ
```
┌─────────────────────┐
│   Feature Tree UI   │
├─────────────────────┤
│  Constraint Solver  │ ← D-Cubed, LGS
├─────────────────────┤
│  Command History    │ ← Undo/Redo
├─────────────────────┤
│   B-rep Kernel      │ ← Open CASCADE
└─────────────────────┘
```

### 実装例

```rust
// parametric_app.rs
struct ParametricCAD {
    framework: CadFramework,
    feature_tree: FeatureTree,
    constraint_solver: ConstraintSolver,
    history: CommandHistory,
}

// Feature Tree
struct FeatureTree {
    root: Feature,
    active_feature: Option<FeatureId>,
}

enum Feature {
    Sketch {
        id: FeatureId,
        plane: Plane,
        constraints: Vec<Constraint>,
        children: Vec<Feature>,
    },
    Extrude {
        id: FeatureId,
        sketch_id: FeatureId,
        depth: Parameter,
        children: Vec<Feature>,
    },
    Fillet {
        id: FeatureId,
        edges: Vec<EdgeId>,
        radius: Parameter,
        children: Vec<Feature>,
    },
}

// パラメータ管理
struct Parameter {
    name: String,
    value: f32,
    expression: Option<String>, // 例: "width * 2"
}

impl ParametricCAD {
    fn add_sketch(&mut self, plane: Plane) -> FeatureId {
        let id = self.feature_tree.add_feature(Feature::Sketch {
            id: FeatureId::new(),
            plane,
            constraints: vec![],
            children: vec![],
        });
        
        // Undo/Redo 対応
        let cmd = AddSketchCommand { plane };
        self.history.execute(Box::new(cmd));
        
        id
    }
    
    fn add_constraint(&mut self, sketch_id: FeatureId, constraint: Constraint) {
        // 制約を追加
        self.feature_tree.add_constraint(sketch_id, constraint.clone());
        
        // ソルバーで解く
        self.constraint_solver.solve();
        
        // ジオメトリを更新
        self.rebuild_geometry();
        
        // Undo/Redo
        let cmd = AddConstraintCommand { sketch_id, constraint };
        self.history.execute(Box::new(cmd));
    }
    
    fn rebuild_geometry(&mut self) {
        // Feature Tree を順番に評価
        for feature in self.feature_tree.iter() {
            match feature {
                Feature::Sketch { constraints, .. } => {
                    self.constraint_solver.solve_sketch(constraints);
                }
                Feature::Extrude { sketch_id, depth, .. } => {
                    let sketch = self.get_sketch(*sketch_id);
                    let solid = self.brep_kernel.extrude(sketch, depth.value);
                    self.framework.add_entity(solid);
                }
                Feature::Fillet { edges, radius, .. } => {
                    self.brep_kernel.fillet(edges, radius.value);
                }
            }
        }
    }
}
```

### UI例

```rust
impl ParametricCAD {
    fn render_feature_tree(&mut self, ui: &mut egui::Ui) {
        ui.heading("Feature Tree");
        
        for feature in self.feature_tree.iter() {
            let label = match feature {
                Feature::Sketch { id, .. } => format!("📐 Sketch {}", id),
                Feature::Extrude { id, depth, .. } => {
                    format!("⬆️ Extrude {} ({}mm)", id, depth.value)
                }
                Feature::Fillet { id, radius, .. } => {
                    format!("🔘 Fillet {} (R{})", id, radius.value)
                }
            };
            
            if ui.selectable_label(
                self.feature_tree.active_feature == Some(feature.id()),
                label
            ).clicked() {
                self.feature_tree.set_active(feature.id());
            }
        }
    }
}
```

---

## 4. Pattern 3: Collaborative Cloud CAD

### 概要
**用途**: チームでの共同設計、リアルタイム編集  
**例**: Onshape風、Figma風

### アーキテクチャ
```
┌──────────────┐         ┌──────────────┐
│  Client A    │ ←─────→ │   Server     │
│ (Browser)    │  WebRTC  │ (Event Store)│
└──────────────┘         └──────────────┘
       ↑                        ↑
       │                        │
       └────────────────────────┘
              WebSocket
```

### 実装例

```rust
// server/main.rs
use axum::{Router, extract::ws::WebSocket};
use tokio::sync::broadcast;

struct CollaborativeServer {
    event_store: Arc<Mutex<EventStore>>,
    clients: Arc<Mutex<HashMap<ClientId, ClientSession>>>,
    broadcast_tx: broadcast::Sender<Event>,
}

impl CollaborativeServer {
    async fn handle_websocket(&self, socket: WebSocket, client_id: ClientId) {
        let mut rx = self.broadcast_tx.subscribe();
        
        loop {
            tokio::select! {
                // クライアントからのイベント
                Some(msg) = socket.recv() => {
                    let event: Event = serde_json::from_str(&msg)?;
                    
                    // イベントを保存
                    self.event_store.lock().await.append(event.clone());
                    
                    // 他のクライアントにブロードキャスト
                    self.broadcast_tx.send(event)?;
                }
                
                // 他のクライアントからのイベント
                Ok(event) = rx.recv() => {
                    socket.send(serde_json::to_string(&event)?).await?;
                }
            }
        }
    }
}

// client/collaborative_client.rs
struct CollaborativeClient {
    framework: CadFramework,
    websocket: WebSocket,
    local_events: Vec<Event>,
    remote_events: Vec<Event>,
}

impl CollaborativeClient {
    async fn sync(&mut self) {
        // ローカルイベントを送信
        for event in self.local_events.drain(..) {
            self.websocket.send(serde_json::to_string(&event)?).await?;
        }
        
        // リモートイベントを受信
        while let Some(msg) = self.websocket.try_recv() {
            let event: Event = serde_json::from_str(&msg)?;
            self.apply_remote_event(event);
        }
    }
    
    fn apply_remote_event(&mut self, event: Event) {
        // CRDT で競合解決
        let resolved = self.crdt.merge(event);
        self.framework.apply_event(resolved);
    }
}
```

### Operational Transformation (OT)

```rust
// 競合解決
fn transform_events(local: Event, remote: Event) -> (Event, Event) {
    match (local, remote) {
        (Event::EntityMoved { id: id1, to: to1, .. },
         Event::EntityMoved { id: id2, to: to2, .. }) if id1 == id2 => {
            // 同じエンティティの移動 → 後勝ち
            (Event::Noop, Event::EntityMoved { id: id2, to: to2 })
        }
        (Event::EntityDeleted { id: id1 },
         Event::EntityMoved { id: id2, .. }) if id1 == id2 => {
            // 削除 vs 移動 → 削除優先
            (Event::EntityDeleted { id: id1 }, Event::Noop)
        }
        _ => (local, remote) // 競合なし
    }
}
```

---

## 5. Pattern 4: Domain-Specific CAD

### 概要
**用途**: 特定分野専用CAD（回路図、配管図、建築平面図）  
**例**: KiCad風、AutoCAD MEP風

### 実装例: 電気回路図エディタ

```rust
// circuit_cad.rs
struct CircuitCAD {
    framework: CadFramework,
    component_library: ComponentLibrary,
    netlist: Netlist,
}

// コンポーネントライブラリ
struct ComponentLibrary {
    components: HashMap<String, ComponentDefinition>,
}

struct ComponentDefinition {
    symbol: Symbol,
    pins: Vec<Pin>,
    properties: HashMap<String, String>,
}

struct Pin {
    name: String,
    position: Point,
    pin_type: PinType,
}

enum PinType {
    Input,
    Output,
    Bidirectional,
    Power,
    Ground,
}

// ネットリスト生成
impl CircuitCAD {
    fn place_component(&mut self, component_name: &str, position: Point) {
        let def = self.component_library.get(component_name).unwrap();
        
        let instance = ComponentInstance {
            id: ComponentId::new(),
            definition: def.clone(),
            position,
            rotation: 0.0,
        };
        
        self.framework.add_entity(Entity::Component(instance));
    }
    
    fn connect_pins(&mut self, pin1: PinId, pin2: PinId) {
        // ワイヤーを描画
        let wire = self.create_wire(pin1, pin2);
        self.framework.add_entity(Entity::Wire(wire));
        
        // ネットリストに追加
        self.netlist.add_connection(pin1, pin2);
    }
    
    fn export_netlist(&self) -> String {
        // SPICE形式で出力
        let mut output = String::new();
        
        for component in &self.netlist.components {
            output.push_str(&format!(
                "{} {} {} {}\n",
                component.id,
                component.pins[0],
                component.pins[1],
                component.value
            ));
        }
        
        output
    }
    
    fn validate_circuit(&self) -> Vec<ValidationError> {
        let mut errors = vec![];
        
        // 浮いているピンをチェック
        for component in &self.netlist.components {
            for pin in &component.pins {
                if !self.netlist.is_connected(pin) {
                    errors.push(ValidationError::FloatingPin {
                        component: component.id,
                        pin: pin.name.clone(),
                    });
                }
            }
        }
        
        // 電源の接続をチェック
        if !self.netlist.has_power_connection() {
            errors.push(ValidationError::NoPowerConnection);
        }
        
        errors
    }
}
```

---

## 6. Pattern 5: AI-Assisted CAD

### 概要
**用途**: AI による設計支援、自動生成  
**例**: Generative Design, AI補完

### アーキテクチャ
```
┌─────────────────┐
│   User Input    │
├─────────────────┤
│   AI Agent      │ ← GPT-4, Claude
├─────────────────┤
│  Agent API      │ ← HTTP Commands
├─────────────────┤
│  CAD Framework  │
└─────────────────┘
```

### 実装例

```rust
// ai_assisted_cad.rs
struct AIAssistedCAD {
    framework: CadFramework,
    ai_client: AIClient,
    agent_server: AgentServer,
}

impl AIAssistedCAD {
    async fn process_natural_language(&mut self, prompt: &str) -> Result<()> {
        // AI にプロンプトを送信
        let response = self.ai_client.complete(&format!(
            "You are a CAD assistant. Generate HTTP commands to create the following design:\n\
             {}\n\n\
             Available commands:\n\
             - POST /api/command {{\"action\": \"draw_line\", \"args\": {{\"start\": [x, y], \"end\": [x, y]}}}}\n\
             - POST /api/command {{\"action\": \"draw_circle\", \"args\": {{\"center\": [x, y], \"radius\": r}}}}\n\
             \n\
             Respond with a JSON array of commands.",
            prompt
        )).await?;
        
        // コマンドを実行
        let commands: Vec<Command> = serde_json::from_str(&response)?;
        for cmd in commands {
            self.agent_server.execute_command(cmd).await?;
        }
        
        Ok(())
    }
}

// 使用例
async fn main() {
    let mut cad = AIAssistedCAD::new();
    
    // 自然言語で指示
    cad.process_natural_language(
        "Create a rectangular floor plan 10m x 8m with a door on the north wall"
    ).await?;
    
    // AIが自動的に以下を実行:
    // 1. draw_rect(0, 0, 10000, 8000)
    // 2. draw_line(4000, 8000, 5000, 8000) // ドア開口
}
```

### Generative Design

```rust
struct GenerativeDesign {
    framework: CadFramework,
    optimizer: GeneticAlgorithm,
}

impl GenerativeDesign {
    fn optimize_bracket(&mut self, constraints: Constraints) -> Design {
        let mut population = self.generate_initial_population();
        
        for generation in 0..100 {
            // 各個体を評価
            let fitness: Vec<f32> = population.iter()
                .map(|design| self.evaluate_fitness(design, &constraints))
                .collect();
            
            // 選択・交叉・突然変異
            population = self.evolve(population, fitness);
        }
        
        // 最良の個体を返す
        population.into_iter()
            .max_by(|a, b| {
                self.evaluate_fitness(a, &constraints)
                    .partial_cmp(&self.evaluate_fitness(b, &constraints))
                    .unwrap()
            })
            .unwrap()
    }
    
    fn evaluate_fitness(&self, design: &Design, constraints: &Constraints) -> f32 {
        let mut score = 0.0;
        
        // 重量を最小化
        score -= design.calculate_mass() * 10.0;
        
        // 強度を確保
        let stress = self.run_fea_simulation(design);
        if stress.max() < constraints.max_stress {
            score += 100.0;
        } else {
            score -= (stress.max() - constraints.max_stress) * 50.0;
        }
        
        score
    }
}
```

---

## 📊 パターン選択ガイド

### プロジェクト規模別

| 規模 | 推奨パターン | 理由 |
|------|------------|------|
| 個人プロジェクト | Simple 2D Drawing | 学習コスト低、迅速な開発 |
| 小規模チーム | Domain-Specific | 特化機能で差別化 |
| 中規模企業 | Parametric CAD | 業務効率化、再利用性 |
| 大規模企業 | Collaborative Cloud | スケーラビリティ、共同作業 |
| 研究開発 | AI-Assisted | 最先端技術、自動化 |

### 技術スタック別

```rust
// パターン1: Webベース (WASM)
#[cfg(target_arch = "wasm32")]
fn main() {
    let app = CadFramework::new()
        .with_renderer(WebGLRenderer::new())
        .with_storage(LocalStorage::new());
    
    wasm_bindgen_futures::spawn_local(async move {
        app.run().await;
    });
}

// パターン2: デスクトップ (Native)
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = CadFramework::new()
        .with_renderer(WgpuRenderer::new())
        .with_storage(FileStorage::new("./documents"));
    
    app.run();
}

// パターン3: サーバー (Headless)
fn main() {
    let server = CadServer::new()
        .with_agent_api(true)
        .with_port(9000);
    
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(server.run());
}
```

---

## 🎯 まとめ

Rust CAD Framework は、以下のパターンで活用できる:

1. **Simple 2D Drawing** - 学習用、プロトタイプ
2. **Parametric CAD** - 本格的な設計ツール
3. **Collaborative Cloud** - チーム開発
4. **Domain-Specific** - 専門分野特化
5. **AI-Assisted** - 次世代CAD

各パターンは独立しており、**段階的に進化**させることも可能:
```
Simple → Domain-Specific → Parametric → Collaborative → AI-Assisted
```

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
