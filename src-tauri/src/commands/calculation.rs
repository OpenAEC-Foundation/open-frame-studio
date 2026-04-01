use crate::state::AppState;
use ofs_core::calculation::{estimate_cost, CostEstimate, PriceTable};
use tauri::State;

#[tauri::command]
pub fn get_cost_estimate(
    state: State<'_, AppState>,
    id: String,
    price_table_json: Option<String>,
) -> Result<CostEstimate, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;

    let prices = match price_table_json {
        Some(json) => serde_json::from_str(&json).map_err(|e| format!("Ongeldige prijstabel: {}", e))?,
        None => PriceTable::default(),
    };

    Ok(estimate_cost(kozijn, &prices))
}

#[tauri::command]
pub fn get_cost_estimate_project(
    state: State<'_, AppState>,
    price_table_json: Option<String>,
) -> Result<Vec<CostEstimate>, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;

    let prices = match price_table_json {
        Some(json) => serde_json::from_str(&json).map_err(|e| format!("Ongeldige prijstabel: {}", e))?,
        None => PriceTable::default(),
    };

    let result: Vec<CostEstimate> = project
        .kozijnen
        .iter()
        .map(|k| estimate_cost(k, &prices))
        .collect();

    Ok(result)
}
