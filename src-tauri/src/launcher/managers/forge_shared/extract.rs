use std::path::Path;
use std::fs::{File, create_dir_all};
use std::io::{copy, Read};
use zip::ZipArchive;

/// Extracts version.json or install_profile.json from a Forge installer JAR.
pub fn extract_version_json_from_jar(jar_path: &Path) -> Result<serde_json::Value, String> {
    let file = File::open(jar_path).map_err(|e| format!("Failed to open jar: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;

    // Try root version.json (Modern)
    if let Ok(mut entry) = archive.by_name("version.json") {
        let mut s = String::new();
        entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
        return serde_json::from_str(&s).map_err(|e| e.to_string());
    }

    // Try install_profile.json -> versionInfo (Legacy)
    if let Ok(mut entry) = archive.by_name("install_profile.json") {
        let mut s = String::new();
        entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
        let profile: serde_json::Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
        if let Some(info) = profile.get("versionInfo") {
            return Ok(info.clone());
        }
    }

    Err("Could not find version.json or install_profile.json in Forge installer JAR".into())
}

/// Extracts the bundled maven/ directory from the installer JAR.
pub fn extract_bundled_maven_libs(jar_path: &Path, libs_dir: &Path) -> Result<(), String> {
    let file = File::open(jar_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();

        if name.starts_with("maven/") && !entry.is_dir() {
            let relative_path = &name["maven/".len()..];
            let target = libs_dir.join(relative_path);
            if !target.exists() {
                if let Some(parent) = target.parent() { create_dir_all(parent).map_err(|e| e.to_string())?; }
                let mut out = File::create(&target).map_err(|e| e.to_string())?;
                copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

/// Recursive directory copy.
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
