use std::collections::HashMap;

/// Sustituye `${key}` placeholders en un string usando la tabla de valores.
pub fn substitute_placeholders(template: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in placeholders {
        let token = format!("${{{}}}", key);
        result = result.replace(&token, value);
    }
    result
}

/// Procesa un array de argumentos del JSON de Mojang.
/// Cada elemento puede ser:
///   - Un string simple → se sustituyen placeholders y se incluye.
///   - Un objeto con `rules` condicionales → se evalúan según OS/features.
pub fn resolve_argument_list(
    args: &[serde_json::Value],
    placeholders: &HashMap<String, String>,
) -> Vec<String> {
    let current_os = match std::env::consts::OS {
        "macos" => "osx",
        "windows" => "windows",
        _ => "linux",
    };

    let mut resolved = Vec::new();

    for arg in args {
        if let Some(s) = arg.as_str() {
            resolved.push(substitute_placeholders(s, placeholders));
        } else if arg.is_object() {
            let rules = match arg.get("rules").and_then(|r| r.as_array()) {
                Some(r) => r,
                None => continue,
            };

            let mut allowed = false;
            for rule in rules {
                let action = rule
                    .get("action")
                    .and_then(|a| a.as_str())
                    .unwrap_or("disallow");
                let mut matches_rule = true;

                // Check OS rule
                if let Some(os_obj) = rule.get("os") {
                    if let Some(os_name) = os_obj.get("name").and_then(|n| n.as_str()) {
                        if os_name != current_os {
                            matches_rule = false;
                        }
                    }
                }

                // Check feature rules – our launcher enables no special features
                if let Some(features) = rule.get("features").and_then(|f| f.as_object()) {
                    for (_feat, required) in features {
                        if required.as_bool().unwrap_or(false) {
                            matches_rule = false;
                        }
                    }
                }

                if matches_rule {
                    allowed = action == "allow";
                }
            }

            if allowed {
                if let Some(value) = arg.get("value") {
                    if let Some(s) = value.as_str() {
                        resolved.push(substitute_placeholders(s, placeholders));
                    } else if let Some(arr) = value.as_array() {
                        for v in arr {
                            if let Some(s) = v.as_str() {
                                resolved.push(substitute_placeholders(s, placeholders));
                            }
                        }
                    }
                }
            }
        }
    }

    resolved
}

/// Busca el binario `java` / `java.exe` dentro de un árbol de directorios JRE.
pub fn get_java_executable(base_path: &str) -> Result<String, String> {
    for entry in walkdir::WalkDir::new(base_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy();
        if (name == "java" || name == "java.exe") && entry.file_type().is_file() {
            return Ok(entry.path().to_string_lossy().to_string());
        }
    }
    Err("No se pudo encontrar el binario de Java dentro del JRE descargado".into())
}

/// Enlaza (hard link) o copia recursos legacy al directorio de instancia aislada.
pub fn enlazar_o_copiar_recursos(
    src: &std::path::Path,
    dst: &std::path::Path,
) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let target_path = dst.join(entry.file_name());

        if ft.is_dir() {
            enlazar_o_copiar_recursos(&entry.path(), &target_path)?;
        } else if !target_path.exists() {
            if std::fs::hard_link(entry.path(), &target_path).is_err() {
                std::fs::copy(entry.path(), &target_path)?;
            }
        }
    }
    Ok(())
}
