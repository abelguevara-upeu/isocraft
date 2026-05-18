use tauri::AppHandle;
use crate::launcher::paths::LauncherPaths;

/// Lists files in a specific folder (mods or resourcepacks) within the instance directory.
#[tauri::command]
pub async fn list_files(
    app: AppHandle,
    instance_name: String,
    folder: String,
) -> Result<Vec<String>, String> {
    let paths = LauncherPaths::new(&app)?;
    let target_dir = paths.instance_dir(&instance_name).join(&folder);

    if !target_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in std::fs::read_dir(&target_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            let lower_name = name.to_lowercase();
            // Filter out OS specific system/meta files
            if lower_name == ".ds_store" || lower_name == "thumbs.db" || lower_name == "desktop.ini" || lower_name == ".directory" {
                continue;
            }
            files.push(name);
        }
    }

    Ok(files)
}

/// Counts of files imported vs skipped (already existed).
#[derive(serde::Serialize)]
pub struct ImportResult {
    pub imported: u32,
    pub skipped: u32,
}

/// Internal helper: tries to copy a single .jar/.zip file into target_dir.
/// Returns true if copied, false if skipped (already exists or wrong extension).
fn copy_mod_file(source: &std::path::Path, target_dir: &std::path::Path) -> std::io::Result<bool> {
    let file_name = match source.file_name() {
        Some(n) => n,
        None => return Ok(false),
    };
    let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "jar" && ext != "zip" {
        return Ok(false);
    }
    let target_path = target_dir.join(file_name);
    if target_path.exists() {
        return Ok(false); // duplicate — skip silently
    }
    std::fs::copy(source, &target_path)?;
    Ok(true)
}

/// Imports a file OR all .jar/.zip files inside a folder into the specified instance folder.
/// Duplicates are silently skipped. Returns counts of imported and skipped files.
#[tauri::command]
pub async fn import_path(
    app: AppHandle,
    instance_name: String,
    folder: String,
    source_path: String,
) -> Result<ImportResult, String> {
    let paths = LauncherPaths::new(&app)?;
    let target_dir = paths.instance_dir(&instance_name).join(&folder);
    std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let source = std::path::Path::new(&source_path);
    if !source.exists() {
        return Err(format!("Path does not exist: {}", source_path));
    }

    let mut imported = 0u32;
    let mut skipped = 0u32;

    if source.is_file() {
        // Single file drop
        match copy_mod_file(source, &target_dir).map_err(|e| e.to_string())? {
            true  => imported += 1,
            false => skipped += 1,
        }
    } else if source.is_dir() {
        // Folder drop — import every .jar/.zip at the top level
        for entry in std::fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                match copy_mod_file(&path, &target_dir).map_err(|e| e.to_string())? {
                    true  => imported += 1,
                    false => skipped += 1,
                }
            }
        }
    }

    Ok(ImportResult { imported, skipped })
}

