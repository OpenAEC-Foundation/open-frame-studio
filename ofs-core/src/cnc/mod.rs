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
    pub gross_length_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc: Option<crate::curved_timber::TimberArc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_cope_radius_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_cope_radius_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub divider_joints: Vec<crate::production::DividerJointItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arch_joints: Vec<crate::production::ArchJointItem>,
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
    let joint_index = prod.joint_index();
    let mut parts = Vec::new();

    for cut in &prod.cut_list {
        // Indicative cutting preview only. Joint, rebate, tooling and machine
        // coordinates require a verified producer system; inventing those
        // operations here would make the preview look production-ready.
        let mut ops = Vec::new();
        let gross = prod.blank_length_mm(cut);
        let arch_joints: Vec<_> = prod.arch_joints.iter().filter(|j|
            j.piece_id == cut.piece_id || j.receiver_piece_id == cut.piece_id).cloned().collect();
        let arch_end = |end| arch_joints.iter().any(|j|
            j.piece_id == cut.piece_id && j.end == end
                || j.receiver_piece_id == cut.piece_id && j.receiver_end == end);
        use crate::production::DividerJointEnd::{Start, End};
        let planar = |end: &Option<crate::curved_timber::DividerEnd>| end.as_ref()
            .map_or(true, |e| e.tenon.is_none() && e.tenon_issue.is_none()
                && matches!(e.segments.as_slice(), [crate::curved_contour::Segment::Line { .. }]));
        if gross.is_some() && cut.arc.is_none() && cut.start_cope_radius_mm.is_none() && planar(&cut.divider_start) && !arch_end(Start) {
            ops.push(CncOperation::CrossCut { position_mm: 0.0, angle_deg: cut.miter_left_deg });
        }
        if let Some(position_mm) = gross.filter(|_| cut.arc.is_none() && cut.end_cope_radius_mm.is_none() && planar(&cut.divider_end) && !arch_end(End)) {
            ops.push(CncOperation::CrossCut {
                position_mm,
                angle_deg: cut.miter_right_deg,
            });
        }

        parts.push(CncPart {
            piece_id: cut.piece_id.clone(),
            kozijn_mark: prod.kozijn_mark.clone(),
            profile_name: cut.profile_name.clone(),
            material: cut.material.clone(),
            gross_length_mm: gross,
            arc: cut.arc.clone(),
            start_cope_radius_mm: cut.start_cope_radius_mm,
            end_cope_radius_mm: cut.end_cope_radius_mm,
            divider_joints: joint_index.get(cut.piece_id.as_str()).map_or_else(Vec::new, |j|
                j.ends.iter().flatten().chain(&j.mortises).map(|item| (**item).clone()).collect()),
            arch_joints,
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
