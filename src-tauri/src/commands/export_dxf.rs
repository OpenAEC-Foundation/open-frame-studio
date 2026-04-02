use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_dxf(
    state: State<'_, AppState>,
    id: String,
    output_path: String,
) -> Result<String, String> {
    let kozijn = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
        project
            .kozijnen
            .iter()
            .find(|k| k.id == id)
            .ok_or("Kozijn niet gevonden")?
            .clone()
    };

    ofs_core::export::dxf::generate_dxf(&kozijn, &output_path)?;

    Ok(output_path)
}
