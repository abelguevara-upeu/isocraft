use futures_util::StreamExt;
use std::io::Write;
use crate::launcher::context::LauncherContext;

/// Downloads and extracts a JRE if it doesn't exist in the cache.
pub async fn ensure_jre(
    ctx: &LauncherContext,
    version_major: u32,
) -> Result<String, String> {
    // Adoptium uses "mac" instead of "osx"
    let os = match std::env::consts::OS {
        "macos" => "mac",
        "windows" => "windows",
        _ => "linux",
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "aarch64",
        _ => "x64",
    };

    // Map versions to the best stable LTS for the platform
    let download_major = if cfg!(target_os = "macos") {
        if version_major <= 8 {
            8 // Legacy versions strictly need Java 8
        } else if version_major < 21 {
            21 // Modern versions (1.13+) are more stable on Java 21 than 17/11 on macOS
        } else {
            version_major
        }
    } else if version_major == 16 {
        17
    } else {
        version_major
    };

    let java_dir = ctx.paths.runtimes.join(format!("java-{}", download_major));

    if java_dir.exists() {
        // Validate integrity: ensure the java binary is actually inside the directory
        let java_binary_found = walkdir::WalkDir::new(&java_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                (name == "java" || name == "java.exe") && e.file_type().is_file()
            });

        if java_binary_found {
            ctx.emit_progress(100, "JRE ready in cache.");
            return Ok(java_dir.to_string_lossy().to_string());
        }

        // Corrupted/incomplete JRE — remove and re-download
        println!("[IsoCraft] ⚠️ JRE {} directory found but binary missing. Re-downloading...", download_major);
        let _ = std::fs::remove_dir_all(&java_dir);
    }

    let url = format!("https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse", download_major, os, arch);

    ctx.emit_progress(0, &format!("Downloading JRE {}...", download_major));

    let resp = ctx.client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() { 
        return Err(format!("Failed to contact Adoptium (HTTP {})", resp.status())); 
    }

    let total_size = resp.content_length().unwrap_or(50 * 1024 * 1024);
    std::fs::create_dir_all(&java_dir).map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    {
        let zip_path = ctx.paths.root.join(format!("jre-{}.zip", version_major));
        download_to_file(ctx, resp, total_size, &zip_path).await?;
        extract_zip(&zip_path, &java_dir)?;
        let _ = std::fs::remove_file(&zip_path);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let tar_path = ctx.paths.root.join(format!("jre-{}.tar.gz", version_major));
        download_to_file(ctx, resp, total_size, &tar_path).await?;
        extract_tar(&tar_path, &java_dir)?;
        let _ = std::fs::remove_file(&tar_path);
    }

    ctx.emit_progress(100, "JRE successfully installed.");

    Ok(java_dir.to_string_lossy().to_string())
}

async fn download_to_file(ctx: &LauncherContext, resp: reqwest::Response, total_size: u64, path: &std::path::Path) -> Result<(), String> {
    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let pct = (downloaded as f64 / total_size as f64 * 80.0) as u32;
        ctx.emit_progress(pct, &format!("Downloading JRE... {}%", pct));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn extract_zip(zip_path: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let out_path = dest.join(entry.mangled_name());
        if entry.is_dir() { std::fs::create_dir_all(&out_path).ok(); }
        else {
            if let Some(p) = out_path.parent() { std::fs::create_dir_all(p).ok(); }
            let mut out = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).ok();
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn extract_tar(tar_path: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::open(tar_path).map_err(|e| e.to_string())?;
    let tar = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(dest).map_err(|e| e.to_string())
}
