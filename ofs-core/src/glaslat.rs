use serde::{Deserialize, Serialize};

/// Glaslat (glazing bead) configuration for a glazed cell.
///
/// Beads run around the glazing rebate (sponning) and hold the glass.
/// Attached to a `Cell` via `glaslat: Option<Glaslat>`; only meaningful for
/// glazed cells (fixed glass and operable sashes).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Glaslat {
    pub position: GlaslatPosition,
    pub material: String,
    /// Bead face width in mm (KVT minimum 13 inside / 15 outside).
    pub width_mm: f64,
    /// Bead height in mm (KVT minimum 17).
    pub height_mm: f64,
    /// Mitred (verstek) corners — true = 45° mitres on all four corners.
    #[serde(default = "default_true")]
    pub mitered: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlaslatPosition {
    Binnen, // inside glazing bead (binnenbeglazing)
    Buiten, // outside glazing bead (buitenbeglazing)
}

impl GlaslatPosition {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::Binnen => "Binnen",
            Self::Buiten => "Buiten",
        }
    }
}

impl Default for Glaslat {
    fn default() -> Self {
        Self {
            position: GlaslatPosition::Binnen,
            material: "meranti".into(),
            width_mm: 15.0,
            height_mm: 17.0,
            mitered: true,
        }
    }
}
