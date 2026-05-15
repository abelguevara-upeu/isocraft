use std::collections::HashMap;
use crate::launcher::utils;

/// Substitutes `${key}` placeholders in a string using the provided values map.
pub fn substitute_placeholders(template: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in placeholders {
        let token = format!("${{{}}}", key);
        result = result.replace(&token, value);
    }
    result
}

/// Resolves a list of arguments from Mojang's JSON, following conditional rules.
pub fn resolve_argument_list(
    args: &[serde_json::Value],
    placeholders: &HashMap<String, String>,
) -> Vec<String> {
    let current_os = utils::get_os_key();
    let mut resolved = Vec::new();

    for arg in args {
        if let Some(s) = arg.as_str() {
            resolved.push(substitute_placeholders(s, placeholders));
        } else if let Some(obj) = arg.as_object() {
            let rules = match obj.get("rules").and_then(|r| r.as_array()) {
                Some(r) => r,
                None => continue,
            };

            if check_rules(rules, current_os) {
                if let Some(value) = obj.get("value") {
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

/// Searches for the `java` / `java.exe` binary within the JRE directory.
pub fn get_java_executable<P: AsRef<std::path::Path>>(base_path: P) -> Result<String, String> {
    for entry in walkdir::WalkDir::new(base_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy();
        if (name == "java" || name == "java.exe") && entry.file_type().is_file() {
            return Ok(entry.path().to_string_lossy().to_string());
        }
    }
    Err("Java binary not found".into())
}

// ─── Private Helpers ─────────────────────────────────────────────────────────

fn check_rules(rules: &[serde_json::Value], current_os: &str) -> bool {
    let mut allowed = false;
    for rule in rules {
        let action = rule.get("action").and_then(|a| a.as_str()).unwrap_or("disallow");
        let mut matches = true;

        if let Some(os_obj) = rule.get("os") {
            if let Some(os_name) = os_obj.get("name").and_then(|n| n.as_str()) {
                if os_name != current_os { matches = false; }
            }
        }

        if let Some(features) = rule.get("features").and_then(|f| f.as_object()) {
            for (_, req) in features {
                if req.as_bool().unwrap_or(false) { matches = false; }
            }
        }

        if matches { allowed = action == "allow"; }
    }
    allowed
}
