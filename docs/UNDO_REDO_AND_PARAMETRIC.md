# Undo/Redo & Parametric Deformation: Complete Guide

> **対象**: CAD設計者、フレームワーク開発者
> 
> **目的**: Undo/Redoとパラメトリック変形の**全ての実現方法**を網羅

---

## 📚 Table of Contents
1. [Undo/Redo Implementation Patterns](#1-undoredo-implementation-patterns)
2. [Parametric Deformation Techniques](#2-parametric-deformation-techniques)
3. [Advanced Architectures](#3-advanced-architectures)

---

## 1. Undo/Redo Implementation Patterns

### 1.1 Command Pattern (コマンドパターン)

**概念**: 操作を「コマンドオブジェクト」としてカプセル化し、`execute()` と `undo()` を実装

```rust
trait Command {
    fn execute(&mut self, state: &mut AppState);
    fn undo(&mut self, state: &mut AppState);
}

struct MoveEntityCommand {
    entity_id: EntityId,
    from: Point,
    to: Point,
}

impl Command for MoveEntityCommand {
    fn execute(&mut self, state: &mut AppState) {
        state.move_entity(self.entity_id, self.to);
    }
    
    fn undo(&mut self, state: &mut AppState) {
        state.move_entity(self.entity_id, self.from);
    }
}

struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    fn execute(&mut self, mut cmd: Box<dyn Command>, state: &mut AppState) {
        cmd.execute(state);
        self.undo_stack.push(cmd);
        self.redo_stack.clear(); // 新しいコマンドでRedoスタックをクリア
    }
    
    fn undo(&mut self, state: &mut AppState) {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(state);
            self.redo_stack.push(cmd);
        }
    }
    
    fn redo(&mut self, state: &mut AppState) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(state);
            self.undo_stack.push(cmd);
        }
    }
}
```

**メリット**:
- ✅ 各コマンドが自己完結
- ✅ メモリ効率が良い（差分のみ保存）
- ✅ 拡張しやすい

**デメリット**:
- ❌ 各コマンドに `undo()` の実装が必要
- ❌ 複雑な操作の逆操作が難しい

---

### 1.2 Memento Pattern (メメントパターン)

**概念**: 状態のスナップショットを保存し、復元時にそのまま戻す

```rust
#[derive(Clone)]
struct DocumentMemento {
    entities: HashMap<EntityId, Entity>,
    camera: Camera,
    // 全ての状態を保存
}

struct MementoHistory {
    snapshots: Vec<DocumentMemento>,
    current_index: usize,
}

impl MementoHistory {
    fn save_snapshot(&mut self, state: &AppState) {
        // 現在位置以降のスナップショットを削除
        self.snapshots.truncate(self.current_index + 1);
        
        // 新しいスナップショットを保存
        self.snapshots.push(DocumentMemento {
            entities: state.entities.clone(),
            camera: state.camera.clone(),
        });
        
        self.current_index += 1;
    }
    
    fn undo(&mut self) -> Option<&DocumentMemento> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.snapshots[self.current_index])
        } else {
            None
        }
    }
    
    fn redo(&mut self) -> Option<&DocumentMemento> {
        if self.current_index + 1 < self.snapshots.len() {
            self.current_index += 1;
            Some(&self.snapshots[self.current_index])
        } else {
            None
        }
    }
}
```

**メリット**:
- ✅ 実装が簡単
- ✅ 任意の時点に瞬時に復元可能

**デメリット**:
- ❌ メモリ消費が大きい（全状態をコピー）
- ❌ 大規模データで破綻

---

### 1.3 Event Sourcing (イベントソーシング)

**概念**: 全ての変更を「イベント」として記録し、再生することで状態を復元

```rust
#[derive(Clone, Serialize, Deserialize)]
enum Event {
    EntityAdded { id: EntityId, entity: Entity },
    EntityMoved { id: EntityId, from: Point, to: Point },
    EntityDeleted { id: EntityId, entity: Entity },
}

struct EventStore {
    events: Vec<Event>,
    current_index: usize, // 現在の「再生位置」
}

impl EventStore {
    fn apply_event(&self, event: &Event, state: &mut AppState) {
        match event {
            Event::EntityAdded { id, entity } => {
                state.entities.insert(*id, entity.clone());
            }
            Event::EntityMoved { id, to, .. } => {
                if let Some(entity) = state.entities.get_mut(id) {
                    entity.position = *to;
                }
            }
            Event::EntityDeleted { id, .. } => {
                state.entities.remove(id);
            }
        }
    }
    
    fn rebuild_state(&self) -> AppState {
        let mut state = AppState::default();
        for event in &self.events[..=self.current_index] {
            self.apply_event(event, &mut state);
        }
        state
    }
    
    fn undo(&mut self) -> AppState {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
        self.rebuild_state()
    }
    
    fn redo(&mut self) -> AppState {
        if self.current_index + 1 < self.events.len() {
            self.current_index += 1;
        }
        self.rebuild_state()
    }
}
```

**メリット**:
- ✅ 完全な監査ログ
- ✅ 任意の時点に復元可能
- ✅ イベントを外部に保存可能（永続化）

**デメリット**:
- ❌ 再生コストが高い（全イベントを再適用）
- ❌ イベント設計が複雑

**最適化**: Snapshot + Event
```rust
struct OptimizedEventStore {
    snapshots: Vec<(usize, AppState)>, // (event_index, state)
    events: Vec<Event>,
    snapshot_interval: usize, // 例: 100イベントごと
}

impl OptimizedEventStore {
    fn rebuild_state(&self, target_index: usize) -> AppState {
        // 最も近いスナップショットを探す
        let snapshot = self.snapshots.iter()
            .rev()
            .find(|(idx, _)| *idx <= target_index);
        
        let (start_idx, mut state) = snapshot
            .map(|(idx, s)| (*idx, s.clone()))
            .unwrap_or((0, AppState::default()));
        
        // スナップショット以降のイベントを再生
        for event in &self.events[start_idx..=target_index] {
            self.apply_event(event, &mut state);
        }
        
        state
    }
}
```

---

### 1.4 Persistent Data Structures (永続データ構造)

**概念**: データ構造自体が「過去のバージョン」を保持

```rust
use im::HashMap; // immutable-rs の HashMap

struct PersistentHistory {
    versions: Vec<HashMap<EntityId, Entity>>,
    current_index: usize,
}

impl PersistentHistory {
    fn new() -> Self {
        Self {
            versions: vec![HashMap::new()],
            current_index: 0,
        }
    }
    
    fn modify(&mut self, f: impl FnOnce(&mut HashMap<EntityId, Entity>)) {
        // 現在のバージョンをコピー（構造共有で高速）
        let mut new_version = self.versions[self.current_index].clone();
        f(&mut new_version);
        
        // 新しいバージョンを追加
        self.versions.truncate(self.current_index + 1);
        self.versions.push(new_version);
        self.current_index += 1;
    }
    
    fn undo(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }
    
    fn redo(&mut self) {
        if self.current_index + 1 < self.versions.len() {
            self.current_index += 1;
        }
    }
    
    fn current(&self) -> &HashMap<EntityId, Entity> {
        &self.versions[self.current_index]
    }
}
```

**メリット**:
- ✅ Undo/Redoが O(1)
- ✅ 構造共有でメモリ効率が良い
- ✅ 実装がシンプル

**デメリット**:
- ❌ 専用のデータ構造が必要（`im`, `rpds` など）
- ❌ 通常の `Vec` や `HashMap` より遅い

---

### 1.5 Differential Dataflow (差分データフロー)

**概念**: 変更の「差分」だけを伝播させる

```rust
struct DifferentialState {
    base_state: AppState,
    deltas: Vec<Delta>,
    current_index: usize,
}

enum Delta {
    Insert(EntityId, Entity),
    Remove(EntityId),
    Update(EntityId, EntityDiff),
}

struct EntityDiff {
    position: Option<Point>,
    color: Option<Color>,
    // 変更されたフィールドのみ
}

impl DifferentialState {
    fn apply_delta(&mut self, delta: &Delta) {
        match delta {
            Delta::Insert(id, entity) => {
                self.base_state.entities.insert(*id, entity.clone());
            }
            Delta::Remove(id) => {
                self.base_state.entities.remove(id);
            }
            Delta::Update(id, diff) => {
                if let Some(entity) = self.base_state.entities.get_mut(id) {
                    if let Some(pos) = diff.position {
                        entity.position = pos;
                    }
                    if let Some(color) = diff.color {
                        entity.color = color;
                    }
                }
            }
        }
    }
    
    fn undo(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            // 差分を逆適用
            self.unapply_delta(&self.deltas[self.current_index]);
        }
    }
}
```

**メリット**:
- ✅ メモリ効率が非常に良い
- ✅ 大規模データに強い
- ✅ リアクティブプログラミングと相性が良い

**デメリット**:
- ❌ 実装が複雑
- ❌ 差分の逆操作が必要

---

## 2. Parametric Deformation Techniques

### 2.1 Constraint Solver (制約ソルバー)

**概念**: 幾何制約を満たすように図形を変形

#### Graph-Based Solver
```rust
struct ConstraintGraph {
    nodes: HashMap<NodeId, GeometricElement>,
    edges: Vec<Constraint>,
}

enum Constraint {
    Distance { a: NodeId, b: NodeId, distance: f32 },
    Parallel { line1: NodeId, line2: NodeId },
    Perpendicular { line1: NodeId, line2: NodeId },
    Tangent { circle: NodeId, line: NodeId },
}

impl ConstraintGraph {
    fn solve(&mut self) {
        // グラフ分解アルゴリズム
        let clusters = self.decompose_into_clusters();
        
        for cluster in clusters {
            self.solve_cluster(cluster);
        }
    }
    
    fn solve_cluster(&mut self, cluster: Vec<NodeId>) {
        // 各クラスタを独立に解く
        // 構築的手法: 順番に制約を適用
        for constraint in &self.edges {
            self.apply_constraint(constraint);
        }
    }
}
```

#### Numerical Solver (Newton法)
```rust
struct NumericalSolver {
    variables: Vec<f32>, // [x1, y1, x2, y2, ...]
    constraints: Vec<Box<dyn ConstraintEquation>>,
}

trait ConstraintEquation {
    fn evaluate(&self, vars: &[f32]) -> f32;
    fn jacobian(&self, vars: &[f32]) -> Vec<f32>;
}

impl NumericalSolver {
    fn solve(&mut self, max_iterations: usize) {
        for _ in 0..max_iterations {
            let residuals: Vec<f32> = self.constraints.iter()
                .map(|c| c.evaluate(&self.variables))
                .collect();
            
            if residuals.iter().all(|r| r.abs() < 1e-6) {
                break; // 収束
            }
            
            // Newton法: x_new = x_old - J^-1 * F
            let jacobian = self.compute_jacobian();
            let delta = solve_linear_system(&jacobian, &residuals);
            
            for (var, d) in self.variables.iter_mut().zip(delta) {
                *var -= d;
            }
        }
    }
}
```

**商用ソルバー**:
- **D-Cubed 3D DCM** (Siemens)
- **LEDAS LGS** (Bricsys)
- **C3D Solver** (C3D Toolkit)

---

### 2.2 Free-Form Deformation (FFD)

**概念**: 制御格子を変形させることで内部の図形を変形

```rust
struct FFDLattice {
    control_points: Vec<Vec<Vec<Point3>>>, // 3D格子
    degree: (usize, usize, usize),
}

impl FFDLattice {
    fn deform(&self, point: Point3) -> Point3 {
        // ベルンシュタイン多項式で補間
        let (u, v, w) = self.world_to_parametric(point);
        
        let mut result = Point3::zero();
        for i in 0..=self.degree.0 {
            for j in 0..=self.degree.1 {
                for k in 0..=self.degree.2 {
                    let b = bernstein(i, self.degree.0, u)
                          * bernstein(j, self.degree.1, v)
                          * bernstein(k, self.degree.2, w);
                    
                    result += self.control_points[i][j][k] * b;
                }
            }
        }
        result
    }
    
    fn move_control_point(&mut self, i: usize, j: usize, k: usize, new_pos: Point3) {
        self.control_points[i][j][k] = new_pos;
    }
}

fn bernstein(i: usize, n: usize, t: f32) -> f32 {
    binomial(n, i) * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32)
}
```

**用途**: 有機的な形状の変形（キャラクターモデル、車体デザイン）

---

### 2.3 Mesh Deformation (メッシュ変形)

#### Laplacian Deformation
```rust
struct LaplacianDeformer {
    mesh: Mesh,
    laplacian_matrix: SparseMatrix,
}

impl LaplacianDeformer {
    fn deform(&mut self, handles: Vec<(VertexId, Point3)>) {
        // ラプラシアン座標を保存
        let laplacian_coords = self.compute_laplacian_coords();
        
        // 制約付き最小二乗問題を解く
        // min ||L * V - δ||^2  s.t. V[handles] = target_positions
        
        let new_positions = self.solve_constrained_least_squares(
            &laplacian_coords,
            &handles
        );
        
        self.mesh.update_vertices(new_positions);
    }
    
    fn compute_laplacian_coords(&self) -> Vec<Vector3> {
        self.mesh.vertices.iter().enumerate().map(|(i, v)| {
            let neighbors = self.mesh.get_neighbors(i);
            let centroid = neighbors.iter()
                .map(|&n| self.mesh.vertices[n])
                .sum::<Point3>() / neighbors.len() as f32;
            
            v.position - centroid
        }).collect()
    }
}
```

**特徴**:
- ✅ 滑らかな変形
- ✅ 詳細を保持
- ❌ 大規模メッシュで遅い

---

### 2.4 Skeleton-Based Deformation (スケルトン変形)

```rust
struct Skeleton {
    bones: Vec<Bone>,
    bind_pose: Vec<Matrix4>,
}

struct Bone {
    parent: Option<usize>,
    transform: Matrix4,
}

impl Skeleton {
    fn deform_mesh(&self, mesh: &Mesh, weights: &[Vec<(usize, f32)>]) -> Mesh {
        let mut deformed = mesh.clone();
        
        for (i, vertex) in deformed.vertices.iter_mut().enumerate() {
            let mut pos = Point3::zero();
            
            for &(bone_idx, weight) in &weights[i] {
                let bone_transform = self.get_world_transform(bone_idx);
                let bind_inverse = self.bind_pose[bone_idx].inverse();
                
                let skinning_matrix = bone_transform * bind_inverse;
                pos += (skinning_matrix * vertex.position) * weight;
            }
            
            vertex.position = pos;
        }
        
        deformed
    }
}
```

**用途**: キャラクターアニメーション、ロボットアームのシミュレーション

---

## 3. Advanced Architectures

### 3.1 CQRS + Event Sourcing

```rust
// Command Side (書き込み)
struct CommandHandler {
    event_store: EventStore,
}

impl CommandHandler {
    fn handle_move_entity(&mut self, id: EntityId, to: Point) {
        // 現在の状態を取得
        let state = self.event_store.rebuild_state();
        
        // ビジネスロジック検証
        if let Some(entity) = state.entities.get(&id) {
            let event = Event::EntityMoved {
                id,
                from: entity.position,
                to,
            };
            
            // イベントを保存
            self.event_store.append(event);
        }
    }
}

// Query Side (読み込み)
struct QueryModel {
    entities: HashMap<EntityId, Entity>,
    spatial_index: QuadTree,
}

impl QueryModel {
    fn update_from_event(&mut self, event: &Event) {
        match event {
            Event::EntityMoved { id, to, .. } => {
                if let Some(entity) = self.entities.get_mut(id) {
                    self.spatial_index.remove(*id, entity.bounds());
                    entity.position = *to;
                    self.spatial_index.insert(*id, entity.bounds());
                }
            }
            // ...
        }
    }
}
```

**メリット**:
- ✅ 読み書きを独立に最適化
- ✅ スケーラビリティ
- ✅ 完全な監査ログ

---

### 3.2 Reactive Dataflow Graph

```rust
struct ReactiveGraph {
    nodes: HashMap<NodeId, Box<dyn ReactiveNode>>,
    edges: Vec<(NodeId, NodeId)>,
}

trait ReactiveNode {
    fn compute(&mut self, inputs: &[Value]) -> Value;
    fn invalidate(&mut self);
}

impl ReactiveGraph {
    fn set_input(&mut self, node: NodeId, value: Value) {
        // 依存グラフを辿って再計算
        let affected = self.get_dependent_nodes(node);
        
        for node_id in affected {
            self.nodes.get_mut(&node_id).unwrap().invalidate();
        }
        
        // 遅延評価: 実際の再計算は get_output 時
    }
    
    fn get_output(&mut self, node: NodeId) -> Value {
        let node = self.nodes.get_mut(&node).unwrap();
        
        if node.is_dirty() {
            let inputs = self.get_inputs(node);
            node.compute(&inputs)
        } else {
            node.cached_value()
        }
    }
}
```

**用途**: Grasshopper, Dynamo などのビジュアルプログラミング

---

## 📊 比較表

### Undo/Redo 方式

| 方式 | メモリ | 速度 | 実装難易度 | 監査ログ |
|------|--------|------|-----------|---------|
| Command | ⭐⭐⭐ | ⭐⭐⭐ | 中 | ❌ |
| Memento | ⭐ | ⭐⭐⭐ | 易 | ❌ |
| Event Sourcing | ⭐⭐ | ⭐ | 難 | ✅ |
| Persistent DS | ⭐⭐ | ⭐⭐⭐ | 中 | ❌ |
| Differential | ⭐⭐⭐ | ⭐⭐ | 難 | ✅ |

### パラメトリック変形

| 方式 | 精度 | 速度 | 用途 |
|------|------|------|------|
| Constraint Solver | ⭐⭐⭐ | ⭐⭐ | 機械設計 |
| FFD | ⭐⭐ | ⭐⭐⭐ | 有機形状 |
| Mesh Deformation | ⭐⭐ | ⭐ | スキャンフィット |
| Skeleton | ⭐⭐⭐ | ⭐⭐⭐ | アニメーション |

---

## 🎯 推奨実装

### 小〜中規模CAD
```rust
// Command + Memento のハイブリッド
struct HybridHistory {
    commands: Vec<Box<dyn Command>>,
    snapshots: Vec<(usize, AppState)>, // 100コマンドごと
}
```

### 大規模CAD
```rust
// Event Sourcing + Snapshot
struct EnterpriseHistory {
    event_store: EventStore,
    snapshot_manager: SnapshotManager,
    query_model: QueryModel, // CQRS
}
```

### リアルタイム共同編集
```rust
// Differential Dataflow + CRDT
struct CollaborativeHistory {
    differential_engine: DifferentialDataflow,
    crdt_state: OpBasedCRDT,
}
```

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
