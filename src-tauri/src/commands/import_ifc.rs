#[tauri::command]
pub fn import_ifc_file(file_path: String) -> Result<String, String> {
    let result = ofs_core::import::ifc_import::parse_ifc_file(&file_path)?;
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_own_ifc_project(
    state: tauri::State<'_, crate::state::AppState>,
    file_path: String,
) -> Result<ofs_core::kozijn::Project, String> {
    let restored = ofs_core::import::ifc_import::reconstruct_own_parts_project_file(&file_path)?;
    let mut current = state.project.lock().map_err(|e| e.to_string())?;
    let mut path = state.project_path.lock().map_err(|e| e.to_string())?;
    if !current.kozijnen.is_empty()
        || !current.vliesgevels.is_empty()
        || !current.combinations.is_empty()
        || !current.custom_profiles.is_empty()
        || !current.custom_sjablonen.is_empty()
        || path.is_some()
    {
        return Err("Maak eerst een nieuw leeg project om een IFC-basisproject te herstellen".into());
    }
    *current = restored.clone();
    *path = None;
    Ok(restored)
}
