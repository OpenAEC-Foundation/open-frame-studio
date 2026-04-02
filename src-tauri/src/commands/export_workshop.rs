use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_workshop_drawing(
    state: State<'_, AppState>,
    id: String,
    output_path: String,
) -> Result<String, String> {
    let (kozijn_json, project_json) = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
        let kozijn = project
            .kozijnen
            .iter()
            .find(|k| k.id == id)
            .ok_or("Kozijn niet gevonden")?;
        let kj = serde_json::to_string(kozijn).map_err(|e| e.to_string())?;
        let pj = serde_json::to_string(&*project).map_err(|e| e.to_string())?;
        (kj, pj)
    };

    let python_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Kan exe directory niet vinden")?
        .join("../python");

    let python_dir = if python_dir.exists() {
        python_dir
    } else {
        std::path::PathBuf::from("python")
    };

    let output = crate::state::python_command()
        .arg(python_dir.join("main.py"))
        .arg("generate-werkplaats-tekening")
        .arg("--output")
        .arg(&output_path)
        .arg("--kozijn-json")
        .arg(&kozijn_json)
        .arg("--project-json")
        .arg(&project_json)
        .output()
        .await
        .map_err(|e| format!("Python sidecar fout: {}", e))?;

    if output.status.success() {
        Ok(output_path)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Werkplaatstekening export mislukt: {}", stderr))
    }
}
