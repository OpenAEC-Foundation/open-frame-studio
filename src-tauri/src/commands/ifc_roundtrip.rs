use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn compare_ifc_files(old_path: String, new_path: String) -> Result<String, String> {
    let old = ofs_core::import::ifc_import::parse_ifc_file(&old_path)?;
    let new = ofs_core::import::ifc_import::parse_ifc_file(&new_path)?;
    let diff = ofs_core::ifc_roundtrip::compare_ifc_imports(&old, &new);
    serde_json::to_string(&diff).map_err(|e| e.to_string())
}

/// Compare an earlier OFS IFC export with the current project. Stable IFC
/// GlobalIds, dimensions, OFS properties and assembly parts are compared.
#[tauri::command]
pub fn compare_ifc_roundtrip(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<String, String> {
    let old = ofs_core::import::ifc_import::parse_ifc_file(&file_path)?;
    let project = state.project.lock().map_err(|e| e.to_string())?.clone();
    let temp_path =
        std::env::temp_dir().join(format!("ofs_roundtrip_{}.ifc", uuid::Uuid::new_v4(),));
    let temp_str = temp_path.to_string_lossy().into_owned();
    let export = if old.assemblies.is_empty() {
        ofs_core::export::ifc::generate_project_ifc_with_lod(
            &project,
            &temp_str,
            ofs_core::export::ifc::LodLevel::Lod300,
        )
    } else {
        ofs_core::export::ifc::generate_ifc_parts_project(&project, &temp_str)
    };
    if let Err(error) = export {
        let _ = std::fs::remove_file(&temp_path);
        return Err(error);
    }
    let parsed = ofs_core::import::ifc_import::parse_ifc_file(&temp_str);
    let _ = std::fs::remove_file(&temp_path);
    let new = parsed?;
    let diff = ofs_core::ifc_roundtrip::compare_ifc_imports(&old, &new);
    serde_json::to_string(&diff).map_err(|e| e.to_string())
}
