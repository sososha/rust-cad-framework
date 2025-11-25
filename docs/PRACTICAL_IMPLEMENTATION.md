# Practical Implementation Patterns

> **対象**: 実践的なCAD機能を実装する開発者
> 
> **目的**: EntityID, ファイルI/O, スナップなど、実装パターンの完全ガイド

---

## 📚 Table of Contents
1. [Entity ID Management](#1-entity-id-management)
2. [File I/O & Serialization](#2-file-io--serialization)
3. [Snap System](#3-snap-system)
4. [Selection System](#4-selection-system)
5. [Layer Management](#5-layer-management)

---

## 1. Entity ID Management

### 1.1 問題: Vec のインデックスは不安定

```rust
// ❌ 悪い例
struct GeometryStore {
    entities: Vec<Entity>,
}

// 削除すると後続のインデックスがずれる
entities.remove(5); // ID 6 が ID 5 になる！
```

---

### 1.2 解決: slotmap による安定ID

```rust
use slotmap::{SlotMap, new_key_type};

// EntityId 型を定義
new_key_type! {
    pub struct EntityId;
}

struct GeometryStore {
    entities: SlotMap<EntityId, Entity>,
}

impl GeometryStore {
    fn new() -> Self {
        Self {
            entities: SlotMap::with_key(),
        }
    }
    
    fn add_entity(&mut self, entity: Entity) -> EntityId {
        self.entities.insert(entity)
    }
    
    fn remove_entity(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(id)
    }
    
    fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }
    
    fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(id)
    }
}
```

---

### 1.3 Serialization 対応

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SerializableDocument {
    entities: Vec<(u64, Entity)>, // (key_data, entity)
}

impl GeometryStore {
    fn to_serializable(&self) -> SerializableDocument {
        SerializableDocument {
            entities: self.entities.iter()
                .map(|(id, entity)| (id.data().as_ffi(), entity.clone()))
                .collect(),
        }
    }
    
    fn from_serializable(doc: SerializableDocument) -> Self {
        let mut store = Self::new();
        for (key_data, entity) in doc.entities {
            let id = EntityId::from(slotmap::KeyData::from_ffi(key_data));
            store.entities.insert_with_key(|_| entity);
        }
        store
    }
}
```

---

## 2. File I/O & Serialization

### 2.1 対応形式

| 形式 | 用途 | 実装難易度 |
|------|------|-----------|
| **JSON** | 独自形式、デバッグ | ⭐ |
| **Binary** | 高速、小サイズ | ⭐⭐ |
| **DXF** | AutoCAD互換 | ⭐⭐⭐⭐ |
| **SVG** | 2Dエクスポート | ⭐⭐ |
| **STEP** | 3D交換 | ⭐⭐⭐⭐⭐ |

---

### 2.2 JSON 形式（独自）

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CadDocument {
    version: String,
    entities: Vec<Entity>,
    layers: Vec<Layer>,
    metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Metadata {
    author: String,
    created: String,
    modified: String,
    units: String, // "mm", "inch"
}

impl CadDocument {
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let doc = serde_json::from_str(&json)?;
        Ok(doc)
    }
}
```

**ファイル例** (`drawing.json`):
```json
{
  "version": "1.0",
  "entities": [
    {
      "Line": {
        "p1": { "x": 0.0, "y": 0.0 },
        "p2": { "x": 100.0, "y": 100.0 }
      }
    }
  ],
  "layers": [
    { "name": "Layer 1", "visible": true, "color": [255, 255, 255] }
  ],
  "metadata": {
    "author": "User",
    "created": "2025-11-25T10:00:00Z",
    "modified": "2025-11-25T10:30:00Z",
    "units": "mm"
  }
}
```

---

### 2.3 Binary 形式（高速）

```rust
use bincode;

impl CadDocument {
    pub fn save_binary(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = bincode::serialize(self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
    
    pub fn load_binary(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        let doc = bincode::deserialize(&bytes)?;
        Ok(doc)
    }
}
```

**性能比較**:
- JSON: 1MB, 50ms
- Binary: 200KB, 5ms (10倍高速)

---

### 2.4 DXF エクスポート

```rust
pub fn export_dxf(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = String::new();
    
    // Header
    output.push_str("0\nSECTION\n2\nHEADER\n");
    output.push_str("9\n$ACADVER\n1\nAC1015\n"); // AutoCAD 2000
    output.push_str("0\nENDSEC\n");
    
    // Entities
    output.push_str("0\nSECTION\n2\nENTITIES\n");
    
    for entity in &self.entities {
        match entity {
            Entity::Line { p1, p2 } => {
                output.push_str(&format!(
                    "0\nLINE\n8\n0\n10\n{}\n20\n{}\n11\n{}\n21\n{}\n",
                    p1.x, p1.y, p2.x, p2.y
                ));
            }
            Entity::Circle { center, radius } => {
                output.push_str(&format!(
                    "0\nCIRCLE\n8\n0\n10\n{}\n20\n{}\n40\n{}\n",
                    center.x, center.y, radius
                ));
            }
            _ => {}
        }
    }
    
    output.push_str("0\nENDSEC\n");
    output.push_str("0\nEOF\n");
    
    std::fs::write(path, output)?;
    Ok(())
}
```

---

### 2.5 SVG エクスポート

```rust
pub fn export_svg(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut svg = String::new();
    
    // SVG Header
    svg.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="1000">"#);
    
    // Entities
    for entity in &self.entities {
        match entity {
            Entity::Line { p1, p2 } => {
                svg.push_str(&format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" />"#,
                    p1.x, p1.y, p2.x, p2.y
                ));
            }
            Entity::Circle { center, radius } => {
                svg.push_str(&format!(
                    r#"<circle cx="{}" cy="{}" r="{}" fill="none" stroke="black" />"#,
                    center.x, center.y, radius
                ));
            }
            _ => {}
        }
    }
    
    svg.push_str("</svg>");
    
    std::fs::write(path, svg)?;
    Ok(())
}
```

---

### 2.6 バージョニング

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
enum VersionedDocument {
    #[serde(rename = "1.0")]
    V1_0(DocumentV1_0),
    #[serde(rename = "2.0")]
    V2_0(DocumentV2_0),
}

impl VersionedDocument {
    pub fn load(path: &Path) -> Result<CadDocument, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let versioned: VersionedDocument = serde_json::from_str(&json)?;
        
        match versioned {
            VersionedDocument::V1_0(doc) => Ok(doc.migrate_to_v2().into()),
            VersionedDocument::V2_0(doc) => Ok(doc.into()),
        }
    }
}
```

---

## 3. Snap System

### 3.1 Grid Snap (グリッドスナップ)

```rust
pub struct SnapSystem {
    grid_size: f32,
    snap_enabled: bool,
}

impl SnapSystem {
    pub fn snap_to_grid(&self, point: Point) -> Point {
        if !self.snap_enabled {
            return point;
        }
        
        Point {
            x: (point.x / self.grid_size).round() * self.grid_size,
            y: (point.y / self.grid_size).round() * self.grid_size,
        }
    }
}
```

---

### 3.2 Object Snap (オブジェクトスナップ)

```rust
pub enum SnapPoint {
    Endpoint,
    Midpoint,
    Center,
    Intersection,
}

impl SnapSystem {
    pub fn find_snap_points(&self, entities: &[Entity], cursor: Point, threshold: f32) -> Vec<(Point, SnapPoint)> {
        let mut snap_points = Vec::new();
        
        for entity in entities {
            match entity {
                Entity::Line { p1, p2 } => {
                    // Endpoint
                    if (cursor - *p1).len() < threshold {
                        snap_points.push((*p1, SnapPoint::Endpoint));
                    }
                    if (cursor - *p2).len() < threshold {
                        snap_points.push((*p2, SnapPoint::Endpoint));
                    }
                    
                    // Midpoint
                    let mid = Point {
                        x: (p1.x + p2.x) / 2.0,
                        y: (p1.y + p2.y) / 2.0,
                    };
                    if (cursor - mid).len() < threshold {
                        snap_points.push((mid, SnapPoint::Midpoint));
                    }
                }
                Entity::Circle { center, radius } => {
                    // Center
                    if (cursor - *center).len() < threshold {
                        snap_points.push((*center, SnapPoint::Center));
                    }
                }
                _ => {}
            }
        }
        
        snap_points
    }
    
    pub fn get_nearest_snap(&self, snap_points: &[(Point, SnapPoint)], cursor: Point) -> Option<Point> {
        snap_points.iter()
            .min_by(|(p1, _), (p2, _)| {
                let d1 = (cursor - *p1).len();
                let d2 = (cursor - *p2).len();
                d1.partial_cmp(&d2).unwrap()
            })
            .map(|(p, _)| *p)
    }
}
```

---

### 3.3 Angle Snap (角度スナップ)

```rust
impl SnapSystem {
    pub fn snap_angle(&self, start: Point, end: Point, angle_step: f32) -> Point {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let angle = dy.atan2(dx);
        
        // 最も近い角度にスナップ
        let snapped_angle = (angle / angle_step).round() * angle_step;
        
        let length = (dx * dx + dy * dy).sqrt();
        
        Point {
            x: start.x + length * snapped_angle.cos(),
            y: start.y + length * snapped_angle.sin(),
        }
    }
}
```

---

## 4. Selection System

### 4.1 基本的な選択

```rust
pub struct SelectionManager {
    selected: HashSet<EntityId>,
}

impl SelectionManager {
    pub fn select(&mut self, id: EntityId) {
        self.selected.insert(id);
    }
    
    pub fn deselect(&mut self, id: EntityId) {
        self.selected.remove(&id);
    }
    
    pub fn toggle(&mut self, id: EntityId) {
        if self.selected.contains(&id) {
            self.selected.remove(&id);
        } else {
            self.selected.insert(id);
        }
    }
    
    pub fn clear(&mut self) {
        self.selected.clear();
    }
    
    pub fn is_selected(&self, id: EntityId) -> bool {
        self.selected.contains(&id)
    }
}
```

---

### 4.2 矩形選択

```rust
impl SelectionManager {
    pub fn select_in_rect(&mut self, rect: Rect, entities: &SlotMap<EntityId, Entity>) {
        for (id, entity) in entities.iter() {
            if entity.intersects_rect(rect) {
                self.selected.insert(id);
            }
        }
    }
}

impl Entity {
    fn intersects_rect(&self, rect: Rect) -> bool {
        match self {
            Entity::Line { p1, p2 } => {
                rect.contains(*p1) || rect.contains(*p2)
            }
            Entity::Circle { center, radius } => {
                rect.contains(*center)
            }
            _ => false,
        }
    }
}
```

---

## 5. Layer Management

### 5.1 Layer 構造

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub color: Color,
    pub entities: HashSet<EntityId>,
}

pub struct LayerManager {
    layers: Vec<Layer>,
    active_layer: usize,
}

impl LayerManager {
    pub fn add_layer(&mut self, name: String) {
        self.layers.push(Layer {
            name,
            visible: true,
            locked: false,
            color: Color::WHITE,
            entities: HashSet::new(),
        });
    }
    
    pub fn set_active(&mut self, index: usize) {
        if index < self.layers.len() {
            self.active_layer = index;
        }
    }
    
    pub fn add_entity_to_active(&mut self, id: EntityId) {
        if let Some(layer) = self.layers.get_mut(self.active_layer) {
            layer.entities.insert(id);
        }
    }
    
    pub fn is_visible(&self, id: EntityId) -> bool {
        self.layers.iter().any(|layer| {
            layer.visible && layer.entities.contains(&id)
        })
    }
}
```

---

## 📊 実装パターン比較

| パターン | 複雑度 | 性能 | 推奨度 |
|---------|--------|------|--------|
| **Vec Index** | ⭐ | ⭐⭐⭐ | ❌ 不安定 |
| **SlotMap** | ⭐⭐ | ⭐⭐⭐ | ✅ 推奨 |
| **HashMap** | ⭐⭐ | ⭐⭐ | △ 可 |
| **JSON** | ⭐ | ⭐ | ✅ デバッグ |
| **Binary** | ⭐⭐ | ⭐⭐⭐ | ✅ 本番 |
| **DXF** | ⭐⭐⭐⭐ | ⭐⭐ | ✅ 互換性 |

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
