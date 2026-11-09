use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub fn export_cnc_gcode(
    state: State<'_, AppState>,
    id: String,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let _ = output_dir;
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    if kozijn.recovered_from_ifc {
        return Err("CNC-export is geblokkeerd voor een IFC-basisproject: systeemprofielen, verbindingen en bewerkingen moeten eerst worden gevalideerd".into());
    }
    Err("CNC-export is nog niet vrijgegeven: er is geen gevalideerde producentconfiguratie met profielen, verbindingen, gereedschappen en machineassen".into())
}

#[tauri::command]
pub fn get_cnc_parts(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ofs_core::cnc::CncPart>, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;
    let id: uuid::Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    let kozijn = project.kozijnen.iter().find(|k| k.id == id)
        .ok_or("Kozijn niet gevonden")?;
    if kozijn.recovered_from_ifc {
        return Err("CNC-onderdelen zijn niet gevalideerd voor een IFC-basisproject".into());
    }
    Ok(ofs_core::cnc::generate_cnc_parts(kozijn))
}
