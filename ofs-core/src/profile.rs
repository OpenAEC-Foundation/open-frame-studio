use serde::{Deserialize, Serialize};

/// Reference to a profile in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRef {
    pub id: String,
    pub name: String,
}

impl ProfileRef {
    pub fn default_frame() -> Self {
        Self {
            id: "wood-meranti-67x114".into(),
            name: "Meranti 67x114mm".into(),
        }
    }

    pub fn default_sill() -> Self {
        Self {
            id: "wood-meranti-67x150".into(),
            name: "Meranti 67x150mm (dorpel)".into(),
        }
    }

    pub fn default_divider() -> Self {
        Self {
            id: "wood-meranti-67x114".into(),
            name: "Meranti 67x114mm".into(),
        }
    }
}

/// Resolved profile dimensions captured on the `Frame` at selection time.
///
/// The 2D drawing, certification checks and future production logic read
/// these instead of hard-coded defaults, so the chosen profile really
/// propagates into the kozijn. Every field is optional: a missing value (or
/// a missing snapshot altogether — every pre-snapshot project) falls back to
/// the classic defaults, keeping old projects and old wasm bundles working.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSnapshot {
    /// Sponningdiepte in mm — depth of the glass/panel pocket in the
    /// bouwdiepte direction (source: `ProfileDefinition.glazing_rebate`).
    #[serde(default)]
    pub sponning_diepte: Option<f64>,
    /// Sponninghoogte in mm — glass edge cover in the face plane, KVT
    /// minimum 17 mm (source: `SponningInfo.depth`). Drives the sponning
    /// hidden line in the 2D front view (previously hard-coded 17 mm).
    #[serde(default)]
    pub sponning_hoogte: Option<f64>,
    /// Glaslat (glazing bead) face width in mm (source:
    /// `ProfileDefinition.glaslat_width`, KVT min 13 binnen / 15 buiten).
    #[serde(default)]
    pub glaslat_breedte: Option<f64>,
    /// Aanzichtbreedte (sightline) in mm — visible face width after glazing
    /// (source: `ProfileDefinition.sightline`).
    #[serde(default)]
    pub aanzichtbreedte: Option<f64>,
}

/// A profile definition from the library
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDefinition {
    pub id: String,
    pub name: String,
    pub material: String,
    pub material_subtype: Option<String>,
    /// Face width in mm
    pub width: f64,
    /// Depth in mm
    pub depth: f64,
    /// Visible face width after glazing
    pub sightline: f64,
    /// Glazing rebate depth in mm
    pub glazing_rebate: f64,
    /// 2D cross-section polygon points
    pub cross_section: Vec<[f64; 2]>,
    /// Thermal transmittance of the frame (W/m²K)
    pub uf_value: f64,
    /// Where this profile can be used
    pub applicable_as: Vec<ProfileApplication>,
    /// Sponning (rabbet/rebate) information
    #[serde(default)]
    pub sponning: Option<SponningInfo>,
    /// KVT profile type classification
    #[serde(default)]
    pub kvt_type: Option<KvtProfileType>,
    /// Profile series: "54", "67", "78", "90"
    #[serde(default)]
    pub profile_series: Option<String>,
    /// Whether this profile is parametric (wood) or fixed catalog (alu/pvc)
    #[serde(default)]
    pub is_parametric: bool,
    /// Shape Manager groove figures
    #[serde(default)]
    pub groove_figures: Vec<GrooveFigure>,
    /// Glaslat (glass bead) width in mm — min 13mm inside, 15mm outside
    #[serde(default)]
    pub glaslat_width: Option<f64>,
    /// Glaslat height in mm — min 17mm
    #[serde(default)]
    pub glaslat_height: Option<f64>,
    /// Achterhout (remaining wood behind sponning) in mm — min 13mm
    #[serde(default)]
    pub achterhout: Option<f64>,
}

