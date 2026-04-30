use tauri::{AppHandle, Manager};

/// Lista los archivos de una carpeta concreta (`mods` o `resourcepacks`) dentro
/// del directorio aislado de la instancia.
#[tauri::command]
pub async fn listar_archivos(
    app: AppHandle,
    instance_name: String,
    folder: String,
) -> Result<Vec<String>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let target_dir = app_data_dir
        .join("instances")
        .join(&instance_name)
        .join(&folder);

    if !target_dir.exists() {
        return Ok(Vec::new());
    }

    let mut archivos = Vec::new();
    for entry in std::fs::read_dir(&target_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry
            .file_type()
            .map_err(|e| e.to_string())?
            .is_file()
        {
            archivos.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    Ok(archivos)
}

/// Copia un archivo externo (`.jar` o `.zip`) a la carpeta de la instancia indicada.
#[tauri::command]
pub async fn importar_archivo(
    app: AppHandle,
    instance_name: String,
    folder: String,
    source_path: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let target_dir = app_data_dir
        .join("instances")
        .join(&instance_name)
        .join(&folder);

    std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let source = std::path::Path::new(&source_path);
    if !source.exists() || !source.is_file() {
        return Err("El archivo origen no existe o no es válido".into());
    }

    let file_name = source.file_name().ok_or("Nombre de archivo inválido")?;
    let target_path = target_dir.join(file_name);

    std::fs::copy(&source, &target_path).map_err(|e| e.to_string())?;
    Ok(())
}
