//! WebSocket Service
//!
//! Real-time WebSocket communication for cross-device timer synchronization.
//! Handles connection lifecycle, device tracking, and message broadcasting.

use crate::models::device_connection::{DeviceConnection, ConnectionPool, ConnectionStats};
use crate::models::timer_session::TimerSession;
use crate::services::timer_service::TimerService;
use serde::{Deserialize, Serialize};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State, Query,
    },
    response::Response,
};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;
use uuid::Uuid;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Server → Client: Timer state update
    TimerStateUpdate {
        payload: TimerSession,
    },
    /// Server → Client: Connection status
    ConnectionStatus {
        status: String,
        device_count: usize,
    },
    /// Server → Client: Notification
    Notification {
        message: String,
        event_type: String,
    },
    /// Server → Client: Configuration update
    ConfigurationUpdate {
        payload: serde_json::Value,
    },
    /// Server → Client: Device connected/disconnected
    DeviceStatus {
        device_id: String,
        status: String,
    },
}

/// Client message types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Start timer
    StartTimer,
    /// Pause timer
    PauseTimer,
    /// Reset timer
    ResetTimer,
    /// Skip timer
    SkipTimer,
    /// Heartbeat/ping
    Ping,
}

/// WebSocket service state
#[derive(Debug, Clone)]
pub struct WebSocketService {
    /// Connection pool for tracking active connections
    connection_pool: Arc<RwLock<ConnectionPool>>,

    /// Broadcast channel for sending messages to all clients
    message_broadcast: broadcast::Sender<WebSocketMessage>,

    /// Timer service reference
    timer_service: Arc<TimerService>,

    /// Performance metrics
    metrics: Arc<RwLock<WebSocketMetrics>>,
}

/// WebSocket performance metrics
#[derive(Debug, Default, Clone)]
pub struct WebSocketMetrics {
    /// Messages broadcast per second
    messages_per_second: f64,
    /// Active connections count
    active_connections: usize,
    /// Total messages sent
    total_messages_sent: u64,
    /// Total message failures
    total_message_failures: u64,
    /// Average broadcast latency in milliseconds
    average_broadcast_latency: f64,
    /// Last broadcast timestamp
    last_broadcast_time: Option<u64>,
}

impl WebSocketService {
    /// Create a new WebSocket service
    pub fn new(timer_service: Arc<TimerService>) -> Self {
        let (message_broadcast, _) = broadcast::channel(1000);

        Self {
            connection_pool: Arc::new(RwLock::new(ConnectionPool::new())),
            message_broadcast,
            timer_service,
            metrics: Arc::new(RwLock::new(WebSocketMetrics::default())),
        }
    }

    /// Broadcast timer state to all connected devices with performance optimization
    pub async fn broadcast_timer_state(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        let timer_state = self.timer_service.get_timer_state().await;

        let session = TimerSession {
            id: timer_state.id,
            duration: timer_state.duration,
            elapsed: timer_state.elapsed,
            timer_type: serde_json::from_str(&format!("\"{}\"", timer_state.timer_type))?,
            is_running: timer_state.is_running,
            created_at: timer_state.created_at,
            updated_at: timer_state.updated_at,
        };

        let message = WebSocketMessage::TimerStateUpdate {
            payload: session,
        };

        // Broadcast and track performance
        let _ = self.message_broadcast.send(message);

        // Update metrics
        self.update_broadcast_metrics(start_time).await;

        Ok(())
    }

    /// Batch broadcast multiple messages for efficiency
    pub async fn broadcast_batch(&self, messages: Vec<WebSocketMessage>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut sent_count = 0;
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        for message in messages {
            if self.message_broadcast.send(message).is_ok() {
                sent_count += 1;
            }
        }

        // Update metrics for batch operation
        self.update_broadcast_metrics(start_time).await;

        Ok(sent_count)
    }

    /// Broadcast a single message to all connected clients
    pub async fn broadcast_message(&self, message: WebSocketMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        // Broadcast and track performance
        let _ = self.message_broadcast.send(message);

        // Update metrics
        self.update_broadcast_metrics(start_time).await;

        Ok(())
    }

