use tauri::State;
use crate::state::AppState;
use ofs_core::dop::{generate_dop, DeclarationOfPerformance};

#[tauri::command]
pub fn generate_dop_for_kozijn(
    state: State<'_, AppState>,
    id: String,
) -> Result<DeclarationOfPerformance, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    let profiles = &project.custom_profiles;
    Ok(generate_dop(kozijn, profiles))
}
