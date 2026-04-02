use std::path::PathBuf;
use tauri::Manager;

/// Load profile JSON files from the profiles/ resource directory.
/// Returns the raw JSON string of the index + all profile data merged.
#[tauri::command]
pub fn load_profile_library(app_handle: tauri::AppHandle) -> Result<String, String> {
    // Try resource dir first (production), then relative path (dev)
    let profiles_dir = app_handle
        .path()
        .resource_dir()
        .ok()
        .map(|p| p.join("profiles"))
        .filter(|p| p.exists())
        .or_else(|| {
            let dev_path = PathBuf::from("../profiles");
            if dev_path.exists() { Some(dev_path) } else { None }
        })
        .ok_or("Profiles directory niet gevonden")?;

    let index_path = profiles_dir.join("index.json");
    let index_str = std::fs::read_to_string(&index_path)
        .map_err(|e| format!("Kan index.json niet lezen: {}", e))?;

    let index: serde_json::Value = serde_json::from_str(&index_str)
        .map_err(|e| format!("index.json parse fout: {}", e))?;

    let categories = index
        .get("categories")
        .and_then(|c| c.as_array())
        .ok_or("Geen categories in index.json")?;

    let mut result = Vec::new();

    for cat in categories {
        let cat_id = cat.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let cat_label = cat.get("label").and_then(|v| v.as_str()).unwrap_or(cat_id);
        let files = cat.get("files").and_then(|f| f.as_array());

        let mut profiles = Vec::new();
        if let Some(files) = files {
            for file_val in files {
                if let Some(file) = file_val.as_str() {
                    let file_path = profiles_dir.join(file);
                    if let Ok(text) = std::fs::read_to_string(&file_path) {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(arr) = data.as_array() {
                                profiles.extend(arr.iter().cloned());
                            } else {
                                profiles.push(data);
                            }
                        }
                    }
                }
            }
        }

        result.push(serde_json::json!({
            "id": cat_id,
            "label": cat_label,
            "profiles": profiles,
        }));
    }

    serde_json::to_string(&result).map_err(|e| e.to_string())
}
