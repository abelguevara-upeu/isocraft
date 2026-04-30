use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Estado global compartido por todos los comandos Tauri.
/// Mantiene un mapa de procesos activos de Minecraft por nombre de instancia.
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
