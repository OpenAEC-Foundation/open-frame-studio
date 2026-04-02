use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_kozijnstaat(
    state: State<'_, AppState>,
    output_path: String,
    format: String,
) -> Result<String, String> {
    let project_json = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        serde_json::to_string(&*project).map_err(|e| e.to_string())?
    };

    let python_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Cannot find exe directory")?
        .join("../python");

    let python_dir = if python_dir.exists() {
        python_dir
    } else {
        std::path::PathBuf::from("python")
    };

    let output = crate::state::python_command()
        .arg(python_dir.join("main.py"))
        .arg("generate-kozijnstaat")
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg(&format)
        .arg("--project-json")
        .arg(&project_json)
        .output()
        .await
        .map_err(|e| format!("Python sidecar fout: {}", e))?;

    if output.status.success() {
        Ok(output_path)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Kozijnstaat export mislukt: {}", stderr))
    }
}
