use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use crate::models::{VersionDetail, LoaderVersionMapping};
use super::ModLoaderManager;
use crate::launcher::context::LauncherContext;
use crate::launcher::managers::forge_shared::{jre_bridge::{ForgeEra}, manifest::ManifestMerger};
use crate::launcher::cache;

pub mod promos;
pub mod legacy;
pub mod modern;

pub struct ForgeManager;

const FORGE_MAVEN: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";

#[async_trait]
impl ModLoaderManager for ForgeManager {
    async fn resolve_manifest(
        &self,
        version_id: &str,
        loader_version: &str,
        vanilla_json: serde_json::Value,
        ctx: &LauncherContext,
    ) -> Result<VersionDetail, String> {
        if loader_version.is_empty() { return Err("No Forge version provided".into()); }

        let era = detect_era(loader_version);
        let forge_full = format!("{}-{}", version_id, loader_version);
        let target_version_dir = ctx.paths.root.join("versions").join(format!("{}-forge-{}", version_id, loader_version));
        let version_json_path = target_version_dir.join("version.json");
        let success_flag = target_version_dir.join(".isocraft_installed");

        // 1. Check Cache
        if success_flag.exists() && version_json_path.exists() {
            println!("[Forge] Cache hit for {}", forge_full);
            let file = std::fs::File::open(&version_json_path).map_err(|e| e.to_string())?;
            return serde_json::from_reader(file).map_err(|e| e.to_string());
        }

        // 2. Download Installer
        let installer_url = format!("{}/{forge_full}/forge-{forge_full}-installer.jar", FORGE_MAVEN, forge_full = forge_full);
        let installer_path = cache::meta_dir().join(format!("forge-installer-{}.jar", forge_full));
        
        crate::launcher::download::Downloader::ensure_file(ctx, &installer_url, &installer_path, &format!("Forge Installer ({})", forge_full)).await?;

        // 3. Process by Era
        let forge_json = match era {
            ForgeEra::Legacy => legacy::LegacyHandler::process(&installer_path, ctx)?,
            ForgeEra::Modern | ForgeEra::NeoForge => {
                modern::ModernHandler::process(ctx, version_id, loader_version, &installer_path, &target_version_dir).await?
            }
        };

        // 4. Merge and Finalize
        let final_json = ManifestMerger::merge(vanilla_json, forge_json)?;
        
        // Save to cache
        std::fs::create_dir_all(&target_version_dir).ok();
        std::fs::write(&version_json_path, serde_json::to_string_pretty(&final_json).unwrap()).ok();
        std::fs::write(success_flag, "OK").ok();

        serde_json::from_value(final_json).map_err(|e| e.to_string())
    }

    async fn apply_launch_patches(
        &self,
        _ctx: &LauncherContext,
        cmd: &mut Command,
        _detail: &VersionDetail,
        _ph: &HashMap<String, String>,
    ) -> Result<(), String> {
        // Shared MacOS crash fix
        #[cfg(target_os = "macos")]
        {
            cmd.arg("-Dforge.earlyWindow=false");
            cmd.arg("-Dforge.earlywindow=false");
        }

        // Era specific patches (like JRE overrides if needed at launch time)
        // Note: The JRE selection already happened in commands/launcher.rs based on resolve_manifest's output
        Ok(())
    }

    async fn filter_compatible_versions(&self, vanilla_releases: Vec<String>) -> Result<Vec<LoaderVersionMapping>, String> {
        promos::fetch_promotions(vanilla_releases).await
    }
}

fn detect_era(loader_version: &str) -> ForgeEra {
    // Forge versioning changed to 25.x for 1.13+
    let major = loader_version.split('.').next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    if major >= 25 { ForgeEra::Modern } else { ForgeEra::Legacy }
}
