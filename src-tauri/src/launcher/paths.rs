use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Centralized path manager for IsoCraft.
#[derive(Clone)]
pub struct LauncherPaths {
    pub root: PathBuf,
    pub instances: PathBuf,
    pub libraries: PathBuf,
    pub assets: PathBuf,
    pub runtimes: PathBuf,
}

impl LauncherPaths {
    /// Creates a new instance of LauncherPaths, initializing the root directory.
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let home = app.path().home_dir().map_err(|e| e.to_string())?;
        let root = home.join(".isocraft");

        if !root.exists() {
            std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        }

        Ok(Self {
            instances: root.join("instances"),
            libraries: root.join("libraries"),
            assets: root.join("assets"),
            runtimes: root.join("runtimes"),
            root,
        })
    }

    /// Returns the directory path for a specific instance.
    pub fn instance_dir(&self, name: &str) -> PathBuf {
        self.instances.join(name)
    }

    /// Returns the configuration file path for a specific instance.
    pub fn instance_config(&self, name: &str) -> PathBuf {
        self.instance_dir(name).join("instance.json")
    }
}
