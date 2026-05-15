use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use crate::models::{VersionDetail, LoaderVersionMapping};
use super::ModLoaderManager;
use crate::launcher::context::LauncherContext;

pub struct VanillaManager;

#[async_trait]
impl ModLoaderManager for VanillaManager {
    async fn resolve_manifest(
        &self,
        _version_id: &str,
        _loader_version: &str,
        vanilla_json: serde_json::Value,
        _ctx: &LauncherContext,
    ) -> Result<VersionDetail, String> {
        serde_json::from_value(vanilla_json).map_err(|e| e.to_string())
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

    async fn filter_compatible_versions(&self, vanilla_releases: Vec<String>) -> Result<Vec<LoaderVersionMapping>, String> {
        Ok(vanilla_releases.into_iter().map(|v| LoaderVersionMapping {
            minecraft: v.clone(),
            loader: v,
        }).collect())
    }
}
