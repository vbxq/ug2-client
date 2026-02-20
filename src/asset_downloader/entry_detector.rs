use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

fn safe_slice_start(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    // Find the closest char boundary <= max_bytes
    let mut idx = max_bytes;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    &s[..idx]
}

fn safe_slice_end(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let start = s.len() - max_bytes;
    // Find the closest char boundary >= start
    let mut idx = start;
    while idx < s.len() && !s.is_char_boundary(idx) {
        idx += 1;
    }
    &s[idx..]
}

static CHUNK_IDS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.push\(\[\[(\d+(?:,\d+)*)\]").unwrap()
});

pub static DEFERRED_DEPS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.O\(\s*(?:0|void 0)\s*,\s*\[(\d+(?:\s*,\s*\d+)*)\]").unwrap()
});

pub fn detect_entry_scripts(build_dir: &Path, ordered_scripts: &[String]) -> Vec<String> {
    let mut included: HashSet<usize> = HashSet::new();
    let mut required_chunk_ids: HashSet<u64> = HashSet::new();

    if !ordered_scripts.is_empty() {
        included.insert(0);
    }

    let scan_limit = ordered_scripts.len().min(30);
    let mut chunk_id_map: Vec<(usize, Vec<u64>)> = Vec::new();

    for (i, script) in ordered_scripts.iter().enumerate().skip(1).take(scan_limit - 1) {
        let filename = script.trim_start_matches("/assets/");
        let path = build_dir.join(filename);

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if !is_webpack_chunk(&content) {
            included.insert(i);
            continue;
        }

        let chunk_ids = extract_chunk_ids(&content);
        chunk_id_map.push((i, chunk_ids));

        let tail_start = content.len().saturating_sub(3000);
        let tail = &content[tail_start..];

        if has_entry_factory(tail) {
            included.insert(i);
            tracing::debug!("Script {} ({}) is an entry chunk", i, filename);

            for cap in DEFERRED_DEPS_RE.captures_iter(tail) {
                if let Some(m) = cap.get(1) {
                    for id_str in m.as_str().split(',') {
                        if let Ok(id) = id_str.trim().parse::<u64>() {
                            required_chunk_ids.insert(id);
                        }
                    }
                }
            }
        }
    }

    if !required_chunk_ids.is_empty() {
        for (idx, chunk_ids) in &chunk_id_map {
            if !included.contains(idx) && chunk_ids.iter().any(|id| required_chunk_ids.contains(id))
            {
                included.insert(*idx);
                tracing::debug!(
                    "Script {} included as dependency (provides chunks {:?})",
                    idx,
                    chunk_ids
                );
            }
        }
    }

    let mut indices: Vec<usize> = included.into_iter().collect();
    indices.sort();
    let result: Vec<String> = indices
        .into_iter()
        .map(|i| ordered_scripts[i].clone())
        .collect();

    if result.is_empty() {
        tracing::warn!("Could not detect any entry scripts — falling back to first script");
        return ordered_scripts.iter().take(1).cloned().collect();
    }

    tracing::info!(
        "Detected {} initial HTML scripts out of {} total: {:?}",
        result.len(),
        ordered_scripts.len(),
        result
    );

    result
}

pub fn is_webpack_chunk(content: &str) -> bool {
    let head = safe_slice_start(content, 500);
    head.contains("webpackChunk") && head.contains(".push(")
}

pub fn extract_chunk_ids(content: &str) -> Vec<u64> {
    let head = safe_slice_start(content, 2000);
    if let Some(cap) = CHUNK_IDS_RE.captures(head) {
        if let Some(m) = cap.get(1) {
            return m
                .as_str()
                .split(',')
                .filter_map(|s| s.trim().parse::<u64>().ok())
                .collect();
        }
    }
    Vec::new()
}

pub fn has_entry_factory(tail: &str) -> bool {
    let code = if let Some(pos) = tail.rfind("//# sourceMappingURL=") {
        &tail[..pos]
    } else {
        tail
    };

    let trimmed = code.trim_end();
    if !trimmed.ends_with("]);") {
        return false;
    }

    let check_region = safe_slice_end(trimmed, 500);

    if DEFERRED_DEPS_RE.is_match(check_region) {
        return true;
    }

    if check_region.contains(".s=") && check_region.contains("=>(") {
        return true;
    }

    if check_region.contains(".s=") {
        return true;
    }

    false
}
