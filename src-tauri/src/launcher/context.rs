use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use reqwest::Client;
use crate::launcher::paths::LauncherPaths;
use crate::models::ProgressPayload;

/// LauncherContext acts as a shared state for the launch process.
#[derive(Clone)]
pub struct LauncherContext {
    pub app: AppHandle,
    pub paths: LauncherPaths,
    pub client: Client,
    pub instance_name: String,
}

impl LauncherContext {
    pub fn new(app: AppHandle, instance_name: String) -> Result<Self, String> {
        let paths = LauncherPaths::new(&app)?;
        let client = Client::builder()
            .user_agent("IsoCraft-Launcher/1.0")
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            app,
            paths,
            client,
            instance_name,
        })
    }

    /// Emits a progress event to the frontend in a standardized format.
    pub fn emit_progress(&self, progress: u32, message: &str) {
        let _ = self.app.emit("jre-progress", ProgressPayload {
            instance_name: self.instance_name.clone(),
            progress,
            message: message.to_string(),
        });
    }

    /// Returns the OS key compatible with Minecraft manifests (osx, windows, linux).
    pub fn os_key(&self) -> &'static str {
        crate::launcher::utils::get_os_key()
    }

    /// Gets the current instance directory path.
    pub fn instance_dir(&self) -> PathBuf {
        self.paths.instance_dir(&self.instance_name)
    }

    /// Gets the current instance configuration file path.
    pub fn instance_config_path(&self) -> PathBuf {
        self.paths.instance_config(&self.instance_name)
    }
}
