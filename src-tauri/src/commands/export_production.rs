use crate::state::AppState;
use ofs_core::production::compute_production_data;
use tauri::State;

#[tauri::command]
pub async fn export_production_lists(
    state: State<'_, AppState>,
    output_path: String,
    format: String,
) -> Result<String, String> {
    // Compute production data and serialize while holding the lock,
    // then drop the lock before the async Python call.
    let production_json = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        let production_data: Vec<_> = project
            .kozijnen
            .iter()
            .map(|k| compute_production_data(k))
            .collect();
        serde_json::to_string(&production_data).map_err(|e| e.to_string())?
    }; // MutexGuard dropped here

    let output = crate::state::python_command()
        .current_dir("../python")
        .arg("main.py")
        .arg("generate-production-lists")
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg(&format)
        .arg("--production-json")
        .arg(&production_json)
        .output()
        .await
        .map_err(|e| format!("Python sidecar fout: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Productiestaten generatie mislukt: {}", stderr));
    }

    Ok(output_path)
}
