use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};

use crate::launcher::{args, assets, game, jre, libraries, natives};
use crate::models::{GameLaunchPayload, InstanceConfig, ProgressPayload};
use crate::state::GameState;

/// Pipeline completo de lanzamiento para una instancia:
/// manifiesto → JRE → cliente → assets → librerías → natives → spawn.
#[tauri::command]
pub async fn iniciar_pipeline_dinamico(
    app: AppHandle,
    instance_name: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    // ── Leer configuración de la instancia ────────────────────────────────────
    let instance_dir = app_data_dir.join("instances").join(&instance_name);
    let config_path = instance_dir.join("instance.json");
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("No se pudo leer '{}': {}", instance_name, e))?;
    let mut instance_config: InstanceConfig =
        serde_json::from_str(&config_content).map_err(|e| format!("Config inválida: {}", e))?;

    let usuario = instance_config.username.clone();
    let version_id = instance_config.version_id.clone();
    let max_memory = instance_config.max_memory.clone();

    println!(
        "[IsoCraft] Lanzando '{}' → {} (usuario: {})",
        instance_name, version_id, usuario
    );

    emit_progress(&app, &instance_name, 5, "Descargando manifiesto de versión...")?;

    // ── a) Manifiesto Mojang ──────────────────────────────────────────────────
    let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let manifest: serde_json::Value = reqwest::get(manifest_url)
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let version_url = manifest["versions"]
        .as_array()
        .ok_or("Manifiesto inválido")?
        .iter()
        .find(|v| v["id"].as_str() == Some(&version_id))
        .and_then(|v| v["url"].as_str())
        .ok_or("Versión no encontrada en el manifiesto")?;

    // ── b) Detalle de versión ─────────────────────────────────────────────────
    emit_progress(&app, &instance_name, 10, "Descargando detalles de versión...")?;
    let detail: crate::models::VersionDetail = reqwest::get(version_url)
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let java_major = detail
        .java_version
        .as_ref()
        .map(|j| j.major_version)
        .unwrap_or(8);

    // ── c) JRE ────────────────────────────────────────────────────────────────
    emit_progress(&app, &instance_name, 15, "Verificando JRE...")?;
    let ruta_jre = jre::asegurar_jre(&app, java_major, &instance_name).await?;

    // ── d) Cliente Minecraft ──────────────────────────────────────────────────
    emit_progress(&app, &instance_name, 50, "Verificando cliente Minecraft...")?;
    let ruta_jar =
        game::asegurar_juego(&app, &version_id, &detail.downloads.client, &instance_name).await?;

    // ── e) Assets ─────────────────────────────────────────────────────────────
    emit_progress(&app, &instance_name, 60, "Verificando assets...")?;
    let assets_dir = if let Some(ref ai) = detail.asset_index {
        assets::asegurar_assets(&app, ai, &instance_name).await?
    } else {
        app_data_dir.join("assets").to_string_lossy().to_string()
    };

    // Soporte para recursos legacy (MC ≤1.6): copiar/linkear a instancia
    let global_resources = app_data_dir.join("resources");
    if global_resources.exists() {
        let instance_resources = instance_dir.join("resources");
        let _ = args::enlazar_o_copiar_recursos(&global_resources, &instance_resources);
    }

    // ── f) Librerías ──────────────────────────────────────────────────────────
    emit_progress(&app, &instance_name, 70, "Descargando librerías...")?;
    let mut classpath_parts =
        libraries::asegurar_librerias(&app, &detail.libraries, &instance_name).await?;
    classpath_parts.push(ruta_jar);

    #[cfg(target_os = "windows")]
    let sep = ";";
    #[cfg(not(target_os = "windows"))]
    let sep = ":";
    let classpath_final = classpath_parts.join(sep);

    // ── g) Natives ────────────────────────────────────────────────────────────
    emit_progress(&app, &instance_name, 85, "Extrayendo natives...")?;
    let natives_dir =
        natives::asegurar_natives(&app, &detail.libraries, &instance_dir).await?;

    // ── h) Binario java ───────────────────────────────────────────────────────
    let java_exe = args::get_java_executable(&ruta_jre)?;
    println!("[IsoCraft] Java: {}", java_exe);

    emit_progress(&app, &instance_name, 95, "Construyendo comando de lanzamiento...")?;

    // ── i) Placeholders ───────────────────────────────────────────────────────
    let game_dir_str = instance_dir.to_string_lossy().to_string();
    let asset_index_id = detail
        .assets
        .as_deref()
        .unwrap_or(&version_id)
        .to_string();

    let mut ph: HashMap<String, String> = HashMap::new();
    ph.insert("auth_player_name".into(), usuario.clone());
    ph.insert("version_name".into(), version_id.clone());
    ph.insert("game_directory".into(), game_dir_str.clone());
    ph.insert("assets_root".into(), assets_dir.clone());
    ph.insert("assets_index_name".into(), asset_index_id.clone());
    ph.insert("auth_uuid".into(), "00000000-0000-0000-0000-000000000000".into());
    ph.insert("auth_access_token".into(), "0".into());
    ph.insert("user_properties".into(), "{}".into());
    ph.insert("user_type".into(), "legacy".into());
    ph.insert("version_type".into(), "release".into());
    ph.insert("natives_directory".into(), natives_dir.clone());
    ph.insert("launcher_name".into(), "IsoCraft".into());
    ph.insert("launcher_version".into(), "1.0".into());
    ph.insert("classpath".into(), classpath_final.clone());
    ph.insert("resolution_width".into(), "854".into());
    ph.insert("resolution_height".into(), "480".into());

    // ── j) Construir comando ──────────────────────────────────────────────────
    let mut cmd = Command::new(&java_exe);

    // macOS: -XstartOnFirstThread solo para LWJGL 3
    #[cfg(target_os = "macos")]
    {
        let uses_lwjgl3 = detail
            .libraries
            .iter()
            .any(|l| l.name.starts_with("org.lwjgl:") && !l.name.starts_with("org.lwjgl.lwjgl:"));
        if uses_lwjgl3 {
            cmd.arg("-XstartOnFirstThread");
            println!("[IsoCraft] LWJGL 3 → -XstartOnFirstThread");
        } else {
            println!("[IsoCraft] LWJGL 2 → sin -XstartOnFirstThread");
        }
    }

    if let Some(ref arguments) = detail.arguments {
        // Formato moderno (MC ≥1.13)
        for arg in args::resolve_argument_list(&arguments.jvm, &ph) {
            cmd.arg(arg);
        }
        cmd.arg(format!("-Xmx{}", max_memory));
        cmd.arg(&detail.main_class);
        for arg in args::resolve_argument_list(&arguments.game, &ph) {
            cmd.arg(arg);
        }
    } else if let Some(ref mc_args) = detail.minecraft_arguments {
        // Formato legacy (MC ≤1.12.2)
        cmd.arg(format!("-Xmx{}", max_memory));
        cmd.arg(format!("-Djava.library.path={}", natives_dir));
        cmd.arg("-cp");
        cmd.arg(&classpath_final);
        cmd.arg(&detail.main_class);
        for token in mc_args.split_whitespace() {
            cmd.arg(args::substitute_placeholders(token, &ph));
        }
    } else {
        return Err("El JSON de versión no contiene ni 'arguments' ni 'minecraftArguments'".into());
    }

    cmd.current_dir(&instance_dir);

    let child = cmd
        .spawn()
        .map_err(|e| format!("Fallo al iniciar Java: {}", e))?;
    let pid = child.id();
    let child_arc = Arc::new(Mutex::new(child));

    // Registrar proceso activo
    {
        let state = app.state::<GameState>();
        let mut guard = state.child_processes.lock().unwrap();
        if let Some(old) = guard.insert(instance_name.clone(), child_arc.clone()) {
            let mut old_child = old.lock().unwrap();
            let _ = old_child.kill();
        }
    }

    // Actualizar last_played
    instance_config.last_played = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
    );
    let updated_json =
        serde_json::to_string_pretty(&instance_config).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, updated_json).map_err(|e| e.to_string())?;

    app.emit(
        "game-launched",
        GameLaunchPayload {
            instance_name: instance_name.clone(),
            pid,
            info: format!(
                "IsoCraft: [{}] PID {} ({}, Java {})",
                instance_name, pid, version_id, java_major
            ),
        },
    )
    .map_err(|e| e.to_string())?;

    // Hilo monitor: detecta cuando se cierra el juego
    let app_clone = app.clone();
    let name_clone = instance_name.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut guard = child_arc.lock().unwrap();
        match guard.try_wait() {
            Ok(Some(status)) => {
                println!("[IsoCraft] Juego '{}' cerrado: {}", name_clone, status);
                let state = app_clone.state::<GameState>();
                if let Ok(mut g) = state.child_processes.lock() {
                    g.remove(&name_clone);
                }
                let _ = app_clone.emit("game-closed", name_clone);
                break;
            }
            Ok(None) => {}
            Err(e) => {
                println!("[IsoCraft] Error monitoreando '{}': {}", name_clone, e);
                break;
            }
        }
    });

    Ok(())
}

// ─── Helper ───────────────────────────────────────────────────────────────────

fn emit_progress(
    app: &AppHandle,
    instance_name: &str,
    progress: u32,
    message: &str,
) -> Result<(), String> {
    app.emit(
        "jre-progress",
        ProgressPayload {
            instance_name: instance_name.to_string(),
            progress,
            message: message.to_string(),
        },
    )
    .map_err(|e| e.to_string())
}
