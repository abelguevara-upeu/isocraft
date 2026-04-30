use std::io::Write;
use tauri::{AppHandle, Manager};

use crate::models::LibraryEntry;

/// Descarga y extrae native libraries (`.dylib` / `.so` / `.dll` / `.jnilib`)
/// al directorio `<instanceDir>/natives/` de la instancia.
///
/// Los JARs nativos se cachean en `<appData>/libraries/` para no re-descargarlos.
pub async fn asegurar_natives(
    app: &AppHandle,
    libraries: &[LibraryEntry],
    instance_dir: &std::path::Path,
) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let natives_dir = instance_dir.join("natives");
    std::fs::create_dir_all(&natives_dir).map_err(|e| e.to_string())?;

    let os_key = match std::env::consts::OS {
        "macos" => "osx",
        "windows" => "windows",
        _ => "linux",
    };

    let mut count = 0usize;

    for lib in libraries {
        // Solo librerías que declaran nativos
        let natives_map = match &lib.natives {
            Some(n) => n,
            None => continue,
        };
        let classifier_key = match natives_map.get(os_key) {
            Some(k) => k,
            None => continue,
        };
        let downloads = match &lib.downloads {
            Some(d) => d,
            None => continue,
        };
        let classifiers = match &downloads.classifiers {
            Some(c) => c,
            None => continue,
        };
        let native_artifact = match classifiers.get(classifier_key) {
            Some(a) => a,
            None => continue,
        };

        if native_artifact.url.is_empty() {
            continue;
        }

        // ── Descargar JAR nativo al directorio compartido de librerías ────────
        let native_jar = app_data_dir
            .join("libraries")
            .join(&native_artifact.path);

        if !native_jar.exists() {
            if let Some(parent) = native_jar.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let resp = reqwest::get(&native_artifact.url)
                .await
                .map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                let mut f =
                    std::fs::File::create(&native_jar).map_err(|e| e.to_string())?;
                f.write_all(&bytes).map_err(|e| e.to_string())?;
            }
        }

        // ── Extraer archivos nativos del JAR (que es un ZIP) ──────────────────
        let file = std::fs::File::open(&native_jar).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();

            if name.starts_with("META-INF") || name.ends_with('/') {
                continue;
            }

            let is_native = name.ends_with(".dylib")
                || name.ends_with(".so")
                || name.ends_with(".dll")
                || name.ends_with(".jnilib");

            if is_native {
                let file_name = std::path::Path::new(&name)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let out_path = natives_dir.join(&file_name);
                let mut out_file =
                    std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }

    println!(
        "[IsoCraft] {} natives extraídas en {}",
        count,
        natives_dir.display()
    );
    Ok(natives_dir.to_string_lossy().to_string())
}
