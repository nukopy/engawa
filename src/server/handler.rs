//! WebSocket connection handlers.

use std::sync::Arc;

use axum::{
    Json,
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;

use crate::{
    domain::{ClientId, Room, Timestamp},
    infrastructure::dto::{
        http::{ParticipantDetailDto, RoomDetailDto, RoomSummaryDto},
        websocket::{
            ChatMessage, MessageType, ParticipantInfo, ParticipantJoinedMessage,
            ParticipantLeftMessage, RoomConnectedMessage,
        },
    },
    time::{get_jst_timestamp, timestamp_to_jst_rfc3339},
};

use super::state::{AppState, ClientInfo, ConnectQuery};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ConnectQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let client_id = query.client_id;

    // Create a channel for this client to receive messages
    let (tx, rx) = mpsc::unbounded_channel();

    // Get current timestamp in JST
    let connected_at = get_jst_timestamp();

    // Check if client_id is already connected and register the new client
    {
        let mut clients = state.connected_clients.lock().await;
        if clients.contains_key(&client_id) {
            tracing::warn!(
                "Client with ID '{}' is already connected. Rejecting connection.",
                client_id
            );
            return Err(StatusCode::CONFLICT);
        }
        // Register the client_id with its connection info
        let client_info = ClientInfo {
            sender: tx,
            connected_at,
        };
        clients.insert(client_id.clone(), client_info);
    }

    // Add participant to domain model
    {
        let mut room = state.room.lock().await;
        if let Err(e) = room.add_participant(crate::domain::Participant::new(
            ClientId::new(client_id.clone()).expect("ClientId should be valid"),
            Timestamp::new(connected_at),
        )) {
            tracing::warn!("Failed to add participant '{}' to room: {}", client_id, e);
            // Remove from connected clients since we couldn't add to domain model
            let mut clients = state.connected_clients.lock().await;
            clients.remove(&client_id);
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    tracing::info!("Client '{}' connected and registered", client_id);

    Ok(ws.on_upgrade(|socket| handle_socket(socket, state, client_id, rx)))
}

pub async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    client_id: String,
    mut rx: mpsc::UnboundedReceiver<String>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Send current room participants to the newly connected client
    let connected_at = {
        let clients = state.connected_clients.lock().await;
        let participants: Vec<ParticipantInfo> = clients
            .iter()
            .map(|(id, info)| ParticipantInfo {
                client_id: id.clone(),
                connected_at: info.connected_at,
            })
            .collect();

        let room_msg = RoomConnectedMessage {
            r#type: MessageType::RoomConnected,
            participants,
        };

        let room_json = serde_json::to_string(&room_msg).unwrap();
        if let Err(e) = sender.send(Message::Text(room_json.into())).await {
            tracing::error!("Failed to send room connected to '{}': {}", client_id, e);
            return;
        }
        tracing::info!("Sent room connected list to '{}'", client_id);

        // Get this client's connected_at timestamp for broadcasting
        clients
            .get(&client_id)
            .map(|info| info.connected_at)
            .unwrap()
    };

    // Broadcast participant-joined to all other clients
    {
        let clients = state.connected_clients.lock().await;
        let joined_msg = ParticipantJoinedMessage {
            r#type: MessageType::ParticipantJoined,
            client_id: client_id.clone(),
            connected_at,
        };

        let joined_json = serde_json::to_string(&joined_msg).unwrap();
        for (id, client_info) in clients.iter() {
            if id != &client_id {
                // Send to other clients only
                if client_info.sender.send(joined_json.clone()).is_err() {
                    tracing::warn!("Failed to send participant-joined to client '{}'", id);
                }
            }
        }
        tracing::info!("Broadcasted participant-joined for '{}'", client_id);
    }

    let client_id_clone = client_id.clone();
    let state_clone = state.clone();

    // Spawn a task to receive messages from this client
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            };

            match msg {
                Message::Text(text) => {
                    tracing::info!("Received text: {}", text);

                    // Parse the incoming message
                    let chat_msg = match serde_json::from_str::<ChatMessage>(&text) {
                        Ok(msg) => msg,
                        Err(e) => {
                            tracing::warn!("Failed to parse message as JSON: {}", e);
                            // If not JSON, treat as plain text and wrap it
                            ChatMessage {
                                r#type: MessageType::Chat,
                                client_id: "unknown".to_string(),
                                content: text.to_string(),
                                timestamp: 0,
                            }
                        }
                    };

                    // Create response with type "chat" and preserve client_id
                    let response = ChatMessage {
                        r#type: MessageType::Chat,
                        client_id: chat_msg.client_id.clone(),
                        content: chat_msg.content.clone(),
                        timestamp: chat_msg.timestamp,
                    };

                    let response_json = serde_json::to_string(&response).unwrap();
                    tracing::info!(
                        "Broadcasting message from '{}' to other clients: {}",
                        response.client_id,
                        response.content
                    );

                    // Add message to domain model
                    {
                        let mut room = state_clone.room.lock().await;
                        if let Err(e) = room.add_message(response.clone().into()) {
                            tracing::warn!("Failed to add message to room history: {}", e);
                        }
                    }

                    // Send to all connected clients EXCEPT the sender
                    let clients = state_clone.connected_clients.lock().await;
                    for (id, client_info) in clients.iter() {
                        if id != &client_id_clone {
                            // Send to other clients only
                            if client_info.sender.send(response_json.clone()).is_err() {
                                tracing::warn!("Failed to send message to client '{}'", id);
                            }
                        }
                    }
                }
                Message::Ping(_) => {
                    tracing::debug!("Received ping");
                    // Ping/pong is handled automatically by the WebSocket protocol
                }
                Message::Close(_) => {
                    tracing::info!("Client '{}' requested close", client_id_clone);
                    break;
                }
                _ => {}
            }
        }
    });

    // Spawn a task to receive messages from other clients and send to this client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // Send the message to this client
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // If any one of the tasks completes, abort the other
    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    };

    // Remove client_id from connected clients and broadcast participant-left
    {
        let mut clients = state.connected_clients.lock().await;
        clients.remove(&client_id);
        tracing::info!(
            "Client '{}' disconnected and removed from registry",
            client_id
        );

        // Broadcast participant-left to all remaining clients
        let disconnected_at = get_jst_timestamp();
        let left_msg = ParticipantLeftMessage {
            r#type: MessageType::ParticipantLeft,
            client_id: client_id.clone(),
            disconnected_at,
        };

        let left_json = serde_json::to_string(&left_msg).unwrap();
        for (id, client_info) in clients.iter() {
            if client_info.sender.send(left_json.clone()).is_err() {
                tracing::warn!("Failed to send participant-left to client '{}'", id);
            }
        }
        tracing::info!("Broadcasted participant-left for '{}'", client_id);
    }

    // Remove participant from domain model
    {
        let mut room = state.room.lock().await;
        let client_id_vo = ClientId::new(client_id.clone()).expect("ClientId should be valid");
        room.remove_participant(&client_id_vo);
    }
}

