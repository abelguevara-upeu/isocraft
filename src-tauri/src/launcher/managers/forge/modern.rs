use serde_json::Value;
use crate::launcher::context::LauncherContext;
use crate::launcher::managers::forge_shared::{installer::HeadlessInstaller, jre_bridge::ForgeEra};

pub struct ModernHandler;

impl ModernHandler {
    pub async fn process(
        ctx: &LauncherContext,
        mc_version: &str,
        loader_version: &str,
        installer_path: &std::path::Path,
        target_version_dir: &std::path::Path,
    ) -> Result<Value, String> {
        println!("[Forge] Processing Modern era (≥ 1.13)...");
        HeadlessInstaller::run(ctx, mc_version, loader_version, ForgeEra::Modern, installer_path, target_version_dir).await?;
        
        let file = std::fs::File::open(target_version_dir.join("version.json")).map_err(|e| e.to_string())?;
        let json = serde_json::from_reader(file).map_err(|e| e.to_string())?;
        Ok(json)
    }
}
