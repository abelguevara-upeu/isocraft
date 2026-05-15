use std::path::Path;
use zip::ZipArchive;
use crate::models::{LibraryEntry};
use crate::launcher::context::LauncherContext;
use crate::launcher::download::Downloader;

/// Extracts natives for the current OS/Arch.
pub async fn ensure_natives(
    ctx: &LauncherContext,
    libraries: &[LibraryEntry],
) -> Result<String, String> {
    let natives_dir = ctx.instance_dir().join("natives");
    let our_os = ctx.os_key();
    let our_arch = crate::launcher::utils::get_arch_key();

    println!("[Natives] 🔍 Platform: {} | Arch: {}", our_os, our_arch);

    // Clean old natives to prevent arch-mismatch crashes (e.g. Rosetta vs Silicon)
    if natives_dir.exists() {
        let _ = std::fs::remove_dir_all(&natives_dir);
    }
    std::fs::create_dir_all(&natives_dir).map_err(|e| e.to_string())?;

    for lib in libraries {
        if let Some(natives_map) = &lib.natives {
            if let Some(classifier_key) = natives_map.get(our_os) {
                // Handle Apple Silicon natives specifically
                let mut best_classifier = classifier_key.clone();
                if our_arch == "arm64" {
                    let arm_key = format!("{}-arm64", classifier_key);
                    if lib.downloads.as_ref().and_then(|d| d.classifiers.as_ref()).and_then(|c| c.get(&arm_key)).is_some() {
                        best_classifier = arm_key;
                    }
                }

                if let Some(artifact) = lib.downloads.as_ref().and_then(|d| d.classifiers.as_ref()).and_then(|c| c.get(&best_classifier)) {
                    if !artifact.url.is_empty() {
                        let native_jar = ctx.paths.libraries.join(artifact.path.as_deref().unwrap_or(""));
                        
                        // Use unified downloader
                        Downloader::ensure_file(ctx, &artifact.url, &native_jar, &format!("Natives JAR: {}", best_classifier)).await?;
                        
                        println!("[Natives] 📦 Extracting: {}", best_classifier);
                        extract_natives_from_jar(&native_jar, &natives_dir)?;
                    }
                }
            }
        }
    }

    Ok(natives_dir.to_string_lossy().to_string())
}

fn extract_natives_from_jar(jar_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(jar_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();

        let is_binary = name.ends_with(".dylib") || name.ends_with(".so") ||
                        name.ends_with(".dll") || name.ends_with(".jnilib");

        if is_binary {
            let file_name = std::path::Path::new(&name).file_name().unwrap_or_default();
            let out_path = dest_dir.join(file_name);
            let mut out_file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
            let bytes_copied = std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
            
            println!("[Natives] ✅ Extracted {} ({} bytes)", file_name.to_string_lossy(), bytes_copied);

            // Set execute permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&out_path).map_err(|e| e.to_string())?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&out_path, perms).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
