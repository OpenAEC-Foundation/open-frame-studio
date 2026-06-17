//! Declaration of Performance (DoP) generation — module 6 (compliance depth).
//!
//! Produces a CPR (Construction Products Regulation 305/2011) Declaration of
//! Performance for a kozijn under the harmonised standard EN 14351-1
//! (windows and external pedestrian doorsets). The declared performances are
//! aggregated from the existing thermal and performance-classification logic;
//! fields that require manufacturer/notified-body input (which OFS does not
//! model) are emitted as clearly-marked placeholders to be completed.

use serde::{Deserialize, Serialize};

use crate::kozijn::{Kozijn, Material, PanelType, WoodType};
use crate::performance_class::classify_performance;
use crate::profile::ProfileDefinition;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DopCharacteristic {
    /// Essential characteristic (Dutch label).
    pub characteristic: String,
    /// Declared performance / class, or "NPD" (No Performance Determined).
    pub performance: String,
    /// Reference test/classification standard.
    pub standard: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclarationOfPerformance {
    pub dop_number: String,
    pub product_type: String,
    pub kozijn_mark: String,
    pub kozijn_name: String,
    pub intended_use: String,
    pub harmonised_standard: String,
    pub avcp_system: String,
    pub notified_body: String,
    pub manufacturer: String,
    pub material: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub declared_uw: f64,
    pub characteristics: Vec<DopCharacteristic>,
    pub conformity_statement: String,
}

fn material_label(m: &Material) -> String {
    match m {
        Material::Wood(w) => match w {
            WoodType::Meranti => "Hout (Meranti)",
            WoodType::Accoya => "Hout (Accoya)",
            WoodType::Vuren => "Hout (Vuren)",
            WoodType::Eiken => "Hout (Eiken)",
        }
        .to_string(),
        Material::Aluminum => "Aluminium".to_string(),
        Material::Pvc => "Kunststof (PVC)".to_string(),
        Material::WoodAluminum => "Hout-aluminium".to_string(),
    }
}

/// Whether any cell is an external pedestrian door (affects intended use wording).
fn has_door(kozijn: &Kozijn) -> bool {
    kozijn.cells.iter().any(|c| matches!(c.panel_type, PanelType::Door))
}

/// Generate a draft Declaration of Performance for the given kozijn.
pub fn generate_dop(kozijn: &Kozijn, profiles: &[ProfileDefinition]) -> DeclarationOfPerformance {
    let perf = classify_performance(kozijn, profiles);
    let uw = crate::thermal::calculate_uw(kozijn, profiles).uw_value;

    let material = material_label(&kozijn.frame.material);
    let type_label = kozijn
        .cells
        .first()
        .map(|c| c.panel_type.label_nl())
        .unwrap_or("Kozijn");
    let product_type = format!("{} — {}", material, type_label);

    let intended_use = if has_door(kozijn) {
        "Ramen en buitendeuren voor gebruik in gebouwen (EN 14351-1)".to_string()
    } else {
        "Ramen voor gebruik in gebouwen (EN 14351-1)".to_string()
    };

    let characteristics = vec![
        DopCharacteristic {
            characteristic: "Waterdichtheid".into(),
            performance: perf.water_tightness.clone(),
            standard: "EN 1027 / EN 12208".into(),
        },
        DopCharacteristic {
            characteristic: "Weerstand tegen windbelasting".into(),
            performance: perf.wind_resistance.clone(),
            standard: "EN 12211 / EN 12210".into(),
        },
        DopCharacteristic {
            characteristic: "Luchtdoorlatendheid".into(),
            performance: perf.air_permeability.clone(),
            standard: "EN 1026 / EN 12207".into(),
        },
        DopCharacteristic {
            characteristic: "Warmtegeleidingscoëfficiënt (Uw)".into(),
            performance: format!("{:.2} W/m\u{00b2}K", uw),
            standard: "EN ISO 10077-1".into(),
        },
        DopCharacteristic {
            characteristic: "Luchtgeluidsisolatie (Rw)".into(),
            performance: perf.sound_insulation.clone(),
            standard: "EN ISO 717-1".into(),
        },
        DopCharacteristic {
            characteristic: "Inbraakwerendheid".into(),
            performance: perf.burglar_resistance.clone(),
            standard: "EN 1627".into(),
        },
        DopCharacteristic {
            characteristic: "Draagvermogen veiligheidsvoorzieningen".into(),
            performance: "NPD".into(),
            standard: "EN 14351-1".into(),
        },
        DopCharacteristic {
            characteristic: "Stootvastheid".into(),
            performance: "NPD".into(),
            standard: "EN 13049".into(),
        },
        DopCharacteristic {
            characteristic: "Gevaarlijke stoffen".into(),
            performance: "Geen (NPD)".into(),
            standard: "EN 14351-1".into(),
        },
    ];

    DeclarationOfPerformance {
        dop_number: format!("OFS-DoP-{}", kozijn.mark),
        product_type,
        kozijn_mark: kozijn.mark.clone(),
        kozijn_name: kozijn.name.clone(),
        intended_use,
        harmonised_standard: "EN 14351-1:2006+A2:2016".into(),
        avcp_system: "Systeem 3".into(),
        notified_body: "[Aangemelde instantie / NB-nummer invullen]".into(),
        manufacturer: "[Fabrikantgegevens invullen]".into(),
        material,
        width_mm: kozijn.frame.outer_width,
        height_mm: kozijn.frame.outer_height,
        declared_uw: (uw * 100.0).round() / 100.0,
        characteristics,
        conformity_statement:
            "De prestaties van het hierboven omschreven product zijn conform de \
             aangegeven prestaties. Deze prestatieverklaring wordt verstrekt onder de \
             uitsluitende verantwoordelijkheid van de fabrikant (CPR 305/2011, art. 4)."
                .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::Kozijn;

    #[test]
    fn dop_has_essential_characteristics() {
        let k = Kozijn::new("Test", "T01", 1000.0, 1500.0);
        let dop = generate_dop(&k, &[]);

        assert_eq!(dop.dop_number, "OFS-DoP-T01");
        assert_eq!(dop.harmonised_standard, "EN 14351-1:2006+A2:2016");
        // Uw must be one of the declared characteristics.
        assert!(dop
            .characteristics
            .iter()
            .any(|c| c.characteristic.contains("Uw")));
        // EN 14351-1 minimum set: water, wind, air, thermal present.
        assert!(dop.characteristics.len() >= 6);
        assert!(dop.declared_uw > 0.0);
        assert_eq!(dop.width_mm, 1000.0);
    }

    #[test]
    fn door_changes_intended_use() {
        let mut k = Kozijn::new("Deur", "D01", 1000.0, 2300.0);
        k.cells[0].panel_type = PanelType::Door;
        let dop = generate_dop(&k, &[]);
        assert!(dop.intended_use.contains("buitendeuren"));
    }
}
