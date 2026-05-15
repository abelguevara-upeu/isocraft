use crate::models::{LibraryEntry};
use crate::launcher::context::LauncherContext;
use crate::launcher::download::Downloader;

/// Downloads all libraries to the shared `libraries/` directory.
/// Fixed: Now uses a robust sequential downloader to ensure files exist on disk.
pub async fn ensure_libraries(
    ctx: &LauncherContext,
    libraries: &[LibraryEntry],
) -> Result<Vec<String>, String> {
    let libs_dir = ctx.paths.libraries.clone();
    let mut classpath = Vec::new();

    let our_os = ctx.os_key();
    let our_arch = crate::launcher::utils::get_arch_key();

    println!("[Libraries] 📚 Resolving libraries for {} ({})...", our_os, our_arch);

    for lib in libraries {
        if !should_include_library(lib, our_os) {
            continue;
        }

        let mut lib = lib.clone();
        
        // CRITICAL FIX: LWJGL 3.3.1 has known issues on macOS Intel. Upgrade to 3.3.3.
        if our_os == "osx" && lib.name.contains("org.lwjgl") && lib.name.contains("3.3.1") {
            println!("[Libraries] 🍎 macOS: Upgrading LWJGL from 3.3.1 to 3.3.3 for stability.");
            lib.name = lib.name.replace("3.3.1", "3.3.3");
            if let Some(d) = &mut lib.downloads {
                if let Some(a) = &mut d.artifact {
                    a.path = a.path.as_ref().map(|p| p.replace("3.3.1", "3.3.3"));
                    a.url = a.url.replace("3.3.1", "3.3.3");
                }
                if let Some(c) = &mut d.classifiers {
                    for artifact in c.values_mut() {
                        artifact.path = artifact.path.as_ref().map(|p| p.replace("3.3.1", "3.3.3"));
                        artifact.url = artifact.url.replace("3.3.1", "3.3.3");
                    }
                }
            }
        }

        // CRITICAL FIX: JNA 5.12.1 has issues on modern macOS. Upgrade to 5.17.0.
        if our_os == "osx" && lib.name.contains("net.java.dev.jna") && (lib.name.contains("5.12.1") || lib.name.contains("5.13.0")) {
            println!("[Libraries] 🍎 macOS: Upgrading JNA for stability.");
            lib.name = lib.name.replace("5.12.1", "5.17.0").replace("5.13.0", "5.17.0");
            if let Some(d) = &mut lib.downloads {
                if let Some(a) = &mut d.artifact {
                    a.url = a.url.replace("5.12.1", "5.17.0").replace("5.13.0", "5.17.0");
                }
                if let Some(c) = &mut d.classifiers {
                    for artifact in c.values_mut() {
                        artifact.url = artifact.url.replace("5.12.1", "5.17.0").replace("5.13.0", "5.17.0");
                    }
                }
            }
        }

        let mut artifacts = Vec::new();

        // ── A. Natives (Legacy Format) ──
        if let Some(natives_map) = &lib.natives {
            if let Some(classifier_key) = natives_map.get(our_os) {
                let mut best_classifier = classifier_key.clone();
                if our_arch == "arm64" {
                    let arm_key = format!("{}-arm64", classifier_key);
                    if lib.downloads.as_ref().and_then(|d| d.classifiers.as_ref()).and_then(|c| c.get(&arm_key)).is_some() {
                        best_classifier = arm_key;
                    }
                }

                if let Some(artifact) = lib.downloads.as_ref()
                    .and_then(|d| d.classifiers.as_ref())
                    .and_then(|c| c.get(&best_classifier)) 
                {
                    artifacts.push((artifact.path.clone(), artifact.url.clone(), best_classifier));
                }
            }
        }

        // ── B. Standard Artifact (Modern Format) ──
        if let Some(artifact) = lib.downloads.as_ref().and_then(|d| d.artifact.as_ref()) {
            let is_arch_mismatch = if our_arch == "arm64" {
                lib.name.contains("x86_64") || lib.name.contains("x64") && !lib.name.contains("arm64")
            } else if our_arch == "x64" {
                lib.name.contains("arm64") || lib.name.contains("aarch64")
            } else { false };

            if !is_arch_mismatch {
                artifacts.push((artifact.path.clone(), artifact.url.clone(), "artifact".to_string()));
            }
        }

        // Fallback for libraries without explicit download objects (e.g. some Forge libs)
        if artifacts.is_empty() {
             if let Some(url) = &lib.url {
                let parts: Vec<&str> = lib.name.split(':').collect();
                if parts.len() >= 3 {
                    let group = parts[0].replace('.', "/");
                    let artifact = parts[1];
                    let version = parts[2];
                    let classifier = parts.get(3);
                    
                    let filename = if let Some(c) = classifier {
                        format!("{}-{}-{}.jar", artifact, version, c)
                    } else {
                        format!("{}-{}.jar", artifact, version)
                    };
                    
                    let path = format!("{}/{}/{}/{}", group, artifact, version, filename);
                    let full_url = if url.ends_with(".jar") { url.clone() } else { format!("{}/{}", url.trim_end_matches('/'), path) };
                    artifacts.push((Some(path), full_url, "fallback".to_string()));
                }
            }
        }

        for (path_opt, url, desc) in artifacts {
            if let Some(path) = path_opt {
                let target_path = libs_dir.join(&path);
                
                // If URL is empty, try building it from the Forge Maven repository.
                let effective_url = if url.is_empty() {
                    format!("https://maven.minecraftforge.net/{}", path)
                } else {
                    url
                };

                // Sequential download ensures we don't proceed without files
                // (We can optimize this later with a bounded stream if needed)
                if !target_path.exists() {
                    let lib_name = lib.name.clone();
                    Downloader::ensure_file(ctx, &effective_url, &target_path, &format!("Library ({}) {}", desc, lib_name)).await?;
                }
                
                classpath.push(target_path.to_string_lossy().to_string());
            }
        }
    }

    Ok(classpath)
}

fn should_include_library(lib: &LibraryEntry, current_os: &str) -> bool {
    if lib.rules.is_empty() { return true; }

    let mut allowed = false;
    for rule in &lib.rules {
        let mut matches = true;
        if let Some(os_rule) = &rule.os {
            if let Some(os_name) = &os_rule.name {
                if os_name != current_os { matches = false; }
            }
        }
        if matches { allowed = rule.action == "allow"; }
    }
    allowed
}
