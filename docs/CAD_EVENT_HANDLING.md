# CAD Command Applicability & Event Handling Patterns

> **対象**: CADタイプ別の適用範囲とイベント処理の実装を理解したい開発者
> 
> **目的**: 2D/3D CAD、機械/建築CADでのコマンド適用範囲と、実績あるイベント処理パターンを解説

---

## 📚 Table of Contents
1. [Command Applicability by CAD Type](#1-command-applicability-by-cad-type)
2. [Event Handling Best Practices](#2-event-handling-best-practices)
3. [State Machine Pattern](#3-state-machine-pattern)
4. [Drawing Software Common Patterns](#4-drawing-software-common-patterns)
5. [Complete Implementation](#5-complete-implementation)

---

## 1. Command Applicability by CAD Type

### 1.1 CADタイプ別コマンド適用表

| コマンド | 2D CAD | 3D CAD | 機械CAD | 建築CAD | お絵かきソフト |
|---------|--------|--------|---------|---------|---------------|
| **Line** | ✅ 100% | ✅ 80% | ✅ 100% | ✅ 100% | ✅ 100% |
| **Circle** | ✅ 100% | ✅ 60% | ✅ 100% | ✅ 80% | ✅ 100% |
| **Rectangle** | ✅ 100% | ✅ 50% | ✅ 100% | ✅ 100% | ✅ 100% |
| **Arc** | ✅ 100% | ✅ 70% | ✅ 100% | ✅ 90% | ✅ 80% |
| **Polyline** | ✅ 100% | ✅ 60% | ✅ 100% | ✅ 100% | ✅ 90% |
| **Polygon** | ✅ 100% | ✅ 40% | ✅ 80% | ✅ 60% | ✅ 70% |
| **Spline** | ✅ 100% | ✅ 80% | ✅ 90% | ✅ 70% | ✅ 100% |
| **Hatch** | ✅ 100% | ❌ 10% | ✅ 80% | ✅ 100% | ✅ 60% |
| **Double Line** | ✅ 100% | ❌ 5% | ✅ 60% | ✅ 100% | ❌ 10% |
| **Move** | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% |
| **Copy** | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% |
| **Offset** | ✅ 100% | ✅ 30% | ✅ 100% | ✅ 100% | ❌ 20% |
| **Trim** | ✅ 100% | ✅ 40% | ✅ 100% | ✅ 100% | ✅ 50% |
| **Extend** | ✅ 100% | ✅ 40% | ✅ 100% | ✅ 100% | ✅ 40% |
| **Fillet** | ✅ 100% | ✅ 90% | ✅ 100% | ✅ 80% | ❌ 30% |
| **Mirror** | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% |
| **Rotate** | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% |
| **Scale** | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% |
| **Array** | ✅ 100% | ✅ 90% | ✅ 100% | ✅ 100% | ✅ 60% |
| **Stretch** | ✅ 100% | ✅ 60% | ✅ 100% | ✅ 100% | ✅ 70% |
| **Endpoint Snap** | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 80% |
| **Midpoint Snap** | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 70% |
| **Perpendicular Snap** | ✅ 100% | ✅ 90% | ✅ 100% | ✅ 100% | ❌ 30% |
| **Window Selection** | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% |
| **Crossing Selection** | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 100% | ✅ 90% |
| **Extrude** | ❌ 0% | ✅ 100% | ✅ 100% | ✅ 80% | ❌ 0% |
| **Revolve** | ❌ 0% | ✅ 100% | ✅ 100% | ✅ 60% | ❌ 0% |
| **Union/Subtract** | ❌ 0% | ✅ 100% | ✅ 100% | ✅ 70% | ❌ 0% |

---

### 1.2 CADタイプ別の特徴

#### 2D CAD (JWW CAD, AutoCAD 2D)
- **用途**: 平面図、断面図、詳細図
- **重要コマンド**: Line, Offset, Trim, Hatch, Double Line
- **特徴**: 精密な寸法管理、レイヤー管理

#### 3D CAD (SolidWorks, Fusion 360)
- **用途**: 立体モデリング、アセンブリ
- **重要コマンド**: Extrude, Revolve, Union, Fillet
- **特徴**: パラメトリック設計、シミュレーション

#### 機械CAD
- **用途**: 部品設計、組立図
- **重要コマンド**: Fillet, Chamfer, Array, Dimension
- **特徴**: 公差管理、材料指定

#### 建築CAD
- **用途**: 平面図、立面図、パース
- **重要コマンド**: Double Line (壁), Hatch (床材), Offset
- **特徴**: スケール管理、レイアウト

#### お絵かきソフト (Photoshop, Illustrator)
- **用途**: イラスト、デザイン
- **重要コマンド**: Spline (ベジェ曲線), Move, Rotate, Scale
- **特徴**: レイヤー、ブラシ、エフェクト

---

## 2. Event Handling Best Practices

### 2.1 CADソフトウェアのイベント処理原則

#### ❌ やってはいけないこと

```rust
// ❌ 悪い例: イベントハンドラ内でインタラクティブ操作
impl Tool for BadTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        // ❌ イベント内でユーザー入力を待つ
        let user_input = prompt_user("Enter distance:");
        
        // ❌ イベント内でダイアログを表示
        show_dialog("Select options");
        
        // ❌ 同じイベントを再トリガー（無限ループ）
        state.trigger_mouse_down(pos);
    }
}
```

#### ✅ 正しい実装

```rust
// ✅ 良い例: 状態を記録し、次のイベントで処理
impl Tool for GoodTool {
    fn mouse_down(&mut self, pos: Point, state: &mut AppState) {
        // ✅ 状態を記録
        self.state = ToolState::WaitingForSecondPoint;
        self.first_point = Some(pos);
        
        // ✅ UI更新は別スレッドまたは次のフレーム
        state.request_ui_update();
    }
    
    fn mouse_move(&mut self, pos: Point, state: &mut AppState) {
        // ✅ プレビューを更新（読み取り専用）
        if let Some(start) = self.first_point {
            self.preview_line = Some((start, pos));
        }
    }
}
```

---

### 2.2 イベント処理のベストプラクティス

#### 1. **イベントは情報提供のみ**
```rust
// ✅ イベントは状態を読み取るだけ
fn on_object_modified(&self, entity_id: EntityId, state: &AppState) {
    // ✅ 読み取りのみ
    if let Some(entity) = state.geometry.get_entity(entity_id) {
        log::info!("Entity modified: {:?}", entity);
    }
    
    // ❌ 変更しない
    // state.geometry.modify_entity(entity_id, ...);
}
```

#### 2. **イベント順序に依存しない**
```rust
// ❌ 悪い例: イベント順序に依存
struct BadEventHandler {
    command_started: bool,
}

impl BadEventHandler {
    fn on_command_start(&mut self) {
        self.command_started = true;
    }
    
    fn on_document_created(&mut self) {
        // ❌ command_started が true であることを期待
        if self.command_started {
            // ...
        }
    }
}

// ✅ 良い例: 各イベントを独立して処理
impl GoodEventHandler {
    fn on_command_start(&mut self, command: &Command) {
        log::info!("Command started: {}", command.name);
    }
    
    fn on_document_created(&mut self, doc: &Document) {
        log::info!("Document created: {}", doc.name);
    }
}
```

#### 3. **無限ループを防ぐ**
```rust
// ❌ 悪い例: 無限ループ
fn on_object_opened(&mut self, id: EntityId, state: &mut AppState) {
    // ❌ 同じオブジェクトを再度開く → 無限ループ
    state.open_object(id);
}

// ✅ 良い例: フラグで制御
struct SafeHandler {
    processing: bool,
}

impl SafeHandler {
    fn on_object_opened(&mut self, id: EntityId, state: &mut AppState) {
        if self.processing {
            return; // 再入を防ぐ
        }
        
        self.processing = true;
        // 処理
        self.processing = false;
    }
}
```

---

## 3. State Machine Pattern

### 3.1 ツールの状態遷移

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolState {
    Idle,
    WaitingForFirstPoint,
    WaitingForSecondPoint,
    WaitingForThirdPoint,
    Dragging,
    Previewing,
    Completed,
}

struct StateMachineTool {
    state: ToolState,
    data: ToolData,
}

struct ToolData {
    first_point: Option<Point>,
    second_point: Option<Point>,
    preview: Option<Entity>,
}

impl Tool for StateMachineTool {
    fn mouse_down(&mut self, pos: Point, app_state: &mut AppState) {
        match self.state {
            ToolState::Idle => {
                self.data.first_point = Some(pos);
                self.state = ToolState::WaitingForSecondPoint;
            }
            ToolState::WaitingForSecondPoint => {
                self.data.second_point = Some(pos);
                self.create_entity(app_state);
                self.state = ToolState::Completed;
            }
            _ => {}
        }
    }
    
    fn mouse_move(&mut self, pos: Point, app_state: &mut AppState) {
        match self.state {
            ToolState::WaitingForSecondPoint => {
                if let Some(start) = self.data.first_point {
                    self.data.preview = Some(Entity::Line {
                        p1: start,
                        p2: pos,
                    });
                    self.state = ToolState::Previewing;
                }
            }
            ToolState::Previewing => {
                if let Some(start) = self.data.first_point {
                    self.data.preview = Some(Entity::Line {
                        p1: start,
                        p2: pos,
                    });
                }
            }
            _ => {}
        }
    }
    
    fn key_down(&mut self, key: Key, app_state: &mut AppState) {
        match key {
            Key::Escape => {
                // どの状態からでもキャンセル可能
                self.reset();
            }
            Key::Enter => {
                if self.state == ToolState::Previewing {
                    self.create_entity(app_state);
                    self.state = ToolState::Completed;
                }
            }
            _ => {}
        }
    }
    
    fn reset(&mut self) {
        self.state = ToolState::Idle;
        self.data = ToolData::default();
    }
}
```

---

### 3.2 状態遷移図

```
Idle
  ↓ mouse_down
WaitingForFirstPoint
  ↓ mouse_down
WaitingForSecondPoint
  ↓ mouse_move
Previewing
  ↓ mouse_down / Enter
Completed
  ↓ reset
Idle

[Escape] → Idle (どの状態からでも)
```

---

## 4. Drawing Software Common Patterns

### 4.1 Photoshop / Illustrator のイベント処理

#### イベントリスナーパターン

```rust
// Photoshop の Script Events Manager に相当
struct EventManager {
    listeners: HashMap<EventType, Vec<Box<dyn EventListener>>>,
}

#[derive(Hash, Eq, PartialEq)]
enum EventType {
    DocumentOpened,
    DocumentSaved,
    ToolChanged,
    LayerAdded,
    SelectionChanged,
}

trait EventListener {
    fn on_event(&mut self, event: &Event);
}

impl EventManager {
    fn register(&mut self, event_type: EventType, listener: Box<dyn EventListener>) {
        self.listeners.entry(event_type)
            .or_insert_with(Vec::new)
            .push(listener);
    }
    
    fn trigger(&mut self, event_type: EventType, event: &Event) {
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            for listener in listeners {
                listener.on_event(event);
            }
        }
    }
}

// 使用例
struct AutoSaveListener;

impl EventListener for AutoSaveListener {
    fn on_event(&mut self, event: &Event) {
        if let Event::DocumentModified { .. } = event {
            // 自動保存処理
            log::info!("Auto-saving document...");
        }
    }
}
```

---

### 4.2 アクション/マクロシステム

```rust
// Illustrator の Actions に相当
struct ActionRecorder {
    recording: bool,
    actions: Vec<RecordedAction>,
}

#[derive(Clone)]
enum RecordedAction {
    MouseDown { pos: Point },
    MouseMove { pos: Point },
    MouseUp { pos: Point },
    ToolChanged { tool: String },
    CommandExecuted { command: String, args: Vec<String> },
}

impl ActionRecorder {
    fn start_recording(&mut self) {
        self.recording = true;
        self.actions.clear();
    }
    
    fn stop_recording(&mut self) -> Vec<RecordedAction> {
        self.recording = false;
        self.actions.clone()
    }
    
    fn record_action(&mut self, action: RecordedAction) {
        if self.recording {
            self.actions.push(action);
        }
    }
    
    fn replay(&self, state: &mut AppState) {
        for action in &self.actions {
            match action {
                RecordedAction::MouseDown { pos } => {
                    state.active_tool.mouse_down(*pos, state);
                }
                RecordedAction::CommandExecuted { command, args } => {
                    state.execute_command(command, args);
                }
                _ => {}
            }
        }
    }
}
```

---

### 4.3 レンダリングパイプラインとの統合

```rust
// GPU レンダリングとイベント処理の分離
struct Application {
    event_queue: VecDeque<Event>,
    render_queue: VecDeque<RenderCommand>,
    
    // イベント処理スレッド
    event_thread: Option<JoinHandle<()>>,
    
    // レンダリングスレッド
    render_thread: Option<JoinHandle<()>>,
}

impl Application {
    fn run(&mut self) {
        // イベント処理ループ（別スレッド）
        let event_queue = Arc::new(Mutex::new(self.event_queue.clone()));
        let render_queue = Arc::new(Mutex::new(self.render_queue.clone()));
        
        let event_queue_clone = event_queue.clone();
        let render_queue_clone = render_queue.clone();
        
        self.event_thread = Some(thread::spawn(move || {
            loop {
                // イベントを処理
                if let Some(event) = event_queue_clone.lock().unwrap().pop_front() {
                    let render_cmd = process_event(event);
                    render_queue_clone.lock().unwrap().push_back(render_cmd);
                }
                
                thread::sleep(Duration::from_millis(1));
            }
        }));
        
        // レンダリングループ（メインスレッド）
        loop {
            if let Some(cmd) = render_queue.lock().unwrap().pop_front() {
                self.render(cmd);
            }
            
            // 60 FPS
            thread::sleep(Duration::from_millis(16));
        }
    }
}
```

---

## 5. Complete Implementation

### 5.1 統合イベントシステム

```rust
pub struct CADEventSystem {
    // イベント管理
    event_manager: EventManager,
    
    // ツール状態機械
    active_tool: Box<dyn Tool>,
    tool_state: ToolState,
    
    // アクション記録
    action_recorder: ActionRecorder,
    
    // レンダリング
    render_dirty: bool,
}

impl CADEventSystem {
    pub fn handle_mouse_down(&mut self, pos: Point, state: &mut AppState) {
        // 1. アクション記録
        if self.action_recorder.recording {
            self.action_recorder.record_action(RecordedAction::MouseDown { pos });
        }
        
        // 2. ツールにイベント送信
        self.active_tool.mouse_down(pos, state);
        
        // 3. イベントリスナーに通知
        self.event_manager.trigger(EventType::MouseDown, &Event::MouseDown { pos });
        
        // 4. レンダリング更新フラグ
        self.render_dirty = true;
    }
    
    pub fn handle_mouse_move(&mut self, pos: Point, state: &mut AppState) {
        // マウス移動は高頻度なので、記録しない場合もある
        if self.action_recorder.recording && self.action_recorder.record_mouse_move {
            self.action_recorder.record_action(RecordedAction::MouseMove { pos });
        }
        
        self.active_tool.mouse_move(pos, state);
        self.render_dirty = true;
    }
    
    pub fn should_render(&self) -> bool {
        self.render_dirty
    }
    
    pub fn clear_render_flag(&mut self) {
        self.render_dirty = false;
    }
}
```

---

### 5.2 マウスボタン割り当て（CAD専用マウス対応）

```rust
pub struct MouseConfig {
    pub left_button: MouseAction,
    pub middle_button: MouseAction,
    pub right_button: MouseAction,
    pub side_button_1: MouseAction,
    pub side_button_2: MouseAction,
    pub scroll_wheel: ScrollAction,
}

pub enum MouseAction {
    Select,
    Pan,
    Zoom,
    Orbit3D,
    ContextMenu,
    Custom(String),
}

pub enum ScrollAction {
    Zoom,
    Pan,
    Custom(String),
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            left_button: MouseAction::Select,
            middle_button: MouseAction::Pan,
            right_button: MouseAction::ContextMenu,
            side_button_1: MouseAction::Custom("Undo".to_string()),
            side_button_2: MouseAction::Custom("Redo".to_string()),
            scroll_wheel: ScrollAction::Zoom,
        }
    }
}

impl CADEventSystem {
    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState, pos: Point, app_state: &mut AppState) {
        let action = match button {
            MouseButton::Left => &self.mouse_config.left_button,
            MouseButton::Middle => &self.mouse_config.middle_button,
            MouseButton::Right => &self.mouse_config.right_button,
            MouseButton::Other(1) => &self.mouse_config.side_button_1,
            MouseButton::Other(2) => &self.mouse_config.side_button_2,
            _ => return,
        };
        
        match (action, state) {
            (MouseAction::Select, ElementState::Pressed) => {
                self.handle_mouse_down(pos, app_state);
            }
            (MouseAction::Pan, ElementState::Pressed) => {
                self.start_pan(pos);
            }
            (MouseAction::Custom(cmd), ElementState::Pressed) => {
                app_state.execute_command(cmd, &[]);
            }
            _ => {}
        }
    }
}
```

---

## 📊 イベント処理パターン比較

| パターン | 用途 | 複雑度 | 推奨度 |
|---------|------|--------|--------|
| **State Machine** | ツール実装 | ⭐⭐⭐ | ✅ 必須 |
| **Event Listener** | プラグイン、拡張 | ⭐⭐ | ✅ 推奨 |
| **Action Recorder** | マクロ、自動化 | ⭐⭐⭐ | ✅ 推奨 |
| **Multi-threading** | 高性能レンダリング | ⭐⭐⭐⭐ | △ 必要に応じて |

---

## 🎯 実装チェックリスト

### 基本
- [ ] State Machine でツール状態管理
- [ ] イベントハンドラ内でインタラクティブ操作をしない
- [ ] イベント順序に依存しない
- [ ] 無限ループを防ぐ

### 高度
- [ ] Event Listener システム
- [ ] Action Recorder (マクロ)
- [ ] マウスボタンカスタマイズ
- [ ] レンダリングとイベント処理の分離

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-26*
