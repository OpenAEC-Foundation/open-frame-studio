use serde::{Deserialize, Serialize};

use crate::kozijn::{Kozijn, Material, WoodType};
use crate::production::{compute_production_data, ProductionData};

// ── NL-SfB Classification ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NlSfbClassification {
    pub code: String,
    pub description: String,
}

pub fn classify_nl_sfb(kozijn: &Kozijn) -> NlSfbClassification {
    let (code, desc) = match &kozijn.frame.material {
        Material::Wood(_) => ("31.21", "Buitenkozijnen, -ramen, -deuren; hout"),
        Material::Aluminum => ("31.22", "Buitenkozijnen, -ramen, -deuren; aluminium"),
        Material::Pvc => ("31.23", "Buitenkozijnen, -ramen, -deuren; kunststof"),
        Material::WoodAluminum => ("31.24", "Buitenkozijnen, -ramen, -deuren; hout-aluminium"),
    };
    NlSfbClassification {
        code: code.into(),
        description: desc.into(),
    }
}

// ── Price Table ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceTable {
    /// Profile price per running meter (EUR/m)
    pub profile_price_per_m: PriceByMaterial,
    /// Glass price per m2 (EUR/m2)
    pub glass_price_per_m2: f64,
    /// Hardware cost per operable cell (EUR)
    pub hardware_cost_per_cell: f64,
    /// Panel price per m2 (EUR/m2)
    pub panel_price_per_m2: f64,
    /// Gasket price per running meter (EUR/m)
    pub gasket_price_per_m: f64,
    /// Labor rate (EUR/hour)
    pub labor_rate_per_hour: f64,
    /// Surface treatment price per m2 (EUR/m2, 0 if not applicable)
    pub surface_treatment_per_m2: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceByMaterial {
    pub wood_meranti: f64,
    pub wood_accoya: f64,
    pub wood_vuren: f64,
    pub wood_eiken: f64,
    pub aluminum: f64,
    pub pvc: f64,
    pub wood_aluminum: f64,
}

impl PriceByMaterial {
    pub fn get(&self, material: &Material) -> f64 {
        match material {
            Material::Wood(WoodType::Meranti) => self.wood_meranti,
            Material::Wood(WoodType::Accoya) => self.wood_accoya,
            Material::Wood(WoodType::Vuren) => self.wood_vuren,
            Material::Wood(WoodType::Eiken) => self.wood_eiken,
            Material::Aluminum => self.aluminum,
            Material::Pvc => self.pvc,
            Material::WoodAluminum => self.wood_aluminum,
        }
    }
}

impl Default for PriceTable {
    fn default() -> Self {
        Self {
            profile_price_per_m: PriceByMaterial {
                wood_meranti: 12.50,
                wood_accoya: 22.00,
                wood_vuren: 8.00,
                wood_eiken: 30.00,
                aluminum: 18.00,
                pvc: 10.00,
                wood_aluminum: 28.00,
            },
            glass_price_per_m2: 45.00,
            hardware_cost_per_cell: 85.00,
            panel_price_per_m2: 55.00,
            gasket_price_per_m: 1.50,
            labor_rate_per_hour: 55.00,
            surface_treatment_per_m2: 18.00,
        }
    }
}

// ── Cost Estimation ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostEstimate {
    pub nl_sfb: NlSfbClassification,
    pub material_cost: f64,
    pub glass_cost: f64,
    pub hardware_cost: f64,
    pub gasket_cost: f64,
    pub panel_cost: f64,
    pub surface_treatment_cost: f64,
    pub labor_hours: f64,
    pub labor_cost: f64,
    pub total_cost: f64,
    pub line_items: Vec<CostLineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostLineItem {
    pub nl_sfb_code: String,
    pub description: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_price: f64,
    pub total: f64,
}

/// Estimate labor hours based on kozijn complexity.
fn estimate_labor_hours(kozijn: &Kozijn, production: &ProductionData) -> f64 {
    let num_cells = kozijn.cells.len();
    let num_operable = kozijn.cells.iter()
        .filter(|c| c.panel_type.is_operable())
        .count();

    // Base: 1 hour per kozijn + 0.5 per cell + 1.0 per operable cell
    let fabrication = 1.0 + (num_cells as f64 * 0.5) + (num_operable as f64 * 1.0);

    // Surface treatment: 0.3 hours per m2
    let outer_area_m2 = (kozijn.frame.outer_width * kozijn.frame.outer_height) / 1e6;
    let treatment = if matches!(kozijn.frame.material, Material::Wood(_) | Material::WoodAluminum) {
        outer_area_m2 * 0.3
    } else {
        0.0
    };

    // Assembly: 0.5 hours per kozijn + 0.25 per cell
    let assembly = 0.5 + (num_cells as f64 * 0.25);

    fabrication + treatment + assembly
}