#[tauri::command]
pub async fn delete_file(
    app: AppHandle,
    instance_name: String,
    folder: String,
    file_name: String,
) -> Result<(), String> {
    let paths = LauncherPaths::new(&app)?;
    let file_path = paths.instance_dir(&instance_name).join(&folder).join(&file_name);

    if !file_path.exists() {
        return Err(format!("File '{}' does not exist", file_name));
    }

    if !file_path.is_file() {
        return Err("Path is not a file".to_string());
    }

    // Security check: Directory Traversal Protection
    let instance_dir = paths.instance_dir(&instance_name);
    let canonical_file = file_path.canonicalize().map_err(|e| e.to_string())?;
    let canonical_instance = instance_dir.canonicalize().map_err(|e| e.to_string())?;
    if !canonical_file.starts_with(&canonical_instance) {
        return Err("Unauthorized file path deletion".to_string());
    }

    std::fs::remove_file(&file_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Lists directories (worlds) inside the 'saves' folder of the instance directory.
#[tauri::command]
pub async fn list_saves(
    app: AppHandle,
    instance_name: String,
) -> Result<Vec<String>, String> {
    let paths = LauncherPaths::new(&app)?;
    let target_dir = paths.instance_dir(&instance_name).join("saves");

    if !target_dir.exists() {
        return Ok(Vec::new());
    }

    let mut folders = Vec::new();
    for entry in std::fs::read_dir(&target_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Filter out OS hidden files
            if name.starts_with('.') {
                continue;
            }
            folders.push(name);
        }
    }

    Ok(folders)
}

/// Deletes a save directory inside the 'saves' folder.
#[tauri::command]
pub async fn delete_save(
    app: AppHandle,
    instance_name: String,
    save_name: String,
) -> Result<(), String> {
    let paths = LauncherPaths::new(&app)?;
    let save_path = paths.instance_dir(&instance_name).join("saves").join(&save_name);

    if !save_path.exists() {
        return Err(format!("Save '{}' does not exist", save_name));
    }

    if !save_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    // Security check: Directory Traversal Protection
    let instance_dir = paths.instance_dir(&instance_name);
    let canonical_file = save_path.canonicalize().map_err(|e| e.to_string())?;
    let canonical_instance = instance_dir.canonicalize().map_err(|e| e.to_string())?;
    if !canonical_file.starts_with(&canonical_instance) {
        return Err("Unauthorized save path deletion".to_string());
    }

    std::fs::remove_dir_all(&save_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Opens a file or sub-path inside the instance directory using the system default application.
#[tauri::command]
pub async fn open_in_system(
    app: AppHandle,
    instance_name: String,
    sub_path: String,
) -> Result<(), String> {
    let paths = LauncherPaths::new(&app)?;
    let target = paths.instance_dir(&instance_name).join(&sub_path);
    if !target.exists() {
        return Err(format!("Path does not exist: {}", sub_path));
    }
    
    // Security check: Directory Traversal Protection
    let instance_dir = paths.instance_dir(&instance_name);
    let canonical_target = target.canonicalize().map_err(|e| e.to_string())?;
    let canonical_instance = instance_dir.canonicalize().map_err(|e| e.to_string())?;
    if !canonical_target.starts_with(&canonical_instance) {
        return Err("Unauthorized path".to_string());
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(canonical_target)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(canonical_target)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(canonical_target)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Helper to recursively copy directories.
fn copy_dir_all(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> std::io::Result<()> {
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

/// Lists datapacks inside saves/<world_name>/datapacks directory.
/// Returns both file names and directory names.
#[tauri::command]
pub async fn list_datapacks(
    app: AppHandle,
    instance_name: String,
    world_name: String,
) -> Result<Vec<String>, String> {
    let paths = LauncherPaths::new(&app)?;
    let target_dir = paths.instance_dir(&instance_name)
        .join("saves")
        .join(&world_name)
        .join("datapacks");

    if !target_dir.exists() {
        return Ok(Vec::new());
    }

    let mut datapacks = Vec::new();
    for entry in std::fs::read_dir(&target_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let lower_name = name.to_lowercase();
        // Skip system metadata files
        if lower_name == ".ds_store" || lower_name == "thumbs.db" || lower_name == "desktop.ini" || lower_name == ".directory" {
            continue;
        }
        datapacks.push(name);
    }
    datapacks.sort();
    Ok(datapacks)
}

/// Deletes a datapack (file or directory) safely.
#[tauri::command]
pub async fn delete_datapack(
    app: AppHandle,
    instance_name: String,
    world_name: String,
    datapack_name: String,
) -> Result<(), String> {
    let paths = LauncherPaths::new(&app)?;
    let target_path = paths.instance_dir(&instance_name)
        .join("saves")
        .join(&world_name)
        .join("datapacks")
        .join(&datapack_name);

    if !target_path.exists() {
        return Err(format!("Datapack '{}' does not exist", datapack_name));
    }

    // Security check: Traversal protection
    let instance_dir = paths.instance_dir(&instance_name);
    let canonical_target = target_path.canonicalize().map_err(|e| e.to_string())?;
    let canonical_instance = instance_dir.canonicalize().map_err(|e| e.to_string())?;
    if !canonical_target.starts_with(&canonical_instance) {
        return Err("Unauthorized file path deletion".to_string());
    }

    if target_path.is_dir() {
        std::fs::remove_dir_all(&target_path).map_err(|e| e.to_string())?;
    } else {
        std::fs::remove_file(&target_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Imports a datapack (file or folder) into saves/<world_name>/datapacks.
#[tauri::command]
pub async fn import_datapack(
    app: AppHandle,
    instance_name: String,
    world_name: String,
    source_path: String,
) -> Result<(), String> {
    let paths = LauncherPaths::new(&app)?;
    let target_dir = paths.instance_dir(&instance_name)
        .join("saves")
        .join(&world_name)
        .join("datapacks");

    std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let source = std::path::Path::new(&source_path);
    if !source.exists() {
        return Err(format!("Source path does not exist: {}", source_path));
    }

    let file_name = source.file_name()
        .ok_or_else(|| "Invalid source file name".to_string())?;
    let target_path = target_dir.join(file_name);

    if target_path.exists() {
        return Err("A datapack with this name already exists in the world".to_string());
    }

    if source.is_dir() {
        copy_dir_all(source, &target_path).map_err(|e| e.to_string())?;
    } else {
        std::fs::copy(source, &target_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

