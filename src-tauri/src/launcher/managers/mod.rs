use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use crate::models::{VersionDetail, ModLoader, LoaderVersionMapping};
use super::context::LauncherContext;

pub mod vanilla;
pub mod fabric;
pub mod forge_shared;
pub mod forge;
pub mod neoforge;

#[async_trait]
pub trait ModLoaderManager: Send + Sync {
    /// Resolves the manifest by merging the base version with the specific loader version.
    async fn resolve_manifest(
        &self, 
        version_id: &str, 
        loader_version: &str,
        vanilla_json: serde_json::Value,
        ctx: &LauncherContext,
    ) -> Result<VersionDetail, String>;

    /// Applies specific arguments or patches to the launch command.
    async fn apply_launch_patches(
        &self,
        ctx: &LauncherContext,
        cmd: &mut Command,
        _detail: &VersionDetail,
        _ph: &HashMap<String, String>
    ) -> Result<(), String>;

    /// Filters Minecraft versions and associates them with their corresponding Loader versions.
    async fn filter_compatible_versions(&self, vanilla_releases: Vec<String>) -> Result<Vec<LoaderVersionMapping>, String>;
}

pub fn get_manager(loader: &ModLoader) -> Box<dyn ModLoaderManager> {
    match loader {
        ModLoader::Vanilla => Box::new(vanilla::VanillaManager),
        ModLoader::Fabric => Box::new(fabric::FabricManager),
        ModLoader::Forge => Box::new(forge::ForgeManager),
        ModLoader::NeoForge => Box::new(neoforge::NeoForgeManager),
    }
}