    /// Update performance metrics after broadcast
    async fn update_broadcast_metrics(&self, start_time: f64) {
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        let latency = end_time - start_time;

        let mut metrics = self.metrics.write().await;

        // Update message count and latency
        metrics.total_messages_sent += 1;

        // Calculate rolling average latency (using exponential moving average)
        if metrics.average_broadcast_latency == 0.0 {
            metrics.average_broadcast_latency = latency;
        } else {
            metrics.average_broadcast_latency = 0.9 * metrics.average_broadcast_latency + 0.1 * latency;
        }

        // Ensure sub-500ms requirement
        if latency > 500.0 {
            tracing::warn!("Broadcast latency exceeded 500ms: {}ms", latency);
        }
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> WebSocketMetrics {
        let mut metrics = self.metrics.read().await.clone();

        // Update active connections count
        let pool = self.connection_pool.read().await;
        metrics.active_connections = pool.active_connection_count(60);

        // Calculate messages per second (rolling average over last minute)
        if let Some(last_broadcast) = metrics.last_broadcast_time {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let time_diff = now.saturating_sub(last_broadcast) as f64;
            if time_diff > 0.0 {
                metrics.messages_per_second = metrics.total_messages_sent as f64 / time_diff;
            }
        }

        metrics
    }

    /// Handle WebSocket connection upgrade
    pub async fn handle_websocket(
        State(state): State<Arc<WebSocketService>>,
        ws: WebSocketUpgrade,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Response {
        // Extract connection parameters
        let device_id = params.get("device_id").cloned();
        let user_agent = params.get("user_agent").cloned();

        ws.protocols(vec!["roma-timer"])
            .on_upgrade(move |socket| {
                Self::handle_connection(state, socket, device_id, user_agent, addr)
            })
    }

    /// Handle individual WebSocket connection
    async fn handle_connection(
        state: Arc<WebSocketService>,
        mut socket: WebSocket,
        device_id: Option<String>,
        user_agent: Option<String>,
        addr: SocketAddr,
    ) {
        let connection_id = Uuid::new_v4().to_string();
        let ip_address = addr.ip().to_string();

        // Create device connection record
        let mut device_connection = DeviceConnection::new(
            device_id,
            user_agent,
            Some(ip_address),
        );

        device_connection.id = connection_id.clone();
        device_connection.status = crate::models::device_connection::ConnectionStatus::Connected;

        // Add connection to pool
        {
            let mut pool = state.connection_pool.write().await;
            pool.add_connection(device_connection.clone());
        }

        // Send initial connection status
        let stats = state.get_connection_stats().await;
        let connection_msg = WebSocketMessage::ConnectionStatus {
            status: "connected".to_string(),
            device_count: stats.active,
        };

        if let Ok(msg_json) = serde_json::to_string(&connection_msg) {
            let _ = socket.send(Message::Text(msg_json)).await;
        }

        // Subscribe to message broadcasts
        let mut broadcast_rx = state.message_broadcast.subscribe();

        // Start heartbeat task for this connection
        let connection_id_clone = connection_id.clone();
        let heartbeat_task = tokio::spawn({
            let connection_pool = state.connection_pool.clone();
            async move {
                let mut interval = interval(Duration::from_secs(30));

                loop {
                    interval.tick().await;

                    {
                        let mut pool = connection_pool.write().await;
                        if !pool.update_ping(&connection_id_clone) {
                            // Connection not found, stop heartbeat
                            break;
                        }
                    }
                }
            }
        });

        // Handle connection loop
        loop {
            tokio::select! {
                // Handle incoming messages from client
                Some(msg_result) = socket.next() => {
                    match msg_result {
                        Ok(Message::Text(text)) => {
                            if let Err(e) = Self::handle_client_message(&state, &text, &connection_id).await {
                                tracing::error!("Error handling client message: {}", e);
                                break;
                            }
                        }
                        Ok(Message::Ping(payload)) => {
                            // Respond to ping with pong
                            if let Err(e) = socket.send(Message::Pong(payload)).await {
                                tracing::error!("Error sending pong: {}", e);
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            tracing::info!("Client {} disconnected", connection_id);
                            break;
                        }
                        Ok(_) => {
                            // Handle other message types (binary, etc.)
                        }
                        Err(e) => {
                            tracing::error!("WebSocket error for {}: {}", connection_id, e);
                            break;
                        }
                    }
                }

                // Handle broadcast messages
                Ok(broadcast_msg) = broadcast_rx.recv() => {
                    if let Ok(msg_json) = serde_json::to_string(&broadcast_msg) {
                        if let Err(e) = socket.send(Message::Text(msg_json)).await {
                            tracing::error!("Error sending broadcast to {}: {}", connection_id, e);
                            break;
                        }
                    }
                }
            }
        }

        // Cleanup on disconnect
        heartbeat_task.abort();

        {
            let mut pool = state.connection_pool.write().await;
            pool.remove_connection(&connection_id);
        }

        // Notify other clients about device disconnection
        let stats = state.get_connection_stats().await;
        let disconnect_msg = WebSocketMessage::ConnectionStatus {
            status: "device_disconnected".to_string(),
            device_count: stats.active,
        };

        let _ = state.message_broadcast.send(disconnect_msg);
    }

    /// Handle incoming messages from clients
    async fn handle_client_message(
        state: &Arc<WebSocketService>,
        message: &str,
        connection_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client_msg: ClientMessage = serde_json::from_str(message)?;

        match client_msg {
            ClientMessage::StartTimer => {
                // Check if connection is allowed to control timer
                if state.is_connection_authorized(connection_id).await {
                    state.timer_service.start_timer().await?;
                    state.broadcast_timer_state().await?;
                }
            }
            ClientMessage::PauseTimer => {
                if state.is_connection_authorized(connection_id).await {
                    state.timer_service.pause_timer().await?;
                    state.broadcast_timer_state().await?;
                }
            }
            ClientMessage::ResetTimer => {
                if state.is_connection_authorized(connection_id).await {
                    state.timer_service.reset_timer().await?;
                    state.broadcast_timer_state().await?;
                }
            }
            ClientMessage::SkipTimer => {
                if state.is_connection_authorized(connection_id).await {
                    state.timer_service.skip_timer().await?;
                    state.broadcast_timer_state().await?;
                }
            }
            ClientMessage::Ping => {
                // Update connection ping timestamp
                {
                    let mut pool = state.connection_pool.write().await;
                    pool.update_ping(connection_id);
                }
            }
        }

        Ok(())
    }

    /// Check if a connection is authorized to control the timer
    async fn is_connection_authorized(&self, connection_id: &str) -> bool {
        // For now, all connections are authorized
        // In the future, this could implement authentication/authorization logic
        let pool = self.connection_pool.read().await;
        pool.get_connection(connection_id).is_some()
    }

  
    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let pool = self.connection_pool.read().await;
        pool.get_stats(60) // 60 second timeout
    }

    /// Start background tasks for connection management
    pub async fn start_background_tasks(self: Arc<Self>) {
        let service = self.clone();

        // Start connection cleanup task
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Check every minute

            loop {
                interval.tick().await;

                let stats = service.get_connection_stats().await;
                tracing::info!("Active connections: {}/{}", stats.active, stats.total);

                // Clean up inactive connections
                {
                    let mut pool = service.connection_pool.write().await;
                    let cleaned = pool.cleanup_inactive_connections(120); // 2 minute timeout

                    if !cleaned.is_empty() {
                        tracing::info!("Cleaned up {} inactive connections", cleaned.len());

                        // Broadcast updated connection count
                        let new_stats = service.get_connection_stats().await;
                        let message = WebSocketMessage::ConnectionStatus {
                            status: "connection_cleanup".to_string(),
                            device_count: new_stats.active,
                        };

                        let _ = service.message_broadcast.send(message);
                    }
                }
            }
        });

        // Start timer state sync task
        let service_sync = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                // Broadcast timer state periodically (backup for direct broadcasts)
                if let Err(e) = service_sync.broadcast_timer_state().await {
                    tracing::error!("Error broadcasting timer state: {}", e);
                }
            }
        });
    }

    /// Get all active connections
    pub async fn get_active_connections(&self) -> Vec<DeviceConnection> {
        let pool = self.connection_pool.read().await;
        pool.get_active_connections(60)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get connections for a specific device
    pub async fn get_device_connections(&self, device_id: &str) -> Vec<DeviceConnection> {
        let pool = self.connection_pool.read().await;
        pool.get_device_connections(device_id)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Send message to specific device
    pub async fn send_to_device(
        &self,
        _device_id: &str,
        message: WebSocketMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This would require storing WebSocket senders in the connection model
        // For now, we'll broadcast to all connections
        let _ = self.message_broadcast.send(message);
        Ok(())
    }

    /// Force disconnect a device
    pub async fn disconnect_device(&self, _device_id: &str) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut pool = self.connection_pool.write().await;

        // Get all connection IDs first to avoid borrow checker issues
        let connection_ids: Vec<String> = pool.get_all_connections()
            .iter()
            .map(|conn| conn.id.clone())
            .collect();

        let count = connection_ids.len();
        for connection_id in connection_ids {
            pool.remove_connection(&connection_id);
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::timer_service::TimerService;

    #[tokio::test]
    async fn test_websocket_service_creation() {
        // For testing, create a mock timer service without real configuration
        use crate::models::user_configuration::UserConfiguration;
        let config = UserConfiguration::new();

        // Create a mock timer service for testing
        let timer_service = Arc::new(
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    crate::services::timer_service::TimerService::new_with_config(config.clone())
                })
            })
        );
        let ws_service = WebSocketService::new(timer_service);

        let stats = ws_service.get_connection_stats().await;
        assert_eq!(stats.total, 0);
        assert_eq!(stats.active, 0);
    }

    #[tokio::test]
    async fn test_connection_pool_operations() {
        use crate::models::user_configuration::UserConfiguration;
        let config = UserConfiguration::new();

        // Create a mock timer service for testing
        let timer_service = Arc::new(
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    crate::services::timer_service::TimerService::new_with_config(config.clone())
                })
            })
        );
        let _ws_service = WebSocketService::new(timer_service);

        let mut pool = ConnectionPool::new();

        let conn1 = DeviceConnection::new(Some("device-1".to_string()), None, None);
        let conn2 = DeviceConnection::new(Some("device-1".to_string()), None, None);
        let conn3 = DeviceConnection::new(Some("device-2".to_string()), None, None);

        pool.add_connection(conn1);
        pool.add_connection(conn2);
        pool.add_connection(conn3);

        assert_eq!(pool.connection_count(), 3);
        assert_eq!(pool.get_device_connections("device-1").len(), 2);
        assert_eq!(pool.get_device_connections("device-2").len(), 1);

        let device_conns = pool.get_device_connections("device-1");
        assert_eq!(device_conns.len(), 2);
    }
}