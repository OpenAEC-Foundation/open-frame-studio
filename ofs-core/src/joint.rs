use serde::{Deserialize, Serialize};

/// Joint type at connection points between frame members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JointType {
    /// Pen/slis (mortise & tenon) — traditional wood joint
    PenSlis,
    /// Verstek (miter) — 45° cut, typical for aluminum/PVC
    Verstek,
    /// Contramal — counter-profile joint
    Contramal,
    /// Stomp (butt joint) — flat against flat
    Stomp,
}

impl Default for JointType {
    fn default() -> Self {
        Self::PenSlis
    }
}

/// Which member runs through (determines cut lengths)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThroughMember {
    /// Stijl (vertical) runs through — standard for wood
    Stijl,
    /// Dorpel (horizontal) runs through
    Dorpel,
}

impl Default for ThroughMember {
    fn default() -> Self {
        Self::Stijl
    }
}

/// A joint configuration at a connection point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Joint {
    pub joint_type: JointType,
    pub through_member: ThroughMember,
    /// Cut angle in degrees (90 = square, 45 = miter)
    #[serde(default = "default_angle")]
    pub angle: f64,
    /// Pen length in mm (for pen/slis joints, typically 20mm)
    #[serde(default = "default_pen_length")]
    pub pen_length: f64,
}

fn default_angle() -> f64 {
    90.0
}

fn default_pen_length() -> f64 {
    20.0
}

impl Default for Joint {
    fn default() -> Self {
        Self {
            joint_type: JointType::PenSlis,
            through_member: ThroughMember::Stijl,
            angle: 90.0,
            pen_length: 20.0,
        }
    }
}

// ── Shared corner model ─────────────────────────────────────────────
//
// The saw list (`production.rs`) and the front-view drawing (`geometry.rs`)
// used to derive corners independently, so the picture and the cut list could
// disagree (issue 7). These helpers are the ONE place that decides, per corner,
// which member runs through and whether the corner is a verstek (miter) — both
// modules call them so they always agree.

use crate::kozijn::Material;

/// `Kozijn::new` auto-populates four `Joint::default()` entries (pen/slis,
/// stijl through, 90°, pen 20 mm) regardless of material. An untouched default
/// set therefore carries no user intent: treat it as "not configured" so
/// aluminium/PVC frames keep their verstek default and wood frames keep the
/// historic stijl-through corner.
pub fn is_untouched_default_joint(j: &Joint) -> bool {
    j.joint_type == JointType::PenSlis
        && j.through_member == ThroughMember::Stijl
        && (j.angle - 90.0).abs() < 1e-6
        && (j.pen_length - 20.0).abs() < 1e-6
}

/// The four corner joints in [top-left, top-right, bottom-left, bottom-right]
/// order, or `None` when the stored joints are absent, incomplete, or still the
/// untouched auto-populated default set — in which case callers fall back to
/// the material-based default.
pub fn effective_corner_joints(corner_joints: &[Joint]) -> Option<[&Joint; 4]> {
    if corner_joints.len() < 4 {
        return None;
    }
    if corner_joints[..4].iter().all(is_untouched_default_joint) {
        return None;
    }
    Some([
        &corner_joints[0],
        &corner_joints[1],
        &corner_joints[2],
        &corner_joints[3],
    ])
}

/// Drawing/joint model of one frame corner, shared by production and geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerModel {
    /// Mitered (45°) corner — both members stop on the diagonal.
    pub verstek: bool,
    /// Which member runs through (to the outer edge) at this corner.
    pub through: ThroughMember,
    /// Saw/draw angle at the corner (45° for verstek, else 90°).
    pub angle: f64,
}

/// Resolve the four corners [TL, TR, BL, BR] from material + stored joints.
/// Falls back to the material default when the joints are untouched/absent:
/// wood & wood-aluminium → stijl-through square (butt); aluminium & PVC →
/// verstek 45°. This is exactly the decision production's saw list uses, so the
/// front-view picture and the cut list always agree (issue 7).
pub fn frame_corner_model(material: &Material, corner_joints: &[Joint]) -> [CornerModel; 4] {
    match effective_corner_joints(corner_joints) {
        Some(js) => {
            let model = |j: &Joint| {
                let verstek = j.joint_type == JointType::Verstek;
                CornerModel {
                    verstek,
                    through: j.through_member,
                    angle: if verstek {
                        if j.angle > 0.0 && j.angle < 90.0 {
                            j.angle
                        } else {
                            45.0
                        }
                    } else {
                        90.0
                    },
                }
            };
            [model(js[0]), model(js[1]), model(js[2]), model(js[3])]
        }
        None => {
            let verstek = matches!(material, Material::Aluminum | Material::Pvc);
            let c = CornerModel {
                verstek,
                through: ThroughMember::Stijl,
                angle: if verstek { 45.0 } else { 90.0 },
            };
            [c, c, c, c]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::{Material, WoodType};

    fn defaults() -> Vec<Joint> {
        vec![Joint::default(), Joint::default(), Joint::default(), Joint::default()]
    }

    #[test]
    fn untouched_defaults_use_material_fallback() {
        // Wood default → stijl-through butt, no verstek.
        let m = frame_corner_model(&Material::Wood(WoodType::Meranti), &defaults());
        assert!(m.iter().all(|c| !c.verstek && c.through == ThroughMember::Stijl));
        // Aluminium / PVC default → verstek 45° everywhere.
        for mat in [Material::Aluminum, Material::Pvc] {
            let m = frame_corner_model(&mat, &defaults());
            assert!(m.iter().all(|c| c.verstek && (c.angle - 45.0).abs() < 1e-6));
        }
        // The untouched default set is treated as "not configured".
        assert!(effective_corner_joints(&defaults()).is_none());
    }

    #[test]
    fn configured_verstek_beats_material_default_on_wood() {
        let verstek = Joint {
            joint_type: JointType::Verstek,
            through_member: ThroughMember::Stijl,
            angle: 45.0,
            pen_length: 0.0,
        };
        let joints = vec![verstek.clone(), verstek.clone(), verstek.clone(), verstek];
        let m = frame_corner_model(&Material::Wood(WoodType::Meranti), &joints);
        assert!(m.iter().all(|c| c.verstek && (c.angle - 45.0).abs() < 1e-6));
        assert!(effective_corner_joints(&joints).is_some());
    }

    #[test]
    fn configured_dorpel_through_is_honored() {
        let dorpel = Joint {
            joint_type: JointType::PenSlis,
            through_member: ThroughMember::Dorpel,
            angle: 90.0,
            pen_length: 20.0,
        };
        let joints = vec![dorpel.clone(), dorpel.clone(), dorpel.clone(), dorpel];
        let m = frame_corner_model(&Material::Aluminum, &joints);
        // Explicit config wins over the material verstek default.
        assert!(m.iter().all(|c| !c.verstek && c.through == ThroughMember::Dorpel));
    }
}
