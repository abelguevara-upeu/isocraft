use std::io::Write;
use tauri::{AppHandle, Manager, Emitter};

use crate::models::{LibraryEntry, ProgressPayload};

/// Descarga todas las librerías al directorio compartido `<appData>/libraries/`
/// y devuelve el classpath con las rutas absolutas de cada JAR.
pub async fn asegurar_librerias(
    app: &AppHandle,
    libraries: &[LibraryEntry],
    instance_name: &str,
) -> Result<Vec<String>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let libs_dir = app_data_dir.join("libraries");

    let mut classpath = Vec::new();
    let total = libraries.len();

    for (index, lib) in libraries.iter().enumerate() {
        let downloads = match &lib.downloads {
            Some(d) => d,
            None => continue,
        };
        let artifact = match &downloads.artifact {
            Some(a) => a,
            None => continue,
        };

        // URL vacía → librería sin descarga (runtime, natives-only, etc.)
        if artifact.url.is_empty() {
            continue;
        }

        let target_path = libs_dir.join(&artifact.path);
        classpath.push(target_path.to_string_lossy().to_string());

        if target_path.exists() {
            continue; // Ya en caché
        }

        // Progreso cada 5 librerías para no saturar el IPC
        if index % 5 == 0 {
            let pct = (index as f64 / total as f64 * 100.0) as u32;
            app.emit(
                "jre-progress",
                ProgressPayload {
                    instance_name: instance_name.to_string(),
                    progress: pct,
                    message: format!("Librerías {}/{}", index, total),
                },
            )
            .unwrap_or_default();
        }

        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let resp = reqwest::get(&artifact.url)
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
            let mut file =
                std::fs::File::create(&target_path).map_err(|e| e.to_string())?;
            file.write_all(&bytes).map_err(|e| e.to_string())?;
        }
    }

    Ok(classpath)
}
