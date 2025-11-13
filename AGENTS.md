# Repository Guidelines

## ソフトウェアアーキテクチャ

このプロジェクトは**レイヤードアーキテクチャ**と**ドメイン駆動設計（DDD）**の原則に従っています。

詳細は以下のドキュメントを参照してください：

- **`docs/documentations/software-architecture.md`** - アーキテクチャ全体の設計方針
- **`docs/documentations/ddd.md`** - DDD の基礎知識

## プロジェクト構造とモジュール配置

本プロジェクトは **Cargo Workspace** を使用した複数パッケージ構成です。**サーバロジックを中心に設計**されており、レイヤードアーキテクチャに基づいて構成されています。

### ディレクトリ構成

```txt
.
├── Cargo.toml              # workspace root（依存関係の一元管理）
├── README.md               # プロジェクト概要
├── AGENTS.md               # リポジトリガイドライン（本ファイル）
├── CLAUDE.md               # Claude への指示
├── docs/
│   ├── adr/                # Architecture Decision Records
│   ├── documentations/     # アーキテクチャ・DDD 設計ドキュメント
│   ├── note/               # 開発メモ
│   └── tasks/              # タスク管理ドキュメント
├── packages/
│   ├── shared/             # 共通ユーティリティパッケージ
│   │   └── src/
│   │       ├── time.rs     # 時刻管理（Clock trait, get_jst_timestamp）
│   │       └── logger.rs   # ロガー設定
│   ├── server/             # サーバアプリケーションパッケージ
│   │   └── src/
│   │       ├── bin/
│   │       │   └── server.rs  # サーババイナリエントリーポイント
│   │       ├── domain/        # ドメイン層
│   │       ├── usecase/       # UseCase 層
│   │       ├── infrastructure/ # インフラ層
│   │       └── ui/            # UI 層
│   └── client/             # クライアントアプリケーションパッケージ
│       └── src/
│           ├── bin/
│           │   └── client.rs  # クライアントバイナリエントリーポイント
│           ├── domain.rs      # クライアントドメインロジック
│           ├── formatter.rs   # メッセージフォーマット
│           └── session.rs     # WebSocket セッション管理
└── tests/                  # workspace 全体の統合テスト
    ├── fixtures/           # テスト共有ヘルパー（TestServer, TestClient）
    ├── http_api.rs         # HTTP API 統合テスト
    ├── websocket_connection.rs  # WebSocket 接続テスト
    └── websocket_messaging.rs   # WebSocket メッセージングテスト
```

### パッケージ構成

本プロジェクトは以下の3つのパッケージで構成されています：

1. **shared パッケージ** (`packages/shared/`)
   - 技術的ユーティリティ（時刻管理、ロガー）
   - 他のパッケージから参照される共通基盤

2. **server パッケージ** (`packages/server/`)
   - レイヤードアーキテクチャによる4層構造
   - **UI 層** (`ui/`): HTTP/WebSocket ハンドラー、ルーティング設定、サーバー起動
   - **UseCase 層** (`usecase/`): ビジネスロジック、ユースケース実装
   - **Domain 層** (`domain/`): エンティティ、値オブジェクト、Factory、ドメインエラー
   - **Infrastructure 層** (`infrastructure/`): DTO（Data Transfer Object）、ドメインモデル変換
   - 依存関係: `shared` パッケージに依存

3. **client パッケージ** (`packages/client/`)
   - CLI チャットクライアント実装
   - シンプルな構成（テスト用ユーティリティとしての位置づけ）
   - 依存関係: `shared`, `server/infrastructure/dto` に依存

詳細なモジュール構成と設計方針は [ソフトウェアアーキテクチャ](./docs/documentations/software-architecture.md) および [ADR 0002: Cargo Workspace 構造への移行](./docs/adr/0002-cargo-workspace-structure.md) を参照してください。

## ビルド・テスト・開発コマンド

### ビルド

- `cargo build --workspace` : workspace 全体をビルドします。
- `cargo build -p server` : server パッケージのみビルドします。
- `cargo build -p client` : client パッケージのみビルドします。
- `cargo build -p shared` : shared パッケージのみビルドします。

### テスト

- `cargo test --workspace` : workspace 全体のテストを実行します。フェイル時は `-- --nocapture` で詳細を追跡します。
- `cargo test -p server` : server パッケージのテストのみ実行します。
- `cargo test -p client` : client パッケージのテストのみ実行します。
- `cargo test -p shared` : shared パッケージのテストのみ実行します。

