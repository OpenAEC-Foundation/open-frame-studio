use tauri::State;
use crate::state::AppState;
use ofs_core::plausibility::{calculate_project_plausibility, ProjectPlausibilityResult};

#[tauri::command]
pub fn get_project_plausibility(
    state: State<'_, AppState>,
    wind_pressure_pa: f64,
) -> Result<ProjectPlausibilityResult, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    Ok(calculate_project_plausibility(&project, wind_pressure_pa))
}
