use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static HEX_HASH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""([A-Fa-f0-9]{20})""#).unwrap()
});

static CHUNK_HASH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\d[\w]*:"([a-f0-9]{16})""#).unwrap()
});

static EXPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\.exports=.\..\+"(.*?\..{0,5})""#).unwrap()
});

static ASSET_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"/assets/([a-zA-Z0-9]+\.[a-z0-9]{2,5})"#).unwrap()
});

pub fn extract_asset_refs(content: &str) -> HashSet<String> {
    let mut refs = HashSet::new();

    for cap in HEX_HASH_RE.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            refs.insert(m.as_str().to_string());
        }
    }

    for cap in CHUNK_HASH_RE.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            refs.insert(m.as_str().to_string());
        }
    }

    for cap in EXPORT_RE.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            refs.insert(m.as_str().to_string());
        }
    }

    for cap in ASSET_URL_RE.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            refs.insert(m.as_str().to_string());
        }
    }

    refs
}
