mod commands;
mod launcher;
mod models;
mod state;

use commands::{
    files::{
        import_path, list_files, delete_file, list_saves, delete_save, open_in_system,
        list_datapacks, delete_datapack, import_datapack
    },
    instances::{
        open_instance_folder, create_instance, delete_instance, list_instances,
        get_versions, update_instance,
    },
    launcher::launch_instance,
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
            get_versions,
            create_instance,
            list_instances,
            delete_instance,
            open_instance_folder,
            list_files,
            import_path,
            launch_instance,
            update_instance,
            delete_file,
            list_saves,
            delete_save,
            open_in_system,
            list_datapacks,
            delete_datapack,
            import_datapack,
        ])
        .build(tauri::generate_context!())
        .expect("Error starting IsoCraft")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // Kill all active games when closing the launcher
                let state = app_handle.state::<GameState>();
                let mut guard = state.child_processes.lock().unwrap();
                for (name, child_arc) in guard.drain() {
                    let mut child = child_arc.lock().unwrap();
                    let _ = child.kill();
                    println!("[IsoCraft] Process '{}' terminated due to launcher exit", name);
                }
            }
        });
}
