pub mod models;

use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, ConnectionTrait};

pub async fn connect(database_url: &str) -> Result<DatabaseConnection> {
    let mut opts = ConnectOptions::new(database_url);
    opts.max_connections(10)
        .min_connections(1)
        .idle_timeout(std::time::Duration::from_secs(300));
    let db = Database::connect(opts).await?;
    tracing::info!("Connected to database");
    Ok(db)
}

pub async fn run_migrations(db: &DatabaseConnection) -> Result<()> {
    db.execute_unprepared(include_str!("../../migrations/001_initial.sql")).await?;
    db.execute_unprepared(include_str!("../../migrations/002_add_index_scripts.sql")).await?;
    tracing::info!("Migrations applied");
    Ok(())
}
