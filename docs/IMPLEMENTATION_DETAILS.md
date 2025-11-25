# CAD Implementation Deep Dive: Technical Details

> **対象読者**: CADエンジニア、グラフィックスプログラマー、フレームワーク開発者
> 
> **目的**: 実装時に直面する「マニアックだが重要な」技術課題を網羅的に解説

---

## 📚 Table of Contents
1. [Rendering & Graphics](#1-rendering--graphics)
2. [Coordinate Systems & Transformations](#2-coordinate-systems--transformations)
3. [Floating-Point Precision & Tolerance](#3-floating-point-precision--tolerance)
4. [Spatial Indexing & Performance](#4-spatial-indexing--performance)
5. [File Formats & Interoperability](#5-file-formats--interoperability)
6. [Memory Management](#6-memory-management)

---

## 1. Rendering & Graphics

### 1.1 Anti-Aliasing (アンチエイリアシング)

#### 問題: Jaggies (ジャギー)
```
ピクセルベースの画面では、斜め線が階段状になる:
  ████
    ████
      ████  ← ギザギザ！
```

#### MSAA (Multisample Anti-Aliasing)
**仕組み**:
1. ピクセルの**エッジ部分のみ**を複数サンプリング
2. Fragment Shader は1回だけ実行（効率的）
3. サンプルを平均化して最終色を決定

```rust
// wgpu での MSAA 設定
let msaa_samples = 4; // 2x, 4x, 8x が一般的

let texture_desc = wgpu::TextureDescriptor {
    sample_count: msaa_samples,
    // ...
};
```

**トレードオフ**:
| MSAA Level | 品質 | VRAM使用量 | 性能 |
|-----------|------|-----------|------|
| 1x (OFF)  | ⭐   | 1x        | 最速 |
| 2x        | ⭐⭐  | 2x        | 速い |
| 4x        | ⭐⭐⭐ | 4x        | 中速 |
| 8x        | ⭐⭐⭐⭐ | 8x        | 遅い |

**CADでの推奨**: 4x MSAA（品質と性能のバランス）

#### 他のAA手法

**SSAA (Supersampling)**:
- 全体を高解像度でレンダリング → ダウンサンプル
- ✅ 最高品質
- ❌ 非常に重い（MSAA の 4〜8倍のコスト）

**FXAA (Fast Approximate)**:
- ポストプロセス（画像処理）でエッジを検出してぼかす
- ✅ 超高速
- ❌ テクスチャまでぼやける

**TAA (Temporal)**:
- 前フレームの情報を使って時間的に平滑化
- ✅ 高品質
- ❌ ゴースト（残像）が出やすい

---

### 1.2 Line Rendering (線の描画)

#### 問題: 1ピクセル幅の線は見づらい

**解決策1: Thick Lines (太線)**
```rust
// GPU は「太い線」をネイティブサポートしない
// → 線を矩形（Quad）に変換して描画

fn line_to_quad(start: Point, end: Point, width: f32) -> [Vertex; 4] {
    let dir = (end - start).normalize();
    let normal = Vector2::new(-dir.y, dir.x); // 垂直ベクトル
    let offset = normal * (width / 2.0);
    
    [
        start + offset,
        start - offset,
        end + offset,
        end - offset,
    ]
}
```

**解決策2: Shader-Based Lines**
```wgsl
// Fragment Shader で距離を計算してアンチエイリアス
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = distance_to_line(in.pos, line_start, line_end);
    let alpha = smoothstep(line_width + 1.0, line_width, dist);
    return vec4<f32>(color.rgb, alpha);
}
```

---

### 1.3 Depth Buffer (深度バッファ)

2D CADでも必要な理由:
```
重なり順の管理:
  グリッド (Z=0.0)
  図形    (Z=0.5)
  選択枠  (Z=0.9)
```

```rust
let depth_stencil_state = wgpu::DepthStencilState {
    format: wgpu::TextureFormat::Depth24Plus,
    depth_write_enabled: true,
    depth_compare: wgpu::CompareFunction::Less,
    // ...
};
```

---

## 2. Coordinate Systems & Transformations

### 2.1 座標系の変換パイプライン

```
Local Space (Object)
    ↓ Model Matrix
World Space
    ↓ View Matrix
View Space (Camera)
    ↓ Projection Matrix
Clip Space
    ↓ Perspective Division
NDC (Normalized Device Coordinates)
    ↓ Viewport Transform
Screen Space (Pixels)
```

### 2.2 各座標系の詳細

#### World Space (ワールド空間)
```rust
// CADの「図面」そのもの
// 単位: mm, inch, meter など実寸
let point_in_world = Point::new(1000.0, 500.0); // 1000mm, 500mm
```

#### NDC (Normalized Device Coordinates)
```
範囲: X, Y, Z ∈ [-1.0, 1.0]

  (-1, 1)  ┌─────────┐  (1, 1)
           │         │
           │  (0,0)  │  ← 画面中央
           │         │
  (-1,-1)  └─────────┘  (1,-1)
```

#### Screen Space (スクリーン空間)
```
範囲: X ∈ [0, width], Y ∈ [0, height]
原点: 左上 (0, 0)

  (0,0)    ┌─────────┐  (1920, 0)
           │         │
           │         │
           │         │
  (0,1080) └─────────┘  (1920, 1080)
```

### 2.3 変換行列の実装

```rust
use cgmath::{Matrix4, Vector3, Point2};

// World → NDC 変換
fn world_to_ndc(
    point: Point2<f32>,
    camera_pos: Vector3<f32>,
    zoom: f32,
    screen_size: (u32, u32)
) -> Point2<f32> {
    // 1. カメラ位置を引く (View Transform)
    let x = point.x - camera_pos.x;
    let y = point.y - camera_pos.y;
    
    // 2. ズームを適用
    let x = x * zoom;
    let y = y * zoom;
    
    // 3. Screen → NDC
    let ndc_x = (x / screen_size.0 as f32) * 2.0 - 1.0;
    let ndc_y = -((y / screen_size.1 as f32) * 2.0 - 1.0); // Y軸反転
    
    Point2::new(ndc_x, ndc_y)
}
```

### 2.4 逆変換（Screen → World）

```rust
// マウス座標 → ワールド座標
fn screen_to_world(
    screen_x: f32,
    screen_y: f32,
    camera: &Camera,
    screen_size: (u32, u32)
) -> Point2<f32> {
    // Screen → NDC
    let ndc_x = (screen_x / screen_size.0 as f32) * 2.0 - 1.0;
    let ndc_y = -((screen_y / screen_size.1 as f32) * 2.0 - 1.0);
    
    // NDC → World
    let world_x = (ndc_x / camera.zoom) + camera.position.x;
    let world_y = (ndc_y / camera.zoom) + camera.position.y;
    
    Point2::new(world_x, world_y)
}
```

---

## 3. Floating-Point Precision & Tolerance

### 3.1 IEEE 754 の限界

```rust
// ❌ これは失敗する
let a = 0.1 + 0.2;
let b = 0.3;
assert_eq!(a, b); // Panic! 0.30000000000000004 != 0.3
```

**原因**: 浮動小数点数は2進数で表現される
```
0.1 (10進) = 0.0001100110011... (2進, 無限循環)
→ 丸め誤差が発生
```

### 3.2 Epsilon 比較

```rust
const EPSILON: f32 = 1e-6; // 0.000001

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

// 使用例
if approx_eq(point1.x, point2.x) {
    println!("X座標が一致");
}
```

### 3.3 CADにおける許容誤差

```rust
pub struct GeometricTolerance {
    pub linear: f32,  // 線形許容差 (mm)
    pub angular: f32, // 角度許容差 (度)
}

impl Default for GeometricTolerance {
    fn default() -> Self {
        Self {
            linear: 0.001,  // 1μm
            angular: 0.01,  // 0.01度
        }
    }
}
```

### 3.4 相対誤差 vs 絶対誤差

```rust
// ❌ 絶対誤差のみでは不十分
fn bad_approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-6
}
// 問題: 1000000.0 と 1000000.1 が「等しい」と判定される

// ✅ 相対誤差も考慮
fn good_approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    let abs_diff = (a - b).abs();
    let max_val = a.abs().max(b.abs());
    
    abs_diff <= epsilon * max_val.max(1.0)
}
```

### 3.5 数値安定性のベストプラクティス

```rust
// ❌ 悪い例: 小さい数と大きい数の足し算
let result = 1e-10 + 1e10 - 1e10; // 0.0 になる（精度消失）

// ✅ 良い例: 同じスケールの数同士を計算
let small_sum = 1e-10 + 1e-10 + 1e-10;
let result = small_sum + 1e10 - 1e10;
```

---

## 4. Spatial Indexing & Performance

### 4.1 問題: 線形探索の限界

```rust
// ❌ O(n) - 10万個の図形で破綻
fn find_entity_at(pos: Point, entities: &[Entity]) -> Option<usize> {
    for (i, entity) in entities.iter().enumerate() {
        if entity.contains(pos) {
            return Some(i);
        }
    }
    None
}
```

### 4.2 QuadTree (四分木)

**構造**:
```
┌─────────────┬─────────────┐
│  NW (北西)  │  NE (北東)  │
│             │             │
├─────────────┼─────────────┤
│  SW (南西)  │  SE (南東)  │
│             │             │
└─────────────┴─────────────┘
```

**実装**:
```rust
struct QuadTree {
    bounds: Rect,
    capacity: usize,
    entities: Vec<(EntityId, Rect)>,
    subdivided: bool,
    children: Option<Box<[QuadTree; 4]>>,
}

impl QuadTree {
    fn insert(&mut self, id: EntityId, bounds: Rect) {
        if !self.bounds.intersects(&bounds) {
            return;
        }
        
        if self.entities.len() < self.capacity && !self.subdivided {
            self.entities.push((id, bounds));
        } else {
            if !self.subdivided {
                self.subdivide();
            }
            for child in self.children.as_mut().unwrap().iter_mut() {
                child.insert(id, bounds);
            }
        }
    }
    
    fn query(&self, area: Rect) -> Vec<EntityId> {
        let mut result = Vec::new();
        
        if !self.bounds.intersects(&area) {
            return result;
        }
        
        for (id, bounds) in &self.entities {
            if area.intersects(bounds) {
                result.push(*id);
            }
        }
        
        if let Some(children) = &self.children {
            for child in children.iter() {
                result.extend(child.query(area));
            }
        }
        
        result
    }
}
```

**性能**:
- 挿入: O(log n)
- 検索: O(log n) （理想的な場合）

---

### 4.3 R-Tree

**特徴**:
- Minimum Bounding Rectangle (MBR) でグループ化
- データベース（PostgreSQL/PostGIS）で広く使用

```rust
struct RTreeNode {
    mbr: Rect, // Minimum Bounding Rectangle
    children: Vec<RTreeNode>,
    entities: Vec<EntityId>,
}
```

**QuadTree vs R-Tree**:
| 特性 | QuadTree | R-Tree |
|------|----------|--------|
| 分割方法 | 空間を均等分割 | データに基づいて分割 |
| 動的データ | △ | ✅ |
| 実装難易度 | 易 | 難 |
| クエリ性能 | ✅ | ✅ |
| メモリ効率 | △ | ✅ |

---

### 4.4 Dirty Rectangle (差分描画)

```rust
struct DirtyRegionManager {
    dirty_rects: Vec<Rect>,
}

impl DirtyRegionManager {
    fn mark_dirty(&mut self, rect: Rect) {
        self.dirty_rects.push(rect);
    }
    
    fn get_dirty_region(&self) -> Option<Rect> {
        if self.dirty_rects.is_empty() {
            return None;
        }
        
        // 全ての dirty rect を包含する最小矩形を計算
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        
        for rect in &self.dirty_rects {
            min_x = min_x.min(rect.x);
            min_y = min_y.min(rect.y);
            max_x = max_x.max(rect.x + rect.width);
            max_y = max_y.max(rect.y + rect.height);
        }
        
        Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }
    
    fn clear(&mut self) {
        self.dirty_rects.clear();
    }
}
```

---

## 5. File Formats & Interoperability

### 5.1 主要なCADファイル形式

| 形式 | 拡張子 | 用途 | 特徴 |
|------|--------|------|------|
| **DXF** | .dxf | 2D図面交換 | テキスト形式、広く対応 |
| **DWG** | .dwg | AutoCAD | バイナリ、業界標準 |
| **STEP** | .stp, .step | 3D交換 | ISO標準、B-rep対応 |
| **IGES** | .igs, .iges | 3D交換 | 古い標準 |
| **SVG** | .svg | 2Dベクター | Web標準、XML |

### 5.2 DXF の基本構造

```
0
SECTION
2
ENTITIES
0
LINE
8
0          ← レイヤー名
10
0.0        ← 始点X
20
0.0        ← 始点Y
11
100.0      ← 終点X
21
50.0       ← 終点Y
0
ENDSEC
0
EOF
```

### 5.3 Serde でのシリアライズ

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CadDocument {
    version: String,
    entities: Vec<Entity>,
    layers: Vec<Layer>,
}

// JSON保存
fn save_json(doc: &CadDocument, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(doc)?;
    std::fs::write(path, json)?;
    Ok(())
}

// バイナリ保存 (bincode)
fn save_binary(doc: &CadDocument, path: &Path) -> Result<()> {
    let bytes = bincode::serialize(doc)?;
    std::fs::write(path, bytes)?;
    Ok(())
}
```

---

## 6. Memory Management

### 6.1 Entity ID の管理

```rust
// ❌ 悪い例: Vec のインデックス
// 削除時にインデックスがずれる
entities.remove(5); // ID=6以降が全部ずれる

// ✅ 良い例: Generational Arena
use slotmap::{SlotMap, DefaultKey};

pub type EntityId = DefaultKey;

struct GeometryStore {
    entities: SlotMap<EntityId, Entity>,
}

impl GeometryStore {
    fn add(&mut self, entity: Entity) -> EntityId {
        self.entities.insert(entity)
    }
    
    fn remove(&mut self, id: EntityId) {
        self.entities.remove(id);
    }
    
    fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }
}
```

### 6.2 Copy-on-Write (CoW) for Undo/Redo

```rust
use std::sync::Arc;

struct CadDocument {
    entities: Arc<HashMap<EntityId, Entity>>,
}

impl CadDocument {
    fn modify_entity(&mut self, id: EntityId, new_entity: Entity) {
        // Arc::make_mut は必要な時だけクローンする
        let entities = Arc::make_mut(&mut self.entities);
        entities.insert(id, new_entity);
    }
}
```

### 6.3 GPU メモリ管理

```rust
// Vertex Buffer の動的更新
struct DynamicVertexBuffer {
    buffer: wgpu::Buffer,
    capacity: usize,
}

impl DynamicVertexBuffer {
    fn update(&mut self, device: &wgpu::Device, vertices: &[Vertex]) {
        if vertices.len() > self.capacity {
            // 容量不足 → 再確保
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                size: (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                // ...
            });
            self.capacity = vertices.len();
        }
        
        // データ転送
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(vertices));
    }
}
```

---

## 📖 参考文献

### 論文・書籍
- "Real-Time Rendering" (Tomas Akenine-Möller et al.)
- "Geometric Tools for Computer Graphics" (Philip Schneider)
- "Spatial Indexing with R-Trees" (Antonin Guttman, 1984)

### 標準規格
- IEEE 754 (浮動小数点演算)
- ISO 10303 (STEP)
- ISO 13584 (Parts Library)

### オープンソース実装
- **Open CASCADE**: B-rep カーネル
- **CGAL**: 計算幾何アルゴリズムライブラリ
- **rstar**: Rust の R-Tree 実装

---

## 🎯 まとめ

CAD実装は「見た目」だけでなく、以下の深い技術知識が必要:

1. **グラフィックス**: MSAA, 線描画, 深度バッファ
2. **座標変換**: World/NDC/Screen の理解
3. **数値計算**: Epsilon比較, 許容誤差
4. **データ構造**: QuadTree, R-Tree
5. **ファイルI/O**: DXF, STEP, Serde
6. **メモリ**: Generational Arena, CoW

**Rust CAD Framework** は、これらを適切に抽象化し、
開発者が「CADの本質（作図ロジック）」に集中できるようにする。

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
