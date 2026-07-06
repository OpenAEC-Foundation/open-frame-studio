use crate::state::AppState;
use ofs_core::ids::{IdsCheckResult, IdsRequirement};
use tauri::State;

#[tauri::command]
pub fn validate_project_ids(
    state: State<'_, AppState>,
    requirements_json: Option<String>,
) -> Result<Vec<IdsCheckResult>, String> {
    let project = state.project.lock().map_err(|e| e.to_string())?;

    let requirements: Vec<IdsRequirement> = if let Some(json) = requirements_json {
        serde_json::from_str(&json).map_err(|e| format!("Ongeldige IDS requirements: {}", e))?
    } else {
        ofs_core::ids::default_ids_requirements()
    };

    let profiles = &project.custom_profiles;
    let mut all_results = Vec::new();

    for kozijn in &project.kozijnen {
        let results = ofs_core::ids::validate_kozijn(kozijn, profiles, &requirements);
        all_results.extend(results);

        // Geometric / KVT checks (complementary to the IDS property checks).
        let geometry_errors = ofs_core::validation::validate(kozijn);
        if geometry_errors.is_empty() {
            all_results.push(IdsCheckResult {
                requirement: "Geometrie.KVT".into(),
                kozijn_mark: kozijn.mark.clone(),
                passed: true,
                actual_value: None,
                message: "OK".into(),
            });
        } else {
            for err in geometry_errors {
                all_results.push(IdsCheckResult {
                    requirement: "Geometrie.KVT".into(),
                    kozijn_mark: kozijn.mark.clone(),
                    passed: false,
                    actual_value: None,
                    message: err.to_string(),
                });
            }
        }
    }

    // Vliesgevel geometric checks in the same result shape.
    for vg in &project.vliesgevels {
        let vg_errors = ofs_core::vliesgevel::validation::validate_vliesgevel(vg);
        if vg_errors.is_empty() {
            all_results.push(IdsCheckResult {
                requirement: "Vliesgevel.Geometrie".into(),
                kozijn_mark: vg.mark.clone(),
                passed: true,
                actual_value: None,
                message: "OK".into(),
            });
        } else {
            for err in vg_errors {
                all_results.push(IdsCheckResult {
                    requirement: "Vliesgevel.Geometrie".into(),
                    kozijn_mark: vg.mark.clone(),
                    passed: false,
                    actual_value: None,
                    message: err.to_string(),
                });
            }
        }
    }

    Ok(all_results)
}
