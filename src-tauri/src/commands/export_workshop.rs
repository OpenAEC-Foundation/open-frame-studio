use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_workshop_drawing(
    state: State<'_, AppState>,
    id: String,
    output_path: String,
) -> Result<String, String> {
    let (kozijn, project) = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
        let kozijn = project
            .kozijnen
            .iter()
            .find(|k| k.id == id)
            .ok_or("Kozijn niet gevonden")?
            .clone();
        (kozijn, project.clone())
    };

    ofs_core::export::pdf::generate_workshop_pdf(&kozijn, &project, &output_path)?;

    Ok(output_path)
}
