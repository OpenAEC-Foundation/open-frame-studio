use serde::{Deserialize, Serialize};

/// Vakvulling (infill panel) configuration for a Panel or Ventilation cell.
///
/// Models sandwich panels, solid panels, door panels, ventilation grilles and
/// blind/dummy infills, following KVT / NEN 6702 conventions. Attached to a
/// `Cell` via `panel_filling: Option<PanelFilling>` (only meaningful for
/// `PanelType::Panel` and `PanelType::Ventilation`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelFilling {
    pub filling_type: FillingType,
    pub material: String,
    /// Total panel thickness in mm (sandwich typically 40-90 mm).
    pub thickness_mm: f64,
    /// Panel thermal transmittance, W/(m²·K). NEN 6702 sandwich range ~0.22-0.86.
    pub u_value: f64,
    /// Edge banding material (e.g. "pvc", "aluminium"); None = no banding.
    #[serde(default)]
    pub edge_banding: Option<String>,
    pub color: String,
    #[serde(default)]
    pub manufacturer: Option<String>,
    /// Grille spec — only relevant for `FillingType::Ventilation`.
    #[serde(default)]
    pub ventilation: Option<PanelVentilation>,
    /// Depth position of the fill within the profile depth, in mm: the offset of
    /// the fill's inner face from the frame's inner face (0 = flush with the
    /// inside; positive = set back towards the outside). `None` = auto (centred
    /// by thickness). Lets different vakvullingen sit at different depths (#6).
    #[serde(default)]
    pub setback_mm: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FillingType {
    Sandwich,    // sandwichpaneel (isolatiekern)
    Solid,       // massief paneel (hout/plaat)
    DoorPanel,   // deurpaneel
    Ventilation, // ventilatierooster
    Blind,       // blind/dummy paneel
}

impl FillingType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::Sandwich => "Sandwichpaneel",
            Self::Solid => "Massief paneel",
            Self::DoorPanel => "Deurpaneel",
            Self::Ventilation => "Ventilatierooster",
            Self::Blind => "Blind paneel",
        }
    }
}

/// Ventilation grille spec for a `FillingType::Ventilation` filling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelVentilation {
    pub mounting: VentMounting,
    /// Air capacity in dm³/s.
    pub capacity_dm3s: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VentMounting {
    Kalf,  // in a separate opening on the kalf/transom
    Glass, // applied over the glass pane within a single vak
}

impl VentMounting {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::Kalf => "Op kalf",
            Self::Glass => "Op glas",
        }
    }
}

impl Default for PanelFilling {
    fn default() -> Self {
        Self {
            filling_type: FillingType::Sandwich,
            material: "sandwich-pur".into(),
            thickness_mm: 40.0,
            u_value: 0.6,
            edge_banding: None,
            color: "RAL9010".into(),
            manufacturer: None,
            ventilation: None,
            setback_mm: None,
        }
    }
}
