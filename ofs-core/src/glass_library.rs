use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlassProduct {
    pub id: String,
    pub name: String,
    pub composition: String,       // "4-16-4", "4-16-4-16-4"
    pub total_thickness_mm: f64,
    pub ug_value: f64,
    pub g_value: f64,              // solar heat gain
    pub lt_value: f64,             // light transmittance
    pub rw_db: f64,                // sound insulation
    pub safety_class: String,      // "None", "ESG", "VSG", "P2A", etc.
    pub weight_kg_m2: f64,
}

pub fn builtin_glass_library() -> Vec<GlassProduct> {
    vec![
        GlassProduct { id: "hr++_24".into(), name: "HR++ 4-16-4 (standaard)".into(), composition: "4-16-4".into(), total_thickness_mm: 24.0, ug_value: 1.0, g_value: 0.60, lt_value: 0.80, rw_db: 30.0, safety_class: "None".into(), weight_kg_m2: 20.0 },
        GlassProduct { id: "hr++_28".into(), name: "HR++ 4-20-4".into(), composition: "4-20-4".into(), total_thickness_mm: 28.0, ug_value: 1.0, g_value: 0.60, lt_value: 0.80, rw_db: 31.0, safety_class: "None".into(), weight_kg_m2: 20.0 },
        GlassProduct { id: "hr+++_36".into(), name: "HR+++ 4-16-4-16-4 (triple)".into(), composition: "4-16-4-16-4".into(), total_thickness_mm: 44.0, ug_value: 0.5, g_value: 0.50, lt_value: 0.72, rw_db: 33.0, safety_class: "None".into(), weight_kg_m2: 30.0 },
        GlassProduct { id: "hr++_p2a".into(), name: "HR++ P2A veiligheidsglas".into(), composition: "44.2-16-4".into(), total_thickness_mm: 28.8, ug_value: 1.0, g_value: 0.55, lt_value: 0.75, rw_db: 34.0, safety_class: "P2A".into(), weight_kg_m2: 25.0 },
        GlassProduct { id: "hr++_p4a".into(), name: "HR++ P4A inbraakwerend".into(), composition: "44.4-16-4".into(), total_thickness_mm: 29.6, ug_value: 1.0, g_value: 0.50, lt_value: 0.70, rw_db: 36.0, safety_class: "P4A".into(), weight_kg_m2: 30.0 },
        GlassProduct { id: "hr++_geluid".into(), name: "HR++ akoestisch 6-16-44.2".into(), composition: "6-16-44.2".into(), total_thickness_mm: 30.8, ug_value: 1.1, g_value: 0.55, lt_value: 0.75, rw_db: 40.0, safety_class: "VSG".into(), weight_kg_m2: 27.0 },
        GlassProduct { id: "hr+++_passief".into(), name: "HR+++ Passief 4-18-4-18-4".into(), composition: "4-18-4-18-4".into(), total_thickness_mm: 48.0, ug_value: 0.5, g_value: 0.48, lt_value: 0.70, rw_db: 35.0, safety_class: "None".into(), weight_kg_m2: 30.0 },
        GlassProduct { id: "brandwerend_ei30".into(), name: "Brandwerend EI30".into(), composition: "spec-12-spec".into(), total_thickness_mm: 27.0, ug_value: 1.5, g_value: 0.35, lt_value: 0.60, rw_db: 38.0, safety_class: "EI30".into(), weight_kg_m2: 45.0 },
    ]
}
