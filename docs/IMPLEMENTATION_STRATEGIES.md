# CAD Implementation Strategies: Monolith vs Multi-Crate

> **対象**: アーキテクチャ選択で悩む開発者
> 
> **目的**: モノリス vs マルチクレート、複数の実装戦略を比較し、最適な選択を支援

---

## 📚 Table of Contents
1. [Why Current Phase Design](#1-why-current-phase-design)
2. [Alternative Strategies](#2-alternative-strategies)
3. [Monolith vs Multi-Crate](#3-monolith-vs-multi-crate)
4. [Recommended: Cargo Workspace](#4-recommended-cargo-workspace)
5. [Implementation Comparison](#5-implementation-comparison)

---

## 1. Why Current Phase Design

### 現在の Phase 分けの理由

#### ✅ 採用した理由

1. **依存関係の自然な順序**
   ```
   Phase 1: 基礎 (geometry, rendering)
     ↓
   Phase 2: ツール (tools)
     ↓
   Phase 3: UI (ui)
     ↓
   Phase 4: I/O (io)
     ↓
   Phase 5: 高度な機能 (command, layer)
   ```

2. **早期の動作確認**
   - Phase 0: 十字線が見える → モチベーション維持
   - Phase 1: 図形が描画される → 達成感
   - Phase 2: マウスで描ける → 実用性を実感

3. **学習曲線の最適化**
   - wgpu の難しさを最初に乗り越える
   - 後は比較的簡単な機能追加

4. **リスクの早期発見**
   - レンダリングの問題を最初に解決
   - 後で大きな設計変更を避ける

---

### ❌ この Phase 分けの問題点

1. **モノリス構造**
   - 全てが `src/` 以下に入る
   - モジュール間の境界が曖昧

2. **並列開発が難しい**
   - Phase 1 が終わらないと Phase 2 に進めない
   - チーム開発に不向き

3. **テストが後回し**
   - 動作確認はするが、自動テストは少ない

4. **スケールしにくい**
   - 大規模になると `src/` が肥大化

---

## 2. Alternative Strategies

### 戦略1: Feature-First (機能優先)

#### コンセプト
「最小限の機能を完全に実装してから次へ」

#### Phase 分け
```
Phase 1: Line Tool (完全実装)
  - geometry (Line のみ)
  - rendering (Line のみ)
  - tools (LineTool のみ)
  - ui (最小限)
  - io (Line のみ保存)
  - command (Line の Undo/Redo)
  - tests (Line の完全なテスト)

Phase 2: Circle Tool (完全実装)
  - geometry (Circle 追加)
  - rendering (Circle 追加)
  - tools (CircleTool 追加)
  - io (Circle 保存追加)
  - command (Circle の Undo/Redo)
  - tests (Circle の完全なテスト)

Phase 3: Arc Tool (完全実装)
  ...
```

#### ✅ メリット
- 各機能が完全にテストされる
- リリース可能な状態を常に維持
- 並列開発しやすい（機能ごとにブランチ）

#### ❌ デメリット
- 初期の達成感が少ない
- 重複コードが発生しやすい
- リファクタリングが頻繁に必要

---

### 戦略2: Layer-First (レイヤー優先)

#### コンセプト
「アーキテクチャのレイヤーごとに完成させる」

#### Phase 分け
```
Phase 1: Data Layer (完全実装)
  - geometry (全エンティティ)
  - store (完全な CRUD)
  - tests (データ層の完全なテスト)

Phase 2: Rendering Layer (完全実装)
  - renderer (全エンティティの描画)
  - camera (完全な操作)
  - shaders (最適化済み)
  - tests (レンダリングの完全なテスト)

Phase 3: Business Logic Layer (完全実装)
  - tools (全ツール)
  - command (完全な Undo/Redo)
  - tests (ロジックの完全なテスト)

Phase 4: Presentation Layer (完全実装)
  - ui (完全な UI)
  - tests (UI の完全なテスト)
```

#### ✅ メリット
- アーキテクチャが明確
- レイヤー間の依存が整理される
- テストが書きやすい

#### ❌ デメリット
- 動作確認が遅い（Phase 3 まで何も動かない）
- モチベーション維持が難しい
- 要件変更に弱い

---

### 戦略3: Vertical Slice (垂直スライス)

#### コンセプト
「ユーザーストーリーごとに全レイヤーを実装」

#### Phase 分け
```
Phase 1: "線を引いて保存する"
  - geometry (Line)
  - rendering (Line 描画)
  - tools (LineTool)
  - ui (最小限のツールパレット)
  - io (Line の保存)
  - command (Line の Undo)
  - tests (E2E テスト)

Phase 2: "円を描いて保存する"
  - geometry (Circle)
  - rendering (Circle 描画)
  - tools (CircleTool)
  - ui (ツール切り替え)
  - io (Circle の保存)
  - command (Circle の Undo)
  - tests (E2E テスト)

Phase 3: "図形を選択して移動する"
  - tools (SelectTool, MoveTool)
  - command (Move の Undo)
  - tests (E2E テスト)
```

#### ✅ メリット
- 常に動作するソフトウェア
- ユーザー価値が明確
- 並列開発しやすい
- アジャイル開発に最適

#### ❌ デメリット
- 共通コードの抽出が後回し
- リファクタリングが頻繁
- 設計の一貫性を保つのが難しい

---

## 3. Monolith vs Multi-Crate

### Monolith (モノリス)

#### 構造
```
my-cad/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── geometry/
    ├── rendering/
    ├── tools/
    └── ui/
```

#### ✅ メリット
- シンプル
- 開発開始が早い
- リファクタリングが容易
- ビルドが速い

#### ❌ デメリット
- モジュール境界が曖昧
- 並列開発が難しい
- テストが遅くなる
- スケールしにくい

---

### Multi-Crate (マルチクレート)

#### 構造
```
my-cad/
├── Cargo.toml (workspace)
├── crates/
│   ├── cad-core/          # 幾何データ
│   ├── cad-rendering/     # レンダリング
│   ├── cad-tools/         # ツール
│   ├── cad-ui/            # UI
│   ├── cad-io/            # ファイルI/O
│   └── cad-app/           # アプリケーション
└── apps/
    └── desktop/           # デスクトップアプリ
```

#### ✅ メリット
- モジュール境界が明確
- 並列開発しやすい
- 再利用性が高い
- テストが独立
- スケールしやすい

#### ❌ デメリット
- 初期設定が複雑
- ビルドが遅くなる可能性
- 依存関係管理が大変

---

## 4. Recommended: Cargo Workspace

### 推奨構成

```
my-cad/
├── Cargo.toml              # Workspace 定義
│
├── crates/
│   ├── cad-core/           # コアライブラリ
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── geometry/
│   │       └── math/
│   │
│   ├── cad-rendering/      # レンダリング
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs
│   │       └── camera.rs
│   │
│   ├── cad-tools/          # ツールシステム
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── line_tool.rs
│   │       └── circle_tool.rs
│   │
│   ├── cad-ui/             # UI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── panels.rs
│   │
│   └── cad-io/             # ファイルI/O
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── dxf.rs
│           └── json.rs
│
└── apps/
    └── desktop/            # デスクトップアプリ
        ├── Cargo.toml
        └── src/
            └── main.rs
```

---

### Workspace Cargo.toml

```toml
[workspace]
members = [
    "crates/cad-core",
    "crates/cad-rendering",
    "crates/cad-tools",
    "crates/cad-ui",
    "crates/cad-io",
    "apps/desktop",
]

resolver = "2"

[workspace.dependencies]
# 共通の依存関係
winit = "0.29"
wgpu = "0.18"
egui = "0.24"
serde = { version = "1.0", features = ["derive"] }
```

---

### cad-core/Cargo.toml

```toml
[package]
name = "cad-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
slotmap = "1.0"
```

---

### cad-rendering/Cargo.toml

```toml
[package]
name = "cad-rendering"
version = "0.1.0"
edition = "2021"

[dependencies]
cad-core = { path = "../cad-core" }
wgpu = { workspace = true }
winit = { workspace = true }
bytemuck = "1.14"
```

---

### apps/desktop/Cargo.toml

```toml
[package]
name = "my-cad"
version = "0.1.0"
edition = "2021"

[dependencies]
cad-core = { path = "../../crates/cad-core" }
cad-rendering = { path = "../../crates/cad-rendering" }
cad-tools = { path = "../../crates/cad-tools" }
cad-ui = { path = "../../crates/cad-ui" }
cad-io = { path = "../../crates/cad-io" }

winit = { workspace = true }
wgpu = { workspace = true }
egui = { workspace = true }
```

---

## 5. Implementation Comparison

### 戦略比較表

| 戦略 | 初期速度 | 並列開発 | テスト | スケール | 推奨度 |
|------|---------|---------|--------|---------|--------|
| **Current (Layer-by-Layer)** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ✅ 個人開発 |
| **Feature-First** | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ✅ 小チーム |
| **Layer-First** | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | △ 大規模 |
| **Vertical Slice** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ✅ アジャイル |
| **Monolith** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ✅ プロトタイプ |
| **Multi-Crate** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ✅ 本番 |

---

### 推奨戦略

#### 個人開発・プロトタイプ
```
戦略: Current (Layer-by-Layer)
構造: Monolith
理由: 最速で動作確認できる
```

#### 小チーム (2-5人)
```
戦略: Vertical Slice
構造: Multi-Crate (Workspace)
理由: 並列開発しやすい、常に動作する
```

#### 大規模チーム (5人以上)
```
戦略: Feature-First + Multi-Crate
構造: Workspace + マイクロサービス
理由: 完全に独立した開発が可能
```

---

## 6. Migration Path

### Monolith → Multi-Crate への移行

#### Step 1: Workspace 作成
```bash
# 1. ディレクトリ作成
mkdir -p crates/cad-core
mkdir -p apps/desktop

# 2. Workspace Cargo.toml 作成
cat > Cargo.toml << 'EOF'
[workspace]
members = ["crates/cad-core", "apps/desktop"]
EOF
```

#### Step 2: Core を分離
```bash
# 3. cad-core に移動
mv src/geometry crates/cad-core/src/
mv src/math crates/cad-core/src/

# 4. cad-core/Cargo.toml 作成
cat > crates/cad-core/Cargo.toml << 'EOF'
[package]
name = "cad-core"
version = "0.1.0"
edition = "2021"
EOF

# 5. lib.rs 作成
cat > crates/cad-core/src/lib.rs << 'EOF'
pub mod geometry;
pub mod math;
EOF
```

#### Step 3: App を移動
```bash
# 6. アプリを apps/desktop に移動
mv src apps/desktop/
mv Cargo.toml apps/desktop/

# 7. 依存関係を更新
# apps/desktop/Cargo.toml に追加
[dependencies]
cad-core = { path = "../../crates/cad-core" }
```

#### Step 4: ビルド確認
```bash
cargo build
```

---

## 7. Recommended Implementation Plan

### 推奨: Vertical Slice + Multi-Crate

#### Phase 0: Workspace Setup (1日)
```bash
# Workspace 構造を作成
mkdir -p crates/{cad-core,cad-rendering,cad-tools,cad-ui,cad-io}
mkdir -p apps/desktop

# Cargo.toml を作成
# 各クレートの Cargo.toml を作成
```

#### Phase 1: "線を引く" (3-5日)
```
並列開発:
  - Person A: cad-core (Line)
  - Person B: cad-rendering (Line 描画)
  - Person C: cad-tools (LineTool)

統合: apps/desktop で統合
テスト: E2E テスト
```

#### Phase 2: "円を描く" (3-5日)
```
並列開発:
  - Person A: cad-core (Circle)
  - Person B: cad-rendering (Circle 描画)
  - Person C: cad-tools (CircleTool)

統合: apps/desktop で統合
テスト: E2E テスト
```

#### Phase 3: "保存・読み込み" (2-3日)
```
並列開発:
  - Person A: cad-io (JSON)
  - Person B: cad-io (DXF)
  - Person C: apps/desktop (File menu)

統合: apps/desktop で統合
テスト: E2E テスト
```

---

## 📊 最終推奨

### あなたの状況に応じた推奨

| 状況 | 推奨戦略 | 推奨構造 |
|------|---------|---------|
| **個人開発・学習** | Current (Layer-by-Layer) | Monolith |
| **個人開発・本番** | Vertical Slice | Multi-Crate |
| **2-3人チーム** | Vertical Slice | Multi-Crate |
| **5人以上チーム** | Feature-First | Multi-Crate + マイクロサービス |

### 今すぐ始めるなら

#### 個人開発
```bash
# Monolith で開始
cargo new my-cad
cd my-cad

# Phase 0 から実装
# 後で Multi-Crate に移行可能
```

#### チーム開発
```bash
# Multi-Crate で開始
mkdir my-cad
cd my-cad

# Workspace 作成
cat > Cargo.toml << 'EOF'
[workspace]
members = ["crates/*", "apps/*"]
EOF

# クレート作成
cargo new --lib crates/cad-core
cargo new --lib crates/cad-rendering
cargo new apps/desktop
```

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-26*
