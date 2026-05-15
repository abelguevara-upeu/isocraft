use serde_json::Value;

pub struct ManifestMerger;

impl ManifestMerger {
    /// Merges a Forge version JSON onto a Vanilla base.
    /// This follows a "clean override" strategy.
    pub fn merge(vanilla: Value, forge: Value) -> Result<Value, String> {
        let mut final_json = vanilla.clone();
        
        let v_obj = final_json.as_object_mut().ok_or("Vanilla manifest is not an object")?;
        let f_obj = forge.as_object().ok_or("Forge manifest is not an object")?;

        // 1. Use Forge's main class
        if let Some(main_class) = f_obj.get("mainClass") {
            v_obj.insert("mainClass".to_string(), main_class.clone());
        }

        // 2. Merge arguments (Modern format)
        if let Some(f_args) = f_obj.get("arguments").and_then(|a| a.as_object()) {
            let v_args = v_obj.entry("arguments").or_insert_with(|| serde_json::json!({"game":[], "jvm":[]}));
            if let Some(v_args_obj) = v_args.as_object_mut() {
                // Merge game args
                if let Some(f_game) = f_args.get("game").and_then(|g| g.as_array()) {
                    let v_game = v_args_obj.entry("game").or_insert_with(|| Value::Array(vec![]));
                    if let Some(arr) = v_game.as_array_mut() {
                        arr.extend(f_game.clone());
                    }
                }
                // Merge JVM args (Forge libs first)
                if let Some(f_jvm) = f_args.get("jvm").and_then(|g| g.as_array()) {
                    let v_jvm = v_args_obj.entry("jvm").or_insert_with(|| Value::Array(vec![]));
                    if let Some(arr) = v_jvm.as_array_mut() {
                        let mut merged_jvm = f_jvm.clone();
                        merged_jvm.extend(arr.clone());
                        *arr = merged_jvm;
                    }
                }
            }
        }

        // 3. Merge Legacy arguments
        if let Some(f_mc) = f_obj.get("minecraftArguments").and_then(|a| a.as_str()) {
            if let Some(v_mc) = v_obj.get("minecraftArguments").and_then(|a| a.as_str()) {
                v_obj.insert("minecraftArguments".to_string(), Value::String(format!("{} {}", v_mc, f_mc)));
            }
        }

        // 4. Deduplicate and Merge Libraries
        if let Some(f_libs) = f_obj.get("libraries").and_then(|l| l.as_array()) {
            if let Some(v_libs) = v_obj.get("libraries").and_then(|l| l.as_array()) {
                // Deduplicate byGroupId:ArtifactId (keep Forge versions)
                let forge_coords: std::collections::HashSet<String> = f_libs.iter()
                    .filter_map(|l| l.get("name").and_then(|n| n.as_str()))
                    .map(get_coord)
                    .collect();

                let mut merged_libs = f_libs.clone();
                for lib in v_libs {
                    let coord = lib.get("name").and_then(|n| n.as_str()).map(get_coord).unwrap_or_default();
                    if !forge_coords.contains(&coord) {
                        merged_libs.push(lib.clone());
                    }
                }
                v_obj.insert("libraries".to_string(), Value::Array(merged_libs));
            }
        }

        Ok(final_json)
    }
}

fn get_coord(name: &str) -> String {
    let parts: Vec<&str> = name.splitn(3, ':').collect();
    if parts.len() >= 2 { format!("{}:{}", parts[0], parts[1]) }
    else { name.to_string() }
}
