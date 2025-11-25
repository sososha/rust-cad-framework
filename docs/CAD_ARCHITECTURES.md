# CAD Software Architecture Patterns: A Comprehensive Guide

> **目的**: このドキュメントは、CADソフトウェアの設計パターンを体系的に解説し、「Rust CAD Framework」の設計思想を明確にするための教科書です。

---

## 📚 Table of Contents
1. [Geometric Representation Methods](#1-geometric-representation-methods)
2. [Data Architecture Patterns](#2-data-architecture-patterns)
3. [Application Architecture Patterns](#3-application-architecture-patterns)
4. [Constraint & Parametric Systems](#4-constraint--parametric-systems)
5. [Our Choice: Document-View Pattern](#5-our-choice-document-view-pattern)

---

## 1. Geometric Representation Methods
> **問い**: 「図形をどう表現するか？」

### 1.1 Wireframe Model (ワイヤーフレーム)
```
表現: 点と線のみ
例: [P1]---[P2]---[P3]
```
**特徴**:
- ✅ 最もシンプル
- ❌ 面の情報がない（塗りつぶし不可）
- ❌ 内部/外部の区別ができない

**用途**: 初期スケッチ、構造解析の骨組み

---

### 1.2 Boundary Representation (B-rep)
```
表現: 面・辺・頂点 + トポロジー
例: 
  Face1: [Edge1, Edge2, Edge3, Edge4]
  Edge1: [Vertex1, Vertex2]
```
**特徴**:
- ✅ 正確な形状表現
- ✅ レンダリングが容易
- ✅ 製造（CNC加工）に適している
- ❌ データ構造が複雑
- ❌ Boolean演算が重い

**採用例**: AutoCAD, SolidWorks, Fusion 360
**幾何カーネル**: ACIS, Parasolid, Open CASCADE

**数学的基礎**:
- NURBS (Non-Uniform Rational B-Splines) で曲面を表現
- トポロジー: Euler-Poincaré の公式 `V - E + F = 2` (閉じた多面体)

---

### 1.3 Constructive Solid Geometry (CSG)
```
表現: プリミティブ + Boolean演算
例: 
  Cylinder UNION Sphere
  SUBTRACT Cube
```
**特徴**:
- ✅ 直感的（積み木のように組み立てる）
- ✅ データが軽量
- ✅ 常に「正しい立体」が保証される
- ❌ 面・辺の情報取得が遅い（ツリー全体を評価）
- ❌ 複雑な形状（フィレット等）に不向き

**採用例**: OpenSCAD, Blender (Modifier), ゲームエンジン

**ハイブリッド方式**:
多くの現代CADは、CSGで作成 → B-repに変換して保存

---

### 1.4 Mesh / Polygonal Model
```
表現: 三角形・四角形の集合
例: 
  Triangle1: [V1, V2, V3]
  Triangle2: [V2, V3, V4]
```
**特徴**:
- ✅ GPU描画が超高速
- ✅ 物理シミュレーションに最適
- ❌ 近似表現（曲面が階段状になる）
- ❌ 精密な寸法管理が困難

**用途**: 3Dスキャンデータ、ゲーム、VR/AR

---

## 2. Data Architecture Patterns
> **問い**: 「データをどう管理するか？」

### 2.1 Immediate Mode (即時描画型)
```rust
// 現在の Rust CAD Framework
struct AppState {
    entities: Vec<Entity>,
}

fn render(entities: &[Entity]) {
    for entity in entities {
        draw(entity); // 毎フレーム全描画
    }
}
```
**特徴**:
- ✅ 実装が簡単
- ✅ デバッグしやすい
- ❌ 大量の図形で遅くなる（O(n) 毎フレーム）
- ❌ 空間検索（クリック判定）が遅い

**適用範囲**: 数百〜数千オブジェクトまで

---

### 2.2 Scene Graph (シーングラフ)
```rust
struct Node {
    entity: Entity,
    transform: Matrix4,
    children: Vec<Node>,
}
```
**特徴**:
- ✅ 階層構造（親子関係）を表現
- ✅ 変換（移動・回転）の継承が自然
- ❌ CADには過剰（建築図面に親子関係は不要）

**採用例**: Blender, Unity, Three.js

---

### 2.3 Entity Component System (ECS)
```rust
// bevy_ecs, hecs
world.spawn((
    Position(x, y),
    Geometry::Line { ... },
    Selectable,
    Layer(0),
));

// クエリ
for (pos, geo) in world.query::<(&Position, &Geometry)>() {
    // ...
}
```
**特徴**:
- ✅ データ指向設計（キャッシュ効率が高い）
- ✅ 機能を「コンポーネント」として柔軟に組み合わせ
- ✅ 並列処理が容易
- ❌ 学習コスト高い
- ❌ 2D CADには過剰

**採用例**: ゲームエンジン、一部の3D CAD

---

### 2.4 Document-View Pattern
```rust
// Document（データ層）
struct CadDocument {
    entities: HashMap<EntityId, Entity>,
    layers: Vec<Layer>,
    history: CommandHistory,
}

// View（表示層）
struct CadView {
    camera: Camera,
    selection: SelectionState,
    active_tool: Box<dyn Tool>,
}
```
**特徴**:
- ✅ データと表示が分離
- ✅ ファイル保存が自明（Documentだけ保存）
- ✅ Undo/Redoが自然に実装できる
- ✅ 複数ビュー対応（平面図・立面図を同時表示）

**採用例**: AutoCAD, SolidWorks, Rhino

---

### 2.5 CRDT (Conflict-free Replicated Data Type)
```rust
// リアルタイム共同編集用
struct CrdtDocument {
    operations: Vec<Operation>, // 操作履歴
    // Operational Transformation
}
```
**特徴**:
- ✅ リアルタイム共同編集が可能
- ✅ オフライン編集 → 同期が可能
- ❌ 実装が非常に複雑
- ❌ CADの「厳密な寸法」との相性が悪い

**採用例**: Figma, Miro, Google Docs

---

## 3. Application Architecture Patterns
> **問い**: 「アプリ全体をどう構成するか？」

### 3.1 Layered (N-Tier) Architecture
```
┌─────────────────┐
│  UI Layer       │ (ツールバー、プロパティパネル)
├─────────────────┤
│  Application    │ (作図ロジック、コマンド処理)
├─────────────────┤
│  Geometric      │ (幾何カーネル: ACIS, Parasolid)
│  Kernel         │
├─────────────────┤
│  Data Storage   │ (ファイルI/O, データベース)
└─────────────────┘
```
**特徴**:
- ✅ モジュール化されている
- ✅ レイヤーごとに差し替え可能
- ❌ レイヤー間の通信オーバーヘッド

---

### 3.2 Microkernel Architecture
```
┌──────────────────────┐
│   Minimal Core       │ (幾何カーネルのみ)
└──────────────────────┘
     ↑  ↑  ↑  ↑
     │  │  │  │
  ┌──┴──┴──┴──┴──┐
  │  Plug-ins     │ (DXF, STEP, レンダラー)
  └───────────────┘
```
**特徴**:
- ✅ 拡張性が高い
- ✅ プラグインで機能追加
- ❌ プラグイン間の依存関係が複雑化しやすい

**採用例**: FreeCAD, QCAD

---

### 3.3 Client-Server (Cloud CAD)
```
┌─────────────┐         ┌─────────────┐
│  Client     │ ←───→  │  Server     │
│  (Viewer)   │  HTTP   │  (Compute)  │
└─────────────┘         └─────────────┘
```
**特徴**:
- ✅ リアルタイム共同編集
- ✅ デバイス間でデータ同期
- ❌ オフライン作業不可
- ❌ レイテンシの影響

**採用例**: Onshape, Fusion 360 (一部)

---

### 3.4 Modern Open-Source CAD

#### CADmium (Browser-Based Parametric CAD)
```
Architecture:
  Rust Core (truck B-rep) → WASM
  ↓
  JavaScript Bindings
  ↓
  SvelteKit + Three.js (WebGL)
```
**特徴**:
- ✅ ブラウザのみで動作（インストール不要）
- ✅ Rust + WASM で高速
- ✅ JSON ベースのファイル形式
- ❌ 2024年に開発中止（アーカイブ化）

**分類**: **Client-Server (Browser) + Parametric**

---

#### Chili3D (Browser-Based 3D CAD)
```
Architecture:
  OpenCASCADE (OCCT) → WASM
  ↓
  TypeScript
  ↓
  Three.js (WebGL)
```
**特徴**:
- ✅ OpenCASCADE をブラウザで実行
- ✅ STEP, IGES, BREP 対応
- ✅ ローカルファースト（ブラウザに保存）
- ✅ 多言語対応（中国語・英語）

**分類**: **Client-Server (Browser) + Direct Modeling**

---

#### ennucore/cadmium (AI Agent for CAD)
```
Architecture:
  GPT-4 Agent
  ↓
  CadQuery (Python)
  ↓
  OpenCASCADE
  ↓
  STL Export
```
**特徴**:
- ✅ テキストプロンプトから3Dモデル生成
- ✅ 反復改善（フィードバックループ）
- ❌ GUI なし（CLI のみ）

**分類**: **Programmatic CAD + AI**

---

## 4. Constraint & Parametric Systems
> **問い**: 「寸法と拘束をどう管理するか？」

### 4.1 Direct Modeling (ダイレクトモデリング)
```
操作: 面を直接ドラッグして移動
履歴: なし
```
**特徴**:
- ✅ 直感的
- ✅ 他人のデータを編集しやすい
- ❌ 設計意図が失われる
- ❌ 変更の影響範囲が不明確

**採用例**: SketchUp, Tinkercad

---

### 4.2 Parametric Modeling (パラメトリックモデリング)
```
Feature Tree:
  1. Sketch (拘束: 平行、寸法50mm)
  2. Extrude (高さ: 100mm)
  3. Fillet (半径: 5mm)
```
**特徴**:
- ✅ 設計意図を保持
- ✅ パラメータ変更で自動更新
- ❌ Feature Treeが壊れると修復困難
- ❌ 学習コスト高い

**採用例**: SolidWorks, Fusion 360, Onshape

**制約ソルバー**:
- D-Cubed 3D DCM (Siemens)
- LEDAS LGS
- 数学的手法: Newton法、グラフ分解

---

## 5. Our Choice: Document-View Pattern
> **Rust CAD Framework の立ち位置**

### 5.1 設計思想
```rust
// Document（永続化対象）
pub struct CadDocument {
    entities: HashMap<EntityId, Entity>,
    layers: LayerManager,
    history: CommandHistory,
}

// View（一時的な状態）
pub struct CadView {
    camera: Camera,
    selection: SelectionState,
    active_tool: Box<dyn Tool>,
}
```

### 5.2 なぜこの方式か？

| 要件 | Document-View | ECS | Immediate Mode |
|------|--------------|-----|----------------|
| シンプルさ | ✅ | ❌ | ✅ |
| Undo/Redo | ✅ | △ | ❌ |
| ファイル保存 | ✅ | △ | △ |
| AI操作性 | ✅ | ❌ | ✅ |
| 拡張性 | ✅ | ✅ | ❌ |

### 5.3 適用範囲
✅ **適している**:
- 2D CAD（平面図、回路図、配管図）
- AIによる自動テスト・操作
- 教育用CAD

❌ **適していない**:
- 3Dモデリング（→ Open CASCADE, Parasolid）
- リアルタイム共同編集（→ CRDT）
- ゲーム開発（→ Bevy ECS）

---

## 📖 参考文献

### 学術論文
1. "Geometric Deep Learning on B-rep CAD Models" (arXiv, 2023)
2. "Constraint-Based Geometric Modeling" (Purdue University)
3. "ABC-Dataset: A Million CAD Models" (CVPR)

### 商用幾何カーネル
- **ACIS** (Dassault Systèmes Spatial)
- **Parasolid** (Siemens)
- **Open CASCADE** (Open Source)
- **C3D Modeler** (C3D Labs)

### オープンソースCAD
- **FreeCAD** (Parametric, Open CASCADE)
- **LibreCAD** (2D, Direct)
- **OpenSCAD** (CSG, Programmatic)

---

## 🎯 まとめ

CADアーキテクチャに「銀の弾丸」は存在しない。
**用途に応じて適切なパターンを選択すること**が重要。

**Rust CAD Framework** は、以下の理由で **Document-View** を採用:
1. 2D CADに最適
2. AI操作性を重視
3. 学習コストが低い
4. 拡張性と保守性のバランスが良い

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-25*
