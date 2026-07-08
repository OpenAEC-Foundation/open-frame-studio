use crate::kozijn::Material;
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
    /// Sponninghoogte in mm — full rebate height in the face plane, KVT
    /// minimum 17 mm for wood (source: `SponningInfo.depth`). For PVC this is
    /// the Glasfalzhöhe (VEKA Softline 82 MD: 28 mm), which is larger than
    /// the drawn glass edge cover — see `glasinval`.
    #[serde(default)]
    pub sponning_hoogte: Option<f64>,
    /// Glaslat (glazing bead) face width (oplegbreedte) in mm (source:
    /// `ProfileDefinition.glaslat_width`, KVT min 13 binnen / 15 buiten).
    #[serde(default)]
    pub glaslat_breedte: Option<f64>,
    /// Aanzichtbreedte (sightline) in mm — visible face width after glazing
    /// (source: `ProfileDefinition.sightline`).
    #[serde(default)]
    pub aanzichtbreedte: Option<f64>,
    /// Glasinval (Glaseinstand) in mm — how far the profile covers the glass
    /// edge in the face plane, where that differs from the full rebate
    /// height. PVC systems list both (VEKA Softline 82 MD datasheet:
    /// "Glasfalzhöhe 28 / Glaseinstand 20" — the front view must show 20,
    /// not 28). Wood profiles leave this empty: there `sponning_hoogte`
    /// *is* the glasinval (KVT 12.2). Source: `ProfileDefinition.glasinval`.
    #[serde(default)]
    pub glasinval: Option<f64>,
    /// Glaslathoogte in mm — bead height in the face plane (source:
    /// `ProfileDefinition.glaslat_height`; KVT 12.3.2: >= 17 mm and >= the
    /// sponninghoogte). Drives the bead line in the front view (offset
    /// glaslathoogte − sponninghoogte inside the day line: 0 for a 17x17
    /// bead on a 17 mm sponning, 11 for a 28 mm handelslat) and the KVT
    /// bead cut lengths in production.
    #[serde(default)]
    pub glaslat_hoogte: Option<f64>,
}

/// Norm/system fallback for the glasinval (glass edge cover) per frame
/// material, in mm. Applied when a kozijn *has* a profile snapshot whose
/// sponning values are missing; kozijnen without any snapshot emit nothing
/// (the frontend keeps its classic 17 mm fallback) so old projects render
/// byte-identically.
///
/// Sources (onderzoeksronde 2026-07):
/// - Hout en hout-aluminium (houten kern): 17 mm norm-minimum
///   (KVT 12.2; Uitleg NPR 3577, Kenniscentrum Glas 2018).
/// - PVC: geen universele norm — systeemwaarden 18-20 mm (VEKA Softline 82
///   MD datasheet: Glaseinstand 20 mm; GEALAN S 9000: glasaanslag 18 mm).
///   Fallback 20 mm (VEKA); systemen horen hun eigen waarde in de
///   profieldata te zetten.
/// - Aluminium: systeem-specifiek 13,5-27 mm (Reynaers SlimLine 38: 13,5;
///   MasterLine 8: 27 per reynaers.com/ATG 3067). Fallback 25 mm = de
///   bestaande bibliotheekwaarde voor CS 77 / Schüco AWS, waarvan de echte
///   sponninghoogte niet publiek geverifieerd is.
pub fn norm_glasinval(material: &Material) -> f64 {
    match material {
        Material::Wood(_) | Material::WoodAluminum => 17.0,
        Material::Pvc => 20.0,
        Material::Aluminum => 25.0,
    }
}

/// Fallback glaslat (glazing bead) height per frame material, in mm.
///
/// - Hout / hout-aluminium: KVT 12.3.2 — hoogte >= 17 mm én >= de
///   sponninghoogte (fabrieksstandaard 17x17 op een 17 mm sponning).
/// - PVC: VEKA-glaslatten hebben een vaste cliphoogte van 25 mm; de breedte
///   volgt uit de glasdikte (Baltic Technikheft VEKA 82MD p.35 — één bron).
/// - Aluminium: Reynaers MasterLine 8 kliklat 25 mm (reynaers.com technical
///   info + ML8-bestek).
pub fn norm_glaslat_hoogte(material: &Material, sponning_hoogte: f64) -> f64 {
    match material {
        Material::Wood(_) | Material::WoodAluminum => sponning_hoogte.max(17.0),
        Material::Pvc | Material::Aluminum => 25.0,
    }
}

impl ProfileSnapshot {
    /// Glass edge cover for the 2D front view, in mm: the explicit
    /// `glasinval` when the profile carries one (PVC: Glaseinstand), else the
    /// sponninghoogte (identical for wood), else the material norm.
    pub fn resolved_glasinval(&self, material: &Material) -> f64 {
        self.glasinval
            .or(self.sponning_hoogte)
            .unwrap_or_else(|| norm_glasinval(material))
    }

    /// Physical rebate height for size chains, in mm — the sponningmaat is
    /// dagmaat + 2 × this value (KVT 12.2). Prefers the full sponninghoogte
    /// (PVC: Falzhöhe) over the glasinval; falls back to the material norm.
    pub fn resolved_sponning_hoogte(&self, material: &Material) -> f64 {
        self.sponning_hoogte
            .or(self.glasinval)
            .unwrap_or_else(|| norm_glasinval(material))
    }

