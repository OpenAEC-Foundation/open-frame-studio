use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn compare_ifc_files(
    old_path: String,
    new_path: String,
) -> Result<String, String> {
    let old = ofs_core::import::ifc_import::parse_ifc_file(&old_path)?;
    let new = ofs_core::import::ifc_import::parse_ifc_file(&new_path)?;
    let diff = ofs_core::ifc_roundtrip::compare_ifc_imports(&old, &new);
    serde_json::to_string(&diff).map_err(|e| e.to_string())
}

/// Compare the current project's kozijnen against the windows/doors in an
/// existing IFC file (roundtrip check).
///
/// The given file is treated as the "old" state and the current project as
/// the "new" state. Each kozijn in the project is exported to a temporary
/// IFC file and parsed back, so both sides of the comparison go through the
/// same import machinery before the diff is computed.
#[tauri::command]
pub fn compare_ifc_roundtrip(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<String, String> {
    let mut old = ofs_core::import::ifc_import::parse_ifc_file(&file_path)?;

    let kozijnen = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        project.kozijnen.clone()
    };

    let mut new = ofs_core::import::ifc_import::IfcImportResult {
        windows: Vec::new(),
        doors: Vec::new(),
        openings: Vec::new(),
    };

    let temp_dir = std::env::temp_dir();
    for kozijn in &kozijnen {
        let temp_path = temp_dir.join(format!("ofs_roundtrip_{}.ifc", kozijn.id));
        let temp_str = temp_path.to_string_lossy().into_owned();
        ofs_core::export::ifc::generate_ifc(kozijn, &temp_str)?;
        let parsed = ofs_core::import::ifc_import::parse_ifc_file(&temp_str);
        let _ = std::fs::remove_file(&temp_path);
        let parsed = parsed?;
        new.windows.extend(parsed.windows);
        new.doors.extend(parsed.doors);
        new.openings.extend(parsed.openings);
    }

    // The IFC writer regenerates GUIDs on every export, so GUIDs cannot be
    // used to match elements across a roundtrip. OFS exports carry the kozijn
    // merkteken in the parsed name field, so re-key both sides by name where
    // available; elements without a name keep their GUID as key.
    rekey_by_name(&mut old);
    rekey_by_name(&mut new);

    let diff = ofs_core::ifc_roundtrip::compare_ifc_imports(&old, &new);
    serde_json::to_string(&diff).map_err(|e| e.to_string())
}

fn rekey_by_name(result: &mut ofs_core::import::ifc_import::IfcImportResult) {
    for w in &mut result.windows {
        if !w.name.is_empty() {
            w.guid = w.name.clone();
        }
    }
    for d in &mut result.doors {
        if !d.name.is_empty() {
            d.guid = d.name.clone();
        }
    }
}
