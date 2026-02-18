use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// entry in the top-level builds.json index
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildIndexEntry {
    pub date: i64,
    pub path: String,
}

/// the builds.json index: hash -> entry
pub type BuildIndex = HashMap<String, BuildIndexEntry>;

/// full build data from individual JSON files
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildData {
    pub id: String,
    pub date: String,
    /// GLOBAL_ENV can be a full object or `{}` (empty) for some builds
    #[serde(rename = "GLOBAL_ENV")]
    pub global_env: serde_json::Value,
    pub scripts: Vec<String>,
}

/// simplified build info but just for internal use
#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    pub build_hash: String,
    pub channel: String,
    /// all assets from the build (1000+)
    pub scripts: Vec<String>,
    /// the 4 bootstrap scripts from the index.html (loaded in <script> tags) (new build seems to have one god-file though ?)
    pub index_scripts: Vec<String>,
    pub timestamp: i64,
    pub global_env: serde_json::Value,
}
