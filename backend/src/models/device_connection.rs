//! Device Connection Model
//!
//! Represents an active WebSocket connection for cross-device synchronization.
//! Tracks device metadata, connection lifecycle, and heartbeat status.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Device connection for tracking active WebSocket connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConnection {
    /// Unique identifier for this connection
    pub id: String,

    /// Device identifier (persistent across reconnections)
    pub device_id: Option<String>,

    /// User agent string for device identification
    pub user_agent: Option<String>,

    /// IP address of the connected device
    pub ip_address: Option<String>,

    /// Connection timestamp
    pub connected_at: u64,

    /// Last activity timestamp
    pub last_ping: u64,

    /// Connection status
    pub status: ConnectionStatus,

    /// Session identifier this device is part of
    pub session_id: Option<String>,

    /// Device metadata (JSON)
    pub metadata: serde_json::Value,
}

/// Connection status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Connection is active and healthy
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection is inactive (no recent heartbeat)
    Inactive,
    /// Connection is being closed
    Disconnecting,
    /// Connection was closed
    Disconnected,
}

impl DeviceConnection {
    /// Create a new device connection
    pub fn new(
        device_id: Option<String>,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: Uuid::new_v4().to_string(),
            device_id,
            user_agent,
            ip_address,
            connected_at: now,
            last_ping: now,
            status: ConnectionStatus::Connecting,
            session_id: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Update last ping timestamp
    pub fn update_ping(&mut self) {
        self.last_ping = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // If connection was inactive, mark as connected
        if self.status == ConnectionStatus::Inactive {
            self.status = ConnectionStatus::Connected;
        }
    }

    /// Mark connection as inactive
    pub fn mark_inactive(&mut self) {
        self.status = ConnectionStatus::Inactive;
    }

    /// Mark connection as disconnected
    pub fn mark_disconnected(&mut self) {
        self.status = ConnectionStatus::Disconnected;
    }

    /// Check if connection is healthy based on heartbeat
    pub fn is_healthy(&self, timeout_seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        matches!(self.status, ConnectionStatus::Connected) &&
        (now - self.last_ping) < timeout_seconds
    }

    /// Get connection age in seconds
    pub fn age(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now.saturating_sub(self.connected_at)
    }

    /// Get time since last ping in seconds
    pub fn time_since_last_ping(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now.saturating_sub(self.last_ping)
    }

    /// Set session identifier
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Update device metadata
    pub fn update_metadata(&mut self, metadata: serde_json::Value) {
        self.metadata = metadata;
    }

    /// Get device type from user agent
    pub fn device_type(&self) -> DeviceType {
        if let Some(user_agent) = &self.user_agent {
            if user_agent.contains("Mobile") || user_agent.contains("Android") || user_agent.contains("iPhone") {
                DeviceType::Mobile
            } else if user_agent.contains("Tablet") || user_agent.contains("iPad") {
                DeviceType::Tablet
            } else {
                DeviceType::Desktop
            }
        } else {
            DeviceType::Unknown
        }
    }

    /// Get browser from user agent
    pub fn browser(&self) -> Browser {
        if let Some(user_agent) = &self.user_agent {
            if user_agent.contains("Chrome") && !user_agent.contains("Edg") {
                Browser::Chrome
            } else if user_agent.contains("Firefox") {
                Browser::Firefox
            } else if user_agent.contains("Safari") && !user_agent.contains("Chrome") {
                Browser::Safari
            } else if user_agent.contains("Edg") {
                Browser::Edge
            } else {
                Browser::Unknown
            }
        } else {
            Browser::Unknown
        }
    }

    /// Check if this is a mobile device
    pub fn is_mobile(&self) -> bool {
        matches!(self.device_type(), DeviceType::Mobile)
    }

    /// Check if this is a tablet device
    pub fn is_tablet(&self) -> bool {
        matches!(self.device_type(), DeviceType::Tablet)
    }

    /// Check if this is a desktop device
    pub fn is_desktop(&self) -> bool {
        matches!(self.device_type(), DeviceType::Desktop)
    }
}

/// Device type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Unknown,
}

/// Browser enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Browser {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Unknown,
}

/// Connection pool for managing multiple device connections
#[derive(Debug)]
pub struct ConnectionPool {
    connections: std::collections::HashMap<String, DeviceConnection>,
    device_index: std::collections::HashMap<String, Vec<String>>, // device_id -> connection_ids
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new() -> Self {
        Self {
            connections: std::collections::HashMap::new(),
            device_index: std::collections::HashMap::new(),
        }
    }

    /// Add a new connection to the pool
    pub fn add_connection(&mut self, connection: DeviceConnection) {
        let connection_id = connection.id.clone();
        let device_id = connection.device_id.clone();

        self.connections.insert(connection_id.clone(), connection);

        // Update device index
        if let Some(device_id) = device_id {
            self.device_index
                .entry(device_id)
                .or_insert_with(Vec::new)
                .push(connection_id);
        }
    }

    /// Remove a connection from the pool
    pub fn remove_connection(&mut self, connection_id: &str) -> Option<DeviceConnection> {
        let connection = self.connections.remove(connection_id);

        if let Some(ref conn) = connection {
            if let Some(device_id) = &conn.device_id {
                if let Some(connections) = self.device_index.get_mut(device_id) {
                    connections.retain(|id| id != connection_id);
                    if connections.is_empty() {
                        self.device_index.remove(device_id);
                    }
                }
            }
        }

        connection
    }

    /// Get a connection by ID
    pub fn get_connection(&self, connection_id: &str) -> Option<&DeviceConnection> {
        self.connections.get(connection_id)
    }