### Lint・フォーマット

- `cargo fmt --workspace` : workspace 全体を Rustfmt で整形します。PR 直前に必ず実行してください。
- `cargo clippy --workspace --all-targets --all-features` : Axum マクロや Tokio の非同期コードを含め lint します。

### 実行

- `cargo run -p server --bin server` : WebSocket サーバを起動します。`RUST_LOG=info` で通信ログを確認できます。
- `cargo run -p client --bin client -- --client-id alice` : 任意の `client_id` でクライアントを起動します（例：別ターミナルで bob）。

**重要**: 実装タスクを行ったとき、必ず `cargo fmt --workspace`、`cargo clippy --workspace --all-targets --all-features`、`cargo test --workspace` を実行してください。これらが通るまでタスクを終了しないでください。

## コーディングスタイルと命名

### Rust

- Rust 2024 edition / 4 スペースインデント / `snake_case` 関数・変数、`PascalCase` 型、`SCREAMING_SNAKE_CASE` 定数。
- 共有モジュールは `mod transport;` のように `src/` 直下へ切り出し、サーバ・クライアントから再利用します。
- ログは `tracing::info!` 系を使い、イベント名（`participant_joined` など）をフィールドとして付与します。
- エラーハンドリングでは `anyhow` を使用せず、ドメインロジックのエラーは `thiserror` を使って各レイヤーの `error.rs` に定義します。各エラー型は明確なビジネスロジックの失敗を表現してください。
- **インポート規約**: ワイルドカードインポート（`use path::*;`）は使用しない。明示的にインポートする項目を列挙する。
  - ✅ 良い例: `use fixtures::{TestServer, TestClient};`
  - ❌ 悪い例: `use fixtures::*;`
  - **例外**: ユニットテスト内での `use super::*;` のみ許可される。

**モジュール命名規約の詳細は [software-architecture.md](./docs/documentations/software-architecture.md) を参照してください。**

### Markdown

- **コードブロック記法**: ドキュメント内でテキストのみのコードブロックを記述する場合は、言語指定に `txt` を使用する。
  - ✅ 良い例: ````txt ...````
  - ❌ 悪い例: ```` ... ````（言語指定なし）
  - 対象: ディレクトリツリー、依存関係図、フロー図など、特定の言語ではないテキスト
- **ドキュメント参照記法**: ドキュメント内で他のファイルを参照する場合は、Markdown リンク形式で相対パスを使用する。
  - ✅ 良い例: `[アーキテクチャ設計](./docs/documentations/software-architecture.md)`
  - ✅ 良い例: `[タスク](./docs/tasks/20251112-032514_introduce-message-pusher.md)`
  - ❌ 悪い例: `` `docs/documentations/software-architecture.md` ``（バッククォートのみ）
  - ❌ 悪い例: `docs/documentations/software-architecture.md`（リンクなし）
  - 理由: リンク形式にすることで、エディタやビューアーでクリック可能になり、ドキュメント間の移動が容易になる

## テスト指針

### テスト階層

プロジェクトは3層のテスト戦略を採用しています：

1. **単体テスト（Unit Tests）**: ドメインロジックの純粋関数をテスト
2. **統合テスト（Integration Tests）**: プロセスベースで実際のサーバー・クライアント間通信をテスト
3. **手動 E2E テスト**: 実際のユーザーシナリオを手動で検証
   - 複数クライアントでのリアルタイムチャット
   - UI/UX の確認（プロンプト表示、カーソル制御など）

### テスト実装ガイドライン

- 非同期テストは `#[tokio::test(flavor = "multi_thread")]` を使用（統合テストでは不要）
- ドメインロジックは可能な限り純粋関数として抽出し、I/O から分離
- 副作用のある処理（時刻取得など）は trait で抽象化し、テスト時は FixedClock などを使用
- 統合テストでは `std::process::Command` を使って実際の cargo プロセスを起動
- テスト実装は twada の TDD ワークフロー（https://t-wada.hatenablog.jp/entry/canon-tdd-by-kent-beck）に従い、Red → Green → Refactor のサイクルで進める

### テストフォーマット

すべてのテストは以下のフォーマットに従って記述します：

