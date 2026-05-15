use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Global state shared across all Tauri commands.
/// Maintains a map of active Minecraft child processes by instance name.
pub struct GameState {
    pub child_processes: Mutex<HashMap<String, Arc<Mutex<std::process::Child>>>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            child_processes: Mutex::new(HashMap::new()),
        }
    }
}
