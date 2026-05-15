use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use crate::models::{VersionDetail, LoaderVersionMapping};
use super::ModLoaderManager;
use crate::launcher::context::LauncherContext;
use crate::launcher::managers::forge_shared::{installer::HeadlessInstaller, jre_bridge::ForgeEra, manifest::ManifestMerger};
use crate::launcher::cache::{self, TTL_6H};

pub struct NeoForgeManager;

const MAVEN_URL: &str = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
const MAVEN_CACHE: &str = "neoforge-maven.json";

#[async_trait]
impl ModLoaderManager for NeoForgeManager {
    async fn resolve_manifest(
        &self,
        version_id: &str,
        loader_version: &str,
        vanilla_json: serde_json::Value,
        ctx: &LauncherContext,
    ) -> Result<VersionDetail, String> {
        let installer_url = format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
            loader_version, loader_version
        );
        let forge_full = format!("{}-{}", version_id, loader_version);
        let target_version_dir = ctx.paths.root.join("versions").join(format!("{}-neoforge-{}", version_id, loader_version));
        let version_json_path = target_version_dir.join("version.json");
        let success_flag = target_version_dir.join(".isocraft_installed");

        if success_flag.exists() && version_json_path.exists() {
            println!("[NeoForge] Cache hit for {}", forge_full);
            let file = std::fs::File::open(&version_json_path).map_err(|e| e.to_string())?;
            return serde_json::from_reader(file).map_err(|e| e.to_string());
        }

        let installer_path = cache::meta_dir().join(format!("neoforge-installer-{}.jar", forge_full));
        crate::launcher::download::Downloader::ensure_file(ctx, &installer_url, &installer_path, &format!("NeoForge Installer ({})", forge_full)).await?;

        // NeoForge is ALWAYS modern
        println!("[NeoForge] Running installer...");
        HeadlessInstaller::run(ctx, version_id, loader_version, ForgeEra::NeoForge, &installer_path, &target_version_dir).await?;
        
        let file = std::fs::File::open(target_version_dir.join("version.json")).map_err(|e| e.to_string())?;
        let forge_json: serde_json::Value = serde_json::from_reader(file).map_err(|e| e.to_string())?;

        let final_json = ManifestMerger::merge(vanilla_json, forge_json)?;
        
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
        // NeoForge 1.20+ requirements
        cmd.arg("--add-modules=jdk.naming.dns");
        cmd.arg("--add-opens=java.base/java.util=ALL-UNNAMED");
        cmd.arg("--add-opens=java.base/java.lang=ALL-UNNAMED");

        #[cfg(target_os = "macos")]
        {
            cmd.arg("-Dforge.earlyWindow=false");
            cmd.arg("-Dforge.earlywindow=false");
        }

        Ok(())
    }

    async fn filter_compatible_versions(&self, vanilla_releases: Vec<String>) -> Result<Vec<LoaderVersionMapping>, String> {
        // Simplified XML parsing from before
        let cache_path = cache::meta_dir().join(MAVEN_CACHE);
        let xml_text = if let Ok(cached) = cache::fetch_cached(MAVEN_URL, &cache_path, TTL_6H).await {
            cached.as_str().unwrap_or("").to_string()
        } else {
            let resp = reqwest::get(MAVEN_URL).await.map_err(|e| e.to_string())?;
            let text = resp.text().await.map_err(|e| e.to_string())?;
            cache::write_cache(&cache_path, &serde_json::Value::String(text.clone()));
            text
        };

        let mut best_mappings: HashMap<String, String> = HashMap::new();
        for line in xml_text.lines() {
            if line.contains("<version>") {
                let v = line.trim().replace("<version>", "").replace("</version>", "");
                let parts: Vec<&str> = v.split('.').collect();
                if parts.len() < 2 { continue; }

                let major = parts[0];
                let minor = parts[1];
                
                let mc_version = if major == "20" && minor == "1" { "1.20.1".into() }
                else if major == "20" { format!("1.20.{}", minor) }
                else if major == "21" && minor == "0" { "1.21".into() }
                else if major == "21" { format!("1.21.{}", minor) }
                else { continue; };

                if vanilla_releases.contains(&mc_version) { best_mappings.insert(mc_version, v); }
            }
        }

        Ok(vanilla_releases.into_iter().filter_map(|r| {
            best_mappings.get(&r).map(|lv| LoaderVersionMapping { minecraft: r.clone(), loader: lv.clone() })
        }).collect())
    }
}
