use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_kozijnstaat(
    state: State<'_, AppState>,
    output_path: String,
    format: String,
) -> Result<String, String> {
    let project = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        project.clone()
    };

    match format.as_str() {
        "xlsx" => {
            ofs_core::export::xlsx::generate_kozijnstaat_xlsx(&project, &output_path)?;
        }
        _ => {
            ofs_core::export::pdf::generate_kozijnstaat_pdf(&project, &output_path)?;
        }
    }

    Ok(output_path)
}
