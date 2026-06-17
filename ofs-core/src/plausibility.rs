//! Plausibility checks — module 4 (plausibility & code conformance).
//!
//! Provides an **indicative** static wind-load deflection pre-check for the
//! primary frame members (stiles, head/sill and the intermediate mullions and
//! transoms). It is a serviceability plausibility signal in the spirit of
//! EN 12210 (frontal deflection ≤ span/200), **not** a structural calculation:
//! it cannot replace a proper verification per EN 1991-1-4 / Eurocode.
//!
//! Assumptions (clearly indicative):
//! - Each member is modelled as a simply-supported beam under a uniform line
//!   load `q = wind_pressure × tributary_width`, deflection
//!   `δ = 5 q L⁴ / (384 E I)`.
//! - Section is the (solid) frame profile `I = width · depth³ / 12`; hollow
//!   PVC/aluminium sections and any steel reinforcement are ignored (the low
//!   default E values compensate conservatively).
//! - Tributary width = half the adjacent light(s); span = outer dim for the
//!   frame edges, inner dim for intermediate dividers.
//! - Default design wind pressure 1.0 kN/m² (≈ NL low-rise serviceability).

use serde::{Deserialize, Serialize};

use crate::kozijn::{Kozijn, Material, WoodType};

/// Frontal deflection limit (relative): span / 200 (EN 12210 reference).
const DEFLECTION_LIMIT_RATIO: f64 = 200.0;

