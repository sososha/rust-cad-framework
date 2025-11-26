# CAD Data Structures, Storage & Memory Layout

> **対象**: CADタイプ別のデータ構造、保存形式、メモリ/Redis展開を理解したい開発者
> 
> **目的**: 2D/3D CAD、ベクター/ラスターの違い、ファイル形式、メモリレイアウト、Redis活用を詳細解説

---

## 📚 Table of Contents
1. [Data Structure Comparison](#1-data-structure-comparison)
2. [File Format Deep Dive](#2-file-format-deep-dive)
3. [Memory Layout](#3-memory-layout)
4. [Redis for Collaborative CAD](#4-redis-for-collaborative-cad)
5. [Implementation Examples](#5-implementation-examples)

---

## 1. Data Structure Comparison

### 1.1 ベクターグラフィックス vs ラスターグラフィックス

| 特性 | ベクター (CAD) | ラスター (画像) |
|------|---------------|----------------|
| **データ表現** | 数式・命令 | ピクセル配列 |
| **メモリ使用量** | 小 (命令数に依存) | 大 (width × height × 4 bytes) |
| **拡大縮小** | 無限に拡大可能 | 拡大で劣化 |
| **編集** | 個別オブジェクト | ピクセル単位 |
| **ファイルサイズ** | 小 | 大 |

#### ベクターデータ構造

```rust
// ベクター: 数学的定義
struct VectorLine {
    p1: Point,           // 8 bytes (f32 × 2)
    p2: Point,           // 8 bytes
    color: Color,        // 4 bytes (RGBA)
    thickness: f32,      // 4 bytes
}
// 合計: 24 bytes

// どれだけ拡大しても、24 bytes のまま
```

#### ラスターデータ構造

```rust
// ラスター: ピクセル配列
struct RasterImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,     // width × height × 4 bytes (RGBA)
}

// 1920×1080 の画像
// メモリ: 1920 × 1080 × 4 = 8,294,400 bytes (約 8MB)
```

---

### 1.2 2D CAD vs 3D CAD データ構造

#### 2D CAD (DXF/DWG)

```rust
// 2D CAD: 平面図形
#[derive(Clone, Serialize, Deserialize)]
pub enum Entity2D {
    Line {
        p1: Point2D,          // (x, y)
        p2: Point2D,
        layer: String,
        color: Color,
    },
    Circle {
        center: Point2D,
        radius: f32,
        layer: String,
    },
    Arc {
        center: Point2D,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    },
    Polyline {
        vertices: Vec<Point2D>,
        closed: bool,
    },
    Text {
        position: Point2D,
        content: String,
        height: f32,
        rotation: f32,
    },
}

// メモリレイアウト
// Line: 32 bytes (2 points + layer + color)
// Circle: 24 bytes (center + radius + layer)
```

#### 3D CAD (STEP/IGES)

```rust
// 3D CAD: NURBS/B-Rep
#[derive(Clone, Serialize, Deserialize)]
pub enum Entity3D {
    // NURBS 曲面
    NURBSSurface {
        control_points: Vec<Vec<Point3D>>,  // 制御点グリッド
        knots_u: Vec<f32>,                  // U方向ノットベクトル
        knots_v: Vec<f32>,                  // V方向ノットベクトル
        degree_u: usize,
        degree_v: usize,
    },
    
    // B-Rep (Boundary Representation)
    BRep {
        vertices: Vec<Point3D>,
        edges: Vec<Edge>,
        faces: Vec<Face>,
        shells: Vec<Shell>,
    },
    
    // メッシュ (テッセレーション後)
    Mesh {
        vertices: Vec<Point3D>,
        triangles: Vec<[usize; 3]>,  // インデックス
        normals: Vec<Vector3>,
    },
}

// メモリレイアウト
// NURBS Surface (10×10 制御点): 
//   100 points × 12 bytes = 1,200 bytes + knots
// Mesh (1000 triangles):
//   ~3000 vertices × 12 bytes = 36,000 bytes
```

---

## 2. File Format Deep Dive

### 2.1 DXF (Drawing Interchange Format)

#### 構造

```
DXF File Structure:
┌─────────────────┐
│ HEADER Section  │ ← システム変数
├─────────────────┤
│ CLASSES Section │ ← カスタムクラス定義
├─────────────────┤
│ TABLES Section  │ ← レイヤー、スタイル定義
├─────────────────┤
│ BLOCKS Section  │ ← ブロック定義
├─────────────────┤
│ ENTITIES Section│ ← 図形エンティティ (メイン)
├─────────────────┤
│ OBJECTS Section │ ← 非図形オブジェクト
└─────────────────┘
```

#### DXF フォーマット例

```dxf
0
SECTION
2
HEADER
9
$ACADVER
1
AC1015
0
ENDSEC
0
SECTION
2
ENTITIES
0
LINE
8
0
10
0.0
20
0.0
11
100.0
21
100.0
0
CIRCLE
8
0
10
50.0
20
50.0
40
25.0
0
ENDSEC
0
EOF
```

**グループコード**:
- `0`: エンティティタイプ
- `8`: レイヤー名
- `10, 20`: 始点 (x, y)
- `11, 21`: 終点 (x, y)
- `40`: 半径

#### Rust 実装

```rust
pub struct DXFWriter {
    output: String,
}

impl DXFWriter {
    pub fn write_line(&mut self, p1: Point, p2: Point, layer: &str) {
        self.output.push_str(&format!(
            "0\nLINE\n8\n{}\n10\n{}\n20\n{}\n11\n{}\n21\n{}\n",
            layer, p1.x, p1.y, p2.x, p2.y
        ));
    }
    
    pub fn write_circle(&mut self, center: Point, radius: f32, layer: &str) {
        self.output.push_str(&format!(
            "0\nCIRCLE\n8\n{}\n10\n{}\n20\n{}\n40\n{}\n",
            layer, center.x, center.y, radius
        ));
    }
}
```

---

### 2.2 DWG (Drawing - AutoCAD Native)

#### 構造

```
DWG File Structure (Binary):
┌─────────────────────┐
│ Header              │ ← バージョン、CRC
├─────────────────────┤
│ Class Definitions   │ ← クラスメタデータ
├─────────────────────┤
│ Object Data         │ ← エンティティ (バイナリ)
├─────────────────────┤
│ Object Map          │ ← ハンドル → オフセット
├─────────────────────┤
│ Padding/Template    │ ← 互換性
└─────────────────────┘
```

**特徴**:
- **バイナリ形式**: 人間が読めない
- **ビットレベル圧縮**: 非常に効率的
- **ハンドルシステム**: オブジェクト間参照

#### メモリ効率

| 形式 | 同じ図面のサイズ |
|------|-----------------|
| DXF (ASCII) | 1.0 MB |
| DWG (Binary) | 0.3 MB |

---

### 2.3 STEP (Standard for the Exchange of Product model data)

#### 構造

```step
ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('CAD Model'),'2;1');
FILE_NAME('model.stp','2025-11-26T12:00:00',('Author'),('Org'),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
ENDSEC;
DATA;
#1=CARTESIAN_POINT('',(0.,0.,0.));
#2=DIRECTION('',(0.,0.,1.));
#3=AXIS2_PLACEMENT_3D('',#1,#2,#4);
#4=DIRECTION('',(1.,0.,0.));
#5=CYLINDRICAL_SURFACE('',#3,10.0);
ENDSEC;
END-ISO-10303-21;
```

**特徴**:
- **NURBS/B-Rep**: 数学的に正確
- **無限精度**: 拡大しても劣化なし
- **編集可能**: 完全な形状情報を保持

#### データ構造

```rust
pub struct STEPEntity {
    id: usize,
    entity_type: String,
    attributes: Vec<STEPValue>,
}

pub enum STEPValue {
    Integer(i64),
    Real(f64),
    String(String),
    EntityRef(usize),
    List(Vec<STEPValue>),
}

// 例: CARTESIAN_POINT
STEPEntity {
    id: 1,
    entity_type: "CARTESIAN_POINT".to_string(),
    attributes: vec![
        STEPValue::String("".to_string()),
        STEPValue::List(vec![
            STEPValue::Real(0.0),
            STEPValue::Real(0.0),
            STEPValue::Real(0.0),
        ]),
    ],
}
```

---

### 2.4 SVG (Scalable Vector Graphics)

#### 構造

```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="1000">
  <line x1="0" y1="0" x2="100" y2="100" stroke="black" stroke-width="2"/>
  <circle cx="50" cy="50" r="25" fill="none" stroke="black" stroke-width="2"/>
  <path d="M 10 10 L 90 90 L 90 10 Z" fill="blue"/>
</svg>
```

**メモリレイアウト**:
- **DOM ツリー**: XML 要素のオブジェクトモデル
- **属性ベース**: 各要素が属性を持つ

```rust
pub struct SVGElement {
    tag: String,
    attributes: HashMap<String, String>,
    children: Vec<SVGElement>,
}

// メモリ使用量: 要素数 × (タグ + 属性) のサイズ
// 複雑なSVG (1000要素): ~100KB
```

---

## 3. Memory Layout

### 3.1 2D CAD メモリレイアウト

```rust
// Structure of Arrays (SoA) - キャッシュ効率が良い
pub struct GeometryStore2D {
    // 全ての線
    lines: Vec<Line2D>,
    
    // 全ての円
    circles: Vec<Circle2D>,
    
    // 全ての円弧
    arcs: Vec<Arc2D>,
    
    // 空間インデックス (高速検索)
    spatial_index: RTree<EntityId>,
}

// メモリレイアウト
// lines: [Line1][Line2][Line3]...
// circles: [Circle1][Circle2]...
// 
// 利点: 同じ型のデータが連続 → キャッシュヒット率向上
```

#### メモリ使用量計算

```rust
// 10,000 エンティティの場合
struct MemoryUsage {
    lines: 10000 * 32,      // 320 KB
    circles: 5000 * 24,     // 120 KB
    arcs: 3000 * 40,        // 120 KB
    spatial_index: 50,      // 50 KB (R-Tree)
    total: 610,             // 610 KB
}
```

---

### 3.2 3D CAD メモリレイアウト

```rust
// 3D メッシュデータ
pub struct Mesh3D {
    // 頂点配列 (SoA)
    positions: Vec<[f32; 3]>,     // x, y, z
    normals: Vec<[f32; 3]>,       // nx, ny, nz
    uvs: Vec<[f32; 2]>,           // u, v
    
    // インデックス配列
    indices: Vec<u32>,            // 三角形インデックス
    
    // GPU バッファ
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
}

// メモリレイアウト
// CPU:
//   positions: [v1.x, v1.y, v1.z][v2.x, v2.y, v2.z]...
//   normals:   [n1.x, n1.y, n1.z][n2.x, n2.y, n2.z]...
//   indices:   [0, 1, 2][3, 4, 5]...
//
// GPU:
//   Vertex Buffer: positions + normals + uvs (インターリーブ)
//   Index Buffer: indices
```

#### メモリ使用量計算

```rust
// 100,000 三角形のメッシュ
struct Mesh3DMemory {
    vertices: 300000 * 12,      // 3.6 MB (positions)
    normals: 300000 * 12,       // 3.6 MB
    uvs: 300000 * 8,            // 2.4 MB
    indices: 300000 * 4,        // 1.2 MB
    total_cpu: 10.8,            // 10.8 MB (CPU)
    total_gpu: 10.8,            // 10.8 MB (GPU)
    grand_total: 21.6,          // 21.6 MB
}
```

---

### 3.3 Canvas (ラスター) メモリレイアウト

```rust
// HTML Canvas 相当
pub struct CanvasBuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,  // RGBA
}

impl CanvasBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            pixels: vec![0; size],
        }
    }
    
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let index = ((y * self.width + x) * 4) as usize;
        self.pixels[index..index + 4].copy_from_slice(&color);
    }
}

// メモリレイアウト
// pixels: [R1, G1, B1, A1][R2, G2, B2, A2]...
//
// 1920×1080 Canvas:
//   1920 × 1080 × 4 = 8,294,400 bytes (約 8 MB)
```

---

## 4. Redis for Collaborative CAD

### 4.1 なぜ Redis か

| 要件 | Redis の機能 |
|------|-------------|
| **リアルタイム同期** | Pub/Sub |
| **高速アクセス** | インメモリ (μs レベル) |
| **複雑なデータ** | Hash, List, Set, JSON |
| **永続化** | RDB, AOF |
| **スケーラビリティ** | クラスタリング |

---

### 4.2 Redis データ構造

#### エンティティの保存

```rust
// Redis Hash でエンティティを保存
use redis::Commands;

pub struct RedisCADStore {
    client: redis::Client,
}

impl RedisCADStore {
    pub fn save_entity(&mut self, id: EntityId, entity: &Entity) -> Result<(), redis::RedisError> {
        let mut con = self.client.get_connection()?;
        let key = format!("entity:{}", id.data().as_ffi());
        
        match entity {
            Entity::Line { p1, p2 } => {
                con.hset_multiple(&key, &[
                    ("type", "line"),
                    ("p1_x", &p1.x.to_string()),
                    ("p1_y", &p1.y.to_string()),
                    ("p2_x", &p2.x.to_string()),
                    ("p2_y", &p2.y.to_string()),
                ])?;
            }
            Entity::Circle { center, radius } => {
                con.hset_multiple(&key, &[
                    ("type", "circle"),
                    ("center_x", &center.x.to_string()),
                    ("center_y", &center.y.to_string()),
                    ("radius", &radius.to_string()),
                ])?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    pub fn load_entity(&mut self, id: EntityId) -> Result<Entity, redis::RedisError> {
        let mut con = self.client.get_connection()?;
        let key = format!("entity:{}", id.data().as_ffi());
        
        let entity_type: String = con.hget(&key, "type")?;
        
        match entity_type.as_str() {
            "line" => {
                let p1_x: f32 = con.hget(&key, "p1_x")?;
                let p1_y: f32 = con.hget(&key, "p1_y")?;
                let p2_x: f32 = con.hget(&key, "p2_x")?;
                let p2_y: f32 = con.hget(&key, "p2_y")?;
                
                Ok(Entity::Line {
                    p1: Point::new(p1_x, p1_y),
                    p2: Point::new(p2_x, p2_y),
                })
            }
            "circle" => {
                let center_x: f32 = con.hget(&key, "center_x")?;
                let center_y: f32 = con.hget(&key, "center_y")?;
                let radius: f32 = con.hget(&key, "radius")?;
                
                Ok(Entity::Circle {
                    center: Point::new(center_x, center_y),
                    radius,
                })
            }
            _ => Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Unknown entity type"))),
        }
    }
}
```

---

### 4.3 Pub/Sub でリアルタイム同期

```rust
use redis::Commands;

pub struct CollaborativeCAD {
    redis_client: redis::Client,
    session_id: String,
}

impl CollaborativeCAD {
    pub fn publish_change(&mut self, entity_id: EntityId, entity: &Entity) -> Result<(), redis::RedisError> {
        let mut con = self.redis_client.get_connection()?;
        
        let message = serde_json::json!({
            "session_id": self.session_id,
            "entity_id": entity_id.data().as_ffi(),
            "entity": entity,
        });
        
        con.publish("cad:changes", message.to_string())?;
        Ok(())
    }
    
    pub fn subscribe_changes(&mut self, callback: impl Fn(EntityId, Entity)) -> Result<(), redis::RedisError> {
        let mut con = self.redis_client.get_connection()?;
        let mut pubsub = con.as_pubsub();
        
        pubsub.subscribe("cad:changes")?;
        
        loop {
            let msg = pubsub.get_message()?;
            let payload: String = msg.get_payload()?;
            
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&payload) {
                // 自分のセッションは無視
                if data["session_id"] == self.session_id {
                    continue;
                }
                
                let entity_id_data = data["entity_id"].as_u64().unwrap();
                let entity_id = EntityId::from(slotmap::KeyData::from_ffi(entity_id_data));
                
                let entity: Entity = serde_json::from_value(data["entity"].clone()).unwrap();
                
                callback(entity_id, entity);
            }
        }
    }
}
```

---

### 4.4 Redis JSON for Complex Data

```rust
// RedisJSON を使用した複雑なデータ保存
use redis::Commands;

pub struct RedisJSONStore {
    client: redis::Client,
}

impl RedisJSONStore {
    pub fn save_document(&mut self, doc: &CADDocument) -> Result<(), redis::RedisError> {
        let mut con = self.client.get_connection()?;
        
        let json = serde_json::to_string(doc).unwrap();
        
        // JSON.SET document:123 $ '{"entities": [...], "layers": [...]}'
        redis::cmd("JSON.SET")
            .arg("document:123")
            .arg("$")
            .arg(json)
            .query(&mut con)?;
        
        Ok(())
    }
    
    pub fn get_entity_count(&mut self) -> Result<usize, redis::RedisError> {
        let mut con = self.client.get_connection()?;
        
        // JSON.ARRLEN document:123 $.entities
        let count: usize = redis::cmd("JSON.ARRLEN")
            .arg("document:123")
            .arg("$.entities")
            .query(&mut con)?;
        
        Ok(count)
    }
}
```

---

## 5. Implementation Examples

### 5.1 メモリ効率的なエンティティストレージ

```rust
// Array of Structures (AoS) - 悪い例
struct EntityAoS {
    entities: Vec<Entity>,  // 各エンティティが異なるサイズ
}

// Structure of Arrays (SoA) - 良い例
struct EntitySoA {
    lines: Vec<Line>,       // 全て同じサイズ
    circles: Vec<Circle>,   // 全て同じサイズ
    arcs: Vec<Arc>,         // 全て同じサイズ
}

// メモリレイアウト比較
// AoS: [Line][Circle][Line][Arc][Line]... (キャッシュミス多)
// SoA: [Line][Line][Line]...[Circle]...[Arc]... (キャッシュヒット多)
```

---

### 5.2 ファイル保存の実装

```rust
pub trait FileFormat {
    fn save(&self, doc: &CADDocument, path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    fn load(path: &Path) -> Result<CADDocument, Box<dyn std::error::Error>>;
}

// JSON 実装
pub struct JSONFormat;

impl FileFormat for JSONFormat {
    fn save(&self, doc: &CADDocument, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(doc)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    fn load(path: &Path) -> Result<CADDocument, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let doc = serde_json::from_str(&json)?;
        Ok(doc)
    }
}

// Binary 実装
pub struct BinaryFormat;

impl FileFormat for BinaryFormat {
    fn save(&self, doc: &CADDocument, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = bincode::serialize(doc)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
    
    fn load(path: &Path) -> Result<CADDocument, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        let doc = bincode::deserialize(&bytes)?;
        Ok(doc)
    }
}
```

---

## 📊 メモリ使用量比較

| データタイプ | 10,000 エンティティ | 100,000 エンティティ | 1,000,000 エンティティ |
|-------------|-------------------|---------------------|----------------------|
| **2D Vector (SoA)** | 0.6 MB | 6 MB | 60 MB |
| **3D Mesh** | 10 MB | 100 MB | 1 GB |
| **Raster (1920×1080)** | 8 MB (1枚) | 80 MB (10枚) | 800 MB (100枚) |
| **Redis (Hash)** | 1 MB | 10 MB | 100 MB |
| **DXF (ASCII)** | 2 MB | 20 MB | 200 MB |
| **DWG (Binary)** | 0.5 MB | 5 MB | 50 MB |

---

## 🎯 実装推奨事項

### メモリ効率
1. ✅ Structure of Arrays (SoA) を使用
2. ✅ 空間インデックス (R-Tree) で高速検索
3. ✅ GPU バッファを活用

### ファイル形式
1. ✅ 独自形式: JSON (デバッグ), Binary (本番)
2. ✅ 互換性: DXF (エクスポート), SVG (Web)
3. ✅ 3D: STEP (高精度), STL (3Dプリント)

### Redis 活用
1. ✅ Pub/Sub でリアルタイム同期
2. ✅ Hash でエンティティ保存
3. ✅ RedisJSON で複雑なデータ

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-26*
