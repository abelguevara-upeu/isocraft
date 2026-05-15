use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Returns the base metadata cache directory: ~/.isocraft/meta
pub fn meta_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    let home = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());

    #[cfg(not(target_os = "windows"))]
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());

    Path::new(&home).join(".isocraft").join("meta")
}

/// Reads a JSON cache file if it exists and has not expired.
/// - `ttl_secs = 0` means "never expire" (permanent cache).
pub fn read_cache(path: &Path, ttl_secs: u64) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    let wrapper: serde_json::Value = serde_json::from_str(&content).ok()?;

    if ttl_secs > 0 {
        let cached_at = wrapper.get("cached_at")?.as_u64()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        if now.saturating_sub(cached_at) > ttl_secs {
            return None; // Expired
        }
    }

    wrapper.get("data").cloned()
}

/// Writes a JSON value to a cache file, wrapping it with a timestamp.
pub fn write_cache(path: &Path, data: &serde_json::Value) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    let wrapper = serde_json::json!({
        "cached_at": now,
        "data": data
    });

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let _ = std::fs::write(path, serde_json::to_string(&wrapper).unwrap_or_default());
}

/// Convenience: fetch a URL, using a disk cache with a given TTL.
/// - `ttl_secs = 0` means permanent cache.
pub async fn fetch_cached(url: &str, cache_path: &Path, ttl_secs: u64) -> Result<serde_json::Value, String> {
    if let Some(cached) = read_cache(cache_path, ttl_secs) {
        return Ok(cached);
    }

    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} fetching {}", resp.status(), url));
    }
    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    write_cache(cache_path, &data);
    Ok(data)
}

/// 1 hour in seconds
pub const TTL_1H: u64 = 3600;
/// 6 hours in seconds
pub const TTL_6H: u64 = 21_600;
/// Permanent (no expiry)
pub const TTL_PERMANENT: u64 = 0;
