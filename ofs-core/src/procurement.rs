use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseProposal {
    pub id: String,
    pub category: String,  // "Hout", "Glas", "Beslag", "Rubber"
    pub items: Vec<PurchaseItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_piece_ids: Vec<String>,
    pub total_excl_btw: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseItem {
    pub description: String,
    pub article_number: Option<String>,
    pub quantity: f64,
    pub unit: String,
    pub unit_price: f64,
    pub total: f64,
    pub kozijn_marks: Vec<String>,
}

pub fn generate_purchase_proposals(
    project: &crate::kozijn::Project,
) -> Vec<PurchaseProposal> {
    let mut hout_items: Vec<PurchaseItem> = Vec::new();
    let mut unresolved_piece_ids = Vec::new();
    let mut glas_items: Vec<PurchaseItem> = Vec::new();
    let mut beslag_items: Vec<PurchaseItem> = Vec::new();

    for kozijn in &project.kozijnen {
        let prod = crate::production::compute_production_data(kozijn);
        let mark = kozijn.mark.clone();

        // Aggregate cut list -> hout
        for cut in &prod.cut_list {
            let Some(gross) = prod.blank_length_mm(cut) else {
                unresolved_piece_ids.push(cut.piece_id.clone());
                continue;
            };
            hout_items.push(PurchaseItem {
                description: format!("{} {}", cut.profile_name, cut.member_label_nl()),
                article_number: None,
                quantity: cut.quantity as f64,
                unit: "stuk".into(),
                unit_price: gross / 1000.0 * 12.50, // estimate
                total: cut.quantity as f64 * gross / 1000.0 * 12.50,
                kozijn_marks: vec![mark.clone()],
            });
        }

        // Glass
        for glass in &prod.glass_list {
            glas_items.push(PurchaseItem {
                description: format!("{} {}mm", glass.glass_type, glass.thickness_mm),
                article_number: None,
                quantity: glass.quantity as f64,
                unit: "stuk".into(),
                unit_price: glass.area_m2 * 45.0,
                total: glass.quantity as f64 * glass.area_m2 * 45.0,
                kozijn_marks: vec![mark.clone()],
            });
        }

        // Hardware
        for hw in &prod.hardware_list {
            beslag_items.push(PurchaseItem {
                description: format!("{}: {}", hw.component, hw.description),
                article_number: None,
                quantity: hw.quantity as f64,
                unit: "stuk".into(),
                unit_price: 25.0,
                total: hw.quantity as f64 * 25.0,
                kozijn_marks: vec![mark.clone()],
            });
        }
    }

    let mut proposals = Vec::new();
    if !hout_items.is_empty() || !unresolved_piece_ids.is_empty() {
        let total: f64 = hout_items.iter().map(|i| i.total).sum();
        proposals.push(PurchaseProposal { id: uuid::Uuid::new_v4().to_string(), category: "Hout".into(), items: hout_items, unresolved_piece_ids, total_excl_btw: total });
    }
    if !glas_items.is_empty() {
        let total: f64 = glas_items.iter().map(|i| i.total).sum();
        proposals.push(PurchaseProposal { id: uuid::Uuid::new_v4().to_string(), category: "Glas".into(), items: glas_items, unresolved_piece_ids: Vec::new(), total_excl_btw: total });
    }
    if !beslag_items.is_empty() {
        let total: f64 = beslag_items.iter().map(|i| i.total).sum();
        proposals.push(PurchaseProposal { id: uuid::Uuid::new_v4().to_string(), category: "Beslag".into(), items: beslag_items, unresolved_piece_ids: Vec::new(), total_excl_btw: total });
    }
    proposals
}
