//! WebAssembly bindings for ofs-core.
//!
//! Exposes the same domain logic as the Tauri backend, but callable
//! from JavaScript in a browser. All data passes as JSON strings.

use wasm_bindgen::prelude::*;
use std::sync::Mutex;

use ofs_core::kozijn::{Kozijn, Project};
use ofs_core::production::compute_production_data;

// ── State ──────────────────────────────────────────────────────

static PROJECT: Mutex<Option<Project>> = Mutex::new(None);

fn with_project<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&mut Project) -> R,
{
    let mut guard = PROJECT.lock().map_err(|e| e.to_string())?;
    let project = guard.get_or_insert_with(|| Project::new("New project", ""));
    Ok(f(project))
}

fn find_kozijn(project: &Project, id: &str) -> Result<usize, String> {
    let uuid: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    project
        .kozijnen
        .iter()
        .position(|k| k.id == uuid)
        .ok_or_else(|| "Kozijn niet gevonden".into())
}

// ── Project commands ───────────────────────────────────────────

#[wasm_bindgen]
pub fn get_project() -> Result<String, String> {
    with_project(|p| serde_json::to_string(p).unwrap())
}

#[wasm_bindgen]
pub fn new_project(name: &str, number: &str) -> Result<String, String> {
    let mut guard = PROJECT.lock().map_err(|e| e.to_string())?;
    let project = Project::new(name, number);
    let json = serde_json::to_string(&project).map_err(|e| e.to_string())?;
    *guard = Some(project);
    Ok(json)
}

#[wasm_bindgen]
pub fn open_project_json(json: &str) -> Result<String, String> {
    let project: Project = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let result = serde_json::to_string(&project).map_err(|e| e.to_string())?;
    let mut guard = PROJECT.lock().map_err(|e| e.to_string())?;
    *guard = Some(project);
    Ok(result)
}

#[wasm_bindgen]
pub fn save_project_json() -> Result<String, String> {
    with_project(|p| serde_json::to_string(p).unwrap())
}

// ── Kozijn CRUD ────────────────────────────────────────────────

#[wasm_bindgen]
pub fn create_kozijn(name: &str, mark: &str, width: f64, height: f64) -> Result<String, String> {
    with_project(|p| {
        let k = Kozijn::new(name, mark, width, height);
        let json = serde_json::to_string(&k).unwrap();
        p.kozijnen.push(k);
        json
    })
}

#[wasm_bindgen]
pub fn get_kozijn(id: &str) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        Ok(serde_json::to_string(&p.kozijnen[idx]).unwrap())
    })?
}

#[wasm_bindgen]
pub fn get_all_kozijnen() -> Result<String, String> {
    with_project(|p| serde_json::to_string(&p.kozijnen).unwrap())
}

#[wasm_bindgen]
pub fn remove_kozijn(id: &str) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        p.kozijnen.remove(idx);
        Ok("ok".into())
    })?
}

#[wasm_bindgen]
pub fn duplicate_kozijn(id: &str, new_mark: &str) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        let mut dup = p.kozijnen[idx].clone();
        dup.id = uuid::Uuid::new_v4();
        dup.mark = new_mark.to_string();
        let json = serde_json::to_string(&dup).unwrap();
        p.kozijnen.push(dup);
        Ok(json)
    })?
}

// ── Kozijn mutations ───────────────────────────────────────────

#[wasm_bindgen]
pub fn update_kozijn_dimensions(id: &str, width: f64, height: f64) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        let k = &mut p.kozijnen[idx];
        k.frame.outer_width = width;
        k.frame.outer_height = height;
        // Recalculate grid
        let fw = k.frame.frame_width;
        if k.grid.columns.len() == 1 {
            k.grid.columns[0].size = width - 2.0 * fw;
        }
        if k.grid.rows.len() == 1 {
            k.grid.rows[0].size = height - 2.0 * fw;
        }
        Ok(serde_json::to_string(&p.kozijnen[idx]).unwrap())
    })?
}

#[wasm_bindgen]
pub fn update_cell_type(
    id: &str,
    cell_index: usize,
    panel_type: &str,
    opening_direction: Option<String>,
) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        let k = &mut p.kozijnen[idx];
        if cell_index < k.cells.len() {
            if let Ok(pt) = serde_json::from_str::<ofs_core::kozijn::PanelType>(&format!("\"{}\"", panel_type)) {
                k.cells[cell_index].panel_type = pt;
            }
            k.cells[cell_index].opening_direction = opening_direction
                .as_deref()
                .and_then(|d| serde_json::from_str(&format!("\"{}\"", d)).ok());
        }
        Ok(serde_json::to_string(&p.kozijnen[idx]).unwrap())
    })?
}

#[wasm_bindgen]
pub fn add_column(id: &str, position: f64) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        p.kozijnen[idx].add_column(position);
        Ok(serde_json::to_string(&p.kozijnen[idx]).unwrap())
    })?
}

#[wasm_bindgen]
pub fn add_row(id: &str, position: f64) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        p.kozijnen[idx].add_row(position);
        Ok(serde_json::to_string(&p.kozijnen[idx]).unwrap())
    })?
}

// ── Geometry ───────────────────────────────────────────────────

#[wasm_bindgen]
pub fn get_kozijn_geometry(id: &str) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        let geom = ofs_core::geometry::compute_2d_geometry(&p.kozijnen[idx]);
        Ok(serde_json::to_string(&geom).unwrap())
    })?
}

// ── Production ─────────────────────────────────────────────────

#[wasm_bindgen]
pub fn get_production_data(id: &str) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        let prod = compute_production_data(&p.kozijnen[idx]);
        Ok(serde_json::to_string(&prod).unwrap())
    })?
}

#[wasm_bindgen]
pub fn get_production_data_project() -> Result<String, String> {
    with_project(|p| {
        let data: Vec<_> = p.kozijnen.iter().map(|k| compute_production_data(k)).collect();
        serde_json::to_string(&data).unwrap()
    })
}

// ── Thermal ────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn calculate_thermal(id: &str) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        let result = ofs_core::thermal::calculate_uw(&p.kozijnen[idx], &[]);
        Ok(serde_json::to_string(&result).unwrap())
    })?
}

// ── Export data (returns JSON for JS-side file generation) ──────

#[wasm_bindgen]
pub fn get_export_data(id: &str) -> Result<String, String> {
    with_project(|p| {
        let idx = find_kozijn(p, id)?;
        let k = &p.kozijnen[idx];
        let prod = compute_production_data(k);
        let result = serde_json::json!({
            "kozijn": k,
            "production": prod,
        });
        Ok(serde_json::to_string(&result).unwrap())
    })?
}

#[wasm_bindgen]
pub fn get_project_export_data() -> Result<String, String> {
    with_project(|p| {
        let data: Vec<_> = p.kozijnen.iter().map(|k| {
            let prod = compute_production_data(k);
            serde_json::json!({ "kozijn": k, "production": prod })
        }).collect();
        serde_json::to_string(&data).unwrap()
    })
}
