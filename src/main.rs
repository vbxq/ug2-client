use ug2_client::*;

use anyhow::{Context, Result};
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("ug2-client patcher.");

    let config = config::AppConfig::load()?;
    tracing::info!("Configuration loaded");

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "clone" => return run_clone(&config).await,
            "import" => return run_import(&config, args.get(2).map(|s| s.as_str())).await,
            other => {
                eprintln!("Unknown command: {}", other);
                eprintln!("Usage:");
                eprintln!("  ug2-client              Start the HTTP server");
                eprintln!("  ug2-client clone        Clone the Discord Build Logger repo (you should already have it, look at data/builds-repo), use this if you didn't download ug2-client from the repo.");
                eprintln!("  ug2-client import [dir] Import builds from cloned repo into DB");
                std::process::exit(1);
            }
        }
    }

    let db = db::connect(&config.database_url).await?;
    tracing::info!("Database connected");

    let redis = cache::redis_cache::connect(&config.redis_url).await?;
    tracing::info!("Redis connected");

    server::run(config, db, redis).await?;

    Ok(())
}

async fn run_clone(config: &config::AppConfig) -> Result<()> {
    let repo_url = format!("https://github.com/{}.git", config.github_builds_repo);
    let target_dir = std::path::Path::new("./data/builds-repo");

    if target_dir.exists() {
        tracing::info!("Repo already cloned at {:?}, pulling latest...", target_dir);
        let status = tokio::process::Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(target_dir)
            .status()
            .await
            .context("Failed to run git pull")?;
        if !status.success() {
            anyhow::bail!("git pull failed with exit code {:?}", status.code());
        }
        tracing::info!("Repo updated successfully");
    } else {
        tracing::info!("Cloning {} into {:?}...", repo_url, target_dir);
        if let Some(parent) = target_dir.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let status = tokio::process::Command::new("git")
            .args(["clone", "--depth", "1", &repo_url, &target_dir.to_string_lossy()])
            .status()
            .await
            .context("Failed to run git clone")?;
        if !status.success() {
            anyhow::bail!("git clone failed with exit code {:?}", status.code());
        }
        tracing::info!("Repo cloned successfully");
    }

    tracing::info!("Now run: cargo run -- import");
    Ok(())
}

async fn run_import(config: &config::AppConfig, data_dir: Option<&str>) -> Result<()> {
    let data_dir = data_dir.unwrap_or("./data/builds-repo");
    tracing::info!("Importing builds from {}", data_dir);

    let db = db::connect(&config.database_url).await?;
    db::run_migrations(&db).await?;

    let index_path = std::path::Path::new(data_dir).join("builds.json");
    let index_str = tokio::fs::read_to_string(&index_path).await
        .context(format!("Could not read {}. Run 'cargo run -- clone' first.", index_path.display()))?;
    let index: discord_scraper::BuildIndex = serde_json::from_str(&index_str)?;

    let total = index.len();
    tracing::info!("Found {} builds in index", total);

    let mut imported = 0u32;
    let mut errors = 0u32;

    for (hash, entry) in &index {
        let build_path = std::path::Path::new(data_dir)
            .join(&entry.path)
            .join(format!("{}.json", hash));

        if !build_path.exists() {
            if errors < 5 { tracing::warn!("Build file not found: {:?}", build_path); }
            errors += 1;
            continue;
        }

        match tokio::fs::read_to_string(&build_path).await {
            Ok(content) => {
                match serde_json::from_str::<discord_scraper::BuildData>(&content) {
                    Ok(build_data) => {
                        match discord_scraper::build_parser::parse_build(&build_data) {
                            Ok(info) => {
                                use sea_orm::*;
                                use db::models::discord_build;

                                let ts = chrono::DateTime::from_timestamp_millis(info.timestamp)
                                    .unwrap_or_default()
                                    .fixed_offset();

                                let global_env_db = if build_data.global_env.as_object().map_or(false, |m| m.is_empty()) {
                                    None
                                } else {
                                    Some(info.global_env)
                                };

                                let active = discord_build::ActiveModel {
                                    build_hash: Set(info.build_hash.clone()),
                                    channel: Set(info.channel.clone()),
                                    build_date: Set(ts),
                                    global_env: Set(global_env_db),
                                    scripts: Set(serde_json::to_value(&info.scripts).unwrap()),
                                    index_scripts: Set(serde_json::json!([])),
                                    is_patched: Set(false),
                                    is_active: Set(false),
                                    ..Default::default()
                                };

                                match discord_build::Entity::insert(active)
                                    .on_conflict(
                                        sea_orm::sea_query::OnConflict::column(discord_build::Column::BuildHash)
                                            .do_nothing()
                                            .to_owned(),
                                    )
                                    .do_nothing()
                                    .exec_without_returning(&db)
                                    .await
                                {
                                    Ok(_) => imported += 1,
                                    Err(e) => {
                                        tracing::warn!("Insert error for {}: {}", hash, e);
                                        errors += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                if errors < 5 { tracing::warn!("Parse error for {}: {}", hash, e); }
                                errors += 1;
                            }
                        }
                    }
                    Err(e) => {
                        if errors < 5 { tracing::warn!("JSON error for {}: {}", hash, e); }
                        errors += 1;
                    }
                }
            }
            Err(e) => {
                if errors < 5 { tracing::warn!("Read error for {}: {}", hash, e); }
                errors += 1;
            }
        }

        let processed = imported + errors;
        if processed % 500 == 0 {
            tracing::info!(
                "Progress: {}/{} ({} imported, {} errors)",
                processed, total, imported, errors
            );
        }
    }

    tracing::info!(
        "Import complete: {} imported, {} errors out of {} total",
        imported, errors, total
    );
    Ok(())
}
