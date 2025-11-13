# 本プロジェクトにおけるレイヤードアーキテクチャの適用

本プロジェクトのソフトウェアアーキテクチャは [レイヤードアーキテクチャ](./layered-architecture.md) を適用しています。

## プロジェクト構成

本プロジェクトは **Cargo Workspace** を使用した複数パッケージ構成です。サーバアプリケーション（`packages/server/`）にレイヤードアーキテクチャを適用しています。詳細は [ADR 0002: Cargo Workspace 構造への移行](../adr/0002-cargo-workspace-structure.md) を参照してください。

## レイヤードアーキテクチャの各レイヤーの定義

参照元：[@docs/documentations/layered-architecture.md](./layered-architecture.md)

- 「境界づけられたコンテキスト」単位で定義された Rust モジュールは、以下の 4 つの層（レイヤー）を中心に構成する
  - UserInterface 層（UI 層、Presentation 層とも呼ばれる。本プロジェクトでは UserInterface 層と呼ぶ。）
  - UseCase 層（Application 層とも呼ばれるが、本プロジェクトでは UseCase 層と呼ぶ）
  - Domain 層
  - Infrastructure 層
- それぞれのレイヤーは同名の package にまとめる

### UserInterface 層

- UI 層、Presentation 層とも呼ばれる
- API のクライアント（Web 画面や他システム等）との入出力を司る層
- 主な責務
  - クライアントから送信されたリクエストヘッダ、クエリパラメータ、フォームデータ等を取得
  - 入力パラメータのバリデーションチェック
  - Application 層の アプリケーションサービス のメソッドを呼び出し
  - アプリケーションサービスの返却結果からレスポンスを構築

### UseCase 層

- Domain 層のエンティティ単位で アプリケーションサービス を宣言する
- ユースケース単位で アプリケーションサービス のメソッドを作成する
- 各メソッドは基本的にはドメインオブジェクトのメソッド呼び出しとエラーのハンドリングのみ実施する
  - 条件判断や計算等の業務ロジックは アプリケーションサービス には記述せず、ドメインオブジェクト操作の処理順序のみが記述されているのが理想的
  - 業務ロジックは後述する Domain 層に閉じ込める
- DB トランザクションもアプリケーションサービスのメソッド単位で Commit/Rollback する形を基本とする

### Domain 層

- ドメインオブジェクトで構成される層（エンティティ、値オブジェクト、リポジトリ等）
- 条件判断や計算等の業務ロジックはこの層のオブジェクトに極力入れるのが望ましい

### Infrastructure 層

- DB やファイル操作、外部 API 呼び出し等の外部システムへ接続して実施する操作を司る層

## 各レイヤーの責務と配置場所

### UserInterface 層（UI 層）

**責務**:

- 外部からのリクエストを受け取る
- レスポンスを外部に返す
- DTO（Data Transfer Object）の定義と変換

**配置場所**:

- `packages/server/src/ui/handler/websocket.rs` - WebSocket ハンドラー
- `packages/server/src/ui/handler/http.rs` - HTTP API ハンドラー
- `packages/server/src/ui/server.rs` - サーバー起動とルーティング設定
- `packages/server/src/ui/state.rs` - アプリケーション状態管理

**依存関係**:

- UseCase 層に依存
- Domain 層に依存（DTO ⇔ Domain Model の変換のため）
- Infrastructure 層に依存（DTO を使用）

### UseCase 層（Application 層）

**責務**:

- ビジネスユースケースの実装
- トランザクション管理
- 複数の Domain Model を組み合わせた処理

**配置場所**:

- `packages/server/src/usecase/` - UseCase 層のユースケース実装
  - `connect_participant.rs` - 参加者接続のユースケース
  - `disconnect_participant.rs` - 参加者切断のユースケース
  - `send_message.rs` - メッセージ送信のユースケース
  - `get_rooms.rs` - ルーム一覧取得のユースケース
  - `get_room_detail.rs` - ルーム詳細取得のユースケース
  - `get_room_state.rs` - ルーム状態取得のユースケース
  - `error.rs` - UseCase 層のエラー定義

