use crate::state::AppState;
use ofs_core::geometry::{compute_2d_geometry, KozijnGeometry2D};
use ofs_core::hardware::{self, HardwareSet, SecurityClass};
use ofs_core::kozijn::{FrameShape, Glazing, Kozijn, PanelType, OpeningDirection, ShapeType};
use ofs_core::profile::ProfileRef;
use ofs_core::thermal;
use ofs_core::template::{self, KozijnSjabloon};
use ofs_core::grid;
use tauri::State;

#[tauri::command]
pub fn create_kozijn(
    state: State<'_, AppState>,
    name: String,
    mark: String,
    width: f64,
    height: f64,
) -> Result<Kozijn, String> {
    let kozijn = Kozijn::new(&name, &mark, width, height);
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    project.kozijnen.push(kozijn.clone());
    Ok(kozijn)
}

#[tauri::command]
pub fn create_kozijn_from_template(
    state: State<'_, AppState>,
    template: String,
    width: f64,
    height: f64,
    sjabloon_id: Option<String>,
) -> Result<Kozijn, String> {
    let sj = sjabloon_id
        .map(|id| template::get_sjabloon(&id))
        .unwrap_or_else(template::default_sjabloon);

    let kozijn = match template.as_str() {
        "single_turn_tilt" => grid::template_single_turn_tilt_sj(width, height, &sj),
        "double_turn_tilt" => grid::template_double_turn_tilt_sj(width, height, &sj),
        "sliding_door" => grid::template_sliding_door_sj(width, height, &sj),
        "front_door" => grid::template_front_door_sj(width, height, &sj),
        _ => Kozijn::new_with_sjabloon("Kozijn", "K01", width, height, &sj),
    };
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    project.kozijnen.push(kozijn.clone());
    Ok(kozijn)
}

#[tauri::command]
pub fn get_sjablonen(state: State<'_, AppState>) -> Vec<KozijnSjabloon> {
    let mut all = template::builtin_sjablonen();
    if let Ok(project) = state.project.lock() {
        all.extend(project.custom_sjablonen.clone());
    }
    all
}

#[tauri::command]
pub fn save_custom_sjabloon(
    state: State<'_, AppState>,
    sjabloon_json: String,
) -> Result<(), String> {
    let sjabloon: KozijnSjabloon = serde_json::from_str(&sjabloon_json)
        .map_err(|e| format!("Ongeldig sjabloon JSON: {}", e))?;
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    // Update existing or add new
    if let Some(existing) = project.custom_sjablonen.iter_mut().find(|s| s.id == sjabloon.id) {
        *existing = sjabloon;
    } else {
        project.custom_sjablonen.push(sjabloon);
    }
    Ok(())
}

#[tauri::command]
pub fn delete_custom_sjabloon(
    state: State<'_, AppState>,
    sjabloon_id: String,
) -> Result<(), String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    project.custom_sjablonen.retain(|s| s.id != sjabloon_id);
    Ok(())
}

#[tauri::command]
pub fn get_kozijn(state: State<'_, AppState>, id: String) -> Result<Kozijn, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    project
        .kozijnen
        .iter()
        .find(|k| k.id == id)
        .cloned()
        .ok_or_else(|| "Kozijn niet gevonden".into())
}

#[tauri::command]
pub fn get_all_kozijnen(state: State<'_, AppState>) -> Result<Vec<Kozijn>, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    Ok(project.kozijnen.clone())
}

#[tauri::command]
pub fn update_kozijn_dimensions(
    state: State<'_, AppState>,
    id: String,
    width: f64,
    height: f64,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter_mut()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;

    // Scale columns proportionally
    let old_inner_w = kozijn.inner_width();
    let new_inner_w = width - 2.0 * kozijn.frame.frame_width;
    let scale_w = new_inner_w / old_inner_w;
    for col in &mut kozijn.grid.columns {
        col.size *= scale_w;
    }

    // Scale rows proportionally
    let old_inner_h = kozijn.inner_height();
    let new_inner_h = height - 2.0 * kozijn.frame.frame_width;
    let scale_h = new_inner_h / old_inner_h;
    for row in &mut kozijn.grid.rows {
        row.size *= scale_h;
    }

    kozijn.frame.outer_width = width;
    kozijn.frame.outer_height = height;

    Ok(kozijn.clone())
}

