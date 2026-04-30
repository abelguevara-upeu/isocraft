use futures_util::StreamExt;
use std::io::Write;
use tauri::{AppHandle, Manager, Emitter};

use crate::models::ProgressPayload;

/// Descarga y extrae un JRE de Eclipse Temurin (Adoptium) si no existe en caché.
/// - macOS / Linux → `.tar.gz`
/// - Windows       → `.zip`
///
/// El JRE se almacena en `<appDataDir>/runtimes/java-<major>/` y se reutiliza
/// entre instancias que requieran la misma versión mayor.
pub async fn asegurar_jre(
    app: &AppHandle,
    version_major: u32,
    instance_name: &str,
) -> Result<String, String> {
    let os = match std::env::consts::OS {
        "macos" => "mac",
        "windows" => "windows",
        _ => "linux",
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "aarch64",
        _ => "x64",
    };

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let java_dir = app_data_dir
        .join("runtimes")
        .join(format!("java-{}", version_major));

    // ── Caché hit ────────────────────────────────────────────────────────────
    if java_dir.exists() {
        app.emit(
            "jre-progress",
            ProgressPayload {
                instance_name: instance_name.to_string(),
                progress: 100,
                message: "JRE ya existe en caché local.".into(),
            },
        )
        .map_err(|e| e.to_string())?;
        return Ok(java_dir.to_string_lossy().to_string());
    }

    // Eclipse Temurin REST API v3
    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse",
        version_major, os, arch
    );

    app.emit(
        "jre-progress",
        ProgressPayload {
            instance_name: instance_name.to_string(),
            progress: 0,
            message: format!("Descargando Temurin JRE {}...", version_major),
        },
    )
    .map_err(|e| e.to_string())?;

    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "Fallo al descargar JRE desde Adoptium (HTTP {})",
            resp.status()
        ));
    }

    let total_size = resp.content_length().unwrap_or(50 * 1024 * 1024);
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&java_dir).map_err(|e| e.to_string())?;

    // ── Windows: .zip ────────────────────────────────────────────────────────
    #[cfg(target_os = "windows")]
    {
        let zip_path = app_data_dir.join(format!("jre-{}.zip", version_major));
        download_to_file(app, instance_name, resp, total_size, &zip_path).await?;

        app.emit(
            "jre-progress",
            ProgressPayload {
                instance_name: instance_name.to_string(),
                progress: 85,
                message: "Extrayendo binarios de Java (ZIP)...".into(),
            },
        )
        .map_err(|e| e.to_string())?;

        let zip_file = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let out_path = match entry.enclosed_name() {
                Some(p) => java_dir.join(p),
                None => continue,
            };
            if entry.name().ends_with('/') {
                std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut out_file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
            }
        }
        let _ = std::fs::remove_file(&zip_path);
    }

    // ── macOS / Linux: .tar.gz ───────────────────────────────────────────────
    #[cfg(not(target_os = "windows"))]
    {
        let tar_path = app_data_dir.join(format!("jre-{}.tar.gz", version_major));
        download_to_file(app, instance_name, resp, total_size, &tar_path).await?;

        app.emit(
            "jre-progress",
            ProgressPayload {
                instance_name: instance_name.to_string(),
                progress: 85,
                message: "Extrayendo binarios de Java (TAR)...".into(),
            },
        )
        .map_err(|e| e.to_string())?;

        let tar_gz = std::fs::File::open(&tar_path).map_err(|e| e.to_string())?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(&java_dir).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&tar_path);
    }

    app.emit(
        "jre-progress",
        ProgressPayload {
            instance_name: instance_name.to_string(),
            progress: 100,
            message: "JRE listo.".into(),
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(java_dir.to_string_lossy().to_string())
}

/// Descarga la respuesta HTTP a un archivo local emitiendo progreso.
async fn download_to_file(
    app: &AppHandle,
    instance_name: &str,
    resp: reqwest::Response,
    total_size: u64,
    path: &std::path::Path,
) -> Result<(), String> {
    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let progress = (downloaded as f64 / total_size as f64 * 80.0) as u32;
        app.emit(
            "jre-progress",
            ProgressPayload {
                instance_name: instance_name.to_string(),
                progress,
                message: format!("Descargando JRE... {}%", progress),
            },
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
