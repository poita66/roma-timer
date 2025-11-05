//! Database Selection Example
//!
//! Demonstrates how to use both SQLite and PostgreSQL with Roma Timer

use std::env;
// Since this is a binary example, we need to include the modules directly
mod database;
use database::{DatabaseManager, DatabaseType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🗄️  Roma Timer Database Selection Example\n");

    // Example 1: SQLite (default)
    println!("1. SQLite Example:");
    let sqlite_url = "sqlite:example_timer.db";
    let sqlite_db = DatabaseManager::new(sqlite_url).await?;

    println!("   Database type: {}", sqlite_db.database_type);
    println!("   Connection URL: {}", sqlite_db.masked_database_url());

    // Test connection
    sqlite_db.test_connection().await?;
    println!("   ✅ SQLite connection successful");

    // Run migrations
    sqlite_db.migrate().await?;
    println!("   ✅ SQLite migrations completed");

    println!();

    // Example 2: PostgreSQL (if available)
    println!("2. PostgreSQL Example:");

    // Check for PostgreSQL URL in environment
    let postgres_url = env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost/roma_timer".to_string());

    println!("   Attempting PostgreSQL connection...");
    println!("   Connection URL: {}", DatabaseType::Postgres.example_url());

    match DatabaseManager::new(&postgres_url).await {
        Ok(postgres_db) => {
            println!("   ✅ PostgreSQL connection successful");
            println!("   Database type: {}", postgres_db.database_type);

            // Test connection
            postgres_db.test_connection().await?;
            println!("   ✅ PostgreSQL connection test passed");

            // Run migrations
            postgres_db.migrate().await?;
            println!("   ✅ PostgreSQL migrations completed");
        }
        Err(e) => {
            println!("   ❌ PostgreSQL connection failed: {}", e);
            println!("   💡 To test PostgreSQL:");
            println!("      - Set POSTGRES_URL environment variable");
            println!("      - Or start PostgreSQL server with: docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password -e POSTGRES_DB=roma_timer postgres:15");
        }
    }

    println!();

    // Example 3: Environment-based selection
    println!("3. Environment-based Selection:");
    let env_db = DatabaseManager::from_env().await?;
    println!("   Selected database: {}", env_db.database_type);
    println!("   Connection URL: {}", env_db.masked_database_url());

    // Show database configuration options
    println!();
    println!("🔧 Configuration Options:");
    println!("   SQLite:");
    println!("     export DATABASE_URL=\"sqlite:roma-timer.db\"");
    println!("     cargo run");
    println!();
    println!("   PostgreSQL:");
    println!("     export DATABASE_URL=\"postgres://user:password@localhost:5432/roma_timer\"");
    println!("     cargo run --features postgres");
    println!();
    println!("   Build features:");
    println!("     cargo build --features sqlite          # SQLite only");
    println!("     cargo build --features postgres         # PostgreSQL only");
    println!("     cargo build --features \"sqlite postgres\" # Both databases");

    Ok(())
}