use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, Emitter};

use crate::launcher::{args, assets, game, jre, libraries, natives, warnings, context::LauncherContext, managers};
use crate::models::{GameLaunchPayload, InstanceConfig};
use crate::state::GameState;

/// Clean, decoupled, and Context-oriented launch pipeline.
#[tauri::command]
pub async fn launch_instance(
    app: AppHandle,
    instance_name: String,
) -> Result<(), String> {
    // 1. Create Context (The launch backpack)
    let ctx = LauncherContext::new(app, instance_name.clone())?;
    let config_path = ctx.instance_config_path();

    // 2. Read Configuration
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Could not read config: {}", e))?;
    let mut instance_config: InstanceConfig =
        serde_json::from_str(&config_content).map_err(|e| format!("Invalid config: {}", e))?;

    println!("[IsoCraft] 🚀 Launching: {} ({})", ctx.instance_name, instance_config.version_id);

    // Check for known issues with this version on the current platform
    if let Some(warning) = warnings::get_launch_warning(&instance_config.version_id) {
        let _ = ctx.app.emit("launch-warning", serde_json::json!({
            "instance_name": ctx.instance_name,
            "version_id": instance_config.version_id,
            "message": warning
        }));
    }

    // 3. Resolve Manifest
    ctx.emit_progress(5, "Resolving manifest...");
    let vanilla_manifest = fetch_vanilla_manifest(&ctx, &instance_config.version_id).await?;

    // Pre-download Vanilla JAR and save Vanilla JSON to versions/ directory (required for installer env spoofing)
    let vanilla_dir = ctx.paths.root.join("versions").join(&instance_config.version_id);
    std::fs::create_dir_all(&vanilla_dir).map_err(|e| format!("Failed to create vanilla directory: {}", e))?;

    let vanilla_json_path = vanilla_dir.join(format!("{}.json", instance_config.version_id));
    let vanilla_json_str = serde_json::to_string_pretty(&vanilla_manifest).map_err(|e| e.to_string())?;
    std::fs::write(&vanilla_json_path, vanilla_json_str).map_err(|e| format!("Failed to write vanilla JSON: {}", e))?;

    let vanilla_detail: crate::models::VersionDetail = serde_json::from_value(vanilla_manifest.clone())
        .map_err(|e| format!("Failed to parse vanilla manifest: {}", e))?;
    if let Some(ref downloads) = vanilla_detail.downloads {
        game::ensure_game_jar(&ctx, &instance_config.version_id, &downloads.client).await?;
    }

    let manager = managers::get_manager(&instance_config.loader);
    let detail = manager.resolve_manifest(&instance_config.version_id, &instance_config.loader_version, vanilla_manifest, &ctx).await?;




    let java_major = detail.java_version.as_ref().map(|j| j.major_version).unwrap_or(8);

    // 4. Prepare Dependencies (JRE, Client, Assets, Libraries)
    ctx.emit_progress(15, "Checking JRE...");
    let jre_path = jre::ensure_jre(&ctx, java_major).await?;

    ctx.emit_progress(40, "Checking client...");
    let jar_path = if let Some(ref downloads) = detail.downloads {
        game::ensure_game_jar(&ctx, &instance_config.version_id, &downloads.client).await?
    } else {
        // Fallback: use the vanilla JAR path
        ctx.paths.root.join("versions").join(&instance_config.version_id).join(format!("{}.jar", instance_config.version_id)).to_string_lossy().to_string()
    };

    ctx.emit_progress(60, "Checking assets...");
    let assets_dir = assets::ensure_assets(&ctx, detail.asset_index.as_ref().ok_or("No asset index")?).await?;

    ctx.emit_progress(80, "Downloading libraries...");
    let mut classpath_parts = libraries::ensure_libraries(&ctx, &detail.libraries).await?;
    classpath_parts.push(jar_path.clone());

    // Deduplicate by exact file path — prevents "Duplicate key" errors in Forge's
    // UnionFileSystem when the same physical JAR is referenced from multiple sources.
    let mut seen = std::collections::HashSet::new();
    classpath_parts.retain(|p| seen.insert(p.clone()));

    // Classpath will be built after reordering in step 6

    // 5. Natives
    #[cfg(target_os = "macos")]
    let is_modern = detail.arguments.is_some();
    #[cfg(not(target_os = "macos"))]
    let is_modern = false;

    let natives_dir = if cfg!(target_os = "macos") && is_modern {
        // For modern macOS, we rely on LWJGL internal extraction from classpath
        println!("[IsoCraft] 🍎 Modern macOS detected: Using internal LWJGL native loading.");
        "".to_string()
    } else {
        ctx.emit_progress(90, "Extracting natives...");
        natives::ensure_natives(&ctx, &detail.libraries).await?
    };

    // 6. Build Command
    let java_exe = args::get_java_executable(&jre_path)?;
    println!("[IsoCraft] ☕ Using Java: {}", java_exe);

    let instance_dir = ctx.instance_dir();
    
    // FIX: Create a short-path symlink to bypass macOS snprintf crashes
    #[cfg(target_os = "macos")]
    let short_base = {
        let symlink_path = std::path::Path::new("/tmp/isocraft_link");
        let target = &ctx.paths.root;
        if !symlink_path.exists() {
            let _ = std::os::unix::fs::symlink(target, symlink_path);
        }
        symlink_path.to_string_lossy().to_string()
    };

    let mut final_classpath = vec![jar_path.clone()];
    final_classpath.extend(classpath_parts);
    
    let mut classpath_string = final_classpath.join(if cfg!(target_os = "windows") { ";" } else { ":" });
    
    // Shorten the classpath string using the symlink on macOS
    #[cfg(target_os = "macos")]
    {
        let real_base = ctx.paths.root.to_string_lossy().to_string();
        classpath_string = classpath_string.replace(&real_base, &short_base);
    }

    // 6. Build Command
    let mut java_exe = args::get_java_executable(&jre_path)?;
    let mut instance_dir_str = instance_dir.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        let real_base = ctx.paths.root.to_string_lossy().to_string();
        java_exe = java_exe.replace(&real_base, &short_base);
        instance_dir_str = instance_dir_str.replace(&real_base, &short_base);
    }
    
    println!("[IsoCraft] ☕ Using Java (Short Path): {}", java_exe);

    // FIX: Use an argument file (@argfile)
    let arg_file_path = instance_dir.join("launch_args.txt");
    let arg_file_content = format!("-classpath\n{}", classpath_string);
    std::fs::write(&arg_file_path, arg_file_content).map_err(|e| format!("Failed to write argfile: {}", e))?;

    let ph = build_placeholders(&instance_config, &detail, &ctx, &instance_dir, &assets_dir, &natives_dir, &jar_path, &classpath_string);

    let mut cmd = Command::new(&java_exe);
    cmd.current_dir(std::path::Path::new(&instance_dir_str));

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    #[cfg(target_os = "macos")]
    {
        // Minimal set for macOS
        cmd.arg("-XstartOnFirstThread");
        cmd.arg("--enable-native-access=ALL-UNNAMED");
        cmd.arg("-Djna.nosys=true");
        
        // Clean environment of Tauri/Vite garbage to save buffer space
        cmd.env_clear();
        // Restore ONLY essential vars
        if let Ok(path) = std::env::var("PATH") { cmd.env("PATH", path); }
        if let Ok(home) = std::env::var("HOME") { cmd.env("HOME", home); }
        if let Ok(user) = std::env::var("USER") { cmd.env("USER", user); }
    }

    // Use the @argfile
    let mut arg_file_arg = arg_file_path.to_string_lossy().to_string();
    #[cfg(target_os = "macos")]
    {
        let real_base = ctx.paths.root.to_string_lossy().to_string();
        arg_file_arg = arg_file_arg.replace(&real_base, &short_base);
    }
    cmd.arg(format!("@{}", arg_file_arg));

    if let Some(ref arguments) = detail.arguments {
        for arg in args::resolve_argument_list(&arguments.jvm, &ph) {
            // Skip ALL native-related properties on macOS
            if cfg!(target_os = "macos") {
                if arg.starts_with("-Djava.library.path=") || 
                   arg.starts_with("-Dorg.lwjgl.system.SharedLibraryExtractPath=") ||
                   arg.starts_with("-Djna.tmpdir=") ||
                   arg.starts_with("-Dio.netty.native.workdir=") {
                    continue;
                }
            }
            cmd.arg(arg);
        }
        manager.apply_launch_patches(&ctx, &mut cmd, &detail, &ph).await?;
        cmd.arg(format!("-Xmx{}", instance_config.max_memory));
        cmd.arg(&detail.main_class);
        for arg in args::resolve_argument_list(&arguments.game, &ph) { cmd.arg(arg); }
    } else {
        // Legacy path
        cmd.arg(format!("-Xmx{}", instance_config.max_memory));
        cmd.arg(&detail.main_class);
        if let Some(ref mc_args) = detail.minecraft_arguments {
             for token in mc_args.split_whitespace() {
                cmd.arg(args::substitute_placeholders(token, &ph));
            }
        }
    }

    // 7. Execution
    println!("[IsoCraft] 🛠️ Launching with ArgFile: @{}", arg_file_path.display());
    let child = cmd.spawn().map_err(|e| format!("Error starting Java: {}", e))?;
    let pid = child.id();
    let child_arc = Arc::new(Mutex::new(child));

    register_process(&ctx.app, &ctx.instance_name, child_arc.clone());
    update_instance(&config_path, &mut instance_config)?;

    let _ = ctx.app.emit("game-launched", GameLaunchPayload {
        instance_name: ctx.instance_name.clone(),
        pid,
        info: format!("IsoCraft: {} started (PID {})", ctx.instance_name, pid),
    });

    monitor_process(ctx.app, ctx.instance_name, child_arc);

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn fetch_vanilla_manifest(ctx: &LauncherContext, version_id: &str) -> Result<serde_json::Value, String> {
    let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let manifest: serde_json::Value = ctx.client.get(manifest_url).send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;
    let version_url = manifest["versions"].as_array().ok_or("Invalid manifest")?
        .iter().find(|v| v["id"].as_str() == Some(version_id))
        .and_then(|v| v["url"].as_str()).ok_or("Version not found")?;
    ctx.client.get(version_url).send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())
}

