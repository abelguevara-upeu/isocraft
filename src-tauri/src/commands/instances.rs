use tauri::AppHandle;

use crate::models::{InstanceConfig, VersionManifest, LoaderVersionMapping};
use crate::launcher::{paths::LauncherPaths, managers, cache};

// ─── Versions ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_versions(loader: crate::models::ModLoader) -> Result<Vec<LoaderVersionMapping>, String> {
    // 1. Fetch Mojang version list (cached 1h)
    const MOJANG_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let cache_path = cache::meta_dir().join("mojang-releases.json");
    let raw = cache::fetch_cached(MOJANG_URL, &cache_path, cache::TTL_1H).await?;
    let manifest: VersionManifest = serde_json::from_value(raw).map_err(|e| e.to_string())?;

    let releases: Vec<String> = manifest
        .versions
        .into_iter()
        .filter(|v| v.version_type == "release")
        .map(|v| v.id)
        .collect();

    // 2. Delegate filtering and mapping to the corresponding manager
    let manager = managers::get_manager(&loader);
    manager.filter_compatible_versions(releases).await
}

// ─── Instance Lifecycle ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn create_instance(
    app: AppHandle,
    name: String,
    version_id: String,
    loader: crate::models::ModLoader,
    loader_version: String,
    username: String,
    max_memory: String,
) -> Result<InstanceConfig, String> {
    let paths = LauncherPaths::new(&app)?;
    let instance_dir = paths.instance_dir(&name);

    if instance_dir.exists() {
        return Err(format!("Instance '{}' already exists", name));
    }

    std::fs::create_dir_all(&instance_dir).map_err(|e| e.to_string())?;

    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

    let config = InstanceConfig {
        name,
        version_id,
        loader,
        loader_version,
        created_at: now.to_string(),
        last_played: None,
        max_memory,
        username,
    };

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(instance_dir.join("instance.json"), json).map_err(|e| e.to_string())?;

    Ok(config)
}

#[tauri::command]
pub async fn list_instances(app: AppHandle) -> Result<Vec<InstanceConfig>, String> {
    let paths = LauncherPaths::new(&app)?;
    let instances_dir = paths.instances;

    if !instances_dir.exists() {
        return Ok(vec![]);
    }

    let mut instances = Vec::new();
    for entry in std::fs::read_dir(&instances_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let config_path = entry.path().join("instance.json");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
            if let Ok(cfg) = serde_json::from_str::<InstanceConfig>(&content) {
                instances.push(cfg);
            }
        }
    }

    instances.sort_by(|a, b| b.last_played.cmp(&a.last_played));
    Ok(instances)
}

#[tauri::command]
pub async fn delete_instance(app: AppHandle, name: String) -> Result<(), String> {
    let paths = LauncherPaths::new(&app)?;
    let instance_dir = paths.instance_dir(&name);

    if !instance_dir.exists() {
        return Err(format!("Instance '{}' does not exist", name));
    }

    {
        use tauri::Manager;
        use crate::state::GameState;
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

#[tauri::command]
pub async fn open_instance_folder(app: AppHandle, name: String) -> Result<(), String> {
    let paths = LauncherPaths::new(&app)?;
    let instance_dir = paths.instance_dir(&name);

    if !instance_dir.exists() {
        return Err(format!("Instance '{}' does not exist", name));
    }

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&instance_dir).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&instance_dir).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&instance_dir).spawn().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_instance(
    app: AppHandle,
    current_name: String,
    new_name: String,
    version_id: String,
    loader: crate::models::ModLoader,
    loader_version: String,
    username: String,
    max_memory: String,
) -> Result<InstanceConfig, String> {
    let paths = LauncherPaths::new(&app)?;
    let current_dir = paths.instance_dir(&current_name);

    if !current_dir.exists() {
        return Err(format!("Instance '{}' does not exist", current_name));
    }

    // Check if game is currently running
    {
        use tauri::Manager;
        use crate::state::GameState;
        let state = app.state::<GameState>();
        let guard = state.child_processes.lock().unwrap();
        if guard.contains_key(&current_name) {
            return Err("Cannot modify instance while the game is running".to_string());
        }
    }

    // Handle renaming
    let final_dir = if current_name != new_name {
        let target_dir = paths.instance_dir(&new_name);
        if target_dir.exists() {
            return Err(format!("An instance named '{}' already exists", new_name));
        }

        std::fs::rename(&current_dir, &target_dir)
            .map_err(|e| format!("Failed to rename instance directory: {}", e))?;
        target_dir
    } else {
        current_dir
    };

    let config_path = final_dir.join("instance.json");
    let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut config: InstanceConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    // Update config fields
    config.name = new_name;
    config.version_id = version_id;
    config.loader = loader;
    config.loader_version = loader_version;
    config.username = username;
    config.max_memory = max_memory;

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(config_path, json).map_err(|e| e.to_string())?;

    Ok(config)
}

