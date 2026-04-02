use crate::state::AppState;
use ofs_core::vliesgevel::{CurtainPanel, CurtainPanelType, Vliesgevel};
use ofs_core::vliesgevel::geometry::{compute_vliesgevel_2d, VliesgevelGeometry2D};
use ofs_core::vliesgevel::grid;
use ofs_core::vliesgevel::production::{compute_vliesgevel_production, VliesgevalProductionData};
use tauri::State;

#[tauri::command]
pub fn create_vliesgevel(
    state: State<'_, AppState>,
    name: String,
    mark: String,
    width: f64,
    height: f64,
    mullion_spacing: f64,
    transom_spacing: f64,
) -> Result<Vliesgevel, String> {
    let mut vg = grid::create_regular_grid(width, height, mullion_spacing, transom_spacing);
    vg.name = name;
    vg.mark = mark;
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    project.vliesgevels.push(vg.clone());
    Ok(vg)
}

#[tauri::command]
pub fn create_vliesgevel_from_template(
    state: State<'_, AppState>,
    template: String,
    width: f64,
    height: f64,
) -> Result<Vliesgevel, String> {
    let vg = match template.as_str() {
        "stick_system" => grid::template_stick_system(width, height),
        "unitized" => grid::template_unitized(width, height),
        "shopfront" => grid::template_shopfront(width, height),
        _ => grid::create_regular_grid(width, height, 1500.0, 1200.0),
    };
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    project.vliesgevels.push(vg.clone());
    Ok(vg)
}

#[tauri::command]
pub fn get_vliesgevel(state: State<'_, AppState>, id: String) -> Result<Vliesgevel, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    project
        .vliesgevels
        .iter()
        .find(|v| v.id == id)
        .cloned()
        .ok_or_else(|| "Vliesgevel niet gevonden".into())
}

#[tauri::command]
pub fn get_all_vliesgevels(state: State<'_, AppState>) -> Result<Vec<Vliesgevel>, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    Ok(project.vliesgevels.clone())
}

#[tauri::command]
pub fn vliesgevel_add_mullion(
    state: State<'_, AppState>,
    id: String,
    x_position: f64,
) -> Result<Vliesgevel, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let vg = project.vliesgevels.iter_mut().find(|v| v.id == id)
        .ok_or("Vliesgevel niet gevonden")?;
    vg.add_mullion(x_position);
    Ok(vg.clone())
}

#[tauri::command]
pub fn vliesgevel_add_transom(
    state: State<'_, AppState>,
    id: String,
    y_position: f64,
) -> Result<Vliesgevel, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let vg = project.vliesgevels.iter_mut().find(|v| v.id == id)
        .ok_or("Vliesgevel niet gevonden")?;
    vg.add_transom(y_position);
    Ok(vg.clone())
}

#[tauri::command]
pub fn vliesgevel_remove_mullion(
    state: State<'_, AppState>,
    id: String,
    mullion_index: usize,
) -> Result<Vliesgevel, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let vg = project.vliesgevels.iter_mut().find(|v| v.id == id)
        .ok_or("Vliesgevel niet gevonden")?;
    vg.remove_mullion(mullion_index);
    Ok(vg.clone())
}

#[tauri::command]
pub fn vliesgevel_remove_transom(
    state: State<'_, AppState>,
    id: String,
    transom_index: usize,
) -> Result<Vliesgevel, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let vg = project.vliesgevels.iter_mut().find(|v| v.id == id)
        .ok_or("Vliesgevel niet gevonden")?;
    vg.remove_transom(transom_index);
    Ok(vg.clone())
}

#[tauri::command]
pub fn vliesgevel_update_panel(
    state: State<'_, AppState>,
    id: String,
    col: usize,
    row: usize,
    panel_type: CurtainPanelType,
) -> Result<Vliesgevel, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let vg = project.vliesgevels.iter_mut().find(|v| v.id == id)
        .ok_or("Vliesgevel niet gevonden")?;
    let panel = vg.panel_at_mut(col, row)
        .ok_or("Paneel niet gevonden")?;
    panel.panel_type = panel_type;
    Ok(vg.clone())
}

#[tauri::command]
pub fn get_vliesgevel_geometry(
    state: State<'_, AppState>,
    id: String,
) -> Result<VliesgevelGeometry2D, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let vg = project.vliesgevels.iter().find(|v| v.id == id)
        .ok_or("Vliesgevel niet gevonden")?;
    Ok(compute_vliesgevel_2d(vg))
}

#[tauri::command]
pub fn get_vliesgevel_production(
    state: State<'_, AppState>,
    id: String,
) -> Result<VliesgevalProductionData, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let vg = project.vliesgevels.iter().find(|v| v.id == id)
        .ok_or("Vliesgevel niet gevonden")?;
    Ok(compute_vliesgevel_production(vg))
}

#[tauri::command]
pub fn remove_vliesgevel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    project.vliesgevels.retain(|v| v.id != id);
    Ok(())
}