**依存関係**:

- Domain 層に依存
- Infrastructure 層に依存（Repository を使用）
- UserInterface 層には依存しない

### Domain 層

**責務**:

- ビジネスロジックの中核
- ドメインモデル（Entity、Value Object）の定義
- ドメインサービスの実装

**配置場所**:

- `packages/server/src/domain/entity.rs` - Room, Participant, ChatMessage（Entity）
- `packages/server/src/domain/value_object.rs` - ClientId, RoomId, MessageContent, Timestamp（Value Object）
- `packages/server/src/domain/repository.rs` - Repository trait 定義
- `packages/server/src/domain/message_pusher.rs` - MessagePusher trait 定義
- `packages/server/src/domain/factory.rs` - Factory 関数
- `packages/server/src/domain/error.rs` - ドメイン層のエラー定義（thiserror 使用）

**エラーハンドリング**:

- **ドメイン層のエラーは `thiserror` を使って `packages/server/src/domain/error.rs` に定義します**
- Value Object のバリデーションエラーは `ValueObjectError` enum で定義
- エラーメッセージは構造化されたフィールドを持ち、実際の値と期待値を含む
- 例: `ClientIdTooLong { max: usize, actual: usize }`
- エラーは型安全で、文字列の代わりに専用の型を使用

**特徴**:

- 他のレイヤーに依存しない（依存関係の逆転）
- 純粋なビジネスロジックのみを含む
- I/O や外部システムへの依存を持たない
- エラー型も Domain 層で定義し、他層に漏らさない

**依存関係**:

- どのレイヤーにも依存しない（最も内側の層）

### Infrastructure 層

**責務**:

- 外部システムとの接続
- データの永続化
- 外部 API の呼び出し
- **DTO（Data Transfer Object）の定義と変換**

**配置場所**:

- `packages/server/src/infrastructure/dto/` - WebSocket メッセージ、API レスポンスの DTO
  - `websocket.rs` - WebSocket メッセージ DTO
  - `http.rs` - HTTP API レスポンス DTO
- `packages/server/src/infrastructure/conversion.rs` - DTO ⇔ Domain Model の変換ロジック
- `packages/server/src/infrastructure/repository/` - Repository の実装
  - `in_memory_room_repository.rs` - インメモリ Room Repository
- `packages/server/src/infrastructure/message_pusher/` - MessagePusher の実装
  - `websocket_message_pusher.rs` - WebSocket MessagePusher

将来的にデータベースを導入する場合、以下のような構造を推奨します：

  ```sh
  packages/server/src/infrastructure/
  ├── mod.rs
  ├── dto/
  │   ├── mod.rs
  │   ├── websocket.rs              # WebSocket DTO
  │   └── http.rs                   # HTTP API DTO
  ├── conversion.rs                 # DTO 変換ロジック
  ├── repository/
  │   ├── mod.rs
  │   ├── in_memory_room_repository.rs  # 現在の実装
  │   ├── room_repository.rs            # DB 版 Room の永続化
  │   └── message_repository.rs         # DB 版 Message の永続化
  └── external/
      └── notification_client.rs        # 外部通知サービスのクライアント
  ```

**DTO の配置について**:

- **本プロジェクトでは DTO を Infrastructure 層に配置しています**
- DTO は外部とのデータ交換のための型定義であり、通信プロトコルに依存します
- DTO と Domain Model の変換ロジック（conversion）も Infrastructure 層に配置します
- これにより、外部とのインターフェースに関する関心事を Infrastructure 層に集約できます

**具体例**:

- DTO の定義と変換
- データベースアクセス（Repository の実装）
- 外部 API クライアント
- ファイルシステムアクセス
- メッセージキューなど

**依存関係**:

- Domain 層に依存（Repository のインターフェースは Domain 層で定義、DTO 変換時に Domain Model を使用）
- UserInterface 層や UseCase 層には依存しない

