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

fn default_theme() -> String { "light".into() }
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
    let content = if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        // Validate JSON parses, return as-is
        let _: AppSettings = serde_json::from_str(&content).unwrap_or_default();
        content
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
        json
    };
    if std::env::var("OPENAEC_FRAME_CONTAINER").as_deref() != Ok("1") {
        return Ok(content);
    }
    let locale = std::env::var("OPENAEC_FRAME_LOCALE").ok();
    let theme = std::env::var("OPENAEC_THEME").ok();
    let configured = apply_container_preferences(&content, locale.as_deref(), theme.as_deref())?;
    if configured != content {
        std::fs::write(path, &configured).map_err(|e| e.to_string())?;
    }
    Ok(configured)
}

fn apply_container_preferences(
    content: &str,
    locale: Option<&str>,
    theme: Option<&str>,
) -> Result<String, String> {
    let mut settings: serde_json::Value =
        serde_json::from_str(content).map_err(|e| e.to_string())?;
    let values = settings.as_object_mut().ok_or("Invalid settings object")?;
    let mut changed = false;
    for (key, value) in [("locale", locale), ("theme", theme)] {
        if let Some(value) = value.filter(|s| !s.is_empty()) {
            if values.get(key).and_then(serde_json::Value::as_str) == Some(value) {
                continue;
            }
            values.insert(key.into(), value.into());
            changed = true;
        }
    }
    Ok(if changed {
        settings.to_string()
    } else {
        content.to_owned()
    })
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings_json: String) -> Result<(), String> {
    // Validate
    let _: AppSettings = serde_json::from_str(&settings_json)
        .map_err(|e| format!("Invalid settings: {}", e))?;
    let path = settings_path(&app)?;
    std::fs::write(&path, &settings_json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::apply_container_preferences;

    #[test]
    fn container_overrides_preserve_other_preferences_and_omit_unset_values() {
        let stored = r#"{"theme":"blue","locale":"de","unrelated":17}"#;
        assert_eq!(apply_container_preferences(stored, Some("de"), None).unwrap(), stored);
        let configured = apply_container_preferences(stored, Some("tr"), Some("")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&configured).unwrap();
        assert_eq!(settings["theme"], "blue");
        assert_eq!(settings["locale"], "tr");
        assert_eq!(settings["unrelated"], 17);
        let configured = apply_container_preferences(stored, None, Some("dark")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&configured).unwrap();
        assert_eq!(settings["locale"], "de");
        assert_eq!(settings["theme"], "dark");
    }
}
