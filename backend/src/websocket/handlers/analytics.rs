//! Analytics WebSocket Message Handlers
//!
//! Handles WebSocket messages for daily session statistics and analytics.

use crate::models::daily_session_stats::DailySessionStats;
use crate::models::session_reset_event::SessionResetEvent;
use crate::database::DatabaseManager;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{Utc, Datelike};
use tracing::{info, warn, error, instrument};

/// Get Daily Stats Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDailyStatsRequest {
    /// User configuration ID
    pub user_id: String,
    /// Date to get stats for (YYYY-MM-DD format), if None uses today
    pub date: Option<String>,
    /// Number of days to include (for range queries)
    pub days: Option<u32>,
}

/// Get Daily Stats Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDailyStatsResponse {
    /// Success status
    pub success: bool,
    /// Daily statistics
    pub stats: Vec<DailySessionStats>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Get Reset Events Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResetEventsRequest {
    /// User configuration ID
    pub user_id: String,
    /// Start date (YYYY-MM-DD format)
    pub start_date: Option<String>,
    /// End date (YYYY-MM-DD format)
    pub end_date: Option<String>,
    /// Maximum number of events to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

/// Get Reset Events Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResetEventsResponse {
    /// Success status
    pub success: bool,
    /// Reset events
    pub events: Vec<SessionResetEvent>,
    /// Total count (for pagination)
    pub total_count: Option<u32>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Get Session Summary Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionSummaryRequest {
    /// User configuration ID
    pub user_id: String,
    /// Period type: "week", "month", "year"
    pub period: String,
    /// Number of periods to include
    pub count: Option<u32>,
}

/// Session Summary Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryData {
    /// Period label (e.g., "2024-01-15 to 2024-01-21")
    pub period_label: String,
    /// Total work sessions
    pub total_work_sessions: u32,
    /// Total work minutes
    pub total_work_minutes: u32,
    /// Average sessions per day
    pub avg_sessions_per_day: f64,
    /// Productivity score (0-100)
    pub productivity_score: u32,
    /// Number of manual overrides
    pub manual_overrides: u32,
}

/// Get Session Summary Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionSummaryResponse {
    /// Success status
    pub success: bool,
    /// Session summary data
    pub summary: Vec<SessionSummaryData>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Analytics WebSocket Handler
pub struct AnalyticsWebSocketHandler {
    database_manager: Arc<DatabaseManager>,
}

impl AnalyticsWebSocketHandler {
    /// Create a new analytics WebSocket handler
    pub fn new(database_manager: Arc<DatabaseManager>) -> Self {
        Self { database_manager }
    }

    /// Handle get daily stats message
    #[instrument(skip(self))]
    pub async fn handle_get_daily_stats(
        &self,
        request: GetDailyStatsRequest,
    ) -> GetDailyStatsResponse {
        info!("Handling get daily stats request for user {}", request.user_id);

        // Determine target date
        let target_date = request.date.unwrap_or_else(|| {
            Utc::now().format("%Y-%m-%d").to_string()
        });

        // Validate date format
        if !self.is_valid_date_format(&target_date) {
            return GetDailyStatsResponse {
                success: false,
                stats: vec![],
                error: Some("Invalid date format. Use YYYY-MM-DD".to_string()),
            };
        }

        // Load daily stats from database
        let stats = match self.load_daily_session_stats(&request.user_id, &target_date, request.days).await {
            Ok(stats) => stats,
            Err(e) => {
                return GetDailyStatsResponse {
                    success: false,
                    stats: vec![],
                    error: Some(format!("Failed to load daily stats: {}", e)),
                };
            }
        };

        GetDailyStatsResponse {
            success: true,
            stats,
            error: None,
        }
    }

