//! Circularity / material passport (module 12).
//!
//! Derives indicative material masses, recyclability and embodied carbon
//! (GWP, A1–A3) per material category from the production data of each kozijn,
//! producing a project-level material passport in the spirit of NMD / Madaster /
//! MPG reporting.
//!
//! All factors here are **indicative generic placeholders** (NMD-category style),
//! not product-specific EPD data. They are meant to give the user an order-of-
//! magnitude circularity picture; replace them with real EPD figures once a
//! product/EPD database is wired in. Masses are derived as follows:
//! - Profiles (frame, dividers, sash): cross-section (parsed from the "WxH"
//!   profile name, falling back to the frame profile) × a material fill factor
//!   × net cut length × density.
//! - Glass: area × IGU thickness × 2.5 kg/(m²·mm) (solid-glass density applied to
//!   the unit thickness — a conservative upper bound).
//! - Glazing beads (glaslatten): bead cross-section × total mitered length × density.
//! - Panels (vakvullingen): area × filling thickness × an effective panel density.
//! - Hardware: indicative steel mass per component.
//! - Gaskets: EPDM linear mass per metre.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::kozijn::{Kozijn, Project};
use crate::panel_filling::FillingType;
use crate::production::compute_production_data;

// ── Indicative material factors ────────────────────────────────────

/// Solid-glass mass per m² per mm of thickness (density 2500 kg/m³).
const GLASS_KG_PER_M2_MM: f64 = 2.5;
/// Indicative EPDM gasket linear mass (kg/m).
const EPDM_KG_PER_M: f64 = 0.06;

#[derive(Debug, Clone, Copy)]
struct CatProps {
    /// Display category name (Dutch).
    name: &'static str,
    density_kg_m3: f64,
    /// Solid fraction of the bounding cross-section (hollow profiles < 1.0).
    fill_factor: f64,
    /// Embodied carbon A1–A3, kg CO₂-eq per kg (indicative).
    co2_per_kg: f64,
    /// Recyclable fraction at end of life, 0..1 (indicative).
    recyclability: f64,
    /// Biobased / renewable material.
    renewable: bool,
}

const GLASS_PROPS: CatProps = CatProps {
    name: "Glas",
    density_kg_m3: 2500.0,
    fill_factor: 1.0,
    co2_per_kg: 1.3,
    recyclability: 0.90,
    renewable: false,
};

const STEEL_PROPS: CatProps = CatProps {
    name: "Beslag (staal)",
    density_kg_m3: 7850.0,
    fill_factor: 1.0,
    co2_per_kg: 1.7,
    recyclability: 0.90,
    renewable: false,
};

const RUBBER_PROPS: CatProps = CatProps {
    name: "Rubber (EPDM)",
    density_kg_m3: 1100.0,
    fill_factor: 1.0,
    co2_per_kg: 2.9,
    recyclability: 0.25,
    renewable: false,
};

/// Material properties for a profile/bead, keyed on the production material name.
fn profile_props(material: &str) -> CatProps {
    let m = material.to_ascii_lowercase();
    if m.contains("hout-alu") {
        // Wood-aluminium composite: wood core + thin aluminium shell.
        CatProps {
            name: "Hout-aluminium",
            density_kg_m3: 600.0,
            fill_factor: 0.85,
            co2_per_kg: 1.6,
            recyclability: 0.75,
            renewable: false,
        }
    } else if m.contains("alu") {
        CatProps {
            name: "Aluminium",
            density_kg_m3: 2700.0,
            fill_factor: 0.18, // hollow extrusion
            co2_per_kg: 8.6,
            recyclability: 0.95,
            renewable: false,
        }
    } else if m.contains("kunststof") || m.contains("pvc") {
        CatProps {
            name: "Kunststof (PVC)",
            density_kg_m3: 1400.0,
            fill_factor: 0.22, // multi-chamber hollow
            co2_per_kg: 2.4,
            recyclability: 0.80,
            renewable: false,
        }
    } else {
        // Wood species — density varies, the rest is shared.
        let density = match m.as_str() {
            "eiken" => 700.0,
            "meranti" => 550.0,
            "accoya" => 510.0,
            "vuren" => 450.0,
            _ => 500.0,
        };
        CatProps {
            name: "Hout",
            density_kg_m3: density,
            fill_factor: 0.92,
            co2_per_kg: 0.45,
            recyclability: 0.60,
            renewable: true,
        }
    }
}