fn build_placeholders(
    config: &InstanceConfig,
    detail: &crate::models::VersionDetail,
    ctx: &LauncherContext,
    instance_dir: &std::path::Path,
    assets_dir: &str,
    natives_dir: &str,
    jar_path: &str,
    classpath: &str,
) -> HashMap<String, String> {
    let mut ph = HashMap::new();
    ph.insert("auth_player_name".into(), config.username.clone());
    ph.insert("version_name".into(), config.version_id.clone());
    ph.insert("game_directory".into(), instance_dir.to_string_lossy().to_string());
    ph.insert("assets_root".into(), assets_dir.to_string());
    ph.insert("assets_index_name".into(), detail.assets.as_deref().unwrap_or(&config.version_id).to_string());
    ph.insert("auth_uuid".into(), "00000000-0000-0000-0000-000000000000".into());
    ph.insert("auth_access_token".into(), "0".into());
    ph.insert("user_properties".into(), "{}".into());
    ph.insert("user_type".into(), "legacy".into());
    ph.insert("version_type".into(), "release".into());
    ph.insert("natives_directory".into(), natives_dir.to_string());
    ph.insert("launcher_name".into(), "IsoCraft".into());
    ph.insert("launcher_version".into(), "1.0".into());
    ph.insert("library_directory".into(), ctx.paths.libraries.to_string_lossy().to_string());
    ph.insert("minecraft_jar".into(), jar_path.to_string());
    ph.insert("classpath".into(), classpath.to_string());
    ph.insert("resolution_width".into(), "854".into());
    ph.insert("resolution_height".into(), "480".into());
    ph.insert("clientid".into(), "0".into());
    ph.insert("auth_xuid".into(), "0".into());
    ph.insert("xuid".into(), "0".into());
    
    // Forge specific placeholders
    #[cfg(target_os = "windows")]
    ph.insert("classpath_separator".into(), ";".into());
    #[cfg(not(target_os = "windows"))]
    ph.insert("classpath_separator".into(), ":".into());

    ph
}

fn register_process(app: &AppHandle, name: &str, child: Arc<Mutex<std::process::Child>>) {
    let state = app.state::<GameState>();
    let mut guard = state.child_processes.lock().unwrap();
    if let Some(old) = guard.insert(name.to_string(), child) {
        let _ = old.lock().unwrap().kill();
    }
}

fn update_instance<P: AsRef<std::path::Path>>(path: P, config: &mut InstanceConfig) -> Result<(), String> {
    config.last_played = Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs().to_string());
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

fn monitor_process(app: AppHandle, name: String, child_arc: Arc<Mutex<std::process::Child>>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut guard = child_arc.lock().unwrap();
        match guard.try_wait() {
            Ok(Some(status)) => {
                println!("[IsoCraft] Game '{}' closed with status: {}", name, status);
                let state = app.state::<GameState>();
                if let Ok(mut g) = state.child_processes.lock() { g.remove(&name); }
                let _ = app.emit("game-closed", name);
                break;
            }
            Ok(None) => {}
            Err(_) => break,
        }
    });
}
