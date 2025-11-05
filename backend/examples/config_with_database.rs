//! Configuration with Database Selection Example
//!
//! Shows how the Roma Timer configuration system handles different database types

use roma_timer::config::Config;
use roma_timer::database::{DatabaseManager, DatabaseType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  Roma Timer Configuration with Database Selection\n");

    // Example 1: Default configuration (SQLite)
    println!("1. Default Configuration:");
    let default_config = Config::default();
    println!("   Host: {}", default_config.host);
    println!("   Port: {}", default_config.port);
    println!("   Database Type: {}", default_config.database_type);
    println!("   Database URL: {}", default_config.masked_database_url());
    println!();

    // Example 2: Environment-based configuration
    println!("2. Environment-based Configuration:");

    // Set some environment variables for demonstration
    env::set_var("ROMA_TIMER_HOST", "127.0.0.1");
    env::set_var("ROMA_TIMER_PORT", "8080");

    let env_config = Config::from_env()?;
    println!("   Host: {}", env_config.host);
    println!("   Port: {}", env_config.port);
    println!("   Database Type: {}", env_config.database_type);
    println!("   Database URL: {}", env_config.masked_database_url());
    println!();

    // Example 3: SQLite configuration
    println!("3. SQLite Configuration:");
    env::set_var("DATABASE_URL", "sqlite:custom_timer.db");
    let sqlite_config = Config::from_env()?;
    println!("   Database Type: {}", sqlite_config.database_type);
    println!("   Database URL: {}", sqlite_config.masked_database_url());

    // Test the database connection
    let sqlite_db = DatabaseManager::new(&sqlite_config.database_url).await?;
    sqlite_db.test_connection().await?;
    println!("   ✅ SQLite connection successful");
    println!();

    // Example 4: PostgreSQL configuration
    println!("4. PostgreSQL Configuration:");
    env::set_var("DATABASE_URL", "postgres://user:password@localhost:5432/roma_timer");
    let postgres_config = Config::from_env()?;
    println!("   Database Type: {}", postgres_config.database_type);
    println!("   Database URL: {}", postgres_config.masked_database_url());

    // Test PostgreSQL connection (will fail if server not running)
    match DatabaseManager::new(&postgres_config.database_url).await {
        Ok(postgres_db) => {
            postgres_db.test_connection().await?;
            println!("   ✅ PostgreSQL connection successful");
        }
        Err(e) => {
            println!("   ❌ PostgreSQL connection failed: {}", e);
            println!("   💡 Make sure PostgreSQL server is running");
        }
    }

    println!();

    // Example 5: Configuration validation
    println!("5. Configuration Validation:");
    match env_config.validate() {
        Ok(_) => println!("   ✅ Configuration is valid"),
        Err(e) => println!("   ❌ Configuration error: {}", e),
    }

    println!();
    println!("📋 Supported Environment Variables:");
    println!("   ROMA_TIMER_HOST         - Server bind address");
    println!("   ROMA_TIMER_PORT         - Server port");
    println!("   DATABASE_URL            - Database connection URL");
    println!("   ROMA_TIMER_DATABASE_URL - Roma Timer specific database URL");
    println!("   POSTGRES_URL            - PostgreSQL connection URL");
    println!("   ROMA_TIMER_SECRET       - Authentication secret");
    println!("   ROMA_TIMER_LOG_LEVEL    - Log level (error, warn, info, debug, trace)");

    Ok(())
}