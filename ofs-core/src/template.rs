use serde::{Deserialize, Serialize};
use crate::kozijn::Material;
use crate::profile::ProfileRef;

/// A kozijn sjabloon (template) that defines all profiles for a complete frame.
/// Equivalent to MatrixKozijn's "Merk Sjabloon" — one click assigns all profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KozijnSjabloon {
    pub id: String,
    pub name: String,
    pub description: String,
    pub material: Material,
    pub profile_series: String,

    // Frame profiles per KVT position
    /// A: Stijl links/rechts (vertical frame members)
    pub stijl_profile: ProfileRef,
    /// Bovendorpel (top rail)
    pub bovendorpel_profile: ProfileRef,
    /// C: Onderdorpel (bottom sill, with slope)
    pub onderdorpel_profile: ProfileRef,
    /// B: Tussenstijl (mullion, typically wider for double sponning)
    pub tussenstijl_profile: ProfileRef,
    /// D: Tussendorpel (transom)
    pub tussendorpel_profile: ProfileRef,

    // Sash profiles
    /// R: Raamhout (window sash wood)
    pub raamhout_profile: ProfileRef,
    /// Deurhout (door wood, typically heavier)
    pub deurhout_profile: ProfileRef,

    // Accessories
    /// Glaslat (glass bead)
    pub glaslat_profile: ProfileRef,
    /// Spouwlat (cavity batten)
    pub spouwlat_profile: ProfileRef,

    // Defaults
    pub default_glazing: GlazingPreset,
    pub color_inside: String,
    pub color_outside: String,

    // Derived dimensions (for quick access without profile lookup)
    pub frame_width: f64,
    pub frame_depth: f64,
    pub sash_width: f64,
    pub sash_depth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlazingPreset {
    pub glass_type: String,
    pub thickness_mm: f64,
    pub ug_value: f64,
    pub spacer_type: String,
}

impl Default for GlazingPreset {
    fn default() -> Self {
        Self {
            glass_type: "HR++".into(),
            thickness_mm: 24.0,
            ug_value: 1.0,
            spacer_type: "warm-edge".into(),
        }
    }
}

