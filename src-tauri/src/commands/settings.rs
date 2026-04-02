use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default = "default_left_panel_width")]
    pub left_panel_width: f64,
    #[serde(default = "default_right_panel_width")]
    pub right_panel_width: f64,
    #[serde(default)]
    pub left_panel_open: Option<bool>,
    #[serde(default)]
    pub right_panel_open: Option<bool>,
}

fn default_theme() -> String { "default".into() }
fn default_locale() -> String { "nl".into() }
fn default_left_panel_width() -> f64 { 220.0 }
fn default_right_panel_width() -> f64 { 290.0 }

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

#[tauri::command]
pub fn load_settings(app: tauri::AppHandle) -> Result<String, String> {
    let path = settings_path(&app)?;
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        // Validate JSON parses, return as-is
        let _: AppSettings = serde_json::from_str(&content).unwrap_or_default();
        Ok(content)
    } else {
        let defaults = AppSettings {
            theme: default_theme(),
            locale: default_locale(),
            left_panel_width: default_left_panel_width(),
            right_panel_width: default_right_panel_width(),
            left_panel_open: Some(true),
            right_panel_open: Some(true),
        };
        let json = serde_json::to_string_pretty(&defaults).map_err(|e| e.to_string())?;
        std::fs::write(&path, &json).map_err(|e| e.to_string())?;
        Ok(json)
    }
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings_json: String) -> Result<(), String> {
    // Validate
    let _: AppSettings = serde_json::from_str(&settings_json)
        .map_err(|e| format!("Invalid settings: {}", e))?;
    let path = settings_path(&app)?;
    std::fs::write(&path, &settings_json).map_err(|e| e.to_string())
}