## プロジェクト構造と層の対応

現在のプロジェクト構造:

```sh
packages/
├── shared/                      # 共通ユーティリティ
│   └── src/
│       ├── time.rs             # 時刻管理（Clock trait, get_jst_timestamp）
│       └── logger.rs           # ロガー設定
├── server/                      # サーバアプリケーション
│   └── src/
│       ├── bin/
│       │   └── server.rs       # サーバーバイナリエントリーポイント
│       ├── domain/             # Domain 層
│       │   ├── mod.rs
│       │   ├── entity.rs       # Entity（Room, Participant, ChatMessage）
│       │   ├── value_object.rs # Value Object（ClientId, RoomId, etc.）
│       │   ├── repository.rs   # Repository trait
│       │   ├── message_pusher.rs # MessagePusher trait
│       │   ├── factory.rs      # Factory 関数
│       │   └── error.rs        # ドメインエラー定義
│       ├── usecase/            # UseCase 層
│       │   ├── mod.rs
│       │   ├── connect_participant.rs
│       │   ├── disconnect_participant.rs
│       │   ├── send_message.rs
│       │   ├── get_rooms.rs
│       │   ├── get_room_detail.rs
│       │   ├── get_room_state.rs
│       │   └── error.rs
│       ├── infrastructure/     # Infrastructure 層
│       │   ├── mod.rs
│       │   ├── conversion.rs   # DTO ⇔ Domain Entity 変換
│       │   ├── dto/
│       │   │   ├── mod.rs
│       │   │   ├── websocket.rs # WebSocket DTO
│       │   │   └── http.rs      # HTTP API DTO
│       │   ├── repository/
│       │   │   ├── mod.rs
│       │   │   └── in_memory_room_repository.rs
│       │   └── message_pusher/
│       │       ├── mod.rs
│       │       └── websocket_message_pusher.rs
│       └── ui/                 # UserInterface 層
│           ├── mod.rs
│           ├── server.rs       # サーバー起動
│           ├── state.rs        # 状態管理
│           └── handler/
│               ├── mod.rs
│               ├── http.rs     # HTTP ハンドラー
│               └── websocket.rs # WebSocket ハンドラー
└── client/                      # クライアントアプリケーション
    └── src/
        ├── bin/
        │   └── client.rs       # クライアントバイナリエントリーポイント
        ├── domain.rs           # クライアントドメインロジック
        ├── formatter.rs        # メッセージフォーマット
        └── session.rs          # WebSocket セッション管理
```

## エラーハンドリング方針

### 各レイヤーのエラー定義

**基本方針**:

- **各レイヤーは独自のエラー型を定義します**
- すべてのエラーは `thiserror` を使って定義し、型安全性を確保します
- エラーは文字列ではなく、構造化された enum で表現します

**レイヤーごとのエラー配置**:

1. **Domain 層**: `packages/server/src/domain/error.rs`
   - Value Object のバリデーションエラー
   - ビジネスルール違反のエラー
   - 例: `ValueObjectError::ClientIdEmpty`, `ValueObjectError::ClientIdTooLong { max, actual }`

2. **Infrastructure 層**: `packages/server/src/infrastructure/error.rs`（将来実装予定）
   - データベース接続エラー
   - 外部 API 呼び出しエラー
   - DTO 変換エラー（必要に応じて）

3. **UseCase 層**: `packages/server/src/usecase/error.rs`
   - ユースケース固有のエラー
   - トランザクション管理エラー

4. **UserInterface 層**: 各層のエラーを適切な HTTP ステータスコードやレスポンスに変換
   - `packages/server/src/ui/` 内でエラーハンドリング

**エラー変換**:

- 下位層（Domain）のエラーを上位層（UseCase, UserInterface）で変換
- UserInterface 層では、適切な HTTP ステータスコードに変換

## 今後の改善案

1. **データベースの導入**（必要に応じて）
   - `packages/server/src/infrastructure/repository/` に DB 版 Repository を追加
   - インメモリ実装と DB 実装を切り替え可能にする

