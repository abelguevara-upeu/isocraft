use futures_util::StreamExt;
use std::io::Write;
use tauri::{AppHandle, Manager, Emitter};

use crate::models::{DownloadInfo, ProgressPayload};

/// Descarga el JAR del cliente de Minecraft si no está en caché.
/// Se almacena en `<appDataDir>/versions/<version_id>/<version_id>.jar`.
pub async fn asegurar_juego(
    app: &AppHandle,
    version_id: &str,
    download_info: &DownloadInfo,
    instance_name: &str,
) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let version_dir = app_data_dir.join("versions").join(version_id);
    let jar_path = version_dir.join(format!("{}.jar", version_id));

    if jar_path.exists() {
        app.emit(
            "jre-progress",
            ProgressPayload {
                instance_name: instance_name.to_string(),
                progress: 100,
                message: format!("Minecraft {} ya está en caché.", version_id),
            },
        )
        .map_err(|e| e.to_string())?;
        return Ok(jar_path.to_string_lossy().to_string());
    }

    std::fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;

    app.emit(
        "jre-progress",
        ProgressPayload {
            instance_name: instance_name.to_string(),
            progress: 0,
            message: format!("Descargando cliente Minecraft {}...", version_id),
        },
    )
    .map_err(|e| e.to_string())?;

    let resp = reqwest::get(&download_info.url)
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!(
            "Fallo al descargar Minecraft {} (HTTP {})",
            version_id,
            resp.status()
        ));
    }

    let mut file = std::fs::File::create(&jar_path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let progress = (downloaded as f64 / download_info.size as f64 * 100.0) as u32;
        app.emit(
            "jre-progress",
            ProgressPayload {
                instance_name: instance_name.to_string(),
                progress,
                message: format!("Minecraft {} → {}%", version_id, progress),
            },
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(jar_path.to_string_lossy().to_string())
}
