use serde::{Deserialize, Serialize};

/// Edge configuration — how the kozijn connects to the wall at each side
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeConfig {
    /// Randsponning type machined into the kozijn edge
    pub randsponning_type: RandsponningType,
    /// Randsponning depth in mm
    #[serde(default = "default_randsponning_depth")]
    pub randsponning_depth: f64,
    /// Randsponning width in mm
    #[serde(default = "default_randsponning_width")]
    pub randsponning_width: f64,
    /// Spouwlat (cavity batten) configuration
    #[serde(default)]
    pub spouwlat: Option<SpouwlatConfig>,
    /// Seal type between kozijn and wall
    #[serde(default)]
    pub seal_type: SealType,
    /// Interior membrane type
    #[serde(default)]
    pub folie_binnen: FolieType,
    /// Exterior membrane type
    #[serde(default)]
    pub folie_buiten: FolieType,
    /// Anchor spacing in mm (KVT: max 500mm)
    #[serde(default = "default_anchor_spacing")]
    pub anchor_spacing_mm: f64,
    /// Reveal finishing (dagafwerking)
    #[serde(default)]
    pub dagafwerking: Option<String>,
    /// How deep the frame sits in the wall (inbouwdiepte)
    #[serde(default)]
    pub inbouwdiepte_mm: Option<f64>,
}

fn default_randsponning_depth() -> f64 { 5.0 }
fn default_randsponning_width() -> f64 { 46.0 }
fn default_anchor_spacing() -> f64 { 500.0 }

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            randsponning_type: RandsponningType::Haaks,
            randsponning_depth: 5.0,
            randsponning_width: 46.0,
            spouwlat: Some(SpouwlatConfig::default()),
            seal_type: SealType::Compriband,
            folie_binnen: FolieType::Dampremmend,
            folie_buiten: FolieType::Dampopen,
            anchor_spacing_mm: 500.0,
            dagafwerking: None,
            inbouwdiepte_mm: None,
        }
    }
}

/// Randsponning type — groove machined into the edge of the kozijn
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RandsponningType {
    /// Haakse sponning — simple right-angle step. Most common for masonry.
    Haaks,
    /// Kalksponning — V-groove in center for mortar bond. HSB construction.
    Kalksponning,
    /// Renovatiesponning — rebate for installing into existing openings.
    Renovatie,
    /// Vlak — no groove. Requires careful positioning of wall strips.
    Vlak,
}

impl Default for RandsponningType {
    fn default() -> Self { Self::Haaks }
}

impl RandsponningType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::Haaks => "Haakse sponning",
            Self::Kalksponning => "Kalksponning",
            Self::Renovatie => "Renovatiesponning",
            Self::Vlak => "Vlak (geen sponning)",
        }
    }
}

/// Spouwlat (cavity batten) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpouwlatConfig {
    /// Width in mm (100, 120, 140, 160 standard)
    pub width: f64,
    /// Height/thickness in mm (min 27 against wood, 38 against other)
    pub height: f64,
    /// Material
    pub material: String,
    /// Attachment: glued + nailed/stapled
    #[serde(default = "default_true")]
    pub glued: bool,
    /// Nail/staple spacing in mm (max 300 nails, 200 staples)
    #[serde(default = "default_nail_spacing")]
    pub nail_spacing_mm: f64,
}

fn default_true() -> bool { true }
fn default_nail_spacing() -> f64 { 300.0 }

impl Default for SpouwlatConfig {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 27.0,
            material: "vuren".into(),
            glued: true,
            nail_spacing_mm: 300.0,
        }
    }
}

/// Seal type between kozijn and wall
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SealType {
    /// Compriband / cellenband — expanding PU foam tape
    Compriband,
    /// Kit — sealant
    Kit,
    /// Geen afdichting
    Geen,
}

impl Default for SealType {
    fn default() -> Self { Self::Compriband }
}

/// Folie (membrane) type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FolieType {
    /// Dampremmend — vapour-retarding (warm side / interior)
    Dampremmend,
    /// Dampopen — vapour-permeable (cold side / exterior)
    Dampopen,
    /// Geen folie
    Geen,
}

impl Default for FolieType {
    fn default() -> Self { Self::Geen }
}

/// Edge position on the frame
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgePosition {
    Left,
    Right,
    Top,
    Bottom,
}

/// Standard spouwlat widths
pub fn standard_spouwlat_widths() -> Vec<f64> {
    vec![100.0, 120.0, 140.0, 160.0]
}