#[tauri::command]
pub fn update_grid_sizes(
    state: State<'_, AppState>,
    id: String,
    column_sizes: Vec<f64>,
    row_sizes: Vec<f64>,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter_mut()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;

    for (i, col) in kozijn.grid.columns.iter_mut().enumerate() {
        if let Some(&new_size) = column_sizes.get(i) {
            col.size = new_size.max(100.0); // minimum 100mm
        }
    }
    for (i, row) in kozijn.grid.rows.iter_mut().enumerate() {
        if let Some(&new_size) = row_sizes.get(i) {
            row.size = new_size.max(100.0);
        }
    }

    Ok(kozijn.clone())
}

#[tauri::command]
pub fn update_frame_profile(
    state: State<'_, AppState>,
    id: String,
    profile_id: String,
    profile_name: String,
    profile_width: Option<f64>,
    profile_depth: Option<f64>,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter_mut().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    kozijn.frame.profile = ProfileRef { id: profile_id, name: profile_name };
    if let Some(w) = profile_width {
        let old_fw = kozijn.frame.frame_width;
        kozijn.frame.frame_width = w;
        // Rescale grid to fit new frame width
        let old_inner = kozijn.frame.outer_width - 2.0 * old_fw;
        let new_inner = kozijn.frame.outer_width - 2.0 * w;
        if old_inner > 0.0 && new_inner > 0.0 {
            let scale = new_inner / old_inner;
            for col in &mut kozijn.grid.columns {
                col.size *= scale;
            }
        }
        let old_inner_h = kozijn.frame.outer_height - 2.0 * old_fw;
        let new_inner_h = kozijn.frame.outer_height - 2.0 * w;
        if old_inner_h > 0.0 && new_inner_h > 0.0 {
            let scale_h = new_inner_h / old_inner_h;
            for row in &mut kozijn.grid.rows {
                row.size *= scale_h;
            }
        }
    }
    if let Some(d) = profile_depth {
        kozijn.frame.frame_depth = d;
    }
    Ok(kozijn.clone())
}

#[tauri::command]
pub fn update_sill_profile(
    state: State<'_, AppState>,
    id: String,
    profile_id: String,
    profile_name: String,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter_mut().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    kozijn.frame.sill_profile = Some(ProfileRef { id: profile_id, name: profile_name });
    Ok(kozijn.clone())
}

#[tauri::command]
pub fn update_divider_profile(
    state: State<'_, AppState>,
    id: String,
    divider_index: usize,
    is_column: bool,
    profile_id: String,
    profile_name: String,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter_mut().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;

    let profile = ProfileRef { id: profile_id, name: profile_name };
    if is_column {
        let col = kozijn.grid.columns.get_mut(divider_index)
            .ok_or("Kolom niet gevonden")?;
        col.divider_profile = Some(profile);
    } else {
        let row = kozijn.grid.rows.get_mut(divider_index)
            .ok_or("Rij niet gevonden")?;
        row.divider_profile = Some(profile);
    }
    Ok(kozijn.clone())
}

#[tauri::command]
pub fn update_member_profile(
    state: State<'_, AppState>,
    id: String,
    member_type: String,
    member_index: Option<usize>,
    profile_id: String,
    profile_name: String,
    profile_width: Option<f64>,
    _profile_depth: Option<f64>,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter_mut().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;

    let profile = ProfileRef { id: profile_id, name: profile_name };
    match member_type.as_str() {
        "frame_top" => kozijn.frame.top_profile = Some(profile),
        "frame_bottom" => kozijn.frame.bottom_profile = Some(profile),
        "frame_left" => kozijn.frame.left_profile = Some(profile),
        "frame_right" => kozijn.frame.right_profile = Some(profile),
        "divider_v" => {
            if let Some(idx) = member_index {
                // Divider indices: column index + 1 (first column has no divider)
                let col_idx = idx + 1;
                if let Some(col) = kozijn.grid.columns.get_mut(col_idx) {
                    col.divider_profile = Some(profile);
                }
            }
        }
        "divider_h" => {
            if let Some(idx) = member_index {
                let row_idx = idx + 1;
                if let Some(row) = kozijn.grid.rows.get_mut(row_idx) {
                    row.divider_profile = Some(profile);
                }
            }
        }
        _ => return Err(format!("Onbekend member type: {}", member_type)),
    }

    // Update frame dimensions if width/depth provided (for frame members only)
    if member_type.starts_with("frame") {
        if let Some(_w) = profile_width {
            // Don't change frame_width globally — that affects all members
            // Per-member width would need geometry changes (future)
        }
    }

    Ok(kozijn.clone())
}