/// Effective panel properties by filling type. Density already accounts for the
/// sandwich construction (core + facings); mass is area × thickness × density.
fn panel_props(filling: FillingType) -> CatProps {
    match filling {
        FillingType::Sandwich => CatProps {
            name: "Paneel",
            density_kg_m3: 250.0,
            fill_factor: 1.0,
            co2_per_kg: 3.2,
            recyclability: 0.35,
            renewable: false,
        },
        FillingType::Solid => CatProps {
            name: "Paneel",
            density_kg_m3: 600.0,
            fill_factor: 1.0,
            co2_per_kg: 0.9,
            recyclability: 0.55,
            renewable: false,
        },
        FillingType::DoorPanel => CatProps {
            name: "Paneel",
            density_kg_m3: 300.0,
            fill_factor: 1.0,
            co2_per_kg: 2.6,
            recyclability: 0.40,
            renewable: false,
        },
        FillingType::Ventilation => CatProps {
            name: "Paneel",
            density_kg_m3: 800.0,
            fill_factor: 1.0,
            co2_per_kg: 6.0,
            recyclability: 0.80,
            renewable: false,
        },
        FillingType::Blind => CatProps {
            name: "Paneel",
            density_kg_m3: 250.0,
            fill_factor: 1.0,
            co2_per_kg: 2.0,
            recyclability: 0.40,
            renewable: false,
        },
    }
}

/// Indicative steel mass (kg) per hardware component.
fn hardware_mass_kg(component: &str) -> f64 {
    match component {
        "Scharnier" => 0.35,
        "Greep" => 0.40,
        "Sluiting" => 0.65,
        _ => 0.30,
    }
}

/// Parse a `WxH` cross-section (mm) from a profile name, e.g. "67x114",
/// "Dorpel 67x114", "Raamhout 67x114". Returns None if no token matches.
fn parse_cross_section_mm(name: &str) -> Option<(f64, f64)> {
    for token in name.split(|c: char| c.is_whitespace()) {
        let lower = token.to_ascii_lowercase();
        if let Some((a, b)) = lower.split_once('x') {
            if let (Ok(w), Ok(h)) = (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
                if w > 0.0 && h > 0.0 {
                    return Some((w, h));
                }
            }
        }
    }
    None
}

