use tauri::State;
use crate::state::AppState;
use ofs_core::circularity::{calculate_project_circularity, ProjectCircularityResult};

#[tauri::command]
pub fn get_project_circularity(
    state: State<'_, AppState>,
) -> Result<ProjectCircularityResult, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    Ok(calculate_project_circularity(&project))
}
