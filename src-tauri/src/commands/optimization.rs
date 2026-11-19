use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub fn optimize_project_cut_list(
    state: State<'_, AppState>,
    stock_length_mm: Option<f64>,
) -> Result<ofs_core::optimization::CutPlan, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let stock = stock_length_mm.unwrap_or(5800.0);

    Ok(ofs_core::optimization::optimize_frame_cuts(&project.kozijnen, stock))
}
