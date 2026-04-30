use tauri::{AppHandle, Manager};

use crate::models::{InstanceConfig, VersionManifest};
use crate::state::GameState;

// ─── Versiones ────────────────────────────────────────────────────────────────

/// Descarga y retorna la lista de releases desde el manifiesto oficial de Mojang.
#[tauri::command]
pub async fn obtener_versiones() -> Result<Vec<String>, String> {
    let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let manifest: VersionManifest = resp.json().await.map_err(|e| e.to_string())?;

    let releases = manifest
        .versions
        .into_iter()
        .filter(|v| v.version_type == "release")
        .map(|v| v.id)
        .collect();

    Ok(releases)
}

// ─── Ciclo de vida de instancias ──────────────────────────────────────────────

/// Crea una nueva instancia aislada con su directorio y archivo `instance.json`.
#[tauri::command]
pub async fn crear_instancia(
    app: AppHandle,
    name: String,
    version_id: String,
    username: String,
    max_memory: String,
) -> Result<InstanceConfig, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instance_dir = app_data_dir.join("instances").join(&name);

    if instance_dir.exists() {
        return Err(format!("La instancia '{}' ya existe", name));
    }

    std::fs::create_dir_all(&instance_dir).map_err(|e| e.to_string())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let config = InstanceConfig {
        name,
        version_id,
        created_at: now.to_string(),
        last_played: None,
        max_memory,
        username,
    };

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(instance_dir.join("instance.json"), json).map_err(|e| e.to_string())?;

    Ok(config)
}

/// Lista todas las instancias ordenadas por `last_played` descendente.
#[tauri::command]
pub async fn listar_instancias(app: AppHandle) -> Result<Vec<InstanceConfig>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instances_dir = app_data_dir.join("instances");

    if !instances_dir.exists() {
        return Ok(vec![]);
    }

    let mut instances = Vec::new();

    for entry in std::fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let config_path = entry.path().join("instance.json");

        if config_path.exists() {
            let content =
                std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
            if let Ok(cfg) = serde_json::from_str::<InstanceConfig>(&content) {
                instances.push(cfg);
            }
        }
    }

    instances.sort_by(|a, b| b.last_played.cmp(&a.last_played));
    Ok(instances)
}

/// Elimina una instancia: mata el proceso si está en ejecución, luego borra su directorio.
#[tauri::command]
pub async fn eliminar_instancia(app: AppHandle, name: String) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instance_dir = app_data_dir.join("instances").join(&name);

    if !instance_dir.exists() {
        return Err(format!("La instancia '{}' no existe", name));
    }

    // Matar proceso activo si lo hay
    {
        let state = app.state::<GameState>();
        let mut guard = state.child_processes.lock().unwrap();
        if let Some(child_arc) = guard.remove(&name) {
            let mut child = child_arc.lock().unwrap();
            let _ = child.kill();
        }
    }

    std::fs::remove_dir_all(&instance_dir).map_err(|e| e.to_string())?;
    Ok(())
}

/// Abre el directorio de la instancia en el explorador de archivos del SO.
#[tauri::command]
pub async fn abrir_carpeta_instancia(app: AppHandle, name: String) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let instance_dir = app_data_dir.join("instances").join(&name);

    if !instance_dir.exists() {
        return Err(format!("La instancia '{}' no existe", name));
    }

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&instance_dir)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&instance_dir)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&instance_dir)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}