    /// Handle get reset events message
    #[instrument(skip(self))]
    pub async fn handle_get_reset_events(
        &self,
        request: GetResetEventsRequest,
    ) -> GetResetEventsResponse {
        info!("Handling get reset events request for user {}", request.user_id);

        // Validate date formats if provided
        if let Some(ref start_date) = request.start_date {
            if !self.is_valid_date_format(start_date) {
                return GetResetEventsResponse {
                    success: false,
                    events: vec![],
                    total_count: None,
                    error: Some("Invalid start_date format. Use YYYY-MM-DD".to_string()),
                };
            }
        }

        if let Some(ref end_date) = request.end_date {
            if !self.is_valid_date_format(end_date) {
                return GetResetEventsResponse {
                    success: false,
                    events: vec![],
                    total_count: None,
                    error: Some("Invalid end_date format. Use YYYY-MM-DD".to_string()),
                };
            }
        }

        // Load reset events from database
        let (events, total_count) = match self.load_reset_events(
            &request.user_id,
            request.start_date.as_deref(),
            request.end_date.as_deref(),
            request.limit.unwrap_or(50),
            request.offset.unwrap_or(0),
        ).await {
            Ok(result) => result,
            Err(e) => {
                return GetResetEventsResponse {
                    success: false,
                    events: vec![],
                    total_count: None,
                    error: Some(format!("Failed to load reset events: {}", e)),
                };
            }
        };

        GetResetEventsResponse {
            success: true,
            events,
            total_count: Some(total_count),
            error: None,
        }
    }

    /// Handle get session summary message
    #[instrument(skip(self))]
    pub async fn handle_get_session_summary(
        &self,
        request: GetSessionSummaryRequest,
    ) -> GetSessionSummaryResponse {
        info!("Handling get session summary request for user {} (period: {})",
              request.user_id, request.period);

        let count = request.count.unwrap_or(4); // Default to 4 periods

        let summary = match request.period.as_str() {
            "week" => self.get_weekly_summary(&request.user_id, count).await,
            "month" => self.get_monthly_summary(&request.user_id, count).await,
            "year" => self.get_yearly_summary(&request.user_id, count).await,
            _ => {
                return GetSessionSummaryResponse {
                    success: false,
                    summary: vec![],
                    error: Some("Invalid period. Use 'week', 'month', or 'year'".to_string()),
                };
            }
        };

        match summary {
            Ok(summary) => GetSessionSummaryResponse {
                success: true,
                summary,
                error: None,
            },
            Err(e) => GetSessionSummaryResponse {
                success: false,
                summary: vec![],
                error: Some(format!("Failed to generate session summary: {}", e)),
            },
        }
    }

    /// Load daily session stats from database
    async fn load_daily_session_stats(
        &self,
        user_id: &str,
        date: &str,
        days: Option<u32>,
    ) -> Result<Vec<DailySessionStats>, AppError> {
        // For now, return empty vector - this would be implemented with actual database queries
        info!("Loading daily stats for user {} on {} (days: {:?})", user_id, date, days);
        Ok(vec![])
    }

    /// Load reset events from database
    async fn load_reset_events(
        &self,
        user_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<SessionResetEvent>, u32), AppError> {
        // For now, return empty vector - this would be implemented with actual database queries
        info!("Loading reset events for user {} ({} to {}, limit: {}, offset: {})",
              user_id,
              start_date.unwrap_or("beginning"),
              end_date.unwrap_or("now"),
              limit,
              offset);
        Ok((vec![], 0))
    }

    /// Generate weekly summary
    async fn get_weekly_summary(
        &self,
        user_id: &str,
        count: u32,
    ) -> Result<Vec<SessionSummaryData>, AppError> {
        info!("Generating weekly summary for user {} ({} weeks)", user_id, count);
        // This would be implemented with actual database queries
        Ok(vec![])
    }

    /// Generate monthly summary
    async fn get_monthly_summary(
        &self,
        user_id: &str,
        count: u32,
    ) -> Result<Vec<SessionSummaryData>, AppError> {
        info!("Generating monthly summary for user {} ({} months)", user_id, count);
        // This would be implemented with actual database queries
        Ok(vec![])
    }

    /// Generate yearly summary
    async fn get_yearly_summary(
        &self,
        user_id: &str,
        count: u32,
    ) -> Result<Vec<SessionSummaryData>, AppError> {
        info!("Generating yearly summary for user {} ({} years)", user_id, count);
        // This would be implemented with actual database queries
        Ok(vec![])
    }

    /// Validate date format (YYYY-MM-DD)
    fn is_valid_date_format(&self, date_str: &str) -> bool {
        // Simple validation - in production, would use proper date parsing
        date_str.len() == 10 &&
        date_str.chars().nth(4) == Some('-') &&
        date_str.chars().nth(7) == Some('-') &&
        date_str[..4].parse::<u32>().is_ok() &&
        date_str[5..7].parse::<u32>().is_ok() &&
        date_str[8..10].parse::<u32>().is_ok()
    }
}