pub mod postprocessor;
pub mod gcode;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CncPart {
    pub piece_id: String,
    pub kozijn_mark: String,
    pub profile_name: String,
    pub material: String,
    pub gross_length_mm: f64,
    pub operations: Vec<CncOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CncOperation {
    CrossCut { position_mm: f64, angle_deg: f64 },
    Mortise { x: f64, y: f64, width: f64, depth: f64, length: f64 },
    Tenon { x: f64, y: f64, width: f64, length: f64 },
    Drill { x: f64, y: f64, diameter: f64, depth: f64 },
    Groove { start_x: f64, end_x: f64, depth: f64, width: f64 },
}

pub fn generate_cnc_parts(kozijn: &crate::kozijn::Kozijn) -> Vec<CncPart> {
    let prod = crate::production::compute_production_data(kozijn);
    let mut parts = Vec::new();

    for cut in &prod.cut_list {
        // Indicative cutting preview only. Joint, rebate, tooling and machine
        // coordinates require a verified producer system; inventing those
        // operations here would make the preview look production-ready.
        let ops = vec![
            CncOperation::CrossCut { position_mm: 0.0, angle_deg: cut.miter_left_deg },
            CncOperation::CrossCut {
                position_mm: cut.gross_length_mm,
                angle_deg: cut.miter_right_deg,
            },
        ];

        parts.push(CncPart {
            piece_id: cut.piece_id.clone(),
            kozijn_mark: prod.kozijn_mark.clone(),
            profile_name: cut.profile_name.clone(),
            material: cut.material.clone(),
            gross_length_mm: cut.gross_length_mm,
            operations: ops,
        });
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::{generate_cnc_parts, CncOperation};
    use crate::kozijn::Kozijn;

    #[test]
    fn preview_does_not_invent_joint_or_rebate_operations() {
        let kozijn = Kozijn::new("Kozijn", "K01", 1200.0, 1500.0);
        let parts = generate_cnc_parts(&kozijn);
        assert!(!parts.is_empty());
        assert!(parts.iter().all(|part| {
            part.operations.len() == 2
                && part.operations.iter().all(|op| matches!(op, CncOperation::CrossCut { .. }))
        }));
    }
}