2. **Infrastructure 層のエラー定義**
   - `packages/server/src/infrastructure/error.rs` を作成
   - データベース接続エラー、外部 API エラーを定義

3. **DTO の分離**（プロジェクトが大きくなった場合）
   - WebSocket 用と REST API 用で DTO を分離

4. **TUI クライアントの追加**
   - `packages/tui-client/` を追加
   - Ratatui を使用したターミナル UI クライアント

## モジュール命名規約

### 基本原則

- **モジュール名は単数形を使用します**。複数形は使用しません。
  - ✅ 良い例: `value_object.rs`, `model.rs`, `error.rs`
  - ❌ 悪い例: `value_objects.rs`, `models.rs`, `errors.rs`
- **モジュール分割は `<モジュール名>/mod.rs` のパターンを採用します**。
  - 例: `src/infrastructure/mod.rs`, `src/domain/mod.rs`

### 適用例

現在のプロジェクト構造:

```sh
packages/server/src/domain/
├── mod.rs
├── entity.rs          # ✅ 単数形（Entity）
├── value_object.rs    # ✅ 単数形（Value Object）
├── repository.rs      # ✅ 単数形（Repository）
├── message_pusher.rs  # ✅ 単数形（MessagePusher）
├── factory.rs         # ✅ 単数形（Factory）
└── error.rs           # ✅ 単数形（Error）

packages/server/src/infrastructure/
├── mod.rs
├── conversion.rs
├── dto/
│   ├── mod.rs
│   ├── websocket.rs   # WebSocket DTO
│   └── http.rs        # HTTP API DTO
├── repository/
│   ├── mod.rs
│   └── in_memory_room_repository.rs
└── message_pusher/
    ├── mod.rs
    └── websocket_message_pusher.rs
```

## ドメインモデリング（DDD）

このプロジェクトはドメイン駆動設計（DDD）の原則に従ってドメインモデルを設計しています。

### 基礎知識

DDD の基本的な概念と Value Object パターンについては、`docs/documentations/ddd.md` を参照してください。

### Value Objects

以下のプリミティブ型は Value Object として定義されています（`packages/server/src/domain/value_object.rs`）：

- **ClientId**: クライアント識別子（最大100文字、空文字列不可）
- **RoomId**: ルーム識別子（最大100文字、空文字列不可）
- **MessageContent**: メッセージ内容（最大10000文字、空文字列不可）
- **Timestamp**: Unix タイムスタンプ（ミリ秒）

Value Object の特徴：

- 不変（immutable）
- 値が変わると別のオブジェクトになる
- 同一性ではなく値で比較される（`PartialEq` を実装）
- バリデーションロジックをコンストラクタに含む
- `Display` trait を実装して文字列表現を提供

### Domain Entities

ドメインエンティティは `packages/server/src/domain/entity.rs` に定義されています：

- **Room**: チャットルーム（Entity）- RoomId、参加者リスト、メッセージ履歴、容量制限を保持
- **Participant**: 参加者（Entity）- ClientId、接続時刻を保持
- **ChatMessage**: チャットメッセージ（Entity）- 送信者 ClientId、メッセージ内容、タイムスタンプを保持

### DTO と Domain Entity の分離

- **DTO（Data Transfer Object）**: `packages/server/src/infrastructure/dto/` - WebSocket・HTTP API 通信用の型定義
- **Domain Entity**: `packages/server/src/domain/entity.rs` - ビジネスロジック用のエンティティ定義
- **Conversion**: `packages/server/src/infrastructure/conversion.rs` - DTO と Domain Entity の相互変換ロジック

DTO は外部とのインターフェースのみに使用し、内部のビジネスロジックでは Domain Entity を使用してください。

## 参考資料

- `docs/documentations/ddd.md` - DDD の基礎知識
- `docs/documentations/layered-architecture-project.md` - レイヤードアーキテクチャの詳細
- `AGENTS.md` - 開発全般のガイドライン
