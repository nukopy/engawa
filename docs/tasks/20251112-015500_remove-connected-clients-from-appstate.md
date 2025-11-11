# AppState から connected_clients を削除

**作成日**: 2025-11-12 01:55:00 JST
**ステータス**: 📝 **計画中**

## 概要

### 目的

- `AppState` から `connected_clients` を削除し、Repository 経由でのみアクセスするようにする
- UI 層が Repository の内部状態に直接アクセスしている問題を解決
- レイヤー境界を明確にする

### 背景

現状、`AppState` と `InMemoryRoomRepository` が同じ `connected_clients` の Arc を共有しており、以下の問題がある：

```rust
// src/ui/state.rs
pub struct AppState {
    pub repository: Arc<dyn RoomRepository>,
    pub connected_clients: Arc<Mutex<HashMap<String, ClientInfo>>>,  // ← 重複
}

// src/infrastructure/repository/inmemory/room.rs
pub struct InMemoryRoomRepository {
    connected_clients: Arc<Mutex<HashMap<String, ClientInfo>>>,  // ← 同じものを共有
    room: Arc<Mutex<Room>>,
}
```

UI 層（`src/ui/handler/websocket.rs`）が `state.connected_clients` に直接アクセスしている箇所が 4箇所ある。

### スコープ

- ✅ 今回やること:
  - `AppState` から `connected_clients` を削除
  - `RoomRepository` trait に必要なメソッドを追加
  - `websocket.rs` を Repository 経由のアクセスに変更

- ❌ 今回やらないこと:
  - `UnboundedSender<String>` を Domain 層から除去（次のフェーズ）
  - MessageBroker の抽象化（中期的改善）
  - Event 駆動アーキテクチャ（長期的改善）

### 参照

- `docs/tasks/20251112-005146_state-and-sender-architecture.md` - 設計上の課題の全体像

## 方針

### アプローチ

**段階的な改善（短期）**:

1. Repository に以下のメソッドを追加:

   ```rust
   async fn get_client_sender(&self, client_id: &str) -> Option<UnboundedSender<String>>;
   async fn get_all_client_senders(&self) -> HashMap<String, UnboundedSender<String>>;
   async fn get_client_connected_at(&self, client_id: &str) -> Option<i64>;
   ```

2. `AppState` から `connected_clients` を削除

3. `websocket.rs` の 4箇所を修正:
   - Line 98: `connected_at` 取得
   - Line 107: `participant-joined` ブロードキャスト
   - Line 186: メッセージブロードキャスト
   - Line 277: `participant-left` ブロードキャスト

### トレードオフ

**メリット**:

- ✅ UI 層が Repository の内部実装に依存しなくなる
- ✅ レイヤー境界が明確になる
- ✅ `AppState` がシンプルになる

**デメリット（一時的に許容）**:

- ⚠️ Repository に通信の実装詳細（`UnboundedSender`）が残る
- ⚠️ Repository が「データ永続化」と「メッセージ送信」の 2つの責務を持つ

→ これらは次のフェーズ（MessageBroker 抽象化）で解決する

### 品質基準

- `cargo fmt` が通る
- `cargo clippy --all-targets --all-features` が通る
- `cargo test` がすべて成功（80件）
- 特に統合テスト（11件）が失敗しない

## タスク

### Phase 1: Repository trait にメソッド追加

- [x] `src/domain/repository.rs` に以下を追加:
  - [x] `get_client_sender(&self, client_id: &str) -> Option<UnboundedSender<String>>`
  - [x] `get_all_client_senders(&self) -> HashMap<String, UnboundedSender<String>>`
  - [x] `get_client_connected_at(&self, client_id: &str) -> Option<i64>`

### Phase 2: InMemoryRoomRepository に実装追加

- [x] `src/infrastructure/repository/inmemory/room.rs` に実装を追加
  - [x] `get_client_sender` の実装
  - [x] `get_all_client_senders` の実装
  - [x] `get_client_connected_at` の実装

### Phase 3: AppState から connected_clients を削除

- [x] `src/ui/state.rs` の `AppState` を修正
- [x] `src/ui/server.rs` で `AppState` 初期化を修正
- [x] 未使用の import を削除（HashMap, Mutex）

### Phase 4: websocket.rs を Repository 経由に変更

- [x] Line 98: `get_client_connected_at` を使用
- [x] Line 107: `get_all_client_senders` を使用
- [x] Line 186: `get_all_client_senders` を使用
- [x] Line 277: `get_all_client_senders` を使用

### Phase 5: 検証

- [x] `cargo fmt` - 成功
- [x] `cargo clippy --all-targets --all-features` - 成功
- [x] `cargo test` - 全テスト成功（80件）
- [x] 統合テスト（11件）が失敗しないことを確認

## 進捗状況

- **開始日**: 2025-11-12 01:55:00 JST
- **完了日**: 2025-11-12 02:30:00 JST（推定）
- **ステータス**: ✅ **完了**
- **現在のフェーズ**: すべてのフェーズ完了
- **完了タスク数**: 13/13
- **実装時間**: 約 35分
- **最終結果**:
  - AppState から connected_clients を削除 ✅
  - Repository 経由でのみアクセスするように変更 ✅
  - 全テスト（80件）成功 ✅
  - Clippy 警告なし ✅

## 備考

### 設計の変遷

1. **現在**: AppState と Repository が `connected_clients` を共有（二重管理）
2. **このタスク後**: Repository のみが `connected_clients` を管理
3. **次のフェーズ**: MessageBroker を導入して通信を分離
4. **将来**: Event 駆動アーキテクチャで完全に分離

### 関連ファイル

- `src/ui/state.rs` - AppState の定義
- `src/domain/repository.rs` - Repository trait
- `src/infrastructure/repository/inmemory/room.rs` - InMemory 実装
- `src/ui/handler/websocket.rs` - connected_clients の使用箇所（4箇所）
- `src/bin/server.rs` - AppState の初期化

### 参考資料

- `docs/tasks/20251112-005146_state-and-sender-architecture.md` - 設計改善の全体像
