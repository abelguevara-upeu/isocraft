use std::path::Path;
use crate::launcher::context::LauncherContext;

pub struct Downloader;

impl Downloader {
    /// Downloads a file from a URL to a destination path.
    /// Includes basic retry logic and directory creation.
    pub async fn ensure_file(
        ctx: &LauncherContext,
        url: &str,
        dest: &Path,
        description: &str,
    ) -> Result<(), String> {
        if dest.exists() {
            return Ok(());
        }

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        println!("[Downloader] 📥 Downloading {}: {}", description, url);

        let mut retries = 3;
        while retries > 0 {
            let resp = ctx.client.get(url).send().await.map_err(|e| e.to_string());
            
            match resp {
                Ok(r) if r.status().is_success() => {
                    let bytes = r.bytes().await.map_err(|e| e.to_string())?;
                    std::fs::write(dest, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;
                    return Ok(());
                }
                Ok(r) if r.status().as_u16() == 404 => {
                    return Err(format!("File not found (404): {}", url));
                }
                Ok(r) => {
                    println!("[Downloader] ⚠️ HTTP {} for {}. Retrying...", r.status(), description);
                    retries -= 1;
                }
                Err(e) => {
                    println!("[Downloader] ⚠️ Connection error for {}: {}. Retrying...", description, e);
                    retries -= 1;
                }
            }
            if retries > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }

        Err(format!("Failed to download {} after multiple attempts", description))
    }
}
