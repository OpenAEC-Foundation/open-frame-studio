use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub async fn import_dxf_profile(
    _state: State<'_, AppState>,
    file_path: String,
) -> Result<String, String> {
    let profile = ofs_core::import::dxf_profile::parse_dxf_profile(&file_path)?;
    let json = serde_json::to_string(&profile).map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
pub async fn import_catalog(
    _state: State<'_, AppState>,
    file_path: String,
    supplier: Option<String>,
) -> Result<String, String> {
    let profiles = ofs_core::import::catalog::parse_catalog(
        &file_path,
        supplier.as_deref(),
    )?;
    let json = serde_json::to_string(&profiles).map_err(|e| e.to_string())?;
    Ok(json)
}
