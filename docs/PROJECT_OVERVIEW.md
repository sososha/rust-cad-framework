# Project Definition: Rust CAD Framework (The Kit)

> **プロジェクト定義書**
> 
> 本プロジェクトは、特定のCADアプリを作るのではなく、**「汎用的なCADを作成するためのフレームワーク（キット）」** を開発します。
> ユーザーが作りたいCADは、このキットを使用した **「リファレンス実装（作例）」** として実現します。

---

## 1. プロジェクトの正体 (What is this?)

**「Rust CAD Framework」**
= **"Build Your Own CAD" Kit**

CADアプリケーションをゼロから作るための、再利用可能な部品群（クレート）と設計図（ドキュメント）のセットです。

### 🎯 目指すゴール
1. **汎用性**: 建築、機械、回路図など、様々な2D CADの基盤となること。
2. **高性能**: Rust + WGPU による圧倒的な描画パフォーマンスを提供すること。
3. **AI親和性**: AIエージェントが理解・操作しやすい構造（コマンドベース）であること。

---

## 2. 推奨されるCADタイプ (The Recommended Model)

これまでの議論で導き出された、**「最も成功確率が高く、現代的なCADの形」** を定義します。このフレームワークは、このモデルを推奨・サポートします。

### 🏗️ アーキテクチャ: Document-View Pattern
- **データ（Document）** と **表示（View）** を完全に分離する。
- **なぜ？**: ファイル保存、Undo/Redo、複数ビュー表示、AIによるヘッドレス操作を容易にするため。

### 🎨 描画システム: Immediate Mode-like Rendering
- 毎フレーム高速に再描画するが、データ構造は保持する。
- **なぜ？**: 状態管理がシンプルになり、バグが減る。WGPUのパワーで性能も確保できる。

### 🛠️ データ構造: SlotMap + SoA
- エンティティを `Vec` のインデックスではなく、安定した ID (`SlotMap`) で管理する。
- **なぜ？**: 参照の安全性とメモリ効率（キャッシュヒット率）を両立するため。

### 🤖 操作体系: Command Pattern (AI-Native)
- すべてのユーザー操作（描画、削除、移動）を「コマンド」として実装する。
- **なぜ？**: Undo/Redo が自動的に実現でき、AIエージェントがコマンドを発行して操作できるため。

---

## 3. 開発の進め方 (Development Strategy)

「フレームワーク（キット）」と「リファレンス実装（アプリ）」を並行して開発します。

### 📦 構成: Cargo Workspace (Multi-Crate)

```
my-cad/
├── Cargo.toml (Workspace)
├── crates/ (The Kit: フレームワーク本体)
│   ├── cad-core/       # データ構造、計算幾何 (依存なし)
│   ├── cad-rendering/  # WGPUレンダラー、カメラ
│   ├── cad-tools/      # ツール状態マシン、スナップ
│   ├── cad-commands/   # コマンドシステム、Undo/Redo
│   └── cad-io/         # DXF/JSON 入出力
└── apps/ (The Implementation: 具体的なCAD)
    └── reference-cad/  # ユーザーが作りたいCADの実装
```

### 🛣️ フェーズ分け (Phases)

#### Phase 0: The Skeleton (骨格)
- **Kit**: クレート構成の作成。
- **App**: ウィンドウを表示し、Kitをリンクする。

#### Phase 1: The Foundation (描画の基盤)
- **Kit**: `cad-core` (Line, Circle), `cad-rendering` (WGPU描画)。
- **App**: APIを使って、コードで定義した図形を描画する。

#### Phase 2: The Interaction (対話の基盤)
- **Kit**: `cad-tools` (Tool trait, State Machine)。
- **App**: マウスイベントをKitに渡し、線を引くツールを実装する。

#### Phase 3: The Application (アプリ化)
- **Kit**: `cad-commands` (Undo/Redo), `cad-ui` (Widget)。
- **App**: UIを組み込み、ツール切り替えやUndoを実装する。

---

## 4. ユーザーが作りたいものとの関係

- **ユーザーのCAD**: `apps/reference-cad` として実装されます。
- **役割**: フレームワークの機能検証を行い、最初の「成功事例」となります。
- **メリット**: フレームワークが汎用的であるため、将来的に機能（3D対応、Web対応など）を拡張する際に、アプリ側のコードを壊さずに済みます。

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-26*