#[tauri::command]
pub fn update_cell_type(
    state: State<'_, AppState>,
    id: String,
    cell_index: usize,
    panel_type: PanelType,
    opening_direction: Option<OpeningDirection>,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter_mut()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    let cell = kozijn
        .cells
        .get_mut(cell_index)
        .ok_or("Cel niet gevonden")?;
    cell.panel_type = panel_type;
    cell.opening_direction = opening_direction;

    // Auto-generate hardware set when changing to an operable type
    let cell_width = kozijn.grid.columns.get(cell_index % kozijn.grid.columns.len())
        .map(|c| c.size).unwrap_or(600.0);
    let cell_height = kozijn.grid.rows.get(cell_index / kozijn.grid.columns.len())
        .map(|r| r.size).unwrap_or(1200.0);
    cell.hardware_set = hardware::default_hardware_set(
        panel_type,
        opening_direction,
        cell_width,
        cell_height,
        cell.glazing.thickness_mm,
        &kozijn.frame.material,
        SecurityClass::None,
    );

    // Auto-assign sash profile based on panel type (professional workflow)
    match panel_type {
        PanelType::TurnTilt | PanelType::Turn | PanelType::Tilt | PanelType::Sliding => {
            if cell.sash_profile.is_none() {
                cell.sash_profile = Some(ProfileRef {
                    id: "raam-meranti-54x67".into(),
                    name: "Raamhout 54x67mm".into(),
                });
                cell.sash_width = Some(54.0);
                cell.sash_depth = Some(67.0);
            }
        }
        PanelType::Door => {
            if cell.sash_profile.is_none() {
                cell.sash_profile = Some(ProfileRef {
                    id: "deur-meranti-67x114".into(),
                    name: "Deurhout 67x114mm".into(),
                });
                cell.sash_width = Some(67.0);
                cell.sash_depth = Some(114.0);
            }
        }
        _ => {
            cell.sash_profile = None;
            cell.sash_width = None;
            cell.sash_depth = None;
        }
    }

    Ok(kozijn.clone())
}

#[tauri::command]
pub fn add_column(
    state: State<'_, AppState>,
    id: String,
    position: f64,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter_mut()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    kozijn.add_column(position);
    Ok(kozijn.clone())
}

#[tauri::command]
pub fn add_row(
    state: State<'_, AppState>,
    id: String,
    position: f64,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter_mut()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    kozijn.add_row(position);
    Ok(kozijn.clone())
}

#[tauri::command]
pub fn remove_kozijn(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    project.kozijnen.retain(|k| k.id != id);
    Ok(())
}

#[tauri::command]
pub fn get_kozijn_geometry(
    state: State<'_, AppState>,
    id: String,
) -> Result<KozijnGeometry2D, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    Ok(compute_2d_geometry(kozijn))
}

#[tauri::command]
pub fn update_frame_shape(
    state: State<'_, AppState>,
    id: String,
    shape_type: ShapeType,
    arch_height: Option<f64>,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter_mut().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;

    kozijn.frame.shape = FrameShape {
        shape_type,
        arch_radius: None, // computed from arch_height in geometry
        arch_height,
        top_width: None,
        left_angle: None,
        right_angle: None,
    };

    Ok(kozijn.clone())
}

#[tauri::command]
pub fn add_custom_profile(
    state: State<'_, AppState>,
    profile_json: String,
) -> Result<(), String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let profile: ofs_core::profile::ProfileDefinition =
        serde_json::from_str(&profile_json).map_err(|e| format!("Ongeldig profiel: {}", e))?;
    project.custom_profiles.push(profile);
    Ok(())
}

