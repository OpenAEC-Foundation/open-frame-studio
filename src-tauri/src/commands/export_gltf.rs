use crate::state::AppState;
use tauri::State;
use tokio::process::Command;

#[tauri::command]
pub async fn export_gltf(
    state: State<'_, AppState>,
    id: String,
    output_path: String,
) -> Result<String, String> {
    let kozijn_json = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
        let kozijn = project
            .kozijnen
            .iter()
            .find(|k| k.id == id)
            .ok_or("Kozijn niet gevonden")?;
        serde_json::to_string(kozijn).map_err(|e| e.to_string())?
    };

    let output = Command::new("python")
        .current_dir("../python")
        .arg("main.py")
        .arg("generate-gltf")
        .arg("--output")
        .arg(&output_path)
        .arg("--kozijn-json")
        .arg(&kozijn_json)
        .output()
        .await
        .map_err(|e| format!("Python sidecar fout: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("glTF export mislukt: {}", stderr));
    }

    Ok(output_path)
}
