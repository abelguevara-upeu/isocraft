mod commands;
mod launcher;
mod models;
mod state;

use commands::{
    files::{importar_archivo, listar_archivos},
    instances::{
        abrir_carpeta_instancia, crear_instancia, eliminar_instancia, listar_instancias,
        obtener_versiones,
    },
    launcher::iniciar_pipeline_dinamico,
};
use state::GameState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(GameState::new())
        .invoke_handler(tauri::generate_handler![
            obtener_versiones,
            crear_instancia,
            listar_instancias,
            eliminar_instancia,
            abrir_carpeta_instancia,
            listar_archivos,
            importar_archivo,
            iniciar_pipeline_dinamico,
        ])
        .build(tauri::generate_context!())
        .expect("Error al iniciar IsoCraft")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // Matar todos los juegos activos al cerrar el launcher
                let state = app_handle.state::<GameState>();
                let mut guard = state.child_processes.lock().unwrap();
                for (name, child_arc) in guard.drain() {
                    let mut child = child_arc.lock().unwrap();
                    let _ = child.kill();
                    println!("[IsoCraft] Proceso '{}' terminado por cierre del launcher", name);
                }
            }
        });
}