    /// Get all connections for a device
    pub fn get_device_connections(&self, device_id: &str) -> Vec<&DeviceConnection> {
        self.device_index
            .get(device_id)
            .map(|connection_ids| {
                connection_ids
                    .iter()
                    .filter_map(|id| self.connections.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all active connections
    pub fn get_active_connections(&self, timeout_seconds: u64) -> Vec<&DeviceConnection> {
        self.connections
            .values()
            .filter(|conn| conn.is_healthy(timeout_seconds))
            .collect()
    }

    /// Get all connections
    pub fn get_all_connections(&self) -> Vec<&DeviceConnection> {
        self.connections.values().collect()
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Get active connection count
    pub fn active_connection_count(&self, timeout_seconds: u64) -> usize {
        self.connections
            .values()
            .filter(|conn| conn.is_healthy(timeout_seconds))
            .count()
    }

    /// Update connection ping
    pub fn update_ping(&mut self, connection_id: &str) -> bool {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.update_ping();
            true
        } else {
            false
        }
    }

    /// Mark inactive connections
    pub fn mark_inactive_connections(&mut self, timeout_seconds: u64) -> Vec<String> {
        let mut inactive_ids = Vec::new();
        let _now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for (id, connection) in &mut self.connections {
            if !connection.is_healthy(timeout_seconds) {
                connection.mark_inactive();
                inactive_ids.push(id.clone());
            }
        }

        inactive_ids
    }

    /// Remove inactive connections
    pub fn cleanup_inactive_connections(&mut self, timeout_seconds: u64) -> Vec<String> {
        let inactive_ids = self.mark_inactive_connections(timeout_seconds);

        for id in &inactive_ids {
            self.remove_connection(id);
        }

        inactive_ids
    }

    /// Get connection statistics
    pub fn get_stats(&self, timeout_seconds: u64) -> ConnectionStats {
        let total = self.connection_count();
        let active = self.active_connection_count(timeout_seconds);
        let desktop = self.get_all_connections()
            .iter()
            .filter(|conn| conn.is_desktop())
            .count();
        let mobile = self.get_all_connections()
            .iter()
            .filter(|conn| conn.is_mobile())
            .count();
        let tablet = self.get_all_connections()
            .iter()
            .filter(|conn| conn.is_tablet())
            .count();

        ConnectionStats {
            total,
            active,
            desktop,
            mobile,
            tablet,
            inactive: total - active,
        }
    }
}

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub total: usize,
    pub active: usize,
    pub desktop: usize,
    pub mobile: usize,
    pub tablet: usize,
    pub inactive: usize,
}

/// Device connection errors
#[derive(Debug, thiserror::Error)]
pub enum DeviceConnectionError {
    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("Device already has maximum connections: {0}")]
    MaxConnectionsExceeded(String),

    #[error("Invalid connection state")]
    InvalidConnectionState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_connection_creation() {
        let connection = DeviceConnection::new(
            Some("device-123".to_string()),
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/91.0".to_string()),
            Some("192.168.1.100".to_string()),
        );

        assert_eq!(connection.device_id, Some("device-123".to_string()));
        assert_eq!(connection.status, ConnectionStatus::Connecting);
        assert!(connection.is_healthy(60)); // Should be healthy initially
    }

    #[test]
    fn test_device_type_detection() {
        let mobile_conn = DeviceConnection::new(
            None,
            Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_6)".to_string()),
            None,
        );
        assert_eq!(mobile_conn.device_type(), DeviceType::Mobile);
        assert!(mobile_conn.is_mobile());

        let desktop_conn = DeviceConnection::new(
            None,
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/91.0".to_string()),
            None,
        );
        assert_eq!(desktop_conn.device_type(), DeviceType::Desktop);
        assert!(desktop_conn.is_desktop());
    }

    #[test]
    fn test_browser_detection() {
        let chrome_conn = DeviceConnection::new(
            None,
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/91.0".to_string()),
            None,
        );
        assert_eq!(chrome_conn.browser(), Browser::Chrome);

        let firefox_conn = DeviceConnection::new(
            None,
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Firefox/89.0".to_string()),
            None,
        );
        assert_eq!(firefox_conn.browser(), Browser::Firefox);
    }

    #[test]
    fn test_connection_pool() {
        let mut pool = ConnectionPool::new();

        let conn1 = DeviceConnection::new(None, None, None);
        let conn2 = DeviceConnection::new(None, None, None);

        let id1 = conn1.id.clone();
        let id2 = conn2.id.clone();

        pool.add_connection(conn1);
        pool.add_connection(conn2);

        assert_eq!(pool.connection_count(), 2);
        assert!(pool.get_connection(&id1).is_some());
        assert!(pool.get_connection(&id2).is_some());

        pool.remove_connection(&id1);
        assert_eq!(pool.connection_count(), 1);
        assert!(pool.get_connection(&id1).is_none());
        assert!(pool.get_connection(&id2).is_some());
    }

    #[test]
    fn test_device_indexing() {
        let mut pool = ConnectionPool::new();

        let device_id = "device-123".to_string();
        let conn1 = DeviceConnection::new(Some(device_id.clone()), None, None);
        let conn2 = DeviceConnection::new(Some(device_id.clone()), None, None);

        pool.add_connection(conn1);
        pool.add_connection(conn2);

        let device_conns = pool.get_device_connections(&device_id);
        assert_eq!(device_conns.len(), 2);
    }

    #[test]
    fn test_connection_health() {
        let mut connection = DeviceConnection::new(None, None, None);
        assert!(connection.is_healthy(60));

        // Simulate old ping
        connection.last_ping = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - 120; // 2 minutes ago

        assert!(!connection.is_healthy(60)); // Should be unhealthy with 60s timeout
        assert!(connection.is_healthy(180)); // Should be healthy with 180s timeout
    }
}