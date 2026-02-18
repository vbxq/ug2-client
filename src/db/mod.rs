pub mod models;

use anyhow::Result;
use sea_orm::{Database, DatabaseConnection, ConnectionTrait};

pub async fn connect(database_url: &str) -> Result<DatabaseConnection> {
    let db = Database::connect(database_url).await?;
    tracing::info!("Connected to database");
    Ok(db)
}

pub async fn run_migrations(db: &DatabaseConnection) -> Result<()> {
    db.execute_unprepared(include_str!("../../migrations/001_initial.sql")).await?;
    db.execute_unprepared(include_str!("../../migrations/002_add_index_scripts.sql")).await?;
    tracing::info!("Migrations applied");
    Ok(())
}
