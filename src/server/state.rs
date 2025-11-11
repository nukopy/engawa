//! Server state and connection management.

use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::{Mutex, mpsc};

use crate::domain::Room;

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct ConnectQuery {
    pub client_id: String,
}

/// Client connection information
pub struct ClientInfo {
    /// Message sender channel
    pub sender: mpsc::UnboundedSender<String>,
    /// Unix timestamp when connected (in JST, milliseconds)
    pub connected_at: i64,
}

/// Shared application state
pub struct AppState {
    /// Map of client_id to their connection info
    pub connected_clients: Mutex<HashMap<String, ClientInfo>>,
    /// Domain model: chat room with participants and message history
    pub room: Mutex<Room>,
}
