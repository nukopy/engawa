//! Domain logic for server-side operations.
//!
//! This module contains pure functions that implement business logic
//! without side effects, making them easy to test.

#![allow(dead_code)]

use std::collections::HashMap;

use crate::types::ParticipantInfo;

use super::state::ClientInfo;

/// Build a list of participants from connected clients.
///
/// # Arguments
///
/// * `connected_clients` - Map of client_id to their connection info
///
/// # Returns
///
/// A vector of participant information sorted by client_id
pub fn build_participant_list(
    connected_clients: &HashMap<String, ClientInfo>,
) -> Vec<ParticipantInfo> {
    let mut participants: Vec<ParticipantInfo> = connected_clients
        .iter()
        .map(|(client_id, client_info)| ParticipantInfo {
            client_id: client_id.clone(),
            connected_at: client_info.connected_at,
        })
        .collect();

    // Sort by client_id for consistent ordering
    participants.sort_by(|a, b| a.client_id.cmp(&b.client_id));

    participants
}

/// Check if a client_id is already connected.
///
/// # Arguments
///
/// * `connected_clients` - Map of client_id to their connection info
/// * `client_id` - The client ID to check
///
/// # Returns
///
/// `true` if the client_id already exists, `false` otherwise
pub fn is_duplicate_client(
    connected_clients: &HashMap<String, ClientInfo>,
    client_id: &str,
) -> bool {
    connected_clients.contains_key(client_id)
}

/// Get broadcast targets (all clients except the specified one).
///
/// # Arguments
///
/// * `connected_clients` - Map of client_id to their connection info
/// * `exclude_client_id` - The client ID to exclude from the result
///
/// # Returns
///
/// A vector of tuples containing (client_id, ClientInfo) for all clients
/// except the excluded one
pub fn get_broadcast_targets<'a>(
    connected_clients: &'a HashMap<String, ClientInfo>,
    exclude_client_id: &str,
) -> Vec<(&'a String, &'a ClientInfo)> {
    connected_clients
        .iter()
        .filter(|(client_id, _)| client_id.as_str() != exclude_client_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    fn create_test_client_info(connected_at: i64) -> ClientInfo {
        let (sender, _receiver) = mpsc::unbounded_channel();
        ClientInfo {
            sender,
            connected_at,
        }
    }

    #[test]
    fn test_build_participant_list_with_empty_clients() {
        // テスト項目: 接続クライアントが空の場合、空のリストが返される
        // given (前提条件):
        let clients = HashMap::new();

        // when (操作):
        let result = build_participant_list(&clients);

        // then (期待する結果):
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_build_participant_list_with_single_client() {
        // テスト項目: 単一クライアント接続時に正しい参加者リストが生成される
        // given (前提条件):
        let mut clients = HashMap::new();
        clients.insert("alice".to_string(), create_test_client_info(1000));

        // when (操作):
        let result = build_participant_list(&clients);

        // then (期待する結果):
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].client_id, "alice");
        assert_eq!(result[0].connected_at, 1000);
    }

    #[test]
    fn test_build_participant_list_with_multiple_clients() {
        // テスト項目: 複数クライアント接続時に正しい参加者リストが生成される
        // given (前提条件):
        let mut clients = HashMap::new();
        clients.insert("charlie".to_string(), create_test_client_info(3000));
        clients.insert("alice".to_string(), create_test_client_info(1000));
        clients.insert("bob".to_string(), create_test_client_info(2000));

        // when (操作):
        let result = build_participant_list(&clients);

        // then (期待する結果):
        assert_eq!(result.len(), 3);
        // Sorted by client_id
        assert_eq!(result[0].client_id, "alice");
        assert_eq!(result[0].connected_at, 1000);
        assert_eq!(result[1].client_id, "bob");
        assert_eq!(result[1].connected_at, 2000);
        assert_eq!(result[2].client_id, "charlie");
        assert_eq!(result[2].connected_at, 3000);
    }

    #[test]
    fn test_is_duplicate_client_with_empty_clients() {
        // テスト項目: 接続クライアントが空の場合、常に false が返される
        // given (前提条件):
        let clients = HashMap::new();

        // when (操作):
        let result = is_duplicate_client(&clients, "alice");

        // then (期待する結果):
        assert!(!result);
    }

    #[test]
    fn test_is_duplicate_client_with_existing_client() {
        // テスト項目: 既存のクライアント ID をチェックした場合、true が返される
        // given (前提条件):
        let mut clients = HashMap::new();
        clients.insert("alice".to_string(), create_test_client_info(1000));

        // when (操作):
        let result = is_duplicate_client(&clients, "alice");

        // then (期待する結果):
        assert!(result);
    }

    #[test]
    fn test_is_duplicate_client_with_non_existing_client() {
        // テスト項目: 存在しないクライアント ID をチェックした場合、false が返される
        // given (前提条件):
        let mut clients = HashMap::new();
        clients.insert("alice".to_string(), create_test_client_info(1000));

        // when (操作):
        let result = is_duplicate_client(&clients, "bob");

        // then (期待する結果):
        assert!(!result);
    }

    #[test]
    fn test_get_broadcast_targets_with_empty_clients() {
        // テスト項目: 接続クライアントが空の場合、空のリストが返される
        // given (前提条件):
        let clients = HashMap::new();

        // when (操作):
        let result = get_broadcast_targets(&clients, "alice");

        // then (期待する結果):
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_get_broadcast_targets_with_single_client() {
        // テスト項目: 単一クライアントを除外した場合、空のリストが返される
        // given (前提条件):
        let mut clients = HashMap::new();
        clients.insert("alice".to_string(), create_test_client_info(1000));

        // when (操作):
        let result = get_broadcast_targets(&clients, "alice");

        // then (期待する結果):
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_get_broadcast_targets_with_multiple_clients() {
        // テスト項目: 複数クライアント中から指定クライアントを除外したリストが返される
        // given (前提条件):
        let mut clients = HashMap::new();
        clients.insert("alice".to_string(), create_test_client_info(1000));
        clients.insert("bob".to_string(), create_test_client_info(2000));
        clients.insert("charlie".to_string(), create_test_client_info(3000));

        // when (操作):
        let result = get_broadcast_targets(&clients, "alice");

        // then (期待する結果):
        assert_eq!(result.len(), 2);
        let client_ids: Vec<&str> = result.iter().map(|(id, _)| id.as_str()).collect();
        assert!(client_ids.contains(&"bob"));
        assert!(client_ids.contains(&"charlie"));
        assert!(!client_ids.contains(&"alice"));
    }

    #[test]
    fn test_get_broadcast_targets_excluding_non_existing_client() {
        // テスト項目: 存在しないクライアントを除外指定しても全クライアントが返される
        // given (前提条件):
        let mut clients = HashMap::new();
        clients.insert("alice".to_string(), create_test_client_info(1000));
        clients.insert("bob".to_string(), create_test_client_info(2000));

        // when (操作):
        let result = get_broadcast_targets(&clients, "charlie");

        // then (期待する結果):
        assert_eq!(result.len(), 2);
        let client_ids: Vec<&str> = result.iter().map(|(id, _)| id.as_str()).collect();
        assert!(client_ids.contains(&"alice"));
        assert!(client_ids.contains(&"bob"));
    }
}