/// Debug endpoint to get current room state (for testing purposes)
pub async fn debug_room_state(State(state): State<Arc<AppState>>) -> Json<Room> {
    let room = state.room.lock().await;
    Json(room.clone())
}

/// Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

/// Get list of rooms
pub async fn get_rooms(State(state): State<Arc<AppState>>) -> Json<Vec<RoomSummaryDto>> {
    let room = state.room.lock().await;

    let room_summary = RoomSummaryDto {
        id: room.id.as_str().to_string(),
        participants: room
            .participants
            .iter()
            .map(|p| p.id.as_str().to_string())
            .collect(),
        created_at: timestamp_to_jst_rfc3339(room.created_at.value()),
    };

    Json(vec![room_summary])
}

/// Get room detail by ID
pub async fn get_room_detail(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<RoomDetailDto>, StatusCode> {
    let room = state.room.lock().await;

    // Check if the requested room_id matches
    if room.id.as_str() != room_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let room_detail = RoomDetailDto {
        id: room.id.as_str().to_string(),
        participants: room
            .participants
            .iter()
            .map(|p| ParticipantDetailDto {
                client_id: p.id.as_str().to_string(),
                connected_at: timestamp_to_jst_rfc3339(p.connected_at.value()),
            })
            .collect(),
        created_at: timestamp_to_jst_rfc3339(room.created_at.value()),
    };

    Ok(Json(room_detail))
}
