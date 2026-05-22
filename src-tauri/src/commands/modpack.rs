use std::path::{Path, PathBuf};
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

// ─── Modrinth mrpack Structs ─────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ModrinthIndex {
    #[serde(rename = "formatVersion")]
    pub format_version: u32,
    pub game: String,
    #[serde(rename = "versionId")]
    pub version_id: String,
    pub name: String,
    pub dependencies: std::collections::HashMap<String, String>,
    pub files: Vec<ModrinthFile>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ModrinthFile {
    pub path: String,
    pub hashes: ModrinthHashes,
    pub downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    pub file_size: u64,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ModrinthHashes {
    pub sha1: String,
    pub sha512: Option<String>,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

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

/// Helper function to extract a zip/mrpack file.
fn extract_zip(zip_path: &Path, target_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Failed to access zip file index: {}", e))?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };
        
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath).map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath).map_err(|e| format!("Failed to create output file: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to copy file from zip: {}", e))?;
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
    source_path: String,
    instance_name: String,
    username: String,
    max_memory: String,
) -> Result<InstanceConfig, String> {
    let source_path_buf = Path::new(&source_path);
    if !source_path_buf.exists() {
        return Err(format!("Source path does not exist: {}", source_path));
    }

    let paths = LauncherPaths::new(&app)?;
    
    // Determine if source is a file or a folder
    let mut temp_dir: Option<PathBuf> = None;
    let mut is_temp = false;
    
    let source_dir = if source_path_buf.is_file() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let temp_folder = paths.root.join(format!("temp_import_{}", unique_id));
        std::fs::create_dir_all(&temp_folder)
            .map_err(|e| format!("Failed to create temporary folder: {}", e))?;
            
        let _ = app.emit("modpack-progress", ModpackProgressPayload {
            instance_name: instance_name.clone(),
            progress: 2,
            message: "Extracting modpack archive...".to_string(),
        });
        
        if let Err(e) = extract_zip(&source_path_buf, &temp_folder) {
            let _ = std::fs::remove_dir_all(&temp_folder);
            return Err(e);
        }
        
        temp_dir = Some(temp_folder.clone());
        is_temp = true;
        temp_folder
    } else {
        source_path_buf.to_path_buf()
    };

    let result = import_modpack_inner(&app, &source_dir, &instance_name, &username, &max_memory, &paths).await;
    
    // Cleanup temporary folder if one was created
    if is_temp {
        if let Some(ref path) = temp_dir {
            let _ = std::fs::remove_dir_all(path);
        }
    }
    
    result
}

async fn import_modpack_inner<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    source_dir: &Path,
    instance_name: &str,
    username: &str,
    max_memory: &str,
    paths: &LauncherPaths,
) -> Result<InstanceConfig, String> {
    let modrinth_index_path = source_dir.join("modrinth.index.json");
    let curseforge_manifest_path = source_dir.join("manifest.json");

    if modrinth_index_path.exists() {
        import_modrinth_pack(app, source_dir, instance_name, username, max_memory, paths, &modrinth_index_path).await
    } else if curseforge_manifest_path.exists() {
        import_curseforge_pack(app, source_dir, instance_name, username, max_memory, paths, &curseforge_manifest_path).await
    } else {
        Err("Invalid modpack: neither manifest.json (CurseForge) nor modrinth.index.json (Modrinth) was found in the archive.".to_string())
    }
}

