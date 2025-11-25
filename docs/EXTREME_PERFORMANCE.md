# CAD Extreme Performance Techniques

> **対象**: 100万エンティティ、60fps、巨大データセットを扱う実装者
> 
> **目的**: 「不可能」を「可能」にする極限最適化技術の完全網羅

---

## 📚 Table of Contents
1. [Mass Rendering (大量描画)](#1-mass-rendering-大量描画)
2. [Ultra-Fast Rendering (最速描画)](#2-ultra-fast-rendering-最速描画)
3. [Infinite Canvas & Virtual Scrolling](#3-infinite-canvas--virtual-scrolling)
4. [Memory Optimization (メモリ最適化)](#4-memory-optimization-メモリ最適化)
5. [Multi-Threading & Parallelism](#5-multi-threading--parallelism)
6. [Advanced Culling Techniques](#6-advanced-culling-techniques)

---

## 1. Mass Rendering (大量描画)

### 1.1 GPU Instancing (インスタンシング)

**問題**: 100万個の同じボルトを描画したい

**❌ 悪い方法**:
```rust
for bolt in bolts.iter() {
    draw_call(bolt); // 100万回の draw call → 破綻
}
```

**✅ GPU Instancing**:
```rust
// 1. ジオメトリは1回だけ送信
let bolt_mesh = create_bolt_mesh();

// 2. インスタンスごとの変換行列を準備
let instance_data: Vec<Matrix4> = bolts.iter()
    .map(|b| b.transform_matrix())
    .collect();

// 3. 1回の draw call で全描画
draw_instanced(bolt_mesh, instance_data); // 1回だけ！
```

**wgpu での実装**:
```rust
// Instance Buffer の作成
let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Instance Buffer"),
    contents: bytemuck::cast_slice(&instance_data),
    usage: wgpu::BufferUsages::VERTEX,
});

// Draw call
render_pass.draw_indexed(
    0..indices.len() as u32,
    0,
    0..instance_data.len() as u32, // インスタンス数
);
```

**性能**:
- Draw Call: 100万 → **1**
- FPS: 5fps → **60fps**

---

### 1.2 Batching (バッチング)

**問題**: 異なる図形を効率的に描画

```rust
struct Batch {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    material_id: u32,
}

struct BatchManager {
    batches: HashMap<u32, Batch>,
}

impl BatchManager {
    fn add_entity(&mut self, entity: &Entity) {
        let material_id = entity.material_id();
        let batch = self.batches.entry(material_id).or_insert_with(Batch::new);
        
        let vertex_offset = batch.vertices.len() as u32;
        batch.vertices.extend(&entity.vertices);
        batch.indices.extend(entity.indices.iter().map(|i| i + vertex_offset));
    }
    
    fn render(&self, render_pass: &mut RenderPass) {
        for (material_id, batch) in &self.batches {
            render_pass.set_material(*material_id);
            render_pass.draw_batch(batch);
        }
    }
}
```

**効果**:
- 10万個の異なる図形 → **数百の draw call**

---

### 1.3 Indirect Drawing (間接描画)

```rust
// GPU が自分で draw call を発行
struct DrawIndirectCommand {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

// GPU Buffer に draw command を格納
let commands: Vec<DrawIndirectCommand> = entities.iter()
    .map(|e| DrawIndirectCommand {
        vertex_count: e.vertex_count,
        instance_count: 1,
        first_vertex: e.vertex_offset,
        first_instance: 0,
    })
    .collect();

let command_buffer = device.create_buffer_init(&BufferInitDescriptor {
    contents: bytemuck::cast_slice(&commands),
    usage: BufferUsages::INDIRECT,
});

// 1回の call で全描画
render_pass.multi_draw_indirect(&command_buffer, 0, commands.len() as u32);
```

---

## 2. Ultra-Fast Rendering (最速描画)

### 2.1 Compute Shader による前処理

```wgsl
// Compute Shader で頂点変換を並列実行
@compute @workgroup_size(256)
fn transform_vertices(
    @builtin(global_invocation_id) id: vec3<u32>,
) {
    let idx = id.x;
    if (idx >= arrayLength(&input_vertices)) {
        return;
    }
    
    let vertex = input_vertices[idx];
    let transformed = transform_matrix * vec4<f32>(vertex.position, 1.0);
    output_vertices[idx] = Vertex(transformed.xyz, vertex.normal);
}
```

**性能**:
- CPU変換: 10ms
- GPU変換: **0.5ms** (20倍高速)

---

### 2.2 Persistent Mapped Buffers

```rust
// GPU メモリを CPU から直接書き込み可能にする
let buffer = device.create_buffer(&BufferDescriptor {
    size: buffer_size,
    usage: BufferUsages::VERTEX | BufferUsages::MAP_WRITE,
    mapped_at_creation: true,
});

{
    let mut mapped = buffer.slice(..).get_mapped_range_mut();
    // CPU から直接書き込み（コピー不要）
    mapped.copy_from_slice(bytemuck::cast_slice(&vertices));
}
buffer.unmap();
```

**効果**: メモリコピーのオーバーヘッド削減

---

### 2.3 Early-Z Optimization

```rust
// Depth Pre-Pass: 深度だけ先に描画
fn depth_prepass(render_pass: &mut RenderPass, entities: &[Entity]) {
    render_pass.set_pipeline(&depth_only_pipeline);
    for entity in entities {
        render_pass.draw(entity);
    }
}

// Main Pass: 深度テスト済みなので Fragment Shader が高速
fn main_pass(render_pass: &mut RenderPass, entities: &[Entity]) {
    render_pass.set_pipeline(&color_pipeline);
    render_pass.set_depth_compare(CompareFunction::Equal); // 既存の深度と一致のみ
    for entity in entities {
        render_pass.draw(entity); // Fragment Shader は見える部分だけ実行
    }
}
```

---

## 3. Infinite Canvas & Virtual Scrolling

### 3.1 Viewport Culling (ビューポートカリング)

```rust
struct ViewportCuller {
    viewport: Rect,
    spatial_index: QuadTree,
}

impl ViewportCuller {
    fn get_visible_entities(&self) -> Vec<EntityId> {
        // QuadTree で高速検索
        self.spatial_index.query(self.viewport)
    }
    
    fn update_viewport(&mut self, camera: &Camera) {
        // カメラ移動時にビューポート更新
        self.viewport = camera.visible_rect();
    }
}
```

**性能**:
- 全探索: O(n) = 100万エンティティで破綻
- QuadTree: O(log n) = **常に高速**

---

### 3.2 Frustum Culling (視錐台カリング)

```rust
struct Frustum {
    planes: [Plane; 6], // Near, Far, Left, Right, Top, Bottom
}

impl Frustum {
    fn from_camera(camera: &Camera) -> Self {
        let view_proj = camera.view_projection_matrix();
        Self {
            planes: extract_frustum_planes(view_proj),
        }
    }
    
    fn contains(&self, bounds: &AABB) -> bool {
        for plane in &self.planes {
            if plane.distance_to(bounds.center()) < -bounds.radius() {
                return false; // 完全に外側
            }
        }
        true
    }
}

// 使用例
let frustum = Frustum::from_camera(&camera);
let visible: Vec<_> = entities.iter()
    .filter(|e| frustum.contains(&e.bounds))
    .collect();
```

---

### 3.3 Virtual Scrolling (仮想スクロール)

```rust
struct VirtualCanvas {
    chunk_size: f32, // 例: 1000x1000
    loaded_chunks: HashMap<(i32, i32), Chunk>,
    visible_area: Rect,
}

impl VirtualCanvas {
    fn update(&mut self, camera: &Camera) {
        let visible_chunks = self.get_visible_chunk_coords(camera);
        
        // 不要なチャンクをアンロード
        self.loaded_chunks.retain(|coord, _| {
            visible_chunks.contains(coord)
        });
        
        // 新しいチャンクをロード
        for coord in visible_chunks {
            self.loaded_chunks.entry(coord).or_insert_with(|| {
                self.load_chunk(coord)
            });
        }
    }
    
    fn load_chunk(&self, coord: (i32, i32)) -> Chunk {
        // ディスクまたはネットワークから非同期ロード
        load_chunk_async(coord)
    }
}
```

**効果**:
- メモリ使用量: 全データ → **表示領域のみ**
- ロード時間: 初回のみ → **常に高速**

---

### 3.4 Level of Detail (LOD)

```rust
struct LODManager {
    lod_levels: Vec<Mesh>, // LOD 0 (最高), LOD 1, LOD 2...
}

impl LODManager {
    fn select_lod(&self, distance: f32) -> &Mesh {
        let lod_index = match distance {
            d if d < 100.0 => 0,  // 近い: 高詳細
            d if d < 500.0 => 1,  // 中距離: 中詳細
            _ => 2,               // 遠い: 低詳細
        };
        &self.lod_levels[lod_index]
    }
}

// 使用例
for entity in entities {
    let distance = (entity.position - camera.position).length();
    let mesh = entity.lod_manager.select_lod(distance);
    render(mesh);
}
```

**効果**:
- ポリゴン数: 100万 → **10万** (10倍削減)

---

## 4. Memory Optimization (メモリ最適化)

### 4.1 Object Pooling (オブジェクトプール)

```rust
struct EntityPool {
    free_list: Vec<Entity>,
    capacity: usize,
}

impl EntityPool {
    fn acquire(&mut self) -> Entity {
        self.free_list.pop().unwrap_or_else(|| Entity::new())
    }
    
    fn release(&mut self, entity: Entity) {
        if self.free_list.len() < self.capacity {
            self.free_list.push(entity);
        }
    }
}
```

**効果**: アロケーション回数を劇的に削減

---

### 4.2 Struct of Arrays (SoA)

```rust
// ❌ Array of Structs (AoS) - キャッシュミスが多い
struct Entity {
    position: Vec3,
    velocity: Vec3,
    color: Color,
}
let entities: Vec<Entity> = vec![...];

// ✅ Struct of Arrays (SoA) - キャッシュ効率が良い
struct EntityStorage {
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
    colors: Vec<Color>,
}

// 位置だけ更新する場合、positions だけアクセス
for pos in &mut storage.positions {
    *pos += delta;
}
```

**効果**: キャッシュヒット率向上 → **2〜3倍高速化**

---

### 4.3 Compression (圧縮)

```rust
// 頂点座標を16bitに圧縮
#[repr(C)]
struct CompressedVertex {
    pos: [i16; 3], // -32768 ~ 32767
    normal: [i8; 3], // -128 ~ 127
}

fn compress_vertex(v: &Vertex, bounds: &AABB) -> CompressedVertex {
    let normalized = (v.position - bounds.min) / bounds.size();
    CompressedVertex {
        pos: [
            (normalized.x * 32767.0) as i16,
            (normalized.y * 32767.0) as i16,
            (normalized.z * 32767.0) as i16,
        ],
        normal: [
            (v.normal.x * 127.0) as i8,
            (v.normal.y * 127.0) as i8,
            (v.normal.z * 127.0) as i8,
        ],
    }
}
```

**効果**: メモリ使用量 **50%削減**

---

## 5. Multi-Threading & Parallelism

### 5.1 Rayon による並列処理

```rust
use rayon::prelude::*;

// 全エンティティの境界ボックスを並列計算
let bounds: Vec<AABB> = entities.par_iter()
    .map(|e| e.calculate_bounds())
    .collect();

// QuadTree 構築も並列化
let quadtree = QuadTree::build_parallel(&entities);
```

---

### 5.2 Async Loading (非同期ロード)

```rust
use tokio::task;

async fn load_large_model(path: &Path) -> Result<Model> {
    // ファイルI/Oを別スレッドで実行
    let data = task::spawn_blocking(|| {
        std::fs::read(path)
    }).await??;
    
    // パースも並列化
    let model = task::spawn_blocking(move || {
        parse_model(&data)
    }).await?;
    
    Ok(model)
}
```

---

### 5.3 Lock-Free Data Structures

```rust
use crossbeam::queue::SegQueue;

// ロックフリーキュー（複数スレッドから安全にアクセス）
struct CommandQueue {
    queue: SegQueue<DrawCommand>,
}

impl CommandQueue {
    fn push(&self, cmd: DrawCommand) {
        self.queue.push(cmd); // ロック不要
    }
    
    fn drain(&self) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        while let Some(cmd) = self.queue.pop() {
            commands.push(cmd);
        }
        commands
    }
}
```

---

## 6. Advanced Culling Techniques

### 6.1 Occlusion Culling (遮蔽カリング)

```rust
// 手前の大きなオブジェクトで隠れた物体を描画しない
struct OcclusionCuller {
    depth_pyramid: Texture, // Hierarchical Z-Buffer
}

impl OcclusionCuller {
    fn is_visible(&self, bounds: &AABB, camera: &Camera) -> bool {
        let screen_rect = project_to_screen(bounds, camera);
        let mip_level = calculate_mip_level(screen_rect.size());
        
        let depth = self.depth_pyramid.sample(screen_rect.center(), mip_level);
        bounds.min_depth() < depth // 手前にあれば可視
    }
}
```

---

### 6.2 Portal Culling (ポータルカリング)

```rust
// 建築CAD用: 部屋の出入口（ポータル）を通して見える物だけ描画
struct Portal {
    room_a: RoomId,
    room_b: RoomId,
    bounds: Rect,
}

fn render_with_portals(camera: &Camera, rooms: &[Room], portals: &[Portal]) {
    let current_room = find_room(camera.position);
    let mut visible_rooms = HashSet::new();
    visible_rooms.insert(current_room);
    
    // ポータルを再帰的に辿る
    traverse_portals(current_room, camera, portals, &mut visible_rooms);
    
    // 可視な部屋だけ描画
    for room_id in visible_rooms {
        render_room(&rooms[room_id]);
    }
}
```

---

### 6.3 Distance Culling (距離カリング)

```rust
// 遠すぎる物体は描画しない
const MAX_RENDER_DISTANCE: f32 = 10000.0;

let visible: Vec<_> = entities.iter()
    .filter(|e| {
        let distance = (e.position - camera.position).length();
        distance < MAX_RENDER_DISTANCE
    })
    .collect();
```

---

## 📊 性能比較表

| 技術 | 適用前 | 適用後 | 改善率 |
|------|--------|--------|--------|
| GPU Instancing | 5 FPS | 60 FPS | **12x** |
| Frustum Culling | 100万描画 | 1万描画 | **100x** |
| LOD | 1000万ポリゴン | 100万ポリゴン | **10x** |
| Virtual Scrolling | 10GB メモリ | 100MB メモリ | **100x** |
| SoA | 100ms/frame | 30ms/frame | **3.3x** |
| Rayon並列化 | 500ms | 50ms | **10x** |

---

## 🎯 実装優先度

### Phase 1: 必須（100万エンティティ対応）
1. ✅ QuadTree / R-Tree
2. ✅ Frustum Culling
3. ✅ GPU Instancing

### Phase 2: 推奨（1000万エンティティ対応）
4. ✅ Virtual Scrolling
5. ✅ LOD
6. ✅ Batching

### Phase 3: 最適化（無限エンティティ対応）
7. ✅ Occlusion Culling
8. ✅ Compute Shader
9. ✅ Async Loading

---

## 📖 参考文献

### 論文
- "Hierarchical Z-Buffer Visibility" (Greene et al., 1993)
- "Real-Time Rendering of Large-Scale Scenes" (Luebke, 2001)

### 実装例
- **Bevy Engine**: ECS + Parallel Rendering
- **Three.js**: LOD + Frustum Culling
- **Unreal Engine**: Nanite (Virtual Geometry)

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