/// Built-in sjablonen
pub fn builtin_sjablonen() -> Vec<KozijnSjabloon> {
    vec![
        KozijnSjabloon {
            id: "standaard-67-meranti".into(),
            name: "Standaard 67mm Meranti".into(),
            description: "Standaard houten kozijn, 67mm serie, Meranti. Geschikt voor HR++ beglazing.".into(),
            material: Material::Wood(crate::kozijn::WoodType::Meranti),
            profile_series: "67".into(),
            stijl_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 67x114mm".into() },
            bovendorpel_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 67x114mm".into() },
            onderdorpel_profile: ProfileRef { id: "wood-meranti-67x150".into(), name: "Meranti 67x114mm (onderdorpel)".into() },
            tussenstijl_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 67x114mm".into() },
            tussendorpel_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 67x114mm".into() },
            // Raamhout: KVT-draaikiep 69x90 (zichtdeel 55). "54x67" bestaat niet — kleinste KVT is 54x78.
            raamhout_profile: ProfileRef { id: "raamhout-69x90".into(), name: "Raamhout 69x90mm (draaikiep)".into() },
            deurhout_profile: ProfileRef { id: "deur-meranti-67x114".into(), name: "Deurhout 67x114mm".into() },
            glaslat_profile: ProfileRef { id: "glaslat-17x17".into(), name: "Glaslat 17x17mm".into() },
            spouwlat_profile: ProfileRef { id: "spouwlat-22x100".into(), name: "Spouwlat 22x100mm".into() },
            default_glazing: GlazingPreset::default(),
            color_inside: "RAL9010".into(),
            color_outside: "RAL9010".into(),
            frame_width: 67.0,
            frame_depth: 114.0,
            sash_width: 69.0,
            sash_depth: 90.0,
        },
        KozijnSjabloon {
            id: "standaard-67-accoya".into(),
            name: "Standaard 67mm Accoya".into(),
            description: "Duurzaam houten kozijn, 67mm serie, Accoya. Rot-bestendig, geschikt voor HR++.".into(),
            material: Material::Wood(crate::kozijn::WoodType::Accoya),
            profile_series: "67".into(),
            stijl_profile: ProfileRef { id: "wood-accoya-67x114".into(), name: "Accoya 67x114mm".into() },
            bovendorpel_profile: ProfileRef { id: "wood-accoya-67x114".into(), name: "Accoya 67x114mm".into() },
            onderdorpel_profile: ProfileRef { id: "wood-accoya-67x114-dorpel".into(), name: "Accoya 67x114mm (onderdorpel)".into() },
            tussenstijl_profile: ProfileRef { id: "wood-accoya-67x114".into(), name: "Accoya 67x114mm".into() },
            tussendorpel_profile: ProfileRef { id: "wood-accoya-67x114".into(), name: "Accoya 67x114mm".into() },
            raamhout_profile: ProfileRef { id: "raamhout-69x90".into(), name: "Raamhout 69x90mm (draaikiep)".into() },
            deurhout_profile: ProfileRef { id: "deur-meranti-67x114".into(), name: "Deurhout 67x114mm".into() },
            glaslat_profile: ProfileRef { id: "glaslat-17x17".into(), name: "Glaslat 17x17mm".into() },
            spouwlat_profile: ProfileRef { id: "spouwlat-22x100".into(), name: "Spouwlat 22x100mm".into() },
            default_glazing: GlazingPreset::default(),
            color_inside: "RAL9010".into(),
            color_outside: "RAL9010".into(),
            frame_width: 67.0,
            frame_depth: 114.0,
            sash_width: 69.0,
            sash_depth: 90.0,
        },
        KozijnSjabloon {
            id: "zwaar-78-meranti".into(),
            name: "Zwaar 78mm Meranti".into(),
            description: "Zwaar kozijn, 78mm serie. Geschikt voor HR+++ triple beglazing.".into(),
            material: Material::Wood(crate::kozijn::WoodType::Meranti),
            profile_series: "78".into(),
            stijl_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 78x114mm".into() },
            bovendorpel_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 78x114mm".into() },
            onderdorpel_profile: ProfileRef { id: "wood-meranti-67x139-dorpel".into(), name: "Meranti 67x139mm (onderdorpel zwaar)".into() },
            tussenstijl_profile: ProfileRef { id: "wood-meranti-90x114-tussenstijl".into(), name: "Meranti 90x114mm (tussenstijl)".into() },
            tussendorpel_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 78x114mm".into() },
            raamhout_profile: ProfileRef { id: "raamhout-78x90".into(), name: "Raamhout 78x90mm (triple)".into() },
            deurhout_profile: ProfileRef { id: "deur-meranti-67x114".into(), name: "Deurhout 67x114mm".into() },
            glaslat_profile: ProfileRef { id: "glaslat-17x28".into(), name: "Glaslat 17x28mm".into() },
            spouwlat_profile: ProfileRef { id: "spouwlat-22x120".into(), name: "Spouwlat 22x120mm".into() },
            default_glazing: GlazingPreset {
                glass_type: "HR+++".into(),
                thickness_mm: 36.0,
                ug_value: 0.7,
                spacer_type: "super-spacer".into(),
            },
            color_inside: "RAL9010".into(),
            color_outside: "RAL9010".into(),
            frame_width: 78.0,
            frame_depth: 114.0,
            sash_width: 78.0,
            sash_depth: 90.0,
        },
        KozijnSjabloon {
            id: "passief-90-meranti".into(),
            name: "Passief 90mm Meranti".into(),
            description: "Passiefhuis kozijn, 90mm serie. Triple HR+++ glas, maximale isolatie.".into(),
            material: Material::Wood(crate::kozijn::WoodType::Meranti),
            profile_series: "90".into(),
            stijl_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 90x114mm".into() },
            bovendorpel_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 90x114mm".into() },
            onderdorpel_profile: ProfileRef { id: "wood-meranti-67x139-dorpel".into(), name: "Meranti 67x139mm (onderdorpel zwaar)".into() },
            tussenstijl_profile: ProfileRef { id: "wood-meranti-90x114-tussenstijl".into(), name: "Meranti 90x114mm (tussenstijl)".into() },
            tussendorpel_profile: ProfileRef { id: "wood-meranti-67x114".into(), name: "Meranti 90x114mm".into() },
            raamhout_profile: ProfileRef { id: "raamhout-78x90".into(), name: "Raamhout 78x90mm (triple)".into() },
            deurhout_profile: ProfileRef { id: "deur-meranti-67x114".into(), name: "Deurhout 67x114mm".into() },
            glaslat_profile: ProfileRef { id: "glaslat-20x34-5".into(), name: "Glaslat 20x34,5mm".into() },
            spouwlat_profile: ProfileRef { id: "spouwlat-22x140".into(), name: "Spouwlat 22x140mm".into() },
            default_glazing: GlazingPreset {
                glass_type: "Triple".into(),
                thickness_mm: 44.0,
                ug_value: 0.5,
                spacer_type: "super-spacer".into(),
            },
            color_inside: "RAL9010".into(),
            color_outside: "RAL9010".into(),
            frame_width: 90.0,
            frame_depth: 114.0,
            sash_width: 78.0,
            sash_depth: 90.0,
        },
        // ── Kunststof (PVC) ──────────────────────────────────────────────
        // material: Pvc drives verstek (45°) gelaste hoeken in productie/geometrie.
        KozijnSjabloon {
            id: "kunststof-82-veka".into(),
            name: "Kunststof 82mm VEKA Softline".into(),
            description: "Kunststof kozijn, VEKA Softline 82 MD (7-kamer). Verstek-hoeken (gelast), HR++ standaard.".into(),
            material: Material::Pvc,
            profile_series: "82".into(),
            stijl_profile: ProfileRef { id: "veka-softline82".into(), name: "VEKA Softline 82 MD (kozijn)".into() },
            bovendorpel_profile: ProfileRef { id: "veka-softline82".into(), name: "VEKA Softline 82 MD (kozijn)".into() },
            onderdorpel_profile: ProfileRef { id: "veka-softline82".into(), name: "VEKA Softline 82 MD (onderdorpel)".into() },
            tussenstijl_profile: ProfileRef { id: "veka-softline82".into(), name: "VEKA Softline 82 MD (stijl)".into() },
            tussendorpel_profile: ProfileRef { id: "veka-softline82".into(), name: "VEKA Softline 82 MD (dorpel)".into() },
            raamhout_profile: ProfileRef { id: "veka-softline82".into(), name: "VEKA Softline 82 MD (vleugel)".into() },
            deurhout_profile: ProfileRef { id: "veka-softline82".into(), name: "VEKA Softline 82 MD (deur)".into() },
            glaslat_profile: ProfileRef { id: "veka-softline82".into(), name: "VEKA glaslat (clip)".into() },
            spouwlat_profile: ProfileRef { id: "veka-softline82".into(), name: "n.v.t. (kunststof)".into() },
            default_glazing: GlazingPreset::default(),
            color_inside: "RAL9016".into(),
            color_outside: "RAL9016".into(),
            frame_width: 73.0,
            frame_depth: 82.0,
            sash_width: 84.0,
            sash_depth: 82.0,
        },
        KozijnSjabloon {
            id: "kunststof-88-kommerling".into(),
            name: "Kunststof 88mm Kömmerling".into(),
            description: "Kunststof passief kozijn, Kömmerling 88 MD (7-kamer, staalversterkt). Triple glas.".into(),
            material: Material::Pvc,
            profile_series: "88".into(),
            stijl_profile: ProfileRef { id: "kommerling-88md".into(), name: "Kömmerling 88 MD (kozijn)".into() },
            bovendorpel_profile: ProfileRef { id: "kommerling-88md".into(), name: "Kömmerling 88 MD (kozijn)".into() },
            onderdorpel_profile: ProfileRef { id: "kommerling-88md".into(), name: "Kömmerling 88 MD (onderdorpel)".into() },
            tussenstijl_profile: ProfileRef { id: "kommerling-88md".into(), name: "Kömmerling 88 MD (stijl)".into() },
            tussendorpel_profile: ProfileRef { id: "kommerling-88md".into(), name: "Kömmerling 88 MD (dorpel)".into() },
            raamhout_profile: ProfileRef { id: "kommerling-88md".into(), name: "Kömmerling 88 MD (vleugel)".into() },
            deurhout_profile: ProfileRef { id: "kommerling-88md".into(), name: "Kömmerling 88 MD (deur)".into() },
            glaslat_profile: ProfileRef { id: "kommerling-88md".into(), name: "Kömmerling glaslat (clip)".into() },
            spouwlat_profile: ProfileRef { id: "kommerling-88md".into(), name: "n.v.t. (kunststof)".into() },
            default_glazing: GlazingPreset {
                glass_type: "Triple".into(),
                thickness_mm: 44.0,
                ug_value: 0.6,
                spacer_type: "warm-edge".into(),
            },
            color_inside: "RAL9016".into(),
            color_outside: "RAL7016".into(),
            frame_width: 74.0,
            frame_depth: 88.0,
            sash_width: 80.0,
            sash_depth: 88.0,
        },
        // ── Aluminium ────────────────────────────────────────────────────
        // material: Aluminum drives verstek (45°) hoeken in productie/geometrie.
        KozijnSjabloon {
            id: "aluminium-77-reynaers".into(),
            name: "Aluminium 77mm Reynaers CS 77".into(),
            description: "Aluminium kozijn, Reynaers ConceptSystem 77 (thermisch onderbroken). Verstek-hoeken.".into(),
            material: Material::Aluminum,
            profile_series: "77".into(),
            stijl_profile: ProfileRef { id: "reynaers-cs77-standard".into(), name: "Reynaers CS 77 (kozijn)".into() },
            bovendorpel_profile: ProfileRef { id: "reynaers-cs77-standard".into(), name: "Reynaers CS 77 (kozijn)".into() },
            onderdorpel_profile: ProfileRef { id: "reynaers-cs77-standard".into(), name: "Reynaers CS 77 (onderdorpel)".into() },
            tussenstijl_profile: ProfileRef { id: "reynaers-cs77-standard".into(), name: "Reynaers CS 77 (stijl)".into() },
            tussendorpel_profile: ProfileRef { id: "reynaers-cs77-standard".into(), name: "Reynaers CS 77 (dorpel)".into() },
            raamhout_profile: ProfileRef { id: "reynaers-cs77-standard".into(), name: "Reynaers CS 77 (vleugel)".into() },
            deurhout_profile: ProfileRef { id: "reynaers-cs77-standard".into(), name: "Reynaers CS 77 (deur)".into() },
            glaslat_profile: ProfileRef { id: "reynaers-cs77-standard".into(), name: "Reynaers glaslat (klik)".into() },
            spouwlat_profile: ProfileRef { id: "reynaers-cs77-standard".into(), name: "n.v.t. (aluminium)".into() },
            default_glazing: GlazingPreset::default(),
            color_inside: "RAL9005".into(),
            color_outside: "RAL9005".into(),
            frame_width: 51.0,
            frame_depth: 77.0,
            sash_width: 51.0,
            sash_depth: 87.0,
        },
        KozijnSjabloon {
            id: "aluminium-75-schuco".into(),
            name: "Aluminium 75mm Schüco AWS 75.SI+".into(),
            description: "Aluminium super-geïsoleerd kozijn, Schüco AWS 75.SI+. Verstek-hoeken, triple mogelijk.".into(),
            material: Material::Aluminum,
            profile_series: "75".into(),
            stijl_profile: ProfileRef { id: "schuco-aws75si-plus".into(), name: "Schüco AWS 75.SI+ (kozijn)".into() },
            bovendorpel_profile: ProfileRef { id: "schuco-aws75si-plus".into(), name: "Schüco AWS 75.SI+ (kozijn)".into() },
            onderdorpel_profile: ProfileRef { id: "schuco-aws75si-plus".into(), name: "Schüco AWS 75.SI+ (onderdorpel)".into() },
            tussenstijl_profile: ProfileRef { id: "schuco-aws75si-plus".into(), name: "Schüco AWS 75.SI+ (stijl)".into() },
            tussendorpel_profile: ProfileRef { id: "schuco-aws75si-plus".into(), name: "Schüco AWS 75.SI+ (dorpel)".into() },
            raamhout_profile: ProfileRef { id: "schuco-aws75si-plus".into(), name: "Schüco AWS 75.SI+ (vleugel)".into() },
            deurhout_profile: ProfileRef { id: "schuco-aws75si-plus".into(), name: "Schüco AWS 75.SI+ (deur)".into() },
            glaslat_profile: ProfileRef { id: "schuco-aws75si-plus".into(), name: "Schüco glaslat (klik)".into() },
            spouwlat_profile: ProfileRef { id: "schuco-aws75si-plus".into(), name: "n.v.t. (aluminium)".into() },
            default_glazing: GlazingPreset {
                glass_type: "Triple".into(),
                thickness_mm: 44.0,
                ug_value: 0.6,
                spacer_type: "warm-edge".into(),
            },
            color_inside: "RAL7016".into(),
            color_outside: "RAL7016".into(),
            frame_width: 41.0,
            frame_depth: 75.0,
            sash_width: 59.0,
            sash_depth: 85.0,
        },
    ]
}

/// Get a sjabloon by id, falling back to the default (standaard 67mm)
pub fn get_sjabloon(id: &str) -> KozijnSjabloon {
    builtin_sjablonen()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap_or_else(|| builtin_sjablonen().into_iter().next().unwrap())
}

/// Get the default sjabloon
pub fn default_sjabloon() -> KozijnSjabloon {
    get_sjabloon("standaard-67-meranti")
}