async fn import_curseforge_pack<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    source_dir: &Path,
    instance_name: &str,
    username: &str,
    max_memory: &str,
    paths: &LauncherPaths,
    manifest_path: &Path,
) -> Result<InstanceConfig, String> {
    // 1. Read and parse manifest.json
    let manifest_content = std::fs::read_to_string(manifest_path)
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
    let instance_dir = paths.instance_dir(instance_name);
    if instance_dir.exists() {
        return Err(format!("Instance '{}' already exists", instance_name));
    }

    // Emit initial progress
    let emit_progress = |progress: u32, message: &str| {
        let _ = app.emit("modpack-progress", ModpackProgressPayload {
            instance_name: instance_name.to_string(),
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
            
            let api_url = format!("https://api.curse.tools/v1/mods/{}/files/{}", project_id, file_id);
            let resp = client.get(&api_url).send().await.map_err(|e| e.to_string())?;
            
            if !resp.status().is_success() {
                return Err(format!("HTTP {} fetching metadata for project {} file {}", resp.status(), project_id, file_id));
            }
            
            let proxy_resp: CurseForgeProxyResponse = resp.json().await.map_err(|e| e.to_string())?;
            let file_name = proxy_resp.data.file_name;
            
            let download_url = match proxy_resp.data.download_url {
                Some(url) if !url.trim().is_empty() => url,
                _ => {
                    format!("https://edge.forgecdn.net/files/{}/{:03}/{}", file_id / 1000, file_id % 1000, file_name)
                }
            };
            
            let dest_path = mods_dir.join(&file_name);
            if dest_path.exists() {
                return Ok((idx, file_name)); // Already exists, skip download
            }
            
            let download_resp = client.get(&download_url).send().await.map_err(|e| e.to_string())?;
            if !download_resp.status().is_success() {
                return Err(format!("Failed to download mod jar from {}: HTTP {}", download_url, download_resp.status()));
            }
            
            let bytes = download_resp.bytes().await.map_err(|e| e.to_string())?;
            std::fs::write(&dest_path, &bytes).map_err(|e| format!("Failed to write mod jar: {}", e))?;
            
            Ok::<_, String>((idx, file_name))
        }
    });

    let mut stream = futures_util::stream::iter(futures).buffer_unordered(6);
    let mut downloaded = 0;
    
    while let Some(res) = stream.next().await {
        downloaded += 1;
        let progress = 15 + (((downloaded as f32) / (total_files as f32)) * 80.0) as u32;
        
        match res {
            Ok((_idx, name)) => {
                emit_progress(progress, &format!("Downloaded mod {}/{} ({}): {}", downloaded, total_files, progress, name));
            }
            Err(e) => {
                println!("[Modpack Import] Error downloading a mod: {}", e);
                emit_progress(progress, &format!("Warning: Failed to download mod {}/{}: {}", downloaded, total_files, e));
            }
        }
    }

    emit_progress(95, "Finalizing instance configuration...");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let config = InstanceConfig {
        name: instance_name.to_string(),
        version_id: minecraft_version,
        loader,
        loader_version,
        created_at: now.to_string(),
        last_played: None,
        max_memory: max_memory.to_string(),
        username: username.to_string(),
    };

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(instance_dir.join("instance.json"), json).map_err(|e| e.to_string())?;

    emit_progress(100, "Modpack installed successfully!");
    
    Ok(config)
}

