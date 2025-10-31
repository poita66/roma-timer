use std::env;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::path::Path;
use uuid::Uuid;

use crate::TimerState;

// JSON file storage models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSession {
    pub id: String,
    pub session_type: String, // 'work', 'short_break', 'long_break'
    pub duration_seconds: u32,
    pub remaining_seconds: u32,
    pub is_running: bool,
    pub session_count: u32,
    pub work_duration: u32,
    pub short_break_duration: u32,
    pub long_break_duration: u32,
    pub long_break_frequency: u32,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub id: Option<i64>,
    pub work_duration: u32,
    pub short_break_duration: u32,
    pub long_break_duration: u32,
    pub long_break_frequency: u32,
    pub notifications_enabled: bool,
    pub webhook_url: Option<String>,
    pub wait_for_interaction: bool,
    pub theme: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonDatabase {
    timer_sessions: Vec<TimerSession>,
    user_settings: Vec<UserSettings>,
    #[serde(default)]
    users: Vec<User>,
}

pub struct Database {
    file_path: String,
}

impl Database {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = Self::determine_database_path().await?;

        println!("🗄️ Using JSON file storage: {}", file_path);

        // Initialize database file if it doesn't exist
        if !Path::new(&file_path).exists() {
            println!("📝 Creating new database file");
            let initial_db = JsonDatabase {
                timer_sessions: Vec::new(),
                user_settings: Vec::new(),
                users: Vec::new(),
            };
            let json_data = serde_json::to_string_pretty(&initial_db)?;
            let mut file = fs::File::create(&file_path).await?;
            file.write_all(json_data.as_bytes()).await?;
        }

        println!("✅ Database file ready");