    /// Glazing bead height in mm: the profile's own value, else the material
    /// fallback (wood: max(17, sponninghoogte) per KVT 12.3.2).
    pub fn resolved_glaslat_hoogte(&self, material: &Material) -> f64 {
        self.glaslat_hoogte.unwrap_or_else(|| {
            norm_glaslat_hoogte(material, self.resolved_sponning_hoogte(material))
        })
    }
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
    /// Glasinval (Glaseinstand) in mm — glass edge cover in the face plane
    /// where it differs from the full sponning height. PVC systems document
    /// both (VEKA Softline 82 MD: Glasfalzhöhe 28 / Glaseinstand 20); wood
    /// profiles omit it (`sponning.depth` is the glasinval there, KVT 12.2).
    /// JSON key: `glasinval`.
    #[serde(default)]
    pub glasinval: Option<f64>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::WoodType;

    const WOOD: Material = Material::Wood(WoodType::Meranti);

    #[test]
    fn norm_glasinval_per_material() {
        // Hout: KVT 12.2 norm-minimum 17; PVC: VEKA Glaseinstand 20 (geen
        // universele norm); alu: 25 = bestaande bibliotheekwaarde (systemen
        // lopen 13,5-27).
        assert_eq!(norm_glasinval(&WOOD), 17.0);
        assert_eq!(norm_glasinval(&Material::WoodAluminum), 17.0);
        assert_eq!(norm_glasinval(&Material::Pvc), 20.0);
        assert_eq!(norm_glasinval(&Material::Aluminum), 25.0);
    }

    #[test]
    fn resolved_glasinval_prefers_explicit_then_sponning_then_norm() {
        // Empty snapshot → material norm.
        let empty = ProfileSnapshot::default();
        assert_eq!(empty.resolved_glasinval(&WOOD), 17.0);
        assert_eq!(empty.resolved_glasinval(&Material::Pvc), 20.0);
        assert_eq!(empty.resolved_glasinval(&Material::Aluminum), 25.0);

        // Only sponninghoogte set (wood library data) → that value.
        let wood = ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            ..Default::default()
        };
        assert_eq!(wood.resolved_glasinval(&WOOD), 17.0);

        // PVC with Falzhöhe + Glaseinstand (VEKA 82 MD: 28/20): the drawn
        // glass edge cover is the Glaseinstand, not the Falzhöhe.
        let veka = ProfileSnapshot {
            sponning_hoogte: Some(28.0),
            glasinval: Some(20.0),
            ..Default::default()
        };
        assert_eq!(veka.resolved_glasinval(&Material::Pvc), 20.0);
        // ...while size chains (sponningmaat) keep the full rebate height.
        assert_eq!(veka.resolved_sponning_hoogte(&Material::Pvc), 28.0);
    }

    #[test]
    fn resolved_sponning_hoogte_falls_back_to_glasinval_then_norm() {
        let only_glasinval = ProfileSnapshot {
            glasinval: Some(18.0),
            ..Default::default()
        };
        assert_eq!(only_glasinval.resolved_sponning_hoogte(&Material::Pvc), 18.0);
        assert_eq!(
            ProfileSnapshot::default().resolved_sponning_hoogte(&WOOD),
            17.0
        );
    }

    #[test]
    fn resolved_glaslat_hoogte_wood_follows_kvt_12_3_2() {
        // Fabrieksstandaard 17x17 on the 17 mm sponning.
        assert_eq!(ProfileSnapshot::default().resolved_glaslat_hoogte(&WOOD), 17.0);
        // KVT 12.3.2: bead height >= sponninghoogte — a 20 mm sponning
        // (KVT-alternatief) lifts the fallback bead to 20.
        let deep = ProfileSnapshot {
            sponning_hoogte: Some(20.0),
            ..Default::default()
        };
        assert_eq!(deep.resolved_glaslat_hoogte(&WOOD), 20.0);
        // An explicit bead height (28 mm handelslat) wins.
        let handelslat = ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            glaslat_hoogte: Some(28.0),
            ..Default::default()
        };
        assert_eq!(handelslat.resolved_glaslat_hoogte(&WOOD), 28.0);
        // PVC/alu kliklat: 25 mm constant.
        assert_eq!(
            ProfileSnapshot::default().resolved_glaslat_hoogte(&Material::Pvc),
            25.0
        );
        assert_eq!(
            ProfileSnapshot::default().resolved_glaslat_hoogte(&Material::Aluminum),
            25.0
        );
    }

    #[test]
    fn profile_definition_glasinval_deserializes_and_defaults() {
        // Library JSONs without the new key keep working (None)...
        let json = r#"{
            "id": "test", "name": "Test", "material": "pvc",
            "materialSubtype": null, "width": 82.0, "depth": 82.0,
            "sightline": 61.0, "glazingRebate": 28.0, "crossSection": [],
            "ufValue": 1.1, "applicableAs": ["frame"]
        }"#;
        let p: ProfileDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(p.glasinval, None);
        // ...and the new key lands in the field.
        let json2 = json.replace("\"ufValue\"", "\"glasinval\": 20.0, \"ufValue\"");
        let p2: ProfileDefinition = serde_json::from_str(&json2).unwrap();
        assert_eq!(p2.glasinval, Some(20.0));
    }
}