```rust
#[test]
fn test_<名前>() {
    // テスト項目: <テストの説明>
    // given (前提条件):
    <前提条件のセットアップ>

    // when (操作):
    <テスト対象の実行>

    // then (期待する結果):
    <アサーション>
}
```

**注意事項**:

- `#[test]` はテストフレームワークに応じて適切な属性を使用します。
  - `#[test]` は同期テストを実行します。
  - `#[test(flavor = "multi_thread")]` はマルチスレッドで非同期テストを実行します。
  - `#[test(flavor = "single_thread")]` は単一スレッドで非同期テストを実行します。
  - `#[tokio::test]` は非同期テストを実行します。
- `// テスト項目:` の `:` の後には必ず半角スペースを入れる
- given/when/then の各セクション間は空行を 1 行入れる
- 各セクションのコメント後は改行してからコードを書く

## コミットとプルリクエスト

- Git 履歴は「Init cargo project」のように命令形・簡潔な題で統一されています。`component: imperative summary` 形式を推奨します。
- PR では概要、テスト結果（`cargo fmt`, `cargo clippy`, `cargo test`）、関連 Issue、必要に応じクライアント入出力のスクリーンショットやログ抜粋を添付します。
- 大きな変更はサーバとクライアントを別々のコミットに分割し、レビュワーが影響範囲を追いやすくしてください。

## 運用とトラブルシュート

- ローカル実行時は `RUST_LOG=debug cargo run --bin server` で trace ログを確認し、WebSocket の接続/切断イベントを追うと調査が短縮されます。
- 重複 `client_id` エラーは HTTP 409 が返る想定のため、適宜 `curl -i localhost:PORT -H 'client-id: alice'` などでハンドシェイク層も検証してください。

## タスク管理

実装タスクを整理する際は、以下のルールに従ってタスクドキュメントを作成・管理します。

### タスクドキュメントの作成

- ユーザーから「タスクを整理して」「実装タスクを整理」などの文言が含まれるリクエストがあった場合、`docs/tasks/yyyymmdd-hhmmss_<task-summary>.md` 形式でタスクドキュメントを作成します。
- ファイル名の `yyyymmdd-hhmmss` は作成日時（JST）を表します（例: `251111-173839_ddd-and-api-endpoints.md`）。
- `<task-summary>` はタスクの概要を簡潔に表すキーワード（ケバブケース）を使用します。
- 毎回自動的に作成するのではなく、明示的な整理リクエストがあった場合のみ作成します。
- **タスクが肥大化しすぎないように注意してください**。1つのタスクドキュメントは特定の機能実装やリファクタリングに絞り、大きな変更は複数のタスクドキュメントに分割することを推奨します。
- **ドキュメント内の全ての日時（作成日、開始日、更新日時等）は JST（日本標準時）で記載してください**。形式は `YYYY-MM-DD HH:MM:SS` を推奨します（例: `2025-11-11 17:38:39`）。

### タスクドキュメントの構成

タスクドキュメントは `docs/tasks/yyyymmdd-hhmmss_task-summary.md` のテンプレートを参考に作成します。

#### 必須セクション

最低限、以下のセクションを含める必要があります：

1. **タイトル**: 実装タスクの名称
2. **概要**: 目的、背景、スコープ、参照
3. **方針**: アプローチ、設計方針、品質基準
4. **タスク**: チェックリスト形式の具体的な実装タスク

#### 推奨セクション

必要に応じて以下のセクションも追加してください：

1. **進捗状況**: 開始日、現在のフェーズ、完了タスク数、次のアクション、ブロッカー
2. **備考**: 実装時の注意事項やルール
3. **参考資料**: 関連ドキュメントへのリンクや技術的な背景情報

詳細は `docs/tasks/yyyymmdd-hhmmss_task-summary.md` のテンプレートを参照してください。

### タスクの記述形式

- すべてのタスクは `- [ ]` (未完了) または `- [x]` (完了) のチェックリスト形式で記述します。
- サブタスクがある場合は、インデントを使って階層構造を表現します。
- この形式により、タスクの完了状況を一目で確認できます。

#### タスクドキュメントの同期

- 実装の進捗に応じて、タスクドキュメントのチェックリストを更新します。
- 完了したタスクは `- [ ]` から `- [x]` に変更し、進捗状況セクションも更新します。
- 新たなタスクが発生した場合は、適切なフェーズに追加します。
