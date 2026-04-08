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

impl Joint {
    /// Standard wood corner: pen/slis, stijl through
    pub fn wood_corner() -> Self {
        Self::default()
    }

    /// Miter corner: 45° verstek, typical for aluminum/PVC
    pub fn miter_corner() -> Self {
        Self {
            joint_type: JointType::Verstek,
            through_member: ThroughMember::Stijl,
            angle: 45.0,
            pen_length: 0.0,
        }
    }
}