/// Sponning type classification per KVT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SponningType {
    /// Raamsponning aan binnenzijde — standaard draairamen
    Binnensponning,
    /// Raamsponning aan buitenzijde — naar buiten draaiend
    Buitensponning,
    /// Sponning aan beide zijden — tussenstijlen, stolpramen
    DubbeleSponning,
    /// Binnensponning met opdek + dubbele rubber — draaikiepramen
    Draaikiep,
    /// Geen sponning
    Geen,
}

impl Default for SponningType {
    fn default() -> Self {
        Self::Binnensponning
    }
}

/// KVT profile type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KvtProfileType {
    /// Stijl + bovendorpel
    A,
    /// Stijl met waterhol
    A1,
    /// Tussenstijl (dubbele sponning, min 90mm breed)
    B,
    /// Tussenstijl met waterhol
    B1,
    /// Onderdorpel (9° helling, niet mogelijk in 67mm)
    C,
    /// Tussendorpel / kalf
    D,
    /// Raamhout (draaiend deel)
    R,
    /// Vrij getekend profiel
    Custom,
}

/// Sponning (rabbet/rebate) dimensions — extended with KVT fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SponningInfo {
    /// Sponning type
    #[serde(default)]
    pub sponning_type: SponningType,
    /// Sponning width in mm (sponningbreedte)
    pub width: f64,
    /// Sponning depth/height in mm (sponninghoogte, min 17mm)
    pub depth: f64,
    /// Position: "binnen", "buiten", or "midden"
    pub position: String,
    /// Draaikiep: opdekbreedte (raam overlap, typically 13mm)
    #[serde(default)]
    pub opdek_width: Option<f64>,
    /// Draaikiep: number of rubber gasket grooves (2-3)
    #[serde(default)]
    pub rubber_count: Option<u8>,
    /// Dubbele sponning: second sponning width
    #[serde(default)]
    pub second_width: Option<f64>,
    /// Dubbele sponning: second sponning depth
    #[serde(default)]
    pub second_depth: Option<f64>,
    /// Dubbele sponning: kernhout (core wood remaining, min 20mm)
    #[serde(default)]
    pub kernhout: Option<f64>,
    /// Onderdorpel: slope angle in degrees (min 9°)
    #[serde(default)]
    pub slope_degrees: Option<f64>,
}

/// A groove/sponning figure — a sequence of points defining one side of the profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrooveFigure {
    /// Points defining the groove contour as [x, y] offsets
    pub points: Vec<[f64; 2]>,
    /// Groove category
    pub category: GrooveCategory,
    /// Description (e.g., "Raamsponning 12×17")
    #[serde(default)]
    pub label: String,
}

/// Groove figure category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrooveCategory {
    /// Glass/panel groove (vaksponning)
    Vaksponning,
    /// Edge groove — kalksponning
    Randsponning,
    /// Wall connection groove
    Muursponning,
    /// Milling/profiling (rounding, chamfer)
    Omfrezing,
    /// Rubber gasket groove
    RubberGroef,
    /// Hardware groove (Eurogroef 16×13mm)
    Eurogroef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileApplication {
    // Legacy values (backward compatible)
    Frame,
    Sash,
    Divider,
    Sill,
    // New detailed classifications
    /// A: Kozijnstijl (frame stile, vertical)
    KozijnStijl,
    /// Kozijndorpel (frame rail, horizontal)
    KozijnDorpel,
    /// Bovendorpel (top rail)
    Bovendorpel,
    /// B: Tussenstijl (mullion, vertical divider)
    Tussenstijl,
    /// D: Tussendorpel (transom, horizontal divider)
    Tussendorpel,
    /// C: Onderdorpel (bottom sill, min 90mm, 9° slope)
    Onderdorpel,
    /// R: Raamstijl (window sash stile)
    RaamStijl,
    /// Raamdorpel (window sash rail)
    RaamDorpel,
    /// Deurstijl (door stile)
    DeurStijl,
    /// Deurdorpel (door rail)
    DeurDorpel,
    /// Glaslat (glass bead)
    Glaslat,
    /// Spouwlat (cavity batten)
    Spouwlat,
}
