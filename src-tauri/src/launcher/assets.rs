use std::io::Write;
use tauri::{AppHandle, Manager, Emitter};

use crate::models::{AssetIndexInfo, ProgressPayload};

/// Descarga todos los assets (texturas, sonidos, idiomas) de un índice de Mojang.
///
/// Maneja los tres tipos de índice:
/// - **Moderno** (MC ≥1.8): hashes en `assets/objects/`, devuelve `assets/`
/// - **Virtual / legacy** (MC 1.7.x): copia a `assets/virtual/legacy/`, devuelve esa ruta
/// - **map_to_resources** (MC ≤1.6): copia a `<appData>/resources/`
pub async fn asegurar_assets(
    app: &AppHandle,
    asset_index: &AssetIndexInfo,
    instance_name: &str,
) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let assets_dir = app_data_dir.join("assets");
    let indexes_dir = assets_dir.join("indexes");
    let objects_dir = assets_dir.join("objects");

    std::fs::create_dir_all(&indexes_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&objects_dir).map_err(|e| e.to_string())?;

    // ── Descargar índice (con caché) ─────────────────────────────────────────
    let index_path = indexes_dir.join(format!("{}.json", asset_index.id));
    let index_content = if index_path.exists() {
        std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?
    } else {
        app.emit(
            "jre-progress",
            ProgressPayload {
                instance_name: instance_name.to_string(),
                progress: 0,
                message: format!("Descargando índice de assets {}...", asset_index.id),
            },
        )
        .unwrap_or_default();

        let resp = reqwest::get(&asset_index.url)
            .await
            .map_err(|e| e.to_string())?;
        let content = resp.text().await.map_err(|e| e.to_string())?;
        std::fs::write(&index_path, &content).map_err(|e| e.to_string())?;
        content
    };

    let index: serde_json::Value =
        serde_json::from_str(&index_content).map_err(|e| e.to_string())?;
    let objects = index["objects"]
        .as_object()
        .ok_or("Asset index inválido: falta 'objects'")?;

    let is_virtual = index["virtual"].as_bool().unwrap_or(false);
    let map_to_resources = index["map_to_resources"].as_bool().unwrap_or(false);

    if is_virtual || map_to_resources {
        println!(
            "[IsoCraft] Asset index legacy (virtual={}, map_to_resources={})",
            is_virtual, map_to_resources
        );
    }

    let total = objects.len();
    let mut done = 0usize;

    // ── Descargar cada asset por hash SHA1 ───────────────────────────────────
    for (resource_name, obj) in objects {
        let hash = obj["hash"].as_str().unwrap_or_default();
        if hash.len() < 2 {
            continue;
        }

        let prefix = &hash[..2];
        let obj_dir = objects_dir.join(prefix);
        let obj_path = obj_dir.join(hash);

        if !obj_path.exists() {
            std::fs::create_dir_all(&obj_dir).map_err(|e| e.to_string())?;
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                prefix, hash
            );
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                let mut f = std::fs::File::create(&obj_path).map_err(|e| e.to_string())?;
                f.write_all(&bytes).map_err(|e| e.to_string())?;
            }
        }

        // Virtual legacy: copiar a ruta estructurada
        if is_virtual {
            let virt = assets_dir
                .join("virtual")
                .join("legacy")
                .join(resource_name);
            if !virt.exists() {
                if let Some(parent) = virt.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                std::fs::copy(&obj_path, &virt).map_err(|e| e.to_string())?;
            }
        }

        // map_to_resources: copiar a <appData>/resources/
        if map_to_resources {
            let res = app_data_dir.join("resources").join(resource_name);
            if !res.exists() {
                if let Some(parent) = res.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                std::fs::copy(&obj_path, &res).map_err(|e| e.to_string())?;
            }
        }

        done += 1;
        if done.is_multiple_of(100) {
            let pct = (done as f64 / total as f64 * 100.0) as u32;
            app.emit(
                "jre-progress",
                ProgressPayload {
                    instance_name: instance_name.to_string(),
                    progress: pct,
                    message: format!("Assets {}/{}", done, total),
                },
            )
            .unwrap_or_default();
        }
    }

    println!(
        "[IsoCraft] Assets completos: {} procesados de {}",
        done, total
    );

    // Retornar ruta correcta para --assetsDir
    if is_virtual {
        Ok(assets_dir
            .join("virtual")
            .join("legacy")
            .to_string_lossy()
            .to_string())
    } else {
        Ok(assets_dir.to_string_lossy().to_string())
    }
}