        Ok(Database { file_path })
    }

    /// Determines the database file path based on environment variables
    /// Priority order:
    /// 1. DATABASE_URL (for backward compatibility)
    /// 2. ROMA_TIMER_DATA_DIR + "roma_timer.json" (new configurable directory)
    /// 3. Default fallback: "/tmp/roma_timer.json"
    async fn determine_database_path() -> Result<String, Box<dyn std::error::Error>> {
        // First, check for DATABASE_URL (backward compatibility)
        if let Ok(database_url) = env::var("DATABASE_URL") {
            // Only use DATABASE_URL if it's not empty
            if !database_url.is_empty() {
                return Ok(database_url);
            }
        }

        // Check for ROMA_TIMER_DATA_DIR environment variable
        let data_dir = if let Ok(custom_dir) = env::var("ROMA_TIMER_DATA_DIR") {
            custom_dir
        } else {
            // Default to /tmp if no environment variable is set
            "/tmp".to_string()
        };

        // Ensure the data directory exists
        Self::ensure_data_directory_exists(&data_dir).await?;

        // Construct the full path to the database file
        let file_path = Path::new(&data_dir)
            .join("roma_timer.json")
            .to_string_lossy()
            .to_string();

        Ok(file_path)
    }

    /// Ensures the data directory exists, creating it if necessary
    async fn ensure_data_directory_exists(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(dir_path);

        if !path.exists() {
            println!("📁 Creating data directory: {}", dir_path);
            fs::create_dir_all(path).await
                .map_err(|e| format!("Failed to create data directory '{}': {}", dir_path, e))?;

            // Set appropriate permissions for the directory
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(path).await?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o755); // rwxr-xr-x
                fs::set_permissions(path, permissions).await?;
            }

            println!("✅ Data directory created successfully");
        }

        // Verify we can write to the directory (or its parent)
        let test_file = path.join(".roma_timer_write_test");
        match fs::write(&test_file, "test").await {
            Ok(_) => {
                let _ = fs::remove_file(&test_file).await; // Clean up test file
                println!("✅ Data directory is writable");
            }
            Err(e) => {
                return Err(format!("Data directory '{}' is not writable: {}", dir_path, e).into());
            }
        }

        Ok(())
    }

    // Helper method to read the JSON database file
    async fn read_database(&self) -> Result<JsonDatabase, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(&self.file_path).await?;
        let db: JsonDatabase = serde_json::from_str(&contents)?;
        Ok(db)
    }

    // Helper method to write to the JSON database file
    async fn write_database(&self, db: &JsonDatabase) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string_pretty(db)?;
        let mut file = fs::File::create(&self.file_path).await?;
        file.write_all(json_data.as_bytes()).await?;
        Ok(())
    }

    pub async fn get_current_timer_state(&self) -> Result<Option<TimerState>, Box<dyn std::error::Error>> {
        let mut db = self.read_database().await?;

        // Sort by created_at descending and take the latest
        db.timer_sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(session) = db.timer_sessions.first() {
            Ok(Some(self.timer_session_to_state(session.clone())))
        } else {
            Ok(None)
        }
    }

    pub async fn save_timer_state(&self, state: &TimerState) -> Result<(), Box<dyn std::error::Error>> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let mut db = self.read_database().await?;

        // Create new session
        let session = TimerSession {
            id: session_id,
            session_type: state.session_type.clone(),
            duration_seconds: state.remaining_seconds, // Use remaining as current duration
            remaining_seconds: state.remaining_seconds,
            is_running: state.is_running,
            session_count: state.session_count,
            work_duration: state.work_duration,
            short_break_duration: state.short_break_duration,
            long_break_duration: state.long_break_duration,
            long_break_frequency: 4, // Default long break frequency
            last_updated: now,
            created_at: now,
        };

        // Add to sessions
        db.timer_sessions.push(session);

        // Clean up old sessions (keep only last 100)
        db.timer_sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        db.timer_sessions.truncate(100);

        // Write back to file
        self.write_database(&db).await?;

        Ok(())
    }

    pub async fn get_user_settings(&self) -> Result<UserSettings, Box<dyn std::error::Error>> {
        let mut db = self.read_database().await?;

        // Sort by updated_at descending and take the latest
        db.user_settings.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        if let Some(settings) = db.user_settings.first() {
            Ok(settings.clone())
        } else {
            // Return default settings
            Ok(UserSettings {
                id: None,
                work_duration: 25 * 60,
                short_break_duration: 5 * 60,
                long_break_duration: 15 * 60,
                long_break_frequency: 4,
                notifications_enabled: true,
                webhook_url: None,
                wait_for_interaction: false,
                theme: "light".to_string(),
                updated_at: Utc::now(),
            })
        }
    }

    pub async fn update_user_settings(&self, settings: &UserSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut db = self.read_database().await?;

        if let Some(id) = settings.id {
            // Update existing settings by finding and replacing
            if let Some(index) = db.user_settings.iter().position(|s| s.id == Some(id)) {
                db.user_settings[index] = settings.clone();
            } else {
                // ID not found, add as new
                db.user_settings.push(settings.clone());
            }
        } else {
            // Insert new settings with generated ID
            let mut new_settings = settings.clone();
            new_settings.id = Some(db.user_settings.len() as i64 + 1);
            db.user_settings.push(new_settings);
        }

        // Write back to file
        self.write_database(&db).await?;

        Ok(())
    }

    fn timer_session_to_state(&self, session: TimerSession) -> TimerState {
        TimerState {
            is_running: session.is_running,
            remaining_seconds: session.remaining_seconds,
            session_type: session.session_type,
            session_count: session.session_count,
            work_duration: session.work_duration,
            short_break_duration: session.short_break_duration,
            long_break_duration: session.long_break_duration,
            last_updated: session.last_updated.timestamp() as u64,
        }
    }

    // User management methods
    pub async fn create_user(&self, username: &str, password_hash: &str, salt: &str) -> Result<User, Box<dyn std::error::Error>> {
        let mut db = self.read_database().await?;

        // Check if username already exists
        if db.users.iter().any(|u| u.username == username) {
            return Err("Username already exists".into());
        }

        let user = User {
            id: Some(db.users.len() as i64 + 1),
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            salt: salt.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        db.users.push(user.clone());
        self.write_database(&db).await?;

        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let db = self.read_database().await?;
        Ok(db.users.into_iter().find(|u| u.username == username))
    }

    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let db = self.read_database().await?;
        Ok(db.users.into_iter().find(|u| u.id == Some(user_id)))
    }
}

