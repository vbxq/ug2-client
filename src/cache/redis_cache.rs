use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

const ASSET_TTL_SECS: u64 = 86400; // 24 hours

pub async fn connect(redis_url: &str) -> Result<ConnectionManager> {
    let client = redis::Client::open(redis_url)?;
    let cm = ConnectionManager::new(client).await?;
    tracing::info!("Connected to Redis");
    Ok(cm)
}

pub async fn cache_asset(conn: &mut ConnectionManager, key: &str, data: &[u8]) -> Result<()> {
    conn.set_ex::<_, _, ()>(key, data, ASSET_TTL_SECS).await?;
    Ok(())
}

pub async fn get_cached_asset(conn: &mut ConnectionManager, key: &str) -> Result<Option<Vec<u8>>> {
    let result: Option<Vec<u8>> = conn.get(key).await?;
    Ok(result)
}

pub async fn invalidate_build(conn: &mut ConnectionManager, build_hash: &str) -> Result<()> {
    let pattern = format!("asset:{}:*", build_hash);
    let mut cursor: u64 = 0;
    let mut total_deleted = 0usize;

    loop {
        let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(&pattern)
            .arg("COUNT")
            .arg(100)
            .query_async(conn)
            .await?;

        for key in &keys {
            conn.del::<_, ()>(key).await?;
            total_deleted += 1;
        }

        cursor = next_cursor;
        if cursor == 0 {
            break;
        }
    }

    if total_deleted > 0 {
        tracing::info!("Invalidated {} cached assets for build {}", total_deleted, build_hash);
    }

    Ok(())
}

pub fn asset_key(build_hash: &str, asset_name: &str) -> String {
    format!("asset:{}:{}", build_hash, asset_name)
}

const BUILDS_LIST_KEY: &str = "cache:builds_list";

pub fn builds_list_key() -> &'static str {
    BUILDS_LIST_KEY
}

pub async fn get_cached_json(conn: &mut ConnectionManager, key: &str) -> Result<Option<String>> {
    let result: Option<String> = conn.get(key).await?;
    Ok(result)
}

pub async fn cache_json(conn: &mut ConnectionManager, key: &str, json: &str, ttl_secs: u64) -> Result<()> {
    conn.set_ex::<_, _, ()>(key, json, ttl_secs).await?;
    Ok(())
}

pub async fn invalidate_builds_cache(conn: &mut ConnectionManager) -> Result<()> {
    conn.del::<_, ()>(BUILDS_LIST_KEY).await?;
    Ok(())
}