// ── Output structures ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialCategory {
    pub category: String,
    pub mass_kg: f64,
    pub recyclable_kg: f64,
    pub recyclability_pct: f64,
    pub co2_kg: f64,
    pub renewable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KozijnCircularity {
    pub kozijn_mark: String,
    pub kozijn_name: String,
    pub categories: Vec<MaterialCategory>,
    pub total_mass_kg: f64,
    pub total_recyclable_kg: f64,
    pub total_co2_kg: f64,
    /// Recyclable fraction of total mass (0–100).
    pub circularity_score: f64,
    pub renewable_mass_kg: f64,
    pub renewable_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCircularityResult {
    pub kozijn_passports: Vec<KozijnCircularity>,
    pub categories: Vec<MaterialCategory>,
    pub total_mass_kg: f64,
    pub total_recyclable_kg: f64,
    pub total_co2_kg: f64,
    pub circularity_score: f64,
    pub renewable_mass_kg: f64,
    pub renewable_pct: f64,
}

// ── Accumulation helpers ────────────────────────────────────────────

#[derive(Default, Clone, Copy)]
struct Acc {
    mass: f64,
    recyclable: f64,
    co2: f64,
    renewable: bool,
}

fn add(map: &mut HashMap<&'static str, Acc>, p: CatProps, mass_kg: f64) {
    if mass_kg <= 0.0 {
        return;
    }
    let e = map.entry(p.name).or_default();
    e.mass += mass_kg;
    e.recyclable += mass_kg * p.recyclability;
    e.co2 += mass_kg * p.co2_per_kg;
    e.renewable = p.renewable;
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn to_categories(map: HashMap<&'static str, Acc>) -> Vec<MaterialCategory> {
    let mut categories: Vec<MaterialCategory> = map
        .into_iter()
        .map(|(name, a)| MaterialCategory {
            category: name.to_string(),
            mass_kg: round2(a.mass),
            recyclable_kg: round2(a.recyclable),
            recyclability_pct: round1(if a.mass > 0.0 {
                a.recyclable / a.mass * 100.0
            } else {
                0.0
            }),
            co2_kg: round2(a.co2),
            renewable: a.renewable,
        })
        .collect();
    categories.sort_by(|a, b| {
        b.mass_kg
            .partial_cmp(&a.mass_kg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    categories
}

// ── Main computation ────────────────────────────────────────────────

/// Build the indicative material passport for a single kozijn.
pub fn compute_kozijn_circularity(kozijn: &Kozijn) -> KozijnCircularity {
    let prod = compute_production_data(kozijn);
    let mut map: HashMap<&'static str, Acc> = HashMap::new();

    let (frame_w, frame_d) = (kozijn.frame.frame_width, kozijn.frame.frame_depth);

    // Profiles: frame, dividers, sash members.
    for c in &prod.cut_list {
        let p = profile_props(&c.material);
        let (w, h) = parse_cross_section_mm(&c.profile_name).unwrap_or((frame_w, frame_d));
        let area_m2 = (w / 1000.0) * (h / 1000.0);
        let len_m = c.net_length_mm / 1000.0;
        let mass = area_m2 * p.fill_factor * len_m * p.density_kg_m3 * c.quantity as f64;
        add(&mut map, p, mass);
    }

    // Glass panes.
    for g in &prod.glass_list {
        let mass = g.area_m2 * g.thickness_mm * GLASS_KG_PER_M2_MM * g.quantity as f64;
        add(&mut map, GLASS_PROPS, mass);
    }

    // Glazing beads (glaslatten). `total_length_mm` already sums the four beads.
    for gl in &prod.glaslat_list {
        let p = profile_props(&gl.material);
        let area_m2 = (gl.width_mm / 1000.0) * (gl.height_mm / 1000.0);
        let len_m = gl.total_length_mm / 1000.0;
        let mass = area_m2 * p.fill_factor * len_m * p.density_kg_m3;
        add(&mut map, p, mass);
    }

    // Panels (vakvullingen) — thickness/type read from the cell's filling.
    for pl in &prod.panel_list {
        let filling = kozijn
            .cells
            .get(pl.cell_index)
            .and_then(|c| c.panel_filling.as_ref());
        let thickness_mm = filling.map(|f| f.thickness_mm).unwrap_or(40.0);
        let ft = filling.map(|f| f.filling_type).unwrap_or(FillingType::Sandwich);
        let p = panel_props(ft);
        let area_m2 = (pl.width_mm / 1000.0) * (pl.height_mm / 1000.0);
        let mass = area_m2 * (thickness_mm / 1000.0) * p.density_kg_m3 * pl.quantity as f64;
        add(&mut map, p, mass);
    }

    // Hardware (steel).
    for h in &prod.hardware_list {
        let mass = hardware_mass_kg(&h.component) * h.quantity as f64;
        add(&mut map, STEEL_PROPS, mass);
    }

    // Gaskets (EPDM rubber).
    let gasket_len_m: f64 = prod
        .gasket_list
        .iter()
        .map(|g| g.length_mm / 1000.0 * g.quantity as f64)
        .sum();
    add(&mut map, RUBBER_PROPS, gasket_len_m * EPDM_KG_PER_M);

    let categories = to_categories(map);
    let total_mass: f64 = categories.iter().map(|c| c.mass_kg).sum();
    let total_recyclable: f64 = categories.iter().map(|c| c.recyclable_kg).sum();
    let total_co2: f64 = categories.iter().map(|c| c.co2_kg).sum();
    let renewable_mass: f64 = categories
        .iter()
        .filter(|c| c.renewable)
        .map(|c| c.mass_kg)
        .sum();

    KozijnCircularity {
        kozijn_mark: kozijn.mark.clone(),
        kozijn_name: kozijn.name.clone(),
        categories,
        total_mass_kg: round2(total_mass),
        total_recyclable_kg: round2(total_recyclable),
        total_co2_kg: round2(total_co2),
        circularity_score: round1(if total_mass > 0.0 {
            total_recyclable / total_mass * 100.0
        } else {
            0.0
        }),
        renewable_mass_kg: round2(renewable_mass),
        renewable_pct: round1(if total_mass > 0.0 {
            renewable_mass / total_mass * 100.0
        } else {
            0.0
        }),
    }
}

/// Build the project-level material passport, aggregating per-kozijn passports.
pub fn calculate_project_circularity(project: &Project) -> ProjectCircularityResult {
    let passports: Vec<KozijnCircularity> = project
        .kozijnen
        .iter()
        .map(compute_kozijn_circularity)
        .collect();

    // Aggregate categories across all kozijnen.
    let mut agg: HashMap<String, (f64, f64, f64, bool)> = HashMap::new();
    for passport in &passports {
        for c in &passport.categories {
            let e = agg.entry(c.category.clone()).or_insert((0.0, 0.0, 0.0, c.renewable));
            e.0 += c.mass_kg;
            e.1 += c.recyclable_kg;
            e.2 += c.co2_kg;
            e.3 = c.renewable;
        }
    }

    let mut categories: Vec<MaterialCategory> = agg
        .into_iter()
        .map(|(category, (mass, recyclable, co2, renewable))| MaterialCategory {
            category,
            mass_kg: round2(mass),
            recyclable_kg: round2(recyclable),
            recyclability_pct: round1(if mass > 0.0 { recyclable / mass * 100.0 } else { 0.0 }),
            co2_kg: round2(co2),
            renewable,
        })
        .collect();
    categories.sort_by(|a, b| {
        b.mass_kg
            .partial_cmp(&a.mass_kg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_mass: f64 = categories.iter().map(|c| c.mass_kg).sum();
    let total_recyclable: f64 = categories.iter().map(|c| c.recyclable_kg).sum();
    let total_co2: f64 = categories.iter().map(|c| c.co2_kg).sum();
    let renewable_mass: f64 = categories
        .iter()
        .filter(|c| c.renewable)
        .map(|c| c.mass_kg)
        .sum();

    ProjectCircularityResult {
        kozijn_passports: passports,
        categories,
        total_mass_kg: round2(total_mass),
        total_recyclable_kg: round2(total_recyclable),
        total_co2_kg: round2(total_co2),
        circularity_score: round1(if total_mass > 0.0 {
            total_recyclable / total_mass * 100.0
        } else {
            0.0
        }),
        renewable_mass_kg: round2(renewable_mass),
        renewable_pct: round1(if total_mass > 0.0 {
            renewable_mass / total_mass * 100.0
        } else {
            0.0
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::Kozijn;

    #[test]
    fn parses_cross_section_variants() {
        assert_eq!(parse_cross_section_mm("67x114"), Some((67.0, 114.0)));
        assert_eq!(parse_cross_section_mm("Dorpel 67x114"), Some((67.0, 114.0)));
        assert_eq!(parse_cross_section_mm("Raamhout 54x78"), Some((54.0, 78.0)));
        assert_eq!(parse_cross_section_mm("Custom profile"), None);
    }

    #[test]
    fn single_fixed_glass_passport_has_wood_and_glass() {
        let k = Kozijn::new("Test", "T01", 900.0, 1400.0);
        let passport = compute_kozijn_circularity(&k);

        assert!(passport.total_mass_kg > 0.0);
        // A wooden frame with glazing must have at least a wood and a glass category.
        assert!(passport.categories.iter().any(|c| c.category == "Hout"));
        assert!(passport.categories.iter().any(|c| c.category == "Glas"));
        // Score is a valid percentage.
        assert!(passport.circularity_score >= 0.0 && passport.circularity_score <= 100.0);
        // Wood is renewable, so some renewable mass is reported.
        assert!(passport.renewable_mass_kg > 0.0);
    }

    #[test]
    fn project_aggregates_kozijn_masses() {
        let mut project = Project::new("P", "001");
        project.kozijnen.push(Kozijn::new("A", "A01", 1000.0, 1500.0));
        project.kozijnen.push(Kozijn::new("B", "B01", 1000.0, 1500.0));
        let result = calculate_project_circularity(&project);

        assert_eq!(result.kozijn_passports.len(), 2);
        let per_kozijn_total: f64 = result
            .kozijn_passports
            .iter()
            .map(|p| p.total_mass_kg)
            .sum();
        // Project total ≈ sum of per-kozijn totals (allowing rounding drift).
        assert!((result.total_mass_kg - per_kozijn_total).abs() < 1.0);
    }
}