async fn import_modrinth_pack<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    source_dir: &Path,
    instance_name: &str,
    username: &str,
    max_memory: &str,
    paths: &LauncherPaths,
    index_path: &Path,
) -> Result<InstanceConfig, String> {
    // 1. Read and parse modrinth.index.json
    let index_content = std::fs::read_to_string(index_path)
        .map_err(|e| format!("Failed to read modrinth.index.json: {}", e))?;
    let index: ModrinthIndex = serde_json::from_str(&index_content)
        .map_err(|e| format!("Failed to parse modrinth.index.json: {}", e))?;

    // 2. Resolve loader type and version
    let minecraft_version = index.dependencies.get("minecraft")
        .ok_or_else(|| "No minecraft version specified in dependencies".to_string())?.clone();
    
    let mut loader = ModLoader::Vanilla;
    let mut loader_version = String::new();

    if let Some(v) = index.dependencies.get("fabric-loader") {
        loader = ModLoader::Fabric;
        loader_version = v.clone();
    } else if let Some(v) = index.dependencies.get("forge") {
        loader = ModLoader::Forge;
        loader_version = v.clone();
    } else if let Some(v) = index.dependencies.get("neoforge") {
        loader = ModLoader::NeoForge;
        loader_version = v.clone();
    }

    // 3. Setup paths and check existence
    let instance_dir = paths.instance_dir(instance_name);
    if instance_dir.exists() {
        return Err(format!("Instance '{}' already exists", instance_name));
    }

    // Emit progress helper
    let emit_progress = |progress: u32, message: &str| {
        let _ = app.emit("modpack-progress", ModpackProgressPayload {
            instance_name: instance_name.to_string(),
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
        emit_progress(10, "Copying base overrides...");
        copy_dir_all(&overrides_dir, &instance_dir)
            .map_err(|e| format!("Failed to copy base overrides: {}", e))?;
    }

    let client_overrides_dir = source_dir.join("client-overrides");
    if client_overrides_dir.exists() {
        emit_progress(12, "Copying client-specific overrides...");
        copy_dir_all(&client_overrides_dir, &instance_dir)
            .map_err(|e| format!("Failed to copy client overrides: {}", e))?;
    }

    // 5. Download files concurrently
    let client = reqwest::Client::builder()
        .user_agent("IsoCraft-Launcher/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let total_files = index.files.len();
    emit_progress(15, &format!("Resolved Modrinth modpack. Downloading {} files...", total_files));

    let futures = index.files.into_iter().enumerate().map(|(idx, file)| {
        let client = client.clone();
        let instance_dir = instance_dir.clone();
        
        async move {
            let download_url = file.downloads.first()
                .ok_or_else(|| format!("No download URL for file: {}", file.path))?.clone();
            
            let dest_path = instance_dir.join(&file.path);
            
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory for {}: {}", file.path, e))?;
            }

            if dest_path.exists() {
                return Ok((idx, file.path)); // Already exists, skip download
            }

            let download_resp = client.get(&download_url).send().await.map_err(|e| e.to_string())?;
            if !download_resp.status().is_success() {
                return Err(format!("Failed to download {} from {}: HTTP {}", file.path, download_url, download_resp.status()));
            }

            let bytes = download_resp.bytes().await.map_err(|e| e.to_string())?;
            std::fs::write(&dest_path, &bytes)
                .map_err(|e| format!("Failed to write file {}: {}", file.path, e))?;
            
            Ok::<_, String>((idx, file.path))
        }
    });

    let mut stream = futures_util::stream::iter(futures).buffer_unordered(6);
    let mut downloaded = 0;

    while let Some(res) = stream.next().await {
        downloaded += 1;
        let progress = 15 + (((downloaded as f32) / (total_files as f32)) * 80.0) as u32;

        match res {
            Ok((_idx, path)) => {
                emit_progress(progress, &format!("Downloaded file {}/{} ({}%): {}", downloaded, total_files, progress - 15, path));
            }
            Err(e) => {
                println!("[Modrinth Import] Error downloading file: {}", e);
                emit_progress(progress, &format!("Warning: Failed to download: {}", e));
            }
        }
    }

    emit_progress(95, "Finalizing instance configuration...");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let config = InstanceConfig {
        name: instance_name.to_string(),
        version_id: minecraft_version,
        loader,
        loader_version,
        created_at: now.to_string(),
        last_played: None,
        max_memory: max_memory.to_string(),
        username: username.to_string(),
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
        
        let src_dir = Path::new("temp_test_modpack_src");
        let _ = std::fs::remove_dir_all(&src_dir);
        std::fs::create_dir_all(src_dir.join("overrides/config")).unwrap();
        
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
        
        let res = import_modpack(
            app.handle().clone(),
            src_dir.to_string_lossy().to_string(),
            instance_name.clone(),
            "TestUser".to_string(),
            "4G".to_string(),
        ).await;
        
        let _ = std::fs::remove_dir_all(&src_dir);
        
        let config = res.expect("Failed to import modpack");
        assert_eq!(config.name, instance_name);
        assert_eq!(config.version_id, "1.20.1");
        assert_eq!(config.loader, ModLoader::Fabric);
        assert_eq!(config.loader_version, "0.14.22");
        assert_eq!(config.username, "TestUser");
        assert_eq!(config.max_memory, "4G");
        
        let paths = LauncherPaths::new(app.handle()).unwrap();
        let target_dir = paths.instance_dir(&instance_name);
        
        assert!(target_dir.exists());
        assert!(target_dir.join("instance.json").exists());
        assert!(target_dir.join("mods").exists());
        
        let config_file = target_dir.join("config/test.txt");
        assert!(config_file.exists());
        assert_eq!(std::fs::read_to_string(&config_file).unwrap(), "hello test");
        
        let _ = std::fs::remove_dir_all(&target_dir);
    }

    #[tokio::test]
    async fn test_import_modrinth_pack_dry_run() {
        let app = tauri::test::mock_app();
        
        let src_dir = Path::new("temp_test_modrinth_src");
        let _ = std::fs::remove_dir_all(&src_dir);
        std::fs::create_dir_all(src_dir.join("overrides/config")).unwrap();
        std::fs::create_dir_all(src_dir.join("client-overrides/resourcepacks")).unwrap();
        
        let modrinth_json = r#"{
            "formatVersion": 1,
            "game": "minecraft",
            "versionId": "1.0.0",
            "name": "Test Modrinth Modpack",
            "dependencies": {
                "minecraft": "1.20.2",
                "fabric-loader": "0.15.0"
            },
            "files": []
        }"#;
        std::fs::write(src_dir.join("modrinth.index.json"), modrinth_json).unwrap();
        std::fs::write(src_dir.join("overrides/config/base.txt"), "base config").unwrap();
        std::fs::write(src_dir.join("client-overrides/resourcepacks/pack.txt"), "client resourcepack").unwrap();
        
        let instance_name = "TestModrinthInstance_Import_12345".to_string();
        
        let res = import_modpack(
            app.handle().clone(),
            src_dir.to_string_lossy().to_string(),
            instance_name.clone(),
            "ModrinthTester".to_string(),
            "6G".to_string(),
        ).await;
        
        let _ = std::fs::remove_dir_all(&src_dir);
        
        let config = res.expect("Failed to import Modrinth modpack");
        assert_eq!(config.name, instance_name);
        assert_eq!(config.version_id, "1.20.2");
        assert_eq!(config.loader, ModLoader::Fabric);
        assert_eq!(config.loader_version, "0.15.0");
        assert_eq!(config.username, "ModrinthTester");
        assert_eq!(config.max_memory, "6G");
        
        let paths = LauncherPaths::new(app.handle()).unwrap();
        let target_dir = paths.instance_dir(&instance_name);
        
        assert!(target_dir.exists());
        assert!(target_dir.join("instance.json").exists());
        
        // Check base overrides
        let base_config_file = target_dir.join("config/base.txt");
        assert!(base_config_file.exists());
        assert_eq!(std::fs::read_to_string(&base_config_file).unwrap(), "base config");
        
        // Check client overrides
        let client_file = target_dir.join("resourcepacks/pack.txt");
        assert!(client_file.exists());
        assert_eq!(std::fs::read_to_string(&client_file).unwrap(), "client resourcepack");
        
        let _ = std::fs::remove_dir_all(&target_dir);
    }
}
