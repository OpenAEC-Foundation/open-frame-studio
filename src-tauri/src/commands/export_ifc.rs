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

    let lod_level = parse_lod(lod.as_deref());

    ofs_core::export::ifc::generate_ifc_with_lod(&kozijn, &output_path, lod_level)?;

    Ok(output_path)
}

#[tauri::command]
pub async fn export_ifc_project(
    state: State<'_, AppState>,
    output_path: String,
    lod: Option<String>,
) -> Result<String, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?.clone();
    ofs_core::export::ifc::generate_project_ifc_with_lod(
        &project,
        &output_path,
        parse_lod(lod.as_deref()),
    )?;
    Ok(output_path)
}

#[tauri::command]
pub async fn export_ifc_library(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<String, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?.clone();
    ofs_core::export::ifc::generate_ifc_type_library(&project, &output_path)?;
    Ok(output_path)
}

#[tauri::command]
pub async fn export_ifc_parts_project(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<String, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?.clone();
    ofs_core::export::ifc::generate_ifc_parts_project(&project, &output_path)?;
    Ok(output_path)
}

#[tauri::command]
pub async fn export_manufacturer_handoff(
    state: State<'_, AppState>,
    output_dir: String,
) -> Result<String, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?.clone();
    let path = ofs_core::export::manufacturer_handoff::generate_manufacturer_handoff(
        &project,
        std::path::Path::new(&output_dir),
    )?;
    Ok(path.to_string_lossy().into_owned())
}

fn parse_lod(lod: Option<&str>) -> ofs_core::export::ifc::LodLevel {
    match lod {
        Some("200") => ofs_core::export::ifc::LodLevel::Lod200,
        Some("300") => ofs_core::export::ifc::LodLevel::Lod300,
        Some("400") => ofs_core::export::ifc::LodLevel::Lod400,
        // Unknown or absent LOD: fall back to the default level.
        _ => ofs_core::export::ifc::LodLevel::Lod300,
    }
}
