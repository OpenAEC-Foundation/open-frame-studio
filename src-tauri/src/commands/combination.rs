use tauri::State;
use crate::state::AppState;
use ofs_core::combination::*;

#[tauri::command]
pub fn create_combination(
    state: State<'_, AppState>,
    name: String,
    mark: String,
) -> Result<CombinationKozijn, String> {
    let combo = CombinationKozijn::new(&name, &mark);
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    project.combinations.push(combo.clone());
    Ok(combo)
}

#[tauri::command]
pub fn add_to_combination(
    state: State<'_, AppState>,
    combination_id: String,
    kozijn_id: String,
    offset_x: f64,
    offset_y: f64,
) -> Result<CombinationKozijn, String> {
    let combo_id: uuid::Uuid = combination_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let k_id: uuid::Uuid = kozijn_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let combo = project
        .combinations
        .iter_mut()
        .find(|c| c.id == combo_id)
        .ok_or("Combinatie niet gevonden")?;
    combo.add_member(k_id, offset_x, offset_y);
    Ok(combo.clone())
}

#[tauri::command]
pub fn add_coupling(
    state: State<'_, AppState>,
    combination_id: String,
    member_a_id: String,
    member_b_id: String,
    coupling_type: CouplingType,
    coupling_width: f64,
) -> Result<CombinationKozijn, String> {
    let combo_id: uuid::Uuid = combination_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let a_id: uuid::Uuid = member_a_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let b_id: uuid::Uuid = member_b_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let combo = project
        .combinations
        .iter_mut()
        .find(|c| c.id == combo_id)
        .ok_or("Combinatie niet gevonden")?;
    if !combo.members.iter().any(|m| m.kozijn_id == a_id)
        || !combo.members.iter().any(|m| m.kozijn_id == b_id)
    {
        return Err("Kozijn maakt geen deel uit van deze combinatie".into());
    }
    combo.add_coupling(a_id, b_id, coupling_type, coupling_width);
    Ok(combo.clone())
}

#[tauri::command]
pub fn get_combinations(state: State<'_, AppState>) -> Vec<CombinationKozijn> {
    state
        .project
        .lock()
        .map(|p| p.combinations.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn remove_combination(
    state: State<'_, AppState>,
    combination_id: String,
) -> Result<(), String> {
    let combo_id: uuid::Uuid = combination_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    project.combinations.retain(|c| c.id != combo_id);
    Ok(())
}
