use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_ifc(
    state: State<'_, AppState>,
    id: String,
    output_path: String,
    lod: Option<String>,
) -> Result<String, String> {
    let kozijn = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
        project
            .kozijnen
            .iter()
            .find(|k| k.id == id)
            .ok_or("Kozijn niet gevonden")?
            .clone()
    };

    let lod_level = match lod.as_deref() {
        Some("200") => ofs_core::export::ifc::LodLevel::Lod200,
        Some("300") => ofs_core::export::ifc::LodLevel::Lod300,
        Some("400") => ofs_core::export::ifc::LodLevel::Lod400,
        // Unknown or absent LOD: fall back to the default level.
        _ => ofs_core::export::ifc::LodLevel::Lod300,
    };

    ofs_core::export::ifc::generate_ifc_with_lod(&kozijn, &output_path, lod_level)?;

    Ok(output_path)
}
