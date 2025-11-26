# Project Definition: Rust CAD Framework (The Kit)

> **プロジェクト定義書**
> 
> 本プロジェクトは、**「汎用的なCADを作成するためのフレームワーク（キット）」** を開発し、その実証として **「リファレンス実装（作例）」** を提供します。

---

## 1. オープンソース戦略 (OSS Strategy)

「公開しやすく、貢献（PR）してもらいやすい」構造を採用します。

### 📦 リポジトリ構造: Monorepo (Cargo Workspace)

フレームワーク（Kit）とアプリケーション（App）を **1つのリポジトリ** で管理します。

```
rust-cad-framework/
├── Cargo.toml          # Workspace 定義
├── README.md           # プロジェクト全体の解説
├── CONTRIBUTING.md     # 貢献ガイドライン
│
├── crates/             # 【The Kit】フレームワーク本体 (再利用可能)
│   ├── cad-core/       # データ構造 (依存なし)
│   ├── cad-rendering/  # 描画エンジン (WGPU)
│   ├── cad-tools/      # 操作ロジック
│   └── cad-commands/   # Undo/Redo
│
└── apps/               # 【The App】リファレンス実装 (作例)
    └── reference-cad/  # 実際に動くCADアプリ
```

#### ✅ この構造のメリット
1. **開発の同期**: フレームワークの変更を即座にアプリでテストできる。
2. **貢献のしやすさ**: 「アプリのバグを直そうとしたら、フレームワークの修正が必要だった」という場合に、1つのPRで完結できる。
3. **明確な分離**: `crates/` は汎用的、`apps/` は具体的、という境界が物理的に見える。

---

## 2. 開発の進め方: 戦略の再確認

### 🔄 3つの選択肢 (Recap)

| 戦略 | 進め方 | メリット | デメリット |
|------|--------|----------|------------|
| **1. Layer-First** | データ層 → 描画層 → UI層 の順に作る | 設計が綺麗、教科書的 | 最後までアプリが動かない |
| **2. Feature-First** | 線機能(全層) → 円機能(全層) の順に作る | 常に動く、リリースしやすい | 設計の一貫性を保つのが難しい |
| **3. Vertical Slice** | **推奨**: 機能ごとに「Kit」と「App」を同時に作る | **APIの使い勝手を即検証できる** | コンテキストスイッチが発生する |

### 🏆 推奨: Vertical Slice Strategy

フレームワーク開発において最も重要なのは **「そのAPIは本当に使いやすいか？」** を検証することです。

#### 具体的なステップ
1. **Slice 1: "Line"**
   - **Kit**: `cad-core` に `Line` を定義し、`cad-rendering` で描画できるようにする。
   - **App**: そのAPIを使って、画面に線を引く。
   - *検証*: 「APIが複雑すぎないか？」「必要な機能は足りているか？」

2. **Slice 2: "Interaction"**
   - **Kit**: `cad-tools` にマウスイベント処理を追加。
   - **App**: マウスで線を引けるようにする。
   - *検証*: 「イベントの伝播はスムーズか？」

3. **Slice 3: "UI"**
   - **Kit**: ツール切り替えの仕組みを提供。
   - **App**: ツールパレットを表示する。

---

## 3. 推奨されるCADタイプ (The Recommended Model)

### 🏗️ アーキテクチャ: Document-View Pattern
- **データ（Document）** と **表示（View）** を完全に分離。
- **理由**: 汎用性が高く、GUIを持たないAIエージェントや、Webサーバー上でも動作させやすいため。

### 🛠️ データ構造: SlotMap + SoA
- **理由**: Rustの借用規則と相性が良く、パフォーマンス（キャッシュ効率）が最適だから。

### 🤖 操作体系: Command Pattern
- **理由**: 「全ての操作をコマンドにする」ことで、Undo/Redoの実装コストがゼロになり、自動化も容易になるから。

---

## 4. Next Action

**Phase 0: The Skeleton (骨格作り)** を開始します。

1. `cargo new` でワークスペースを作成。
2. `crates/` と `apps/` ディレクトリを作成。
3. 最初の `cad-core` クレートと `reference-cad` アプリを作成し、リンクさせる。

---

*Created by Rust CAD Framework Team*
*Last Updated: 2025-11-26*
