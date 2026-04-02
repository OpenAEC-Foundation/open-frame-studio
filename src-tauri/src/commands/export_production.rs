use crate::state::AppState;
use ofs_core::production::compute_production_data;
use tauri::State;

#[tauri::command]
pub async fn export_production_lists(
    state: State<'_, AppState>,
    output_path: String,
    format: String,
) -> Result<String, String> {
    let production_data = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        project
            .kozijnen
            .iter()
            .map(|k| compute_production_data(k))
            .collect::<Vec<_>>()
    };

    match format.as_str() {
        "csv" => {
            ofs_core::export::csv_production::generate_production_csv(
                &production_data,
                &output_path,
            )?;
        }
        "xlsx" => {
            ofs_core::export::xlsx::generate_production_xlsx(
                &production_data,
                &output_path,
            )?;
        }
        _ => {
            ofs_core::export::pdf::generate_production_pdf(
                &production_data,
                &output_path,
            )?;
        }
    }

    Ok(output_path)
}