#[tauri::command]
pub fn update_cell_hardware(
    state: State<'_, AppState>,
    id: String,
    cell_index: usize,
    hardware_set_json: String,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter_mut()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    let cell = kozijn
        .cells
        .get_mut(cell_index)
        .ok_or("Cel niet gevonden")?;
    let mut hw: HardwareSet = serde_json::from_str(&hardware_set_json)
        .map_err(|e| format!("Ongeldige hardware data: {}", e))?;
    hw.auto_selected = false;
    cell.hardware_set = Some(hw);
    Ok(kozijn.clone())
}

#[tauri::command]
pub fn auto_select_hardware(
    state: State<'_, AppState>,
    id: String,
    cell_index: usize,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter_mut()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    let cell = kozijn
        .cells
        .get_mut(cell_index)
        .ok_or("Cel niet gevonden")?;

    let cell_width = kozijn.grid.columns.get(cell_index % kozijn.grid.columns.len())
        .map(|c| c.size).unwrap_or(600.0);
    let cell_height = kozijn.grid.rows.get(cell_index / kozijn.grid.columns.len())
        .map(|r| r.size).unwrap_or(1200.0);
    let security = cell.hardware_set.as_ref()
        .map(|h| h.security_class)
        .unwrap_or(SecurityClass::None);

    cell.hardware_set = hardware::default_hardware_set(
        cell.panel_type,
        cell.opening_direction,
        cell_width,
        cell_height,
        cell.glazing.thickness_mm,
        &kozijn.frame.material,
        security,
    );

    Ok(kozijn.clone())
}

#[tauri::command]
pub fn update_security_class(
    state: State<'_, AppState>,
    id: String,
    cell_index: usize,
    security_class: SecurityClass,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project
        .kozijnen
        .iter_mut()
        .find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    let cell = kozijn
        .cells
        .get_mut(cell_index)
        .ok_or("Cel niet gevonden")?;

    // Re-run auto-selection with new security class
    let cell_width = kozijn.grid.columns.get(cell_index % kozijn.grid.columns.len())
        .map(|c| c.size).unwrap_or(600.0);
    let cell_height = kozijn.grid.rows.get(cell_index / kozijn.grid.columns.len())
        .map(|r| r.size).unwrap_or(1200.0);

    cell.hardware_set = hardware::default_hardware_set(
        cell.panel_type,
        cell.opening_direction,
        cell_width,
        cell_height,
        cell.glazing.thickness_mm,
        &kozijn.frame.material,
        security_class,
    );

    Ok(kozijn.clone())
}

#[tauri::command]
pub fn update_cell_glazing(
    state: State<'_, AppState>,
    id: String,
    cell_index: usize,
    glazing_json: String,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter_mut().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    let cell = kozijn.cells.get_mut(cell_index)
        .ok_or("Cel niet gevonden")?;

    let glazing: Glazing = serde_json::from_str(&glazing_json)
        .map_err(|e| format!("Ongeldig glazing JSON: {}", e))?;
    cell.glazing = glazing;

    Ok(kozijn.clone())
}

#[tauri::command]
pub fn update_frame_colors(
    state: State<'_, AppState>,
    id: String,
    color_inside: String,
    color_outside: String,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter_mut().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;

    kozijn.frame.color_inside = color_inside;
    kozijn.frame.color_outside = color_outside;

    Ok(kozijn.clone())
}

#[tauri::command]
pub fn duplicate_kozijn(
    state: State<'_, AppState>,
    id: String,
    new_mark: String,
) -> Result<Kozijn, String> {
    let mut project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let original = project.kozijnen.iter().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;

    let duplicate = original.duplicate(&new_mark);
    project.kozijnen.push(duplicate.clone());

    Ok(duplicate)
}

#[tauri::command]
pub fn calculate_thermal(
    state: State<'_, AppState>,
    id: String,
) -> Result<thermal::ThermalResult, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;

    let profiles = &project.custom_profiles;
    Ok(thermal::calculate_uw(kozijn, profiles))
}
