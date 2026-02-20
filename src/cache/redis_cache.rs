use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

pub async fn connect(redis_url: &str) -> Result<ConnectionManager> {
    let client = redis::Client::open(redis_url)?;
    let cm = ConnectionManager::new(client).await?;
    tracing::info!("Connected to Redis");
    Ok(cm)
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
