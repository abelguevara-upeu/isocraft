use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Mojang API structures ───────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionManifest {
    pub versions: Vec<VersionEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionDetail {
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    #[serde(default)]
    pub downloads: Option<VersionDownloads>,
    pub libraries: Vec<LibraryEntry>,
    #[serde(default)]
    pub assets: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndexInfo>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    /// Legacy format (MC ≤1.12.2): flat string with ${placeholders}
    #[serde(rename = "minecraftArguments", default)]
    pub minecraft_arguments: Option<String>,
    /// Modern format (MC ≥1.13): structured game + jvm argument arrays
    #[serde(default)]
    pub arguments: Option<ArgumentsSpec>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArgumentsSpec {
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetIndexInfo {
    pub id: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionDownloads {
    pub client: DownloadInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadInfo {
    pub url: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryEntry {
    pub downloads: Option<LibraryDownloads>,
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    /// OS rules (new format used in 1.19.x+ for platform-specific libraries)
    #[serde(default)]
    pub rules: Vec<LibraryRule>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LibraryRule {
    pub action: String,
    #[serde(default)]
    pub os: Option<LibraryRuleOs>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryRuleOs {
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Artifact {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

// ─── Event payloads ──────────────────────────────────────────────────────────

#[derive(Clone, Serialize)]
pub struct ProgressPayload {
    pub instance_name: String,
    pub progress: u32,
    pub message: String,
}

#[derive(Clone, Serialize)]
pub struct GameLaunchPayload {
    pub instance_name: String,
    pub pid: u32,
    pub info: String,
}

// ─── Instance config ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ModLoader {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
}

impl Default for ModLoader {
    fn default() -> Self {
        Self::Vanilla
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoaderVersionMapping {
    pub minecraft: String,
    pub loader: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceConfig {
    pub name: String,
    pub version_id: String,
    #[serde(default)]
    pub loader: ModLoader,
    /// The exact version of the Mod Loader (e.g., Fabric 0.15.11, NeoForge 26.1.2.31)
    #[serde(default)]
    pub loader_version: String,
    pub created_at: String,
    pub last_played: Option<String>,
    pub max_memory: String,
    pub username: String,
}
