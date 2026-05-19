use std::path::{Path, PathBuf};
use std::process::Command;
use crate::launcher::context::LauncherContext;
use crate::launcher::args;
use super::jre_bridge::{JreBridge, ForgeEra};
use super::extract;

pub struct HeadlessInstaller;

impl HeadlessInstaller {
    pub async fn run(
        ctx: &LauncherContext,
        mc_version: &str,
        loader_version: &str,
        era: ForgeEra,
        installer_path: &Path,
        target_version_dir: &Path,
    ) -> Result<(), String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let temp_dir = std::env::temp_dir().join(format!("isocraft_forge_{}_{}", mc_version, timestamp));
        std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

        // 1. Spoof environment: The installer needs to see a valid MC version directory
        let mc_versions_dir = temp_dir.join("versions").join(mc_version);
        std::fs::create_dir_all(&mc_versions_dir).map_err(|e| e.to_string())?;

        // FIX: Copy BOTH .jar and .json from Vanilla
        let vanilla_source_dir = ctx.paths.root.join("versions").join(mc_version);
        let vanilla_jar = vanilla_source_dir.join(format!("{}.jar", mc_version));
        let vanilla_json = vanilla_source_dir.join(format!("{}.json", mc_version));

        if vanilla_jar.exists() {
            std::fs::copy(&vanilla_jar, mc_versions_dir.join(format!("{}.jar", mc_version)))
                .map_err(|e| format!("Failed to copy vanilla JAR: {}", e))?;
        } else {
            return Err(format!("Vanilla JAR missing at {:?}. Download it first.", vanilla_jar));
        }

        if vanilla_json.exists() {
            std::fs::copy(&vanilla_json, mc_versions_dir.join(format!("{}.json", mc_version)))
                .map_err(|e| format!("Failed to copy vanilla JSON: {}", e))?;
        } else {
            // 1.19.3+ installers will FAIL without this JSON.
            println!("[ForgeShared] ⚠️ Vanilla JSON missing at {:?}. Installer might fail.", vanilla_json);
        }

        // 2. Create fake launcher_profiles.json
        let profiles_json = serde_json::json!({ "profiles": {} });
        std::fs::write(temp_dir.join("launcher_profiles.json"), profiles_json.to_string())
            .map_err(|e| e.to_string())?;

        // 3. Select correct JRE for installation phase
        let jre_dir = JreBridge::ensure_correct_jre(ctx, mc_version, era, true).await?;
        let java_exec = args::get_java_executable(jre_dir)?;

        // 4. Run installer
        println!("[ForgeShared] Running official installer for {} {} (Era: {:?})", mc_version, loader_version, era);
        let status = Command::new(java_exec)
            .arg("-jar")
            .arg(installer_path)
            .arg("--installClient")
            .arg(&temp_dir)
            .current_dir(&temp_dir)
            .status()
            .map_err(|e| format!("Failed to execute installer process: {}", e))?;

        if !status.success() {
            return Err("Forge/NeoForge installer exited with error status. Check logs for details.".into());
        }

        // 5. Post-installation: Copy generated assets
        println!("[ForgeShared] Installer success! Copying generated files...");
        
        // Libraries
        let generated_libs = temp_dir.join("libraries");
        if generated_libs.exists() {
            extract::copy_dir_all(generated_libs, &ctx.paths.libraries).map_err(|e| e.to_string())?;
        }

        // Find the generated version.json (skip the vanilla mc_version dir we spoofed)
        let search_dir = temp_dir.join("versions");
        let found_json = find_json_recursive(&search_dir, mc_version);
        
        if let Some(json_path) = found_json {
            std::fs::create_dir_all(target_version_dir).map_err(|e| e.to_string())?;
            std::fs::copy(json_path, target_version_dir.join("version.json")).map_err(|e| e.to_string())?;
        } else {
            return Err("Installer reported success but no version.json was generated in the temp environment.".into());
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
        
        Ok(())
    }
}

fn find_json_recursive(dir: &Path, skip_dirname: &str) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip the spoofed vanilla version directory
                if path.file_name().and_then(|n| n.to_str()) == Some(skip_dirname) {
                    continue;
                }
                if let Some(p) = find_json_recursive(&path, skip_dirname) { return Some(p); }
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                return Some(path);
            }
        }
    }
    None
}