/// Young's modulus (N/mm² = MPa), indicative and deliberately conservative.
fn youngs_modulus(material: &Material) -> f64 {
    match material {
        Material::Wood(w) => match w {
            WoodType::Eiken => 13000.0,
            WoodType::Meranti => 12000.0,
            WoodType::Accoya => 11000.0,
            WoodType::Vuren => 11000.0,
        },
        Material::Aluminum => 70000.0,
        Material::Pvc => 2700.0, // unreinforced PVC; reinforcement ignored
        Material::WoodAluminum => 12000.0, // wood core governs stiffness
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberDeflection {
    pub member: String,
    pub orientation: String, // "Verticaal" / "Horizontaal"
    pub span_mm: f64,
    pub tributary_mm: f64,
    pub line_load_n_per_mm: f64,
    pub deflection_mm: f64,
    /// Relative stiffness L/δ (higher is stiffer); 0 if deflection is ~0.
    pub deflection_ratio: f64,
    pub limit_ratio: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KozijnPlausibility {
    pub kozijn_mark: String,
    pub kozijn_name: String,
    pub members: Vec<MemberDeflection>,
    /// Worst (smallest) L/δ across members; large value = no concern.
    pub worst_ratio: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPlausibilityResult {
    pub wind_pressure_pa: f64,
    pub limit_ratio: f64,
    pub kozijn_results: Vec<KozijnPlausibility>,
    pub overall_pass: bool,
}

/// Deflection of a simply-supported beam under uniform line load.
/// `q` N/mm, `span`/`E`/`I` in mm, N/mm², mm⁴ → mm.
fn beam_deflection(q_n_per_mm: f64, span_mm: f64, e: f64, i: f64) -> f64 {
    if e <= 0.0 || i <= 0.0 {
        return 0.0;
    }
    5.0 * q_n_per_mm * span_mm.powi(4) / (384.0 * e * i)
}

fn make_member(
    member: String,
    orientation: &str,
    span_mm: f64,
    tributary_mm: f64,
    wind_pressure_pa: f64,
    e: f64,
    i: f64,
) -> MemberDeflection {
    // q [N/mm] = pressure [N/m²] × tributary [m] / 1000
    let q = wind_pressure_pa * (tributary_mm / 1000.0) / 1000.0;
    let deflection = beam_deflection(q, span_mm, e, i);
    let ratio = if deflection > 1e-6 { span_mm / deflection } else { f64::INFINITY };
    let pass = ratio >= DEFLECTION_LIMIT_RATIO;
    MemberDeflection {
        member,
        orientation: orientation.to_string(),
        span_mm: (span_mm * 10.0).round() / 10.0,
        tributary_mm: (tributary_mm * 10.0).round() / 10.0,
        line_load_n_per_mm: (q * 1000.0).round() / 1000.0,
        deflection_mm: (deflection * 100.0).round() / 100.0,
        deflection_ratio: if ratio.is_finite() { ratio.round() } else { 99999.0 },
        limit_ratio: DEFLECTION_LIMIT_RATIO,
        pass,
    }
}

/// Compute the indicative wind-load deflection check for one kozijn.
pub fn compute_kozijn_plausibility(kozijn: &Kozijn, wind_pressure_pa: f64) -> KozijnPlausibility {
    let e = youngs_modulus(&kozijn.frame.material);
    // Second moment of area of the (solid) frame section about the wind axis.
    let width = kozijn.frame.frame_width;
    let depth = kozijn.frame.frame_depth;
    let i_section = width * depth.powi(3) / 12.0;

    let outer_w = kozijn.frame.outer_width;
    let outer_h = kozijn.frame.outer_height;
    let inner_w = kozijn.inner_width();
    let inner_h = kozijn.inner_height();
    let cols = &kozijn.grid.columns;
    let rows = &kozijn.grid.rows;

    let mut members = Vec::new();

    // Vertical members: frame stiles + vertical dividers.
    let first_col = cols.first().map(|c| c.size).unwrap_or(inner_w);
    let last_col = cols.last().map(|c| c.size).unwrap_or(inner_w);
    members.push(make_member(
        "Stijl links".into(), "Verticaal", outer_h, first_col / 2.0, wind_pressure_pa, e, i_section,
    ));
    members.push(make_member(
        "Stijl rechts".into(), "Verticaal", outer_h, last_col / 2.0, wind_pressure_pa, e, i_section,
    ));
    for (idx, _) in cols.iter().enumerate() {
        if let Some(col) = cols.get(idx) {
            if col.divider_profile.is_some() {
                let left = col.size;
                let right = cols.get(idx + 1).map(|c| c.size).unwrap_or(left);
                members.push(make_member(
                    format!("Tussenstijl {}", idx + 1), "Verticaal",
                    inner_h, left / 2.0 + right / 2.0, wind_pressure_pa, e, i_section,
                ));
            }
        }
    }

    // Horizontal members: frame head/sill + horizontal dividers.
    let first_row = rows.first().map(|r| r.size).unwrap_or(inner_h);
    let last_row = rows.last().map(|r| r.size).unwrap_or(inner_h);
    members.push(make_member(
        "Bovendorpel".into(), "Horizontaal", outer_w, first_row / 2.0, wind_pressure_pa, e, i_section,
    ));
    members.push(make_member(
        "Onderdorpel".into(), "Horizontaal", outer_w, last_row / 2.0, wind_pressure_pa, e, i_section,
    ));
    for (idx, _) in rows.iter().enumerate() {
        if let Some(row) = rows.get(idx) {
            if row.divider_profile.is_some() {
                let top = row.size;
                let bottom = rows.get(idx + 1).map(|r| r.size).unwrap_or(top);
                members.push(make_member(
                    format!("Tussendorpel {}", idx + 1), "Horizontaal",
                    inner_w, top / 2.0 + bottom / 2.0, wind_pressure_pa, e, i_section,
                ));
            }
        }
    }

    let worst_ratio = members
        .iter()
        .map(|m| m.deflection_ratio)
        .fold(f64::INFINITY, f64::min);
    let pass = members.iter().all(|m| m.pass);

    KozijnPlausibility {
        kozijn_mark: kozijn.mark.clone(),
        kozijn_name: kozijn.name.clone(),
        members,
        worst_ratio: if worst_ratio.is_finite() { worst_ratio } else { 99999.0 },
        pass,
    }
}

/// Project-level indicative wind-load plausibility.
pub fn calculate_project_plausibility(
    project: &crate::kozijn::Project,
    wind_pressure_pa: f64,
) -> ProjectPlausibilityResult {
    let kozijn_results: Vec<KozijnPlausibility> = project
        .kozijnen
        .iter()
        .map(|k| compute_kozijn_plausibility(k, wind_pressure_pa))
        .collect();
    let overall_pass = kozijn_results.iter().all(|k| k.pass);
    ProjectPlausibilityResult {
        wind_pressure_pa,
        limit_ratio: DEFLECTION_LIMIT_RATIO,
        kozijn_results,
        overall_pass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::{Kozijn, Project};

    #[test]
    fn small_window_passes_under_normal_wind() {
        let k = Kozijn::new("Test", "T01", 1000.0, 1500.0);
        let res = compute_kozijn_plausibility(&k, 1000.0);
        // A 1.0×1.5 m wooden frame at 1 kN/m² should be comfortably stiff.
        assert!(res.pass);
        assert!(res.members.len() >= 4); // 2 stiles + 2 rails
        assert!(res.worst_ratio >= DEFLECTION_LIMIT_RATIO);
    }

    #[test]
    fn larger_span_deflects_more() {
        let small = compute_kozijn_plausibility(&Kozijn::new("S", "S", 1000.0, 1200.0), 1500.0);
        let tall = compute_kozijn_plausibility(&Kozijn::new("T", "T", 1000.0, 3000.0), 1500.0);
        // The taller window's stile must deflect more (smaller L/δ ratio).
        assert!(tall.worst_ratio < small.worst_ratio);
    }

    #[test]
    fn project_aggregates() {
        let mut p = Project::new("P", "1");
        p.kozijnen.push(Kozijn::new("A", "A", 1000.0, 1500.0));
        let r = calculate_project_plausibility(&p, 1000.0);
        assert_eq!(r.kozijn_results.len(), 1);
        assert_eq!(r.wind_pressure_pa, 1000.0);
    }
}
