use serde_json::Value;
use crate::launcher::context::LauncherContext;
use crate::launcher::managers::forge_shared::extract;

pub struct LegacyHandler;

impl LegacyHandler {
    pub fn process(installer_path: &std::path::Path, ctx: &LauncherContext) -> Result<Value, String> {
        println!("[Forge] Processing Legacy era (≤ 1.12.2)...");
        let json = extract::extract_version_json_from_jar(installer_path)?;
        extract::extract_bundled_maven_libs(installer_path, &ctx.paths.libraries)?;
        Ok(json)
    }
}
