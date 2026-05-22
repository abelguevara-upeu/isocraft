use std::path::Path;
use tauri::Emitter;
use serde::{Deserialize, Serialize};
use crate::models::{InstanceConfig, ModLoader};
use crate::launcher::paths::LauncherPaths;
use futures_util::StreamExt;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct CurseForgeManifest {
    pub name: String,
    pub minecraft: CurseForgeMinecraft,
    pub files: Vec<CurseForgeFile>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct CurseForgeMinecraft {
    pub version: String,
    #[serde(rename = "modLoaders")]
    pub mod_loaders: Vec<CurseForgeModLoader>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct CurseForgeModLoader {
    pub id: String,
    pub primary: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct CurseForgeFile {
    #[serde(rename = "projectID")]
    pub project_id: u32,
    #[serde(rename = "fileID")]
    pub file_id: u32,
    pub required: bool,
}

#[derive(Deserialize)]
struct CurseForgeProxyResponse {
    data: CurseForgeProxyData,
}

#[derive(Deserialize)]
struct CurseForgeProxyData {
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "downloadUrl")]
    download_url: Option<String>,
}

/// Helper function to copy directories recursively.
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Payload sent to the frontend during progress updates.
#[derive(Clone, Serialize)]
struct ModpackProgressPayload {
    instance_name: String,
    progress: u32,
    message: String,
}

#[tauri::command]
pub async fn import_modpack<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    folder_path: String,
    instance_name: String,
    username: String,
    max_memory: String,
) -> Result<InstanceConfig, String> {
    let source_dir = Path::new(&folder_path);
    if !source_dir.exists() {
        return Err(format!("Source folder does not exist: {}", folder_path));
    }

    let manifest_path = source_dir.join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("manifest.json not found in {}", folder_path));
    }

    // 1. Read and parse manifest.json
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest.json: {}", e))?;
    let manifest: CurseForgeManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

    // 2. Resolve loader type and version
    let minecraft_version = manifest.minecraft.version.clone();
    let mut loader = ModLoader::Vanilla;
    let mut loader_version = String::new();

    if let Some(primary_loader) = manifest.minecraft.mod_loaders.iter().find(|l| l.primary).or(manifest.minecraft.mod_loaders.first()) {
        let id = &primary_loader.id;
        if id.starts_with("forge-") {
            loader = ModLoader::Forge;
            loader_version = id.trim_start_matches("forge-").to_string();
        } else if id.starts_with("fabric-") {
            loader = ModLoader::Fabric;
            loader_version = id.trim_start_matches("fabric-").to_string();
        } else if id.starts_with("neoforge-") {
            loader = ModLoader::NeoForge;
            loader_version = id.trim_start_matches("neoforge-").to_string();
        }
    }

    // 3. Setup paths and check existence
    let paths = LauncherPaths::new(&app)?;
    let instance_dir = paths.instance_dir(&instance_name);
    if instance_dir.exists() {
        return Err(format!("Instance '{}' already exists", instance_name));
    }

    // Emit initial progress
    let emit_progress = |progress: u32, message: &str| {
        let _ = app.emit("modpack-progress", ModpackProgressPayload {
            instance_name: instance_name.clone(),
            progress,
            message: message.to_string(),
        });
    };

    emit_progress(5, "Creating instance directory...");
    std::fs::create_dir_all(&instance_dir)
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;

    // 4. Copy Overrides if they exist
    let overrides_dir = source_dir.join("overrides");
    if overrides_dir.exists() {
        emit_progress(10, "Copying overrides and local configurations...");
        copy_dir_all(&overrides_dir, &instance_dir)
            .map_err(|e| format!("Failed to copy overrides: {}", e))?;
    }

    // 5. Create mods directory inside instance if not exists
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)
        .map_err(|e| format!("Failed to create mods directory: {}", e))?;

    // 6. Download mods in parallel/concurrently
    let client = reqwest::Client::builder()
        .user_agent("IsoCraft-Launcher/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let total_files = manifest.files.len();
    emit_progress(15, &format!("Resolved modpack. Downloading {} mods...", total_files));

    let futures = manifest.files.into_iter().enumerate().map(|(idx, file)| {
        let client = client.clone();
        let mods_dir = mods_dir.clone();
        
        async move {
            let project_id = file.project_id;
            let file_id = file.file_id;
            
            // Step 6a: Fetch details from proxy API to get file name and download URL
            let api_url = format!("https://api.curse.tools/v1/mods/{}/files/{}", project_id, file_id);
            let resp = client.get(&api_url).send().await.map_err(|e| e.to_string())?;
            
            if !resp.status().is_success() {
                return Err(format!("HTTP {} fetching metadata for project {} file {}", resp.status(), project_id, file_id));
            }
            
            let proxy_resp: CurseForgeProxyResponse = resp.json().await.map_err(|e| e.to_string())?;
            let file_name = proxy_resp.data.file_name;
            
            // If downloadUrl is present, use it. Otherwise construct fallback edge CDN URL.
            let download_url = match proxy_resp.data.download_url {
                Some(url) if !url.trim().is_empty() => url,
                _ => {
                    // Fallback to predictable edge CDN format:
                    // e.g. https://edge.forgecdn.net/files/{file_id_div_1000}/{file_id_mod_1000}/{file_name}
                    format!("https://edge.forgecdn.net/files/{}/{:03}/{}", file_id / 1000, file_id % 1000, file_name)
                }
            };
            
            let dest_path = mods_dir.join(&file_name);
            if dest_path.exists() {
                return Ok((idx, file_name)); // Already exists, skip download
            }
            
            // Step 6b: Download the mod file
            let download_resp = client.get(&download_url).send().await.map_err(|e| e.to_string())?;
            if !download_resp.status().is_success() {
                return Err(format!("Failed to download mod jar from {}: HTTP {}", download_url, download_resp.status()));
            }
            
            let bytes = download_resp.bytes().await.map_err(|e| e.to_string())?;
            std::fs::write(&dest_path, &bytes).map_err(|e| format!("Failed to write mod jar: {}", e))?;
            
            Ok::<_, String>((idx, file_name))
        }
    });

    // Run up to 6 downloads concurrently
    let mut stream = futures_util::stream::iter(futures).buffer_unordered(6);
    let mut downloaded = 0;
    
    while let Some(res) = stream.next().await {
        downloaded += 1;
        
        // Progress goes from 15% to 95%
        let progress = 15 + (((downloaded as f32) / (total_files as f32)) * 80.0) as u32;
        
        match res {
            Ok((_idx, name)) => {
                emit_progress(progress, &format!("Downloaded mod {}/{} ({}): {}", downloaded, total_files, progress, name));
            }
            Err(e) => {
                println!("[Modpack Import] Error downloading a mod: {}", e);
                // We don't fail the whole installation if a single mod fails, but we warn the user
                emit_progress(progress, &format!("Warning: Failed to download mod {}/{}: {}", downloaded, total_files, e));
            }
        }
    }

    emit_progress(95, "Finalizing instance configuration...");

    // 7. Write the instance.json config file
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let config = InstanceConfig {
        name: instance_name.clone(),
        version_id: minecraft_version,
        loader,
        loader_version,
        created_at: now.to_string(),
        last_played: None,
        max_memory,
        username,
        mods_repo: None,
    };

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(instance_dir.join("instance.json"), json).map_err(|e| e.to_string())?;

    emit_progress(100, "Modpack installed successfully!");
    
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let manifest_content = r#"{
            "minecraft": {
                "version": "1.19.2",
                "modLoaders": [
                    {
                        "id": "forge-43.3.13",
                        "primary": true
                    }
                ]
            },
            "manifestType": "minecraftModpack",
            "manifestVersion": 1,
            "name": "Test Modpack",
            "version": "1.0",
            "author": "Author",
            "files": [
                {
                    "projectID": 12345,
                    "fileID": 67890,
                    "required": true
                }
            ]
        }"#;

        let manifest: CurseForgeManifest = serde_json::from_str(manifest_content).unwrap();
        assert_eq!(manifest.minecraft.version, "1.19.2");
        assert_eq!(manifest.minecraft.mod_loaders.len(), 1);
        assert_eq!(manifest.minecraft.mod_loaders[0].id, "forge-43.3.13");
        assert!(manifest.minecraft.mod_loaders[0].primary);
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.files[0].project_id, 12345);
        assert_eq!(manifest.files[0].file_id, 67890);
    }

    #[tokio::test]
    async fn test_import_modpack_dry_run() {
        let app = tauri::test::mock_app();
        
        // Define paths inside the workspace
        let src_dir = Path::new("temp_test_modpack_src");
        let _ = std::fs::remove_dir_all(&src_dir);
        std::fs::create_dir_all(src_dir.join("overrides/config")).unwrap();
        
        // Write manifest with 0 files so we don't trigger download requests
        let manifest_json = r#"{
            "minecraft": {
                "version": "1.20.1",
                "modLoaders": [
                    {
                        "id": "fabric-0.14.22",
                        "primary": true
                    }
                ]
            },
            "manifestType": "minecraftModpack",
            "manifestVersion": 1,
            "name": "Test Fabric Modpack",
            "version": "1.0.0",
            "author": "Tester",
            "files": []
        }"#;
        std::fs::write(src_dir.join("manifest.json"), manifest_json).unwrap();
        std::fs::write(src_dir.join("overrides/config/test.txt"), "hello test").unwrap();
        
        let instance_name = "TestInstance_ImportModpack_12345".to_string();
        
        // Run import
        let res = import_modpack(
            app.handle().clone(),
            src_dir.to_string_lossy().to_string(),
            instance_name.clone(),
            "TestUser".to_string(),
            "4G".to_string(),
        ).await;
        
        // Clean up source directory immediately
        let _ = std::fs::remove_dir_all(&src_dir);
        
        // Assert the result
        let config = res.expect("Failed to import modpack");
        assert_eq!(config.name, instance_name);
        assert_eq!(config.version_id, "1.20.1");
        assert_eq!(config.loader, ModLoader::Fabric);
        assert_eq!(config.loader_version, "0.14.22");
        assert_eq!(config.username, "TestUser");
        assert_eq!(config.max_memory, "4G");
        
        // Assert directories and files in target
        let paths = LauncherPaths::new(app.handle()).unwrap();
        let target_dir = paths.instance_dir(&instance_name);
        
        assert!(target_dir.exists());
        assert!(target_dir.join("instance.json").exists());
        assert!(target_dir.join("mods").exists());
        
        let config_file = target_dir.join("config/test.txt");
        assert!(config_file.exists());
        assert_eq!(std::fs::read_to_string(&config_file).unwrap(), "hello test");
        
        // Clean up target directory
        let _ = std::fs::remove_dir_all(&target_dir);
    }
}

