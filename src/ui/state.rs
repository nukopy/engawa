//! Server state and connection management.

use std::sync::Arc;

use crate::domain::{MessagePusher, RoomRepository};

/// Shared application state
pub struct AppState {
    /// Repository（データアクセス層の抽象化）
    pub repository: Arc<dyn RoomRepository>,
    /// MessagePusher（メッセージ通知の抽象化）
    pub message_pusher: Arc<dyn MessagePusher>,
}
