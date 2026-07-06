use crate::state::AppState;
use ofs_core::production::{compute_production_data, ProductionData};
use tauri::State;

#[tauri::command]
pub fn get_production_data_project(
    state: State<'_, AppState>,
) -> Result<Vec<ProductionData>, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let result: Vec<ProductionData> = project
        .kozijnen
        .iter()
        .map(|k| compute_production_data(k))
        .collect();
    Ok(result)
}
