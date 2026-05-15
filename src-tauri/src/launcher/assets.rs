use futures_util::StreamExt;
use std::sync::{Arc, Mutex};

use crate::models::{AssetIndexInfo};
use crate::launcher::context::LauncherContext;

/// Downloads all assets from a Mojang index in PARALLEL.
pub async fn ensure_assets(
    ctx: &LauncherContext,
    index_info: &AssetIndexInfo,
) -> Result<String, String> {
    let assets_dir = ctx.paths.assets.clone();
    let indexes_dir = assets_dir.join("indexes");
    let objects_dir = assets_dir.join("objects");

    std::fs::create_dir_all(&indexes_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&objects_dir).map_err(|e| e.to_string())?;

    // 1. Download index
    let index_path = indexes_dir.join(format!("{}.json", index_info.id));
    let index_content = if index_path.exists() {
        std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?
    } else {
        ctx.emit_progress(0, &format!("Downloading index {}", index_info.id));
        let resp = ctx.client.get(&index_info.url).send().await.map_err(|e| e.to_string())?;
        let content = resp.text().await.map_err(|e| e.to_string())?;
        std::fs::write(&index_path, &content).map_err(|e| e.to_string())?;
        content
    };

    let index: serde_json::Value = serde_json::from_str(&index_content).map_err(|e| e.to_string())?;
    let objects = index["objects"].as_object().ok_or("Invalid asset index")?;

    let is_virtual = index["virtual"].as_bool().unwrap_or(false);
    let map_to_resources = index["map_to_resources"].as_bool().unwrap_or(false);

    let total = objects.len();
    let done_count = Arc::new(Mutex::new(0usize));

    let tasks: Vec<(String, String)> = objects.iter().map(|(name, obj)| {
        (name.clone(), obj["hash"].as_str().unwrap_or_default().to_string())
    }).collect();

    // 2. Parallel downloads
    let stream = futures_util::stream::iter(tasks).map(|(name, hash)| {
        let ctx = ctx.clone();
        let done_count = done_count.clone();
        let objects_dir = objects_dir.clone();
        let assets_dir = assets_dir.clone();

        async move {
            if hash.len() < 2 { return Ok(()); }
            let prefix = &hash[..2];
            let obj_dir = objects_dir.join(prefix);
            let obj_path = obj_dir.join(&hash);

            if !obj_path.exists() {
                let _ = std::fs::create_dir_all(&obj_dir);
                let url = format!("https://resources.download.minecraft.net/{}/{}", prefix, hash);
                if let Ok(resp) = ctx.client.get(&url).send().await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            let _ = std::fs::write(&obj_path, &bytes);
                        }
                    }
                }
            }

            if is_virtual {
                let virt = assets_dir.join("virtual").join("legacy").join(&name);
                if !virt.exists() {
                    if let Some(p) = virt.parent() { let _ = std::fs::create_dir_all(p); }
                    let _ = std::fs::copy(&obj_path, &virt);
                }
            }
            if map_to_resources {
                let res = ctx.paths.root.join("resources").join(&name);
                if !res.exists() {
                    if let Some(p) = res.parent() { let _ = std::fs::create_dir_all(p); }
                    let _ = std::fs::copy(&obj_path, &res);
                }
            }

            let mut val = done_count.lock().unwrap();
            *val += 1;
            if *val % 100 == 0 {
                let pct = (*val as f64 / total as f64 * 100.0) as u32;
                ctx.emit_progress(pct, &format!("Assets {}/{}", *val, total));
            }
            Ok::<(), String>(())
        }
    }).buffer_unordered(25);

    let results: Vec<_> = stream.collect().await;
    for r in results { r?; }

    if is_virtual {
        Ok(assets_dir.join("virtual").join("legacy").to_string_lossy().to_string())
    } else {
        Ok(assets_dir.to_string_lossy().to_string())
    }
}
