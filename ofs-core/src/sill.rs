use serde::{Deserialize, Serialize};

/// Onderdorpel configuratie — ondersteunt hout, composiet, kunststof en natuursteen
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sill {
    pub sill_type: SillType,
    pub manufacturer: Option<String>,
    pub material: String,
    pub profile_width: f64,   // 52, 67, 90 mm
    pub profile_height: f64,  // 114, 122, 139, 144 mm
    pub sponning_width: f64,  // 51, 57, 67, 72 mm
    pub sponning_height: f64, // typically 17 mm
    pub color: String,
    pub neut: NeutConfig,
    pub drainage: bool,
    pub slope_degrees: f64,
    /// Laagrelief dorpel (low-relief sill with separate strip)
    #[serde(default)]
    pub is_laagrelief: bool,
    /// Height of the laagrelief strip in mm (the non-wood portion)
    #[serde(default)]
    pub laagrelief_height: Option<f64>,
    /// Material of the laagrelief strip
    #[serde(default)]
    pub laagrelief_material: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SillType {
    Wood,
    Composite,    // polyesterbeton (Ekosiet, Holonite)
    Plastic,      // PE-kunststof (DTS)
    NaturalStone, // hardsteen, graniet (Nibostone)
}

impl SillType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::Wood => "Hout",
            Self::Composite => "Composiet",
            Self::Plastic => "Kunststof",
            Self::NaturalStone => "Natuursteen",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeutConfig {
    pub enabled: bool,
    pub height: f64,    // 0-150 mm
    pub width: f64,     // follows stijl width
    pub material: String,
}

impl Default for NeutConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 100.0,
            width: 67.0,
            material: "zelfde als dorpel".into(),
        }
    }
}

impl Default for Sill {
    fn default() -> Self {
        Self {
            sill_type: SillType::Wood,
            manufacturer: None,
            material: "meranti".into(),
            profile_width: 67.0,
            profile_height: 114.0,
            sponning_width: 67.0,
            sponning_height: 17.0,
            color: "RAL9010".into(),
            neut: NeutConfig::default(),
            drainage: true,
            slope_degrees: 3.0,
            is_laagrelief: false,
            laagrelief_height: None,
            laagrelief_material: None,
        }
    }
}
