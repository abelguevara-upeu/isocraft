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

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionDetail {
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    pub downloads: VersionDownloads,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ArgumentsSpec {
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetIndexInfo {
    pub id: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionDownloads {
    pub client: DownloadInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadInfo {
    pub url: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryEntry {
    pub downloads: Option<LibraryDownloads>,
    pub name: String,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Artifact {
    pub path: String,
    #[serde(default)]
    pub url: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceConfig {
    pub name: String,
    pub version_id: String,
    pub created_at: String,
    pub last_played: Option<String>,
    pub max_memory: String,
    pub username: String,
}
