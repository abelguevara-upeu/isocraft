use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use crate::models::{VersionDetail, LoaderVersionMapping};
use super::ModLoaderManager;
use crate::launcher::context::LauncherContext;
use crate::launcher::cache::{self, TTL_1H, TTL_PERMANENT};

pub struct FabricManager;

const LOADER_CACHE:   &str = "fabric-loaders.json";
const GAME_CACHE:     &str = "fabric-games.json";
const LOADER_URL:     &str = "https://meta.fabricmc.net/v2/versions/loader";
const GAME_URL:       &str = "https://meta.fabricmc.net/v2/versions/game";

#[async_trait]
impl ModLoaderManager for FabricManager {
    async fn resolve_manifest(
        &self,
        version_id: &str,
        loader_version: &str,
        vanilla_json: serde_json::Value,
        _ctx: &LauncherContext,
    ) -> Result<VersionDetail, String> {
        // Permanent cache: a Fabric profile for a given MC+Loader pair never changes
        let cache_file = format!("fabric-{}-{}.json", version_id, loader_version);
        let cache_path = cache::meta_dir().join(&cache_file);
        let fabric_url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
            version_id, loader_version
        );

        let fabric_json = cache::fetch_cached(&fabric_url, &cache_path, TTL_PERMANENT).await
            .map_err(|e| format!("Fabric profile not found for MC {} with Loader {}: {}", version_id, loader_version, e))?;

        // Use Vanilla as the authoritative base (it has downloads, assetIndex, javaVersion)
        // Then layer Fabric-specific changes on top
        let mut final_json = vanilla_json.clone();

        if let Some(obj) = final_json.as_object_mut() {
            // 1. Use Fabric's main class (the Fabric Knot launcher)
            if let Some(main_class) = fabric_json.get("mainClass") {
                obj.insert("mainClass".to_string(), main_class.clone());
            }

            // 2. MERGE game arguments only (preserve Vanilla's JVM args which contain -cp)
            if let Some(fabric_args) = fabric_json.get("arguments").and_then(|a| a.as_object()) {
                if let Some(vanilla_args) = obj.get_mut("arguments").and_then(|a| a.as_object_mut()) {
                    if let Some(f_game) = fabric_args.get("game").and_then(|g| g.as_array()) {
                        let v_game = vanilla_args
                            .entry("game")
                            .or_insert_with(|| serde_json::Value::Array(vec![]));
                        if let Some(arr) = v_game.as_array_mut() {
                            arr.extend(f_game.clone());
                        }
                    }
                    // Fabric JVM args (like -DFabricMcEmu) are PREPENDED before Vanilla's
                    if let Some(f_jvm) = fabric_args.get("jvm").and_then(|g| g.as_array()) {
                        let v_jvm = vanilla_args
                            .entry("jvm")
                            .or_insert_with(|| serde_json::Value::Array(vec![]));
                        if let Some(arr) = v_jvm.as_array_mut() {
                            let mut new_jvm = f_jvm.clone();
                            new_jvm.extend(arr.clone());
                            *arr = new_jvm;
                        }
                    }
                }
            }

            // 3. For pre-1.13 versions: APPEND Fabric's legacy args to Vanilla's
            if let Some(f_mc_args) = fabric_json.get("minecraftArguments").and_then(|a| a.as_str()) {
                if let Some(v_mc_args) = obj.get("minecraftArguments").and_then(|a| a.as_str()) {
                    obj.insert(
                        "minecraftArguments".to_string(),
                        serde_json::Value::String(format!("{} {}", v_mc_args, f_mc_args)),
                    );
                }
            }

            // 4. MERGE libraries: Fabric's libraries come FIRST in classpath.
            //    Deduplicate by groupId:artifactId to avoid duplicate class errors
            //    (e.g. Fabric ships asm-9.9 while Vanilla ships asm-9.6 for the same MC version)
            if let Some(fabric_libs) = fabric_json.get("libraries").and_then(|l| l.as_array()) {
                if let Some(vanilla_libs) = obj.get("libraries").and_then(|l| l.as_array()).cloned() {
                    // Build a set of groupId:artifactId already present from Fabric
                    let fabric_coords: std::collections::HashSet<String> = fabric_libs
                        .iter()
                        .filter_map(|lib| lib.get("name").and_then(|n| n.as_str()))
                        .map(|name| {
                            let parts: Vec<&str> = name.splitn(3, ':').collect();
                            if parts.len() >= 2 {
                                format!("{}:{}", parts[0], parts[1])
                            } else {
                                name.to_string()
                            }
                        })
                        .collect();

                    // Only include Vanilla libs not already covered by Fabric
                    let deduped_vanilla: Vec<serde_json::Value> = vanilla_libs
                        .into_iter()
                        .filter(|lib| {
                            let coord = lib.get("name")
                                .and_then(|n| n.as_str())
                                .map(|name| {
                                    let parts: Vec<&str> = name.splitn(3, ':').collect();
                                    if parts.len() >= 2 {
                                        format!("{}:{}", parts[0], parts[1])
                                    } else {
                                        name.to_string()
                                    }
                                })
                                .unwrap_or_default();
                            !fabric_coords.contains(&coord)
                        })
                        .collect();

                    let mut merged = fabric_libs.clone();
                    merged.extend(deduped_vanilla);
                    obj.insert("libraries".to_string(), serde_json::Value::Array(merged));
                }
            }

            // NOTE: javaVersion is intentionally kept from Vanilla (authoritative source)
        }

        serde_json::from_value(final_json)
            .map_err(|e| format!("Failed to parse merged Fabric manifest: {}", e))
    }

    async fn apply_launch_patches(
        &self,
        _ctx: &LauncherContext,
        _cmd: &mut Command,
        _detail: &VersionDetail,
        _ph: &HashMap<String, String>,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn filter_compatible_versions(
        &self,
        vanilla_releases: Vec<String>,
    ) -> Result<Vec<LoaderVersionMapping>, String> {
        let meta = cache::meta_dir();

        // 1. Latest stable loader (cached 1h - updates are rare)
        let loaders: Vec<serde_json::Value> = serde_json::from_value(
            cache::fetch_cached(LOADER_URL, &meta.join(LOADER_CACHE), TTL_1H).await?
        ).map_err(|e| e.to_string())?;

        let latest_stable_loader = loaders
            .iter()
            .find(|v| v.get("stable").and_then(|s| s.as_bool()).unwrap_or(false))
            .and_then(|v| v.get("version").and_then(|s| s.as_str()))
            .ok_or("No stable Fabric Loader version found")?
            .to_string();

        // 2. Fabric-supported stable game versions (cached 1h)
        let fabric_games: Vec<serde_json::Value> = serde_json::from_value(
            cache::fetch_cached(GAME_URL, &meta.join(GAME_CACHE), TTL_1H).await?
        ).map_err(|e| e.to_string())?;

        let fabric_stable_games: std::collections::HashSet<String> = fabric_games
            .into_iter()
            .filter(|v| v.get("stable").and_then(|s| s.as_bool()).unwrap_or(false))
            .filter_map(|v| v.get("version").and_then(|s| s.as_str()).map(|s| s.to_string()))
            .collect();

        // 3. Filter and map, preserving Mojang's ordering
        Ok(vanilla_releases
            .into_iter()
            .filter(|r| fabric_stable_games.contains(r))
            .map(|minecraft| LoaderVersionMapping {
                minecraft,
                loader: latest_stable_loader.clone(),
            })
            .collect())
    }
}
