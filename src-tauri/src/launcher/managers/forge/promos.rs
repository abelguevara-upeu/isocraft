use crate::models::LoaderVersionMapping;
use crate::launcher::cache::{self, TTL_6H};

const PROMOS_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
const PROMOS_CACHE: &str = "forge-promos.json";

pub async fn fetch_promotions(vanilla_releases: Vec<String>) -> Result<Vec<LoaderVersionMapping>, String> {
    let cache_path = cache::meta_dir().join(PROMOS_CACHE);
    let json = cache::fetch_cached(PROMOS_URL, &cache_path, TTL_6H).await?;

    let mut mappings = Vec::new();

    if let Some(promos) = json["promos"].as_object() {
        for r in &vanilla_releases {
            let recommended = format!("{}-recommended", r);
            let latest = format!("{}-latest", r);

            let loader_ver = promos.get(&recommended)
                .or_else(|| promos.get(&latest))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let Some(lv) = loader_ver {
                mappings.push(LoaderVersionMapping {
                    minecraft: r.clone(),
                    loader: lv,
                });
            }
        }
    }

    Ok(mappings)
}