/// Compute a cost estimate for a single kozijn.
pub fn estimate_cost(kozijn: &Kozijn, prices: &PriceTable) -> CostEstimate {
    let nl_sfb = classify_nl_sfb(kozijn);
    let production = compute_production_data(kozijn);

    let mut line_items = Vec::new();

    // Profile material cost
    let total_profile_m: f64 = production.cut_list.iter()
        .map(|c| c.gross_length_mm * c.quantity as f64 / 1000.0)
        .sum();
    let profile_price = prices.profile_price_per_m.get(&kozijn.frame.material);
    let material_cost = total_profile_m * profile_price;
    line_items.push(CostLineItem {
        nl_sfb_code: nl_sfb.code.clone(),
        description: format!("Profiel {}", production.kozijn_mark),
        quantity: total_profile_m,
        unit: "m".into(),
        unit_price: profile_price,
        total: material_cost,
    });

    // Glass cost
    let total_glass_m2: f64 = production.glass_list.iter()
        .map(|g| g.area_m2 * g.quantity as f64)
        .sum();
    let glass_cost = total_glass_m2 * prices.glass_price_per_m2;
    if total_glass_m2 > 0.0 {
        line_items.push(CostLineItem {
            nl_sfb_code: nl_sfb.code.clone(),
            description: "Beglazing".into(),
            quantity: total_glass_m2,
            unit: "m2".into(),
            unit_price: prices.glass_price_per_m2,
            total: glass_cost,
        });
    }

    // Hardware cost
    let operable_count = kozijn.cells.iter()
        .filter(|c| c.panel_type.is_operable())
        .count() as f64;
    let hardware_cost = operable_count * prices.hardware_cost_per_cell;
    if operable_count > 0.0 {
        line_items.push(CostLineItem {
            nl_sfb_code: nl_sfb.code.clone(),
            description: "Hang- en sluitwerk".into(),
            quantity: operable_count,
            unit: "stuks".into(),
            unit_price: prices.hardware_cost_per_cell,
            total: hardware_cost,
        });
    }

    // Gasket cost
    let total_gasket_m: f64 = production.gasket_list.iter()
        .map(|g| g.length_mm * g.quantity as f64 / 1000.0)
        .sum();
    let gasket_cost = total_gasket_m * prices.gasket_price_per_m;
    line_items.push(CostLineItem {
        nl_sfb_code: nl_sfb.code.clone(),
        description: "Afdichtingsrubber".into(),
        quantity: total_gasket_m,
        unit: "m".into(),
        unit_price: prices.gasket_price_per_m,
        total: gasket_cost,
    });

    // Panel cost
    let total_panel_m2: f64 = production.panel_list.iter()
        .map(|p| (p.width_mm * p.height_mm / 1e6) * p.quantity as f64)
        .sum();
    let panel_cost = total_panel_m2 * prices.panel_price_per_m2;
    if total_panel_m2 > 0.0 {
        line_items.push(CostLineItem {
            nl_sfb_code: nl_sfb.code.clone(),
            description: "Panelen".into(),
            quantity: total_panel_m2,
            unit: "m2".into(),
            unit_price: prices.panel_price_per_m2,
            total: panel_cost,
        });
    }

    // Surface treatment
    let outer_area_m2 = (kozijn.frame.outer_width * kozijn.frame.outer_height) / 1e6;
    let surface_treatment_cost = if matches!(kozijn.frame.material, Material::Wood(_) | Material::WoodAluminum) {
        let cost = outer_area_m2 * prices.surface_treatment_per_m2;
        line_items.push(CostLineItem {
            nl_sfb_code: nl_sfb.code.clone(),
            description: "Oppervlaktebehandeling".into(),
            quantity: outer_area_m2,
            unit: "m2".into(),
            unit_price: prices.surface_treatment_per_m2,
            total: cost,
        });
        cost
    } else {
        0.0
    };

    // Labor
    let labor_hours = estimate_labor_hours(kozijn, &production);
    let labor_cost = labor_hours * prices.labor_rate_per_hour;
    line_items.push(CostLineItem {
        nl_sfb_code: nl_sfb.code.clone(),
        description: "Arbeid (fabricage + assemblage)".into(),
        quantity: labor_hours,
        unit: "uur".into(),
        unit_price: prices.labor_rate_per_hour,
        total: labor_cost,
    });

    let total_cost = material_cost + glass_cost + hardware_cost
        + gasket_cost + panel_cost + surface_treatment_cost + labor_cost;

    CostEstimate {
        nl_sfb,
        material_cost,
        glass_cost,
        hardware_cost,
        gasket_cost,
        panel_cost,
        surface_treatment_cost,
        labor_hours,
        labor_cost,
        total_cost,
        line_items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::Kozijn;

    #[test]
    fn test_nl_sfb_classification() {
        let k = Kozijn::new("Test", "T01", 900.0, 1400.0);
        let class = classify_nl_sfb(&k);
        assert_eq!(class.code, "31.21");
    }

    #[test]
    fn test_cost_estimate_basic() {
        let k = Kozijn::new("Test", "T01", 900.0, 1400.0);
        let prices = PriceTable::default();
        let estimate = estimate_cost(&k, &prices);

        assert!(estimate.total_cost > 0.0, "Total cost should be positive");
        assert!(estimate.material_cost > 0.0, "Material cost should be positive");
        assert!(estimate.glass_cost > 0.0, "Glass cost should be positive");
        assert_eq!(estimate.hardware_cost, 0.0, "Fixed glass should have no hardware cost");
        assert!(estimate.labor_hours > 0.0, "Labor hours should be positive");
        assert!(!estimate.line_items.is_empty());

        // All line items should have NL-SfB code
        for item in &estimate.line_items {
            assert_eq!(item.nl_sfb_code, "31.21");
        }
    }
}
