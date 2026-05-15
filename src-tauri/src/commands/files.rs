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
            files.push(entry.file_name().to_string_lossy().to_string());
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
