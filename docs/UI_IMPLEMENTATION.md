# CAD UI Implementation Guide: Production Level

> **対象**: プロダクションレベルのCADアプリケーション開発者
> 
> **目的**: 実際のソフトウェア開発で使える UI 設計・実装の完全ガイド

---

## 📚 Table of Contents
1. [UI Architecture Patterns](#1-ui-architecture-patterns)
2. [Layout Systems](#2-layout-systems)
3. [UI Components](#3-ui-components)
4. [State Management](#4-state-management)
5. [Rust Implementation](#5-rust-implementation)

---

## 1. UI Architecture Patterns

### 1.1 Immediate Mode vs Retained Mode

#### Immediate Mode GUI (IMGUI)
**特徴**: 毎フレーム UI を再構築

```rust
// egui の例
fn ui(&mut self, ctx: &egui::Context) {
    egui::SidePanel::left("tool_panel").show(ctx, |ui| {
        if ui.button("Line").clicked() {
            self.set_tool(Tool::Line);
        }
        if ui.button("Circle").clicked() {
            self.set_tool(Tool::Circle);
        }
    });
}
```

**メリット**:
- ✅ シンプル（状態管理が不要）
- ✅ データと UI が常に同期
- ✅ CAD のビューポートと相性が良い

**デメリット**:
- ❌ CPU 負荷が高い（毎フレーム再構築）
- ❌ 複雑なアニメーションが難しい

**適用**: ツールパレット、プロパティパネル、ビューポートオーバーレイ

---

#### Retained Mode GUI (RMGUI)
**特徴**: UI ツリーを保持し、変更時のみ更新

```rust
// iced の例
enum Message {
    ToolSelected(Tool),
    PropertyChanged(String, f32),
}

fn view(&self) -> Element<Message> {
    Column::new()
        .push(Button::new("Line").on_press(Message::ToolSelected(Tool::Line)))
        .push(Button::new("Circle").on_press(Message::ToolSelected(Tool::Circle)))
        .into()
}

fn update(&mut self, message: Message) {
    match message {
        Message::ToolSelected(tool) => self.active_tool = tool,
        // ...
    }
}
```

**メリット**:
- ✅ 効率的（変更時のみ更新）
- ✅ 複雑なレイアウトに強い
- ✅ アニメーション対応

**デメリット**:
- ❌ 状態管理が複雑
- ❌ データと UI の同期が必要

**適用**: リボンインターフェース、ダイアログ、設定画面

---

### 1.2 混合モード（推奨）

```rust
struct CadUI {
    // Retained Mode: メインUI
    ribbon: RibbonInterface,      // iced
    property_panel: PropertyPanel, // iced
    
    // Immediate Mode: ビューポート
    viewport_overlay: ViewportUI,  // egui
    context_menu: ContextMenu,     // egui
}

impl CadUI {
    fn render(&mut self, ctx: &RenderContext) {
        // Retained Mode UI
        self.ribbon.render();
        self.property_panel.render();
        
        // Immediate Mode Overlay
        egui::Area::new("viewport_overlay").show(&ctx.egui, |ui| {
            self.viewport_overlay.render(ui);
        });
    }
}
```

---

## 2. Layout Systems

### 2.1 Docking System (ドッキングシステム)

**概念**: パネルを自由に配置・ドッキング

```rust
use egui_dock::{DockArea, DockState, NodeIndex, Style};

struct DockingLayout {
    dock_state: DockState<String>,
}

impl DockingLayout {
    fn new() -> Self {
        let mut dock_state = DockState::new(vec!["Viewport".to_string()]);
        
        // 左側にツールパネル
        let [left, _] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.2, // 20% の幅
            vec!["Tools".to_string()],
        );
        
        // 右側にプロパティパネル
        let [_, right] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.2,
            vec!["Properties".to_string()],
        );
        
        Self { dock_state }
    }
    
    fn render(&mut self, ctx: &egui::Context) {
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut TabViewer);
    }
}

struct TabViewer;

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;
    
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Viewport" => render_viewport(ui),
            "Tools" => render_tools(ui),
            "Properties" => render_properties(ui),
            _ => {}
        }
    }
    
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }
}
```

---

### 2.2 Workspace (ワークスペース)

**概念**: レイアウトの保存・切り替え

```rust
#[derive(Serialize, Deserialize, Clone)]
struct Workspace {
    name: String,
    dock_state: DockState<String>,
    ribbon_tabs: Vec<String>,
    visible_panels: HashSet<String>,
}

struct WorkspaceManager {
    workspaces: HashMap<String, Workspace>,
    active_workspace: String,
}

impl WorkspaceManager {
    fn save_current(&mut self, name: String, state: &AppState) {
        let workspace = Workspace {
            name: name.clone(),
            dock_state: state.dock_state.clone(),
            ribbon_tabs: state.ribbon.visible_tabs(),
            visible_panels: state.visible_panels.clone(),
        };
        
        self.workspaces.insert(name.clone(), workspace);
        
        // ファイルに保存
        self.save_to_file(&name);
    }
    
    fn load(&mut self, name: &str) -> Option<Workspace> {
        self.workspaces.get(name).cloned()
    }
    
    fn save_to_file(&self, name: &str) {
        let path = format!("workspaces/{}.json", name);
        if let Some(workspace) = self.workspaces.get(name) {
            let json = serde_json::to_string_pretty(workspace).unwrap();
            std::fs::write(path, json).ok();
        }
    }
}
```

**使用例**:
```rust
// ワークスペースの切り替え
if ui.button("2D Drafting").clicked() {
    app.workspace_manager.load("2D Drafting").map(|ws| {
        app.apply_workspace(ws);
    });
}

if ui.button("3D Modeling").clicked() {
    app.workspace_manager.load("3D Modeling").map(|ws| {
        app.apply_workspace(ws);
    });
}
```

---

## 3. UI Components

### 3.1 Ribbon Interface (リボンインターフェース)

```rust
struct RibbonInterface {
    tabs: Vec<RibbonTab>,
    active_tab: usize,
}

struct RibbonTab {
    name: String,
    groups: Vec<RibbonGroup>,
    contextual: bool, // コンテキストタブ
}

struct RibbonGroup {
    name: String,
    commands: Vec<Command>,
}

impl RibbonInterface {
    fn render(&mut self, ui: &mut egui::Ui) {
        // タブバー
        ui.horizontal(|ui| {
            for (i, tab) in self.tabs.iter().enumerate() {
                if tab.contextual && !self.is_context_active(&tab) {
                    continue; // 非アクティブなコンテキストタブは非表示
                }
                
                if ui.selectable_label(self.active_tab == i, &tab.name).clicked() {
                    self.active_tab = i;
                }
            }
        });
        
        ui.separator();
        
        // アクティブタブの内容
        if let Some(tab) = self.tabs.get(self.active_tab) {
            ui.horizontal(|ui| {
                for group in &tab.groups {
                    self.render_group(ui, group);
                    ui.separator();
                }
            });
        }
    }
    
    fn render_group(&self, ui: &mut egui::Ui, group: &RibbonGroup) {
        ui.vertical(|ui| {
            ui.label(&group.name);
            
            // コマンドボタン
            ui.horizontal_wrapped(|ui| {
                for cmd in &group.commands {
                    if ui.button(&cmd.label).clicked() {
                        cmd.execute();
                    }
                }
            });
        });
    }
}
```

**コンテキストタブ**:
```rust
impl RibbonInterface {
    fn update_contextual_tabs(&mut self, selection: &Selection) {
        // 選択に応じてタブを表示/非表示
        for tab in &mut self.tabs {
            if tab.contextual {
                tab.visible = match tab.name.as_str() {
                    "Solid Editing" => selection.has_solid(),
                    "Surface Editing" => selection.has_surface(),
                    "Sketch Tools" => selection.is_sketch_active(),
                    _ => false,
                };
            }
        }
    }
}
```

---

### 3.2 Property Panel (プロパティパネル)

```rust
struct PropertyPanel {
    selected_entity: Option<EntityId>,
    properties: Vec<Property>,
}

enum Property {
    Float { name: String, value: f32, min: f32, max: f32 },
    Color { name: String, value: Color },
    Enum { name: String, value: String, options: Vec<String> },
    Bool { name: String, value: bool },
}

impl PropertyPanel {
    fn render(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Properties");
        
        if let Some(entity_id) = self.selected_entity {
            let entity = state.get_entity(entity_id);
            
            // 基本情報
            ui.label(format!("Type: {:?}", entity.entity_type()));
            ui.label(format!("ID: {}", entity_id));
            
            ui.separator();
            
            // プロパティ
            egui::ScrollArea::vertical().show(ui, |ui| {
                for property in &mut self.properties {
                    self.render_property(ui, property, entity_id, state);
                }
            });
        } else {
            ui.label("No selection");
        }
    }
    
    fn render_property(
        &mut self,
        ui: &mut egui::Ui,
        property: &mut Property,
        entity_id: EntityId,
        state: &mut AppState
    ) {
        match property {
            Property::Float { name, value, min, max } => {
                ui.horizontal(|ui| {
                    ui.label(name);
                    if ui.add(egui::Slider::new(value, *min..=*max)).changed() {
                        state.update_entity_property(entity_id, name, *value);
                    }
                });
            }
            Property::Color { name, value } => {
                ui.horizontal(|ui| {
                    ui.label(name);
                    if ui.color_edit_button_rgb(value.as_mut()).changed() {
                        state.update_entity_property(entity_id, name, *value);
                    }
                });
            }
            Property::Enum { name, value, options } => {
                ui.horizontal(|ui| {
                    ui.label(name);
                    egui::ComboBox::from_id_source(name)
                        .selected_text(value.as_str())
                        .show_ui(ui, |ui| {
                            for option in options {
                                if ui.selectable_label(*value == *option, option).clicked() {
                                    *value = option.clone();
                                    state.update_entity_property(entity_id, name, value.clone());
                                }
                            }
                        });
                });
            }
            Property::Bool { name, value } => {
                if ui.checkbox(value, name).changed() {
                    state.update_entity_property(entity_id, name, *value);
                }
            }
        }
    }
}
```

**Progressive Disclosure (段階的開示)**:
```rust
fn render_advanced_properties(&mut self, ui: &mut egui::Ui) {
    ui.collapsing("Advanced", |ui| {
        // 高度なプロパティ
        ui.label("Tolerance: 0.001mm");
        ui.label("Material: Steel");
    });
}
```

---

### 3.3 Tool Palette (ツールパレット)

```rust
struct ToolPalette {
    tools: Vec<ToolButton>,
    active_tool: Option<usize>,
}

struct ToolButton {
    icon: egui::Image,
    label: String,
    tooltip: String,
    tool: Tool,
}

impl ToolPalette {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("tool_grid")
            .num_columns(2)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for (i, tool_btn) in self.tools.iter().enumerate() {
                    let selected = self.active_tool == Some(i);
                    
                    if ui.add(
                        egui::ImageButton::new(tool_btn.icon.clone())
                            .selected(selected)
                    ).on_hover_text(&tool_btn.tooltip).clicked() {
                        self.active_tool = Some(i);
                        // ツールを切り替え
                    }
                    
                    if (i + 1) % 2 == 0 {
                        ui.end_row();
                    }
                }
            });
    }
}
```

---

### 3.4 Context Menu (コンテキストメニュー)

```rust
fn render_context_menu(&mut self, ui: &mut egui::Ui, pos: Pos2) {
    ui.menu_button("Right Click", |ui| {
        if ui.button("Copy").clicked() {
            self.copy_selection();
            ui.close_menu();
        }
        if ui.button("Paste").clicked() {
            self.paste();
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Delete").clicked() {
            self.delete_selection();
            ui.close_menu();
        }
    });
}
```

---

## 4. State Management

### 4.1 MVVM Pattern

```rust
// Model
struct CadDocument {
    entities: HashMap<EntityId, Entity>,
    layers: Vec<Layer>,
}

// ViewModel
struct CadViewModel {
    document: CadDocument,
    selection: Selection,
    active_tool: Tool,
    
    // UI State
    property_panel_visible: bool,
    ribbon_collapsed: bool,
}

impl CadViewModel {
    fn select_entity(&mut self, id: EntityId) {
        self.selection.add(id);
        // プロパティパネルを更新
        self.notify_property_changed();
    }
    
    fn notify_property_changed(&mut self) {
        // UI に通知
    }
}

// View
impl CadViewModel {
    fn render(&mut self, ctx: &egui::Context) {
        // Ribbon
        self.render_ribbon(ctx);
        
        // Property Panel
        if self.property_panel_visible {
            self.render_property_panel(ctx);
        }
        
        // Viewport
        self.render_viewport(ctx);
    }
}
```

---

### 4.2 Command Pattern for Undo/Redo

```rust
struct UICommand {
    command: Box<dyn Command>,
    ui_state: UIState, // UI状態も保存
}

impl UICommand {
    fn execute(&mut self, app: &mut App) {
        self.command.execute(&mut app.document);
        app.ui_state = self.ui_state.clone();
    }
    
    fn undo(&mut self, app: &mut App) {
        self.command.undo(&mut app.document);
        // UI状態も復元
        app.ui_state = self.ui_state.clone();
    }
}
```

---

## 5. Rust Implementation

### 5.1 推奨ライブラリ

| 用途 | ライブラリ | 特徴 |
|------|-----------|------|
| **Immediate Mode** | `egui` | シンプル、高速、CAD向き |
| **Retained Mode** | `iced` | Elm風、型安全 |
| **Docking** | `egui_dock` | egui用ドッキング |
| **3D Viewport** | `wgpu` + `egui` | 混合モード |

---

### 5.2 完全な実装例

```rust
use egui::{Context, CentralPanel, SidePanel, TopBottomPanel};
use egui_dock::{DockArea, DockState};

struct CadApplication {
    // Core
    document: CadDocument,
    renderer: Renderer,
    
    // UI
    ribbon: RibbonInterface,
    tool_palette: ToolPalette,
    property_panel: PropertyPanel,
    dock_state: DockState<String>,
    
    // State
    workspace_manager: WorkspaceManager,
}

impl CadApplication {
    fn render(&mut self, ctx: &Context) {
        // Top: Ribbon
        TopBottomPanel::top("ribbon").show(ctx, |ui| {
            self.ribbon.render(ui);
        });
        
        // Bottom: Status Bar
        TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Entities: {}", self.document.entity_count()));
                ui.separator();
                ui.label(format!("Tool: {:?}", self.active_tool));
            });
        });
        
        // Docking Area
        CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.dock_state)
                .show_inside(ui, &mut TabViewer {
                    app: self,
                });
        });
    }
}

struct TabViewer<'a> {
    app: &'a mut CadApplication,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = String;
    
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Viewport" => {
                // 3D Viewport
                let (rect, response) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::click_and_drag()
                );
                
                // wgpu でレンダリング
                self.app.renderer.render(rect);
                
                // Overlay UI
                ui.allocate_ui_at_rect(rect, |ui| {
                    self.app.render_viewport_overlay(ui);
                });
            }
            "Tools" => self.app.tool_palette.render(ui),
            "Properties" => self.app.property_panel.render(ui, &mut self.app.document),
            "Layers" => self.app.render_layer_panel(ui),
            _ => {}
        }
    }
    
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }
}
```

---

### 5.3 パフォーマンス最適化

```rust
impl CadApplication {
    fn render_optimized(&mut self, ctx: &Context) {
        // 変更があった場合のみ再描画
        if self.needs_repaint() {
            ctx.request_repaint();
        }
        
        // 重い処理は遅延実行
        if self.property_panel_dirty {
            self.update_property_panel();
            self.property_panel_dirty = false;
        }
    }
    
    fn needs_repaint(&self) -> bool {
        self.is_animating
            || self.is_dragging
            || self.tool_active
    }
}
```

---

## 📊 UI パターン比較

| パターン | 複雑度 | 性能 | 保守性 | 推奨用途 |
|---------|--------|------|--------|---------|
| Immediate Mode | ⭐ | ⭐⭐ | ⭐⭐⭐ | ツールパレット、オーバーレイ |
| Retained Mode | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | リボン、ダイアログ |
| 混合モード | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | **CAD全般（推奨）** |

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
