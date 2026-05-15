use futures_util::StreamExt;
use std::io::Write;
use crate::models::DownloadInfo;
use crate::launcher::context::LauncherContext;

/// Downloads the official Minecraft client JAR if not cached.
pub async fn ensure_game_jar(
    ctx: &LauncherContext,
    version_id: &str,
    download_info: &DownloadInfo,
) -> Result<String, String> {
    let version_dir = ctx.paths.root.join("versions").join(version_id);
    let jar_path = version_dir.join(format!("{}.jar", version_id));

    if jar_path.exists() {
        ctx.emit_progress(100, &format!("Minecraft {} ready.", version_id));
        return Ok(jar_path.to_string_lossy().to_string());
    }

    std::fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;

    ctx.emit_progress(0, &format!("Downloading official client {}...", version_id));

    let resp = ctx.client.get(&download_info.url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() { 
        return Err(format!("Error downloading Minecraft (HTTP {})", resp.status())); 
    }

    let mut file = std::fs::File::create(&jar_path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let pct = (downloaded as f64 / download_info.size as f64 * 100.0) as u32;
        ctx.emit_progress(pct, &format!("Downloading Minecraft... {}%", pct));
    }

    Ok(jar_path.to_string_lossy().to_string())
}
