use crate::state::AppState;
use ofs_core::purchase_order::PurchaseOrder;
use tauri::State;

#[tauri::command]
pub fn generate_purchase_orders(
    state: State<'_, AppState>,
) -> Result<Vec<PurchaseOrder>, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    Ok(ofs_core::purchase_order::generate_purchase_orders(&project))
}
